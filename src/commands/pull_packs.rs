use anyhow::Result;
use log::debug;
use std::{collections::HashSet, path::Path, time::SystemTime};

use crate::{
    cli::LanguageCode,
    localizer::Localizer,
    pack::PackId,
    scraper::OpTcgScraper,
    storage::{DataStore, PullMode, VegaMetaStats},
    utils,
};

pub fn pull_packs(
    language: LanguageCode,
    output_dir: Option<&Path>,
    user_agent: Option<String>,
) -> Result<()> {
    let default_data_path = utils::get_default_data_dir(language)?;
    let output_dir = output_dir.unwrap_or(&default_data_path);

    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent);
    let store = DataStore::new(output_dir, language);

    eprintln!("fetching list of packs...");
    let start = SystemTime::now();

    let packs = scraper.fetch_packs()?;
    let pack_ids: HashSet<PackId> = packs.keys().cloned().collect();
    store.write_packs(&packs)?;

    println!(
        "downloaded {} packs to: {}",
        packs.len(),
        output_dir.display()
    );

    let duration = start.elapsed()?;

    debug!("pull_packs took: {:?}", duration);

    store.write_vega_stats(VegaMetaStats::new(
        language,
        start.into(),
        duration.as_millis().try_into()?,
        false,
        PullMode::PackListOnly,
        pack_ids,
    ))?;

    Ok(())
}
