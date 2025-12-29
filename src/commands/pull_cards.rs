use anyhow::{bail, Result};
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, path::Path, time::Instant};

use crate::{
    card::Card, cli::LanguageCode, localizer::Localizer, scraper::OpTcgScraper, storage::DataStore,
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
    let start = Instant::now();

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

    let duration = start.elapsed();

    info!("list_cards took: {:?}", duration);
    Ok(())
}

// pub fn download_images_fast(
//     language: LanguageCode,
//     cards: Vec<Card>,
//     output_dir: &Path,
//     user_agent: Option<String>,
// ) -> Result<()> {
//     let localizer = Localizer::load(language)?;
//     let scraper = OpTcgScraper::new(localizer, user_agent);
//
//     if output_dir.exists() {
//         error!("output directory already `{}` exists", output_dir.display());
//         bail!(
//             "cannot create directory `{}` to store images because it already exists",
//             output_dir.display()
//         );
//     }
//
//     match fs::create_dir_all(output_dir) {
//         Ok(_) => info!("successfully created `{}`", output_dir.display()),
//         Err(e) => bail!("failed to create `{}`: {}", output_dir.display(), e),
//     }
//
//     info!("downloading images...");
//     let start = Instant::now();
//
//     let mut handles = vec![];
//
//     let completed_count = Arc::new(AtomicUsize::new(0));
//     let all_cards = cards.len();
//
//     let scraper = Arc::new(scraper);
//     let output_dir = output_dir.to_path_buf();
//
//     for card in cards.into_iter() {
//         let scraper = Arc::clone(&scraper);
//         let output_dir = output_dir.clone();
//         let completed_count = Arc::clone(&completed_count);
//
//         let handle = thread::spawn(move || {
//             let img_url = card.img_url.clone();
//             let img_path = download_card_image(&output_dir, &scraper, card).unwrap();
//             let current = completed_count.fetch_add(1, Ordering::SeqCst) + 1;
//
//             eprintln!(
//                 "[{}/{}] succesfully saved image `{}` to `{}`",
//                 current,
//                 all_cards,
//                 img_url,
//                 img_path.display()
//             );
//
//             debug!(
//                 "[{}/{}] saved image `{}` to `{}`",
//                 current,
//                 all_cards,
//                 img_url,
//                 img_path.display()
//             );
//         });
//         handles.push(handle);
//     }
//
//     for handle in handles {
//         handle.join().unwrap();
//     }
//
//     let duration = start.elapsed();
//     info!("downloading images took: {:?}", duration);
//     Ok(())
// }
