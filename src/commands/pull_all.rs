use anyhow::{bail, Result};
use inquire::{Confirm, Text};
use log::{debug, info};
use rayon::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf, time::Instant};
use yansi::Paint;

use crate::{
    card::Card, cli::LanguageCode, localizer::Localizer, scraper::OpTcgScraper, storage::DataStore,
    utils,
};

fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");

    println!("{}", "+-----------------------------------+".yellow());
    println!(
        "{} {} {}",
        "|".yellow(),
        "vega - One Piece TCG Data Scraper".blue().bold(),
        "|".yellow()
    );
    println!(
        "{} {} {}",
        "|".yellow(),
        format!("version: {version}                   ")
            .white()
            .bold(),
        "|".yellow()
    );
    println!("{}", "+-----------------------------------+\n".yellow());
}

struct InteractiveInputs {
    language: LanguageCode,
    data_dir: PathBuf,
    download_images: bool,
}

fn get_inputs_from_user() -> Result<InteractiveInputs> {
    let language = LanguageCode::select("Choose a language:").prompt()?;

    info!("using language: {:?}", language);

    let download_dir = Text::new("Enter location to save data:")
        .with_default(&utils::get_default_data_dirname(language))
        .prompt()?;

    let download_dir = PathBuf::from(&download_dir);
    if download_dir.exists() {
        handle_existing_dir(&download_dir)?;
    }

    info!("prompting user whether to download images");
    let download_images = Confirm::new("Download images as well?")
        .with_default(false)
        .with_help_message("Downlading images might take some time")
        .prompt()?;

    Ok(InteractiveInputs {
        language,
        data_dir: download_dir,
        download_images,
    })
}

fn handle_existing_dir(data_dir: &PathBuf) -> Result<()> {
    info!(
        "directory `{}` exists, prompting user for removal",
        data_dir.display()
    );

    let replace_existing_dir = Confirm::new(&format!(
        "Directory '{}' already exists. Overwrite?",
        data_dir.display()
    ))
    .with_default(false)
    .with_help_message("This will delete all existing data in this directory")
    .prompt()?;

    info!("user input: {}", replace_existing_dir);

    if replace_existing_dir {
        fs::remove_dir_all(data_dir)?;
        eprintln!("Cleared directory: {}", data_dir.display());
    } else {
        bail!("Aborted, directory has been kept: `{}`", data_dir.display());
    }

    Ok(())
}

pub fn pull_all(
    language: LanguageCode,
    output_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
    user_agent: Option<String>,
) -> Result<()> {
    pull_all_interactive(config_path, user_agent)
}

fn pull_all_interactive(config_path: Option<PathBuf>, user_agent: Option<String>) -> Result<()> {
    print_banner();

    let inputs = get_inputs_from_user()?;

    let localizer = Localizer::load(inputs.language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent);
    let store = DataStore::new(&inputs.data_dir, inputs.language);

    eprintln!("Fetching list of packs...");

    let start = Instant::now();

    let packs = scraper.fetch_packs()?;
    store.write_packs(&packs)?;

    eprintln!("Found {} packs!\n", packs.len());

    let pack_ids = packs.iter().map(|p| p.id.as_str()).collect::<Vec<_>>();

    eprintln!("Now fetching all the cards for each pack...");
    let all_cards = scraper.fetch_all_cards(&pack_ids, true)?;

    for (pack_id, cards) in all_cards.iter() {
        store.write_cards(pack_id, cards)?;
        debug!("wrote cards for: `{}`", pack_id);
    }

    let cards_by_id: HashMap<String, Card> = all_cards
        .into_iter()
        .flat_map(|(_, cards)| cards)
        .map(|card| (card.id.to_owned(), card))
        .collect();

    eprintln!("Wrote data for all {} packs", pack_ids.len());

    if inputs.download_images {
        eprintln!("Downloading all images for every single card...");

        let all_cards = cards_by_id.values().collect::<Vec<_>>();
        let images = scraper.fetch_all_card_images(&all_cards, true)?;

        images.par_iter().for_each(|(card_id, image_data)| {
            let card = cards_by_id
                .get(card_id)
                .unwrap_or_else(|| panic!("card should exist: {card_id}"));

            store
                .write_image(card, image_data.to_vec())
                .unwrap_or_else(|_| panic!("write_image failed for: {card_id}"));
            debug!("wrote image_data for: {}", card_id);
        });
    }

    let duration = start.elapsed();

    eprintln!(
        "\nFinal data is available in: {}",
        inputs.data_dir.display()
    );
    eprintln!("Full download completed after: {:?}", duration);
    Ok(())
}

// fn download_images(scraper: &OpTcgScraper, cards: &HashMap<String, Card>) -> Result<()> {
//     let card_values = cards.values().collect::<Vec<_>>();
//     let images = scraper.fetch_all_card_images(&cards)?;
//
//     images.par_iter().for_each(|(card_id, image_data)| {
//         let card = cards_by_id
//             .get(card_id)
//             .unwrap_or_else(|| panic!("card should exist: {card_id}"));
//
//         store
//             .write_image(card, image_data.to_vec())
//             .unwrap_or_else(|_| panic!("write_image failed for: {card_id}"));
//         debug!("wrote image_data for: {}", card_id);
//     });
//
//     Ok(())
// }
