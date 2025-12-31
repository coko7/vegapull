use anyhow::{bail, Result};
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::SystemTime,
};

use crate::{
    card::Card,
    cli::LanguageCode,
    localizer::Localizer,
    scraper::OpTcgScraper,
    storage::{DataStore, PullMode, VegaMetaStats},
    utils,
};

pub fn pull_cards(
    language: LanguageCode,
    pack_id: &str,
    output_dir: Option<&Path>,
    with_images: bool,
    user_agent: Option<String>,
) -> Result<()> {
    let default_data_path = utils::get_default_data_dir(language)?;
    let output_dir = output_dir.unwrap_or(&default_data_path);

    let localizer = Localizer::load(language)?;
    let scraper = OpTcgScraper::new(localizer, user_agent.clone());
    let store = DataStore::new(output_dir, language);

    eprintln!("fetching all cards for pack {pack_id}...");
    let start = SystemTime::now();

    let cards = scraper.fetch_cards(pack_id)?;
    if cards.is_empty() {
        error!("No cards available for pack {}", pack_id);
        bail!("No cards found");
    }

    store.write_cards(pack_id, &cards)?;

    eprintln!("successfully fetched {} cards!", cards.len());

    if with_images {
        eprintln!("Downloading all images for every single card...");

        let cards_by_id: HashMap<String, Card> = cards
            .into_iter()
            .map(|card| (card.id.to_owned(), card))
            .collect();

        let cards = cards_by_id.values().collect::<Vec<_>>();
        let images = scraper.fetch_all_card_images(&cards, true)?;

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

    println!(
        "downloaded cards for pack {} to: {}",
        pack_id,
        output_dir.display()
    );

    let duration = start.elapsed()?;

    info!("list_cards took: {:?}", duration);

    store.write_vega_stats(VegaMetaStats::new(
        language,
        start.into(),
        duration.as_millis().try_into()?,
        with_images,
        PullMode::SinglePack,
        HashSet::from([pack_id.to_owned()]),
    ))?;

    Ok(())
}
