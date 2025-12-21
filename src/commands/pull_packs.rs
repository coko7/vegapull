use anyhow::Result;
use log::info;
use std::{fs, path::Path, time::Instant};

use crate::{cli::LanguageCode, localizer::Localizer, scraper::OpTcgScraper};

pub fn pull_packs(
    language: LanguageCode,
    output_file: Option<&Path>,
    user_agent: Option<String>,
) -> Result<()> {
    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent);

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
