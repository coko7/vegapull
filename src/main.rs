use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use log::{debug, error, info};
use std::collections::HashSet;
use std::path::PathBuf;
use std::{fs, path::Path, process::ExitCode, time::Instant};

use crate::cli::{Cli, LanguageCode};
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
        cli::Commands::Packs { output_file } => list_packs(args.language, output_file.as_deref()),
        cli::Commands::Cards {
            pack_id,
            output_file,
        } => list_cards(
            args.language,
            &pack_id.to_string_lossy(),
            output_file.as_deref(),
        ),
        cli::Commands::Interactive => interactive::show_interactive(),
        cli::Commands::Images {
            pack_id,
            output_dir,
        } => download_images(args.language, &pack_id.to_string_lossy(), &output_dir),
        cli::Commands::TestConfig => Localizer::find_locales(),
        cli::Commands::Diff { pack_files } => show_diffs(pack_files),
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

fn download_images(language: LanguageCode, pack_id: &str, output_dir: &Path) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(&localizer);

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

    info!("fetching all cards for pack `{}`...", pack_id);
    let start = Instant::now();

    let cards = scraper.fetch_all_cards(pack_id)?;
    if cards.is_empty() {
        error!("no cards available for pack `{}`", pack_id);
        bail!("no cards found for pack `{}`", pack_id);
    }

    info!(
        "successfully fetched {} cards for pack: `{}`!",
        cards.len(),
        pack_id
    );

    let duration = start.elapsed();
    info!("fetching cards took: {:?}", duration);

    info!("downloading images for pack `{}`...", pack_id);
    let start = Instant::now();

    for (idx, card) in cards.iter().enumerate() {
        let img_filename = DataStore::get_img_filename(card)?;
        let img_path = output_dir.join(img_filename);

        let img_data = scraper.download_card_image(card)?;
        DataStore::write_image_to_file(img_data, &img_path)?;

        debug!(
            "[{}/{}] saved image `{}` to `{}`",
            idx + 1,
            cards.len(),
            card.img_url,
            img_path.display()
        );
    }

    let duration = start.elapsed();
    info!("downloading images took: {:?}", duration);
    Ok(())
}

fn list_packs(language: LanguageCode, output_file: Option<&Path>) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(&localizer);

    info!("fetching all pack ids...");
    let start = Instant::now();

    let all_packs = scraper.fetch_all_packs()?;
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

fn list_cards(language: LanguageCode, pack_id: &str, output_file: Option<&Path>) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(&localizer);

    info!("fetching all cards...");
    let start = Instant::now();

    let cards = scraper.fetch_all_cards(pack_id)?;
    if cards.is_empty() {
        error!("No cards available for pack `{}`", pack_id);
        bail!("No cards found");
    }

    info!(
        "successfully fetched {} cards for pack: `{}`!",
        cards.len(),
        pack_id
    );

    let json = serde_json::to_string(&cards)?;
    if let Some(path) = output_file {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
    } else {
        println!("{}", json);
    }

    let duration = start.elapsed();

    info!("list_cards took: {:?}", duration);
    Ok(())
}
