use anyhow::{bail, Context, Result};
use inquire::{Confirm, Text};
use log::{debug, info};
use rayon::prelude::*;
use std::{fs, path::PathBuf, time::Instant};
use yansi::Paint;

use crate::{
    card::Card, cli::LanguageCode, get_default_data_dirname, localizer::Localizer,
    scraper::OpTcgScraper, storage::DataStore,
};

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
        println!("Cleared directory: `{}`", data_dir.display());
        fs::remove_dir_all(data_dir)?;
        info!("removed directory: {}", data_dir.display());
    } else {
        info!("user cancelled directory removal: {}", data_dir.display());
        bail!("Aborted, directory has been kept: `{}`", data_dir.display());
    }

    Ok(())
}

fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", "+-----------------------------+".yellow());
    println!(
        "{} {} {}",
        "|".yellow(),
        "VegaPull - TCG Data Scraper".blue().bold(),
        "|".yellow()
    );
    println!(
        "{} {} {}",
        "|".yellow(),
        format!("version: {version}             ").white().bold(),
        "|".yellow()
    );
    println!("{}", "+-----------------------------+\n".yellow());
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
        .with_default(&get_default_data_dirname())
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

pub fn show_interactive() -> Result<()> {
    print_banner();

    let inputs = get_inputs_from_user()?;

    let localizer = Localizer::load(inputs.language)?;
    let scraper = OpTcgScraper::new(localizer);
    let store = DataStore::new(&inputs.data_dir, inputs.language);

    eprintln!("Fetching list of packs...");

    let start = Instant::now();

    let packs = scraper.fetch_packs()?;
    store.write_packs(&packs)?;

    eprintln!("Found {} packs!\n", packs.len());

    let pack_ids = packs.iter().map(|p| p.id.as_str()).collect::<Vec<_>>();

    eprintln!("Now fetching all the cards for each pack...");
    let all_cards = scraper.fetch_all_cards(&pack_ids, true)?;

    let all_cards_flattened: Vec<Card> = all_cards
        .clone()
        .into_iter()
        .flat_map(|(_k, vs)| vs.into_iter())
        .collect();

    for (pack_id, cards) in all_cards.iter() {
        store.write_cards(pack_id, cards)?;
        debug!("wrote cards for: `{}`", pack_id);
    }

    eprintln!("Wrote data for all {} packs", pack_ids.len());

    if inputs.download_images {
        eprintln!("now downloading all images for every single card");

        let images = scraper.download_all_card_images(&all_cards_flattened)?;
        images.par_iter().for_each(|(card_id, image_data)| {
            let card = all_cards_flattened
                .iter()
                .find(|card| card.id == *card_id)
                .context("card should exist")
                .expect("card should exist"); // Use expect() or handle errors differently
            store
                .write_image(card, image_data.to_vec())
                .expect("write_image failed"); // Handle errors per operation
            debug!("wrote image_data for: {}", card_id);
        });

        // for (card_id, image_data) in images.iter() {
        //     let card = all_cards_flattened
        //         .iter()
        //         .find(|card| card.id == *card_id)
        //         .context("card should exist")?;
        //     store.write_image(card, image_data.to_vec())?;
        //     debug!("wrote image_data for: {}", card_id);
        // }
    }

    // for (pack_id, cards) in all_cards.iter() {
    //     store.write_cards(&pack_id, &cards)?;
    //     info!("wrote cards for: `{}`", pack_id);
    //
    //     if inputs.download_images {
    //         let images = scraper.download_all_card_images(&cards)?;
    //         for (card_id, image_data) in images.iter() {
    //             let card = all_cards_flattened
    //                 .iter()
    //                 .find(|card| card.id == *card_id)
    //                 .context("card should exist")?;
    //             store.write_image(card, image_data.to_vec())?;
    //             debug!("wrote image_data for: {}", card_id);
    //         }
    //     }
    // }

    let duration = start.elapsed();
    info!("fetching cards (and images) took: {:?}", duration);

    eprintln!(
        "\nFinal data is available in: {}",
        inputs.data_dir.display()
    );
    eprintln!("Full download took: {:?}", duration);

    Ok(())
}
