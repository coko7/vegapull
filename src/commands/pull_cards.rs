use anyhow::{bail, Result};
use log::{debug, error, info};
use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use crate::{
    card::Card,
    cli::{CardDownloadMode, LanguageCode},
    localizer::Localizer,
    scraper::OpTcgScraper,
    storage::DataStore,
    utils,
};

pub fn pull_cards(
    language: LanguageCode,
    pack_id: &str,
    output_path: Option<&Path>,
    mode: CardDownloadMode,
    user_agent: Option<String>,
) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent);

    eprintln!("fetching all cards for pack {pack_id}...");
    let start = Instant::now();

    let cards = scraper.fetch_cards(pack_id)?;
    if cards.is_empty() {
        error!("No cards available for pack {}", pack_id);
        bail!("No cards found");
    }

    eprintln!("successfully fetched {} cards!", cards.len());

    let json = serde_json::to_string(&cards)?;

    todo!();

    // match mode {
    //     CardDownloadMode::ImageOnly => save_images_to_fs(language, cards, output_path, user_agent)?,
    //     CardDownloadMode::DataOnly => {
    //         let default_filename = format!("cards_{pack_id}.json");
    //         let default_file_path = current_dir()?.join(default_filename);
    //         let out_json_path = output_path.unwrap_or(&default_file_path);
    //
    //         if let Some(parent) = default_file_path.parent() {
    //             fs::create_dir_all(parent)?;
    //         }
    //
    //         fs::write(out_json_path, &json)?;
    //     }
    //     CardDownloadMode::All => {
    //         let default_data_path = utils::get_default_data_dir()?;
    //         let output_dir = output_path.unwrap_or(&default_data_path);
    //
    //         if let Some(parent) = output_dir.parent() {
    //             fs::create_dir_all(parent)?;
    //         }
    //
    //         let default_filename = format!("cards_{pack_id}.json");
    //         let out_json_path = output_dir.join(default_filename);
    //         fs::write(&out_json_path, &json)?;
    //
    //         save_images_to_fs(language, cards, output_path, user_agent)?;
    //     }
    // };

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

pub fn download_images_fast(
    language: LanguageCode,
    cards: Vec<Card>,
    output_dir: &Path,
    user_agent: Option<String>,
) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent);

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

fn save_images_to_fs(
    language: LanguageCode,
    cards: Vec<Card>,
    output_dir: Option<&Path>,
    user_agent: Option<String>,
) -> Result<()> {
    let default_data_path = utils::get_default_data_dir()?;
    let output_dir = output_dir.unwrap_or(&default_data_path).join("images");

    eprintln!("downloading images to: {}", output_dir.display());
    download_images_fast(language, cards, &output_dir, user_agent)
}
