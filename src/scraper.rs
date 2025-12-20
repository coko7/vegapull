use anyhow::{bail, Context, Result};
use log::{debug, info};
use rayon::prelude::*;
use scraper::Html;
use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant},
};

use crate::{
    card::{Card, CardScraper},
    localizer::Localizer,
    pack::Pack,
};

pub struct OpTcgScraper {
    base_url: String,
    localizer: Localizer,
    client: reqwest::blocking::Client,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

impl OpTcgScraper {
    pub fn new(localizer: Localizer) -> OpTcgScraper {
        OpTcgScraper {
            base_url: localizer.hostname.clone(),
            localizer,
            client: reqwest::blocking::ClientBuilder::new()
                .user_agent(APP_USER_AGENT)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    fn cardlist_endpoint(&self) -> String {
        format!("{}/{}", self.base_url, "cardlist")
    }

    fn get_img_full_url(&self, img_url: &str) -> String {
        let short_img_url = &img_url[3..];
        let full_url = format!("{}/{}", self.base_url, short_img_url);
        debug!("full url: {}", full_url);

        full_url
    }

    pub fn fetch_packs(&self) -> Result<Vec<Pack>> {
        let start = Instant::now();

        let url = self.cardlist_endpoint();
        debug!("GET `{}`", url);

        let response = self.client.get(url).send()?.text()?;

        debug!("parsing HTML document");
        let document = scraper::Html::parse_document(&response);

        let sel = "div.seriesCol>select#series>option";
        debug!("fetching series (packs) ({})...", sel);

        let series_selector = scraper::Selector::parse(sel).unwrap();

        let mut packs = Vec::new();
        for element in document.select(&series_selector) {
            match Pack::new(element) {
                Ok(pack) => {
                    if !pack.id.is_empty() {
                        packs.push(pack);
                    }
                }
                Err(e) => bail!("failed to scrape data about packs: {}", e),
            }
        }

        let duration = start.elapsed();
        debug!("fetching packs took: {:?}", duration);

        Ok(packs)
    }

    pub fn fetch_all_cards(
        &self,
        pack_ids: &[&str],
        report_progress: bool,
    ) -> Result<HashMap<String, Vec<Card>>> {
        pack_ids
            .par_iter()
            .map(|&pid| {
                info!("fetching all cards for pack {} via rayon", pid);
                let pack_id = pid.to_string();
                self.fetch_cards(&pack_id).map(|cards| {
                    if report_progress {
                        eprintln!("Fetched cards for pack {pid}")
                    }
                    (pack_id, cards)
                })
            })
            .collect()
    }

    fn parse_html(response: &str) -> Html {
        let start = Instant::now();
        let document = scraper::Html::parse_document(response);

        let duration = start.elapsed();
        info!("parsing HTML took: {:?}", duration);

        document
    }

    pub fn fetch_cards(&self, pack_id: &str) -> Result<Vec<Card>> {
        let url = self.cardlist_endpoint();
        info!("GET `{}`", url);

        let mut params = HashMap::new();
        params.insert("series", pack_id);

        let start = Instant::now();

        let response = self
            .client
            .get(self.cardlist_endpoint())
            .query(&params)
            .send()?
            .text()?;

        let duration = start.elapsed();
        info!("fetching HTML document took: {:?}", duration);

        let document = Self::parse_html(&response);

        let sel = "div.resultCol>a";
        info!("fetching cards for pack `{}` ({})...", pack_id, sel);

        let card_ids_selector = scraper::Selector::parse(sel).unwrap();

        let start = Instant::now();

        let mut cards = Vec::new();
        for element in document.select(&card_ids_selector) {
            let card_id = element
                .attr("data-src")
                .context("expected `data-src` attr on <a>")?
                .to_string();

            let card_id = &card_id[1..];

            match CardScraper::create_card(&self.localizer, &document, card_id, pack_id) {
                Ok(mut card) => {
                    debug!("computing img_full_url for card: {}", card);
                    card.img_full_url = Some(self.get_img_full_url(&card.img_url));
                    cards.push(card);
                }
                Err(e) => {
                    bail!("failed to scrape data about card `{}`: {}", &card_id, e)
                }
            };
        }

        let duration = start.elapsed();
        info!("processed cards for pack {} in {:?}", pack_id, duration);

        Ok(cards)
    }

    pub fn download_all_card_images(&self, cards: &[Card]) -> Result<HashMap<String, Vec<u8>>> {
        cards
            .par_iter()
            .map(|card| {
                let card_id = card.id.clone();
                debug!("fetching all images via rayon");
                self.download_card_image(card)
                    .map(|images| (card_id, images))
            })
            .collect()
    }

    pub fn download_card_image(&self, card: &Card) -> Result<Vec<u8>> {
        let full_url = self.get_img_full_url(&card.img_url);

        debug!("downloading image `{}`...", full_url);

        let mut retries = 3;
        loop {
            match self.client.get(full_url.as_str()).send() {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        bail!("HTTP {}: {}", status, full_url);
                    }

                    let img_data = response.bytes()?.to_vec();

                    debug!("downloaded {} bytes from {}", img_data.len(), full_url);
                    return Ok(img_data);
                }
                Err(e) => {
                    retries -= 1;
                    if retries == 0 {
                        bail!("failed after 3 retries: {}", e);
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
