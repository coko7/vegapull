use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

use anyhow::{bail, Result};
use clap::ValueEnum;
use log::{error, info};
use yansi::Paint;

use crate::{cli::LanguageCode, localizer::Localizer, scraper::OpTcgScraper, storage::DataStore};

pub fn show_interactive() -> Result<()> {
    println!("{}", "+---------------------------+".yellow());
    println!(
        "{} {} {}",
        "|".yellow(),
        "VegaPull Interactive Mode".blue().bold(),
        "|".yellow()
    );
    println!("{}", "+---------------------------+\n".yellow());

    info!("prompting user for language selection");
    let prompt = format!("Enter language ({}): ", "english".green());
    let input = input_prompt(&prompt)?;
    info!("user input: {}", input);

    let value = if input.is_empty() { "english" } else { &input };
    info!("value to use: {}", value);

    let language = match LanguageCode::from_str(value, true) {
        Ok(val) => val,
        Err(_) => bail!("Failed to parse language code, invalid value: {}", value),
    };
    info!("using language: {:?}", language);

    info!("prompting user for save location");
    let prompt = format!("Enter location to save data ({}): ", "./data".green());
    let input = input_prompt(&prompt)?;
    info!("user input: {}", input);

    let value = if input.is_empty() { "./data" } else { &input };
    info!("value to use: {}", value);

    let data_dir = PathBuf::from(&value);
    if data_dir.exists() {
        info!(
            "directory `{}` exists, prompting user for removal",
            data_dir.display()
        );

        let prompt = format!(
            "Dir `{}` already exists. Replace? ({}/{}): ",
            data_dir.display(),
            "y".green(),
            "N".red()
        );
        let input = input_prompt(&prompt)?;
        info!("user input: {}", input);

        let value = if input.is_empty() { "no" } else { &input };
        info!("value to use: {}", value);

        if is_yes(value) {
            println!("Cleared directory: `{}`", data_dir.display());
            fs::remove_dir_all(&data_dir)?;
            info!("removed directory: {}", data_dir.display());
        } else {
            info!("user cancelled directory removal: {}", data_dir.display());
            bail!("Aborted, directory has been kept: `{}`", data_dir.display());
        }
    }

    info!("prompting user whether to download images");
    let prompt = format!("Download images as well ({}/{}): ", "y".green(), "N".red());
    let input = input_prompt(&prompt)?;
    info!("user input: {}", input);

    let value = if input.is_empty() { "no" } else { &input };
    info!("value to use: {}", value);

    let download_images = is_yes(value);

    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer);
    let store = DataStore::new(&data_dir, language);

    println!("\nFetching packs...");
    let start = Instant::now();

    let packs = scraper.fetch_all_packs()?;
    store.write_packs(&packs)?;

    let duration = start.elapsed();
    info!("fetching packs took: {:?}", duration);

    println!("Successfully stored data for {} packs!\n", packs.len());

    let start = Instant::now();
    for (idx, pack) in packs.iter().enumerate() {
        print!(
            "[{}/{}] Fetching cards for pack `{}`...",
            (idx + 1),
            packs.len(),
            pack.id
        );
        io::stdout().flush()?;

        let cards = scraper.fetch_all_cards(&pack.id)?;
        if cards.is_empty() {
            error!("no cards available for pack `{}`", &pack.id);
            bail!("No cards found");
        }

        store.write_cards(&pack.id, &cards)?;
        info!("fetched and wrote cards for: `{}`", pack.id);
        println!(" OK");

        if download_images {
            println!("Downloading images for pack `{}`...", pack.id);
            for (idx, card) in cards.iter().enumerate() {
                print!(
                    "[{}/{}] Downloading image for card `{}`...",
                    idx + 1,
                    cards.len(),
                    card.id
                );
                io::stdout().flush()?;

                let img_data = scraper.download_card_image(card)?;
                store.write_image(card, img_data)?;
                println!(" OK");
            }
        }
    }

    let duration = start.elapsed();
    info!("fetching cards (and images) took: {:?}", duration);

    println!("Final data is available in: {}", data_dir.display());

    Ok(())
}

fn is_yes(input: &str) -> bool {
    let input = input.trim().to_lowercase();
    matches!(input.as_str(), "yes" | "y")
}

fn input_prompt(text: &str) -> Result<String> {
    print!("{}", text);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    Ok(input)
}
