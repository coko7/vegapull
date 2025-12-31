use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local};
use log::{debug, info, trace};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    card::Card,
    cli::LanguageCode,
    pack::{Pack, PackId},
};

const VEGA_META_FILE: &str = "vega.meta.toml";

pub struct DataStore {
    root_dir: PathBuf,
    language: LanguageCode,
}

#[derive(Debug, Serialize)]
pub enum PullMode {
    All,
    PackListOnly,
    SinglePack,
}

#[derive(Debug, Serialize)]
pub struct VegaMetaStats {
    language: LanguageCode,
    pull_start: DateTime<Local>,
    pull_duration_ms: usize,
    images_included: bool,
    mode: PullMode,
    packs: HashSet<PackId>,
}

impl VegaMetaStats {
    pub fn new(
        language: LanguageCode,
        pull_start: DateTime<Local>,
        pull_duration_ms: usize,
        images_included: bool,
        mode: PullMode,
        packs: HashSet<PackId>,
    ) -> Self {
        Self {
            language,
            pull_start,
            pull_duration_ms,
            images_included,
            mode,
            packs,
        }
    }
}

pub enum StoreLocation<'a> {
    RootDir,
    VegaMetaFile,
    PacksListFile,
    ImagesDir,
    JsonDir,
    CardsFile(&'a str),
    ImageFile(&'a Card),
}

impl DataStore {
    pub fn new(root_dir: &Path, language: LanguageCode) -> Self {
        Self {
            root_dir: root_dir.to_path_buf(),
            language,
        }
    }

    pub fn get_path(&self, location: StoreLocation) -> Result<PathBuf> {
        let path = match location {
            StoreLocation::RootDir => self.root_dir.clone(),
            StoreLocation::VegaMetaFile => {
                self.get_path(StoreLocation::RootDir)?.join(VEGA_META_FILE)
            }
            StoreLocation::ImagesDir => self.get_path(StoreLocation::RootDir)?.join("images/"),
            StoreLocation::JsonDir => self.get_path(StoreLocation::RootDir)?.join("json/"),
            StoreLocation::PacksListFile => {
                self.get_path(StoreLocation::JsonDir)?.join("packs.json")
            }
            StoreLocation::CardsFile(pack_id) => self.get_cards_filename(pack_id)?,
            StoreLocation::ImageFile(card) => {
                let filename = Self::get_img_filename(card)?;
                self.get_path(StoreLocation::ImagesDir)?.join(filename)
            }
        };

        Ok(path.to_path_buf())
    }

    fn get_cards_filename(&self, card_id: &str) -> Result<PathBuf> {
        let parent_dir = self.get_path(StoreLocation::JsonDir)?;
        let filename = format!("cards_{}.json", card_id);
        let path = parent_dir.join(filename);
        Ok(path)
    }

    pub fn get_img_filename(card: &Card) -> Result<String> {
        let last_slash_pos = card.img_url.rfind('/').context("expected to find `/`")?;

        let img_file_name = match card.img_url.find('?') {
            Some(quest_mark_pos) => &card.img_url[last_slash_pos + 1..quest_mark_pos],
            None => &card.img_url[last_slash_pos + 1..],
        };

        debug!("filename for `{}` is: {}", card.id, img_file_name);
        Ok(img_file_name.to_string())
    }

    fn ensure_created(&self, location: StoreLocation) -> Result<()> {
        let root_dir = self.get_path(location)?;
        if root_dir.exists() {
            debug!("data dir already exists at `{}`", root_dir.display());
            return Ok(());
        }

        match fs::create_dir_all(&root_dir) {
            Ok(_) => info!("successfully created `{}`", root_dir.display()),
            Err(e) => bail!("failed to create `{}`: {}", root_dir.display(), e),
        }

        Ok(())
    }

    pub fn write_packs(&self, packs: &HashMap<PackId, Pack>) -> Result<()> {
        self.ensure_created(StoreLocation::JsonDir)?;

        let path = self.get_path(StoreLocation::PacksListFile)?;
        debug!(
            "about to write {} packs to file: `{}`",
            packs.len(),
            path.display()
        );

        let json = serde_json::to_string(&packs)?;
        trace!("serialize data: `{:?} -> {}`", packs, json);

        fs::write(path, json)?;
        debug!("wrote packs data to file");

        Ok(())
    }

    pub fn write_cards(&self, pack_id: &str, cards: &Vec<Card>) -> Result<()> {
        self.ensure_created(StoreLocation::JsonDir)?;

        let path = self.get_path(StoreLocation::CardsFile(pack_id))?;
        debug!(
            "about to write {} cards from `{}` to file: `{}`",
            cards.len(),
            &pack_id,
            path.display()
        );

        let json = serde_json::to_string(&cards)?;
        trace!("serialize data: `{:?} -> {}`", cards, json);

        fs::write(path, json)?;
        debug!("wrote cards data to file");

        Ok(())
    }

    pub fn write_image_to_file(img_data: Vec<u8>, path: &PathBuf) -> Result<()> {
        debug!("about to save image to file: `{}`", path.display());

        let mut file = std::fs::File::create(path)?;
        file.write_all(&img_data)?;
        file.sync_all()?; // Ensure written to disk

        debug!("saved {} bytes to {}", img_data.len(), path.display());

        Ok(())
    }

    pub fn write_image(&self, card: &Card, img_data: Vec<u8>) -> Result<()> {
        self.ensure_created(StoreLocation::ImagesDir)?;

        let path = self.get_path(StoreLocation::ImageFile(card))?;
        Self::write_image_to_file(img_data, &path)?;
        Ok(())
    }

    pub fn write_vega_stats(&self, stats: VegaMetaStats) -> Result<()> {
        let path = self.get_path(StoreLocation::VegaMetaFile)?;
        let toml = toml::to_string_pretty(&stats)?;

        fs::write(&path, toml)?;
        debug!("wrote vega stats to: {} {:#?}", path.display(), stats);
        Ok(())
    }
}
