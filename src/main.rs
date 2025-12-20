use anyhow::{bail, ensure, Context, Result};
use chrono::Local;
use clap::Parser;
use log::{debug, error, info};
use std::collections::HashSet;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::{fs, path::Path, process::ExitCode, time::Instant};

use crate::card::Card;
use crate::cli::{CardDownloadMode, Cli, LanguageCode};
use crate::config::initialize_configs;
use crate::localizer::Localizer;
use crate::pack::Pack;
use crate::scraper::OpTcgScraper;
use crate::storage::DataStore;

mod card;
mod cli;
mod config;
mod interactive;
mod localizer;
mod pack;
mod scraper;
mod storage;

fn main() -> ExitCode {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match process_args(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn process_args(args: Cli) -> Result<()> {
    info!("initialize config");
    initialize_configs()?;

    match args.command {
        cli::Commands::Pull {
            command,
            language,
            config_directory_path,
            user_agent,
        } => match command {
            cli::PullSubCommands::All => interactive::show_interactive(),
            cli::PullSubCommands::Packs { output_file } => {
                list_packs(language, output_file.as_deref())
            }
            cli::PullSubCommands::Cards {
                pack_id,
                output_path,
                mode,
            } => download_cards(
                language,
                &pack_id.to_string_lossy(),
                output_path.as_deref(),
                mode,
            ),
        },
        cli::Commands::Diff { pack_files } => show_diffs(pack_files),
        cli::Commands::Config => Localizer::find_locales(),
    }
}

fn show_diffs(pack_files: Option<Vec<PathBuf>>) -> Result<()> {
    if let Some(pack_files) = pack_files {
        ensure!(pack_files.len() == 2, "exactly two packs must be provided");

        let old_packs_path = pack_files.first().context("there should be a first")?;
        let new_packs_path = pack_files.last().context("there should be a last")?;

        ensure!(Path::exists(old_packs_path), "old_packs file not found");
        ensure!(Path::exists(new_packs_path), "new_packs file not found");

        let old_packs = fs::read_to_string(old_packs_path)?;
        let old_packs: Vec<Pack> = serde_json::from_str(&old_packs)?;
        let old_packs: HashSet<_> = old_packs.iter().collect();
        debug!(
            "successfully loaded {} packs from: `{}`",
            old_packs.len(),
            old_packs_path.display()
        );

        let new_packs = fs::read_to_string(new_packs_path)?;
        let new_packs: Vec<Pack> = serde_json::from_str(&new_packs)?;
        let new_packs: HashSet<_> = new_packs.iter().collect();
        debug!(
            "successfully loaded {} packs from: `{}`",
            new_packs.len(),
            new_packs_path.display()
        );

        let diff_packs: Vec<_> = old_packs.symmetric_difference(&new_packs).collect();
        debug!(
            "found {} diff(s) between both sets: {:#?}",
            diff_packs.len(),
            diff_packs
        );

        let diff_json = serde_json::to_string(&diff_packs)?;
        println!("{}", diff_json);
        return Ok(());
    }

    bail!("missing arguments")
}

pub fn download_images_fast(
    language: LanguageCode,
    cards: Vec<Card>,
    output_dir: &Path,
) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer);

    if output_dir.exists() {
        error!("output directory already `{}` exists", output_dir.display());
        bail!(
            "cannot create directory `{}` to store images because it already exists",
            output_dir.display()
        );
    }

    match fs::create_dir_all(output_dir) {
        Ok(_) => info!("successfully created `{}`", output_dir.display()),
        Err(e) => bail!("failed to create `{}`: {}", output_dir.display(), e),
    }

    info!("downloading images...");
    let start = Instant::now();

    let mut handles = vec![];

    let completed_count = Arc::new(AtomicUsize::new(0));
    let all_cards = cards.len();

    let scraper = Arc::new(scraper);
    let output_dir = output_dir.to_path_buf();

    for card in cards.into_iter() {
        let scraper = Arc::clone(&scraper);
        let output_dir = output_dir.clone();
        let completed_count = Arc::clone(&completed_count);

        let handle = thread::spawn(move || {
            let img_url = card.img_url.clone();
            let img_path = download_card_image(&output_dir, &scraper, card).unwrap();
            let current = completed_count.fetch_add(1, Ordering::SeqCst) + 1;

            eprintln!(
                "[{}/{}] succesfully saved image `{}` to `{}`",
                current,
                all_cards,
                img_url,
                img_path.display()
            );

            debug!(
                "[{}/{}] saved image `{}` to `{}`",
                current,
                all_cards,
                img_url,
                img_path.display()
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    info!("downloading images took: {:?}", duration);
    Ok(())
}

fn download_card_image(
    output_dir: &Path,
    scraper: &Arc<OpTcgScraper>,
    card: Card,
) -> Result<PathBuf> {
    let img_filename = DataStore::get_img_filename(&card)?;
    let img_path = output_dir.join(img_filename);
    let img_data = scraper.download_card_image(&card)?;

    DataStore::write_image_to_file(img_data, &img_path)?;
    Ok(img_path)
}

fn list_packs(language: LanguageCode, output_file: Option<&Path>) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer);

    info!("fetching all pack ids...");
    let start = Instant::now();

    let all_packs = scraper.fetch_packs()?;
    info!("successfully fetched {} packs!", all_packs.len());

    let out_json = serde_json::to_string(&all_packs)?;

    if let Some(path) = output_file {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, out_json)?;
    } else {
        println!("{}", out_json);
    }

    let duration = start.elapsed();

    info!("list_packs took: {:?}", duration);
    Ok(())
}

fn download_cards(
    language: LanguageCode,
    pack_id: &str,
    output_path: Option<&Path>,
    mode: CardDownloadMode,
) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer);

    eprintln!("fetching all cards for pack {pack_id}...");
    let start = Instant::now();

    let cards = scraper.fetch_cards(pack_id)?;
    if cards.is_empty() {
        error!("No cards available for pack {}", pack_id);
        bail!("No cards found");
    }

    eprintln!("successfully fetched {} cards!", cards.len());

    let json = serde_json::to_string(&cards)?;

    match mode {
        CardDownloadMode::ImageOnly => save_images_to_fs(language, cards, output_path)?,
        CardDownloadMode::DataOnly => {
            let default_filename = format!("cards_{pack_id}.json");
            let default_file_path = current_dir()?.join(default_filename);
            let out_json_path = output_path.unwrap_or(&default_file_path);

            if let Some(parent) = default_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(out_json_path, &json)?;
        }
        CardDownloadMode::All => {
            let default_data_path = get_default_data_dir()?;
            let output_dir = output_path.unwrap_or(&default_data_path);

            if let Some(parent) = output_dir.parent() {
                fs::create_dir_all(parent)?;
            }

            let default_filename = format!("cards_{pack_id}.json");
            let out_json_path = output_dir.join(default_filename);
            fs::write(&out_json_path, &json)?;

            save_images_to_fs(language, cards, output_path)?;
        }
    };

    // if mode == CardDownloadMode::DataOnly || mode == CardDownloadMode::All {
    //     if let Some(path) = output_path {
    //         if let Some(parent) = path.parent() {
    //             fs::create_dir_all(parent)?;
    //         }
    //         fs::write(path, json)?;
    //     } else {
    //         println!("{}", json);
    //     }
    // }

    let duration = start.elapsed();

    info!("list_cards took: {:?}", duration);
    Ok(())
}

fn save_images_to_fs(
    language: LanguageCode,
    cards: Vec<Card>,
    output_dir: Option<&Path>,
) -> Result<()> {
    let default_data_path = get_default_data_dir()?;
    let output_dir = output_dir.unwrap_or(&default_data_path).join("images");

    eprintln!("downloading images to: {}", output_dir.display());
    download_images_fast(language, cards, &output_dir)
}

fn get_default_data_dirname() -> String {
    let timestamp = Local::now().format("%y%m%d_%H%M%S").to_string();
    format!("data-{}", timestamp)
}

fn get_default_data_dir() -> Result<PathBuf> {
    let dir_name = get_default_data_dirname();
    Ok(current_dir()?.join(dir_name))
}
