use anyhow::{ensure, Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::{cli::LanguageCode, config};

pub const EN_LOCALE_RAW: &str = include_str!("../config/en.toml");
pub const EN_ASIA_LOCALE_RAW: &str = include_str!("../config/en_asia.toml");
pub const JP_LOCALE_RAW: &str = include_str!("../config/jp.toml");
pub const ZH_HK_LOCALE_RAW: &str = include_str!("../config/zh_hk.toml");
pub const ZH_TW_LOCALE_RAW: &str = include_str!("../config/zh_tw.toml");
pub const TH_LOCALE_RAW: &str = include_str!("../config/th.toml");
pub const FR_LOCALE_RAW: &str = include_str!("../config/fr.toml");

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Aliases {
    #[serde(default)]
    pub colors: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub attributes: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub rarities: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Localizer {
    pub hostname: String,

    pub colors: HashMap<String, String>,
    pub attributes: HashMap<String, String>,
    pub categories: HashMap<String, String>,
    pub rarities: HashMap<String, String>,

    // Optional alias lists to accept multiple labels per canonical key
    #[serde(default)]
    pub aliases: Aliases,
}

impl Localizer {
    fn match_with_alias(
        primary: &HashMap<String, String>,
        aliases: &HashMap<String, Vec<String>>,
        value: &str,
    ) -> Option<String> {
        let v = value.trim();
        // Exact match against primary map values
        if let Some((k, _)) = primary.iter().find(|(_, val)| val.as_str() == v) {
            return Some(k.clone());
        }

        // Case-insensitive match for Latin; exact match for others
        let v_lower = v.to_ascii_lowercase();
        for (k, list) in aliases {
            for a in list {
                if a == v || a.to_ascii_lowercase() == v_lower {
                    return Some(k.clone());
                }
            }
        }

        None
    }

    pub fn match_color(&self, value: &str) -> Option<String> {
        Self::match_with_alias(&self.colors, &self.aliases.colors, value)
    }

    pub fn match_attribute(&self, value: &str) -> Option<String> {
        Self::match_with_alias(&self.attributes, &self.aliases.attributes, value)
    }

    pub fn match_category(&self, value: &str) -> Option<String> {
        Self::match_with_alias(&self.categories, &self.aliases.categories, value)
    }

    pub fn match_rarity(&self, value: &str) -> Option<String> {
        Self::match_with_alias(&self.rarities, &self.aliases.rarities, value)
    }

    pub fn load(language: LanguageCode) -> Result<Localizer> {
        match language {
            LanguageCode::ChineseHongKong => Self::load_from_file("zh_hk"),
            LanguageCode::ChineseSimplified => Self::load_from_file("zh_cn"),
            LanguageCode::ChineseTaiwan => Self::load_from_file("zh_tw"),
            LanguageCode::English => Self::load_from_file("en"),
            LanguageCode::EnglishAsia => Self::load_from_file("en_asia"),
            LanguageCode::Japanese => Self::load_from_file("jp"),
            LanguageCode::Thai => Self::load_from_file("th"),
            LanguageCode::French => Self::load_from_file("fr"),
        }
    }

    pub fn load_from_file(locale: &str) -> Result<Localizer> {
        let config_dir = config::get_config_dir()?;

        ensure!(
            config_dir.exists(),
            format!("config directory not found: {}", config_dir.display())
        );

        let locale_path = config_dir.join(format!("{}.toml", locale));
        ensure!(
            locale_path.exists(),
            format!("locale file not found: {}", locale_path.display())
        );

        info!("load {} locale from: {}", locale, locale_path.display());

        let locale_data = fs::read_to_string(&locale_path)
            .with_context(|| format!("Failed to open file: {}", locale_path.display()))?;
        debug!("loaded {}", locale_data);

        let localizer: Localizer = toml::from_str(&locale_data)?;
        Ok(localizer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_maps() -> (HashMap<String, String>, HashMap<String, Vec<String>>) {
        let mut map = HashMap::new();
        map.insert(String::from("foo"), String::from("Toto"));
        map.insert(String::from("bar"), String::from("Tata"));
        map.insert(String::from("baz"), String::from("Tutu"));

        let mut alias_map: HashMap<String, Vec<String>> = HashMap::new();
        alias_map.insert(
            String::from("foo"),
            vec![String::from("toto"), String::from("TOto")],
        );
        alias_map.insert(String::from("bar"), vec![String::from("tata")]);
        alias_map.insert(String::from("baz"), vec![String::from("tutu")]);

        (map, alias_map)
    }

    #[test]
    fn reverse_search_returns_some() {
        let (map, alias_map) = get_test_maps();

        let actual = Localizer::match_with_alias(&map, &alias_map, "Toto");
        let expected = Some(String::from("foo"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reverse_search_returns_none() {
        let (map, alias_map) = get_test_maps();

        let actual = Localizer::match_with_alias(&map, &alias_map, "Titi");
        let expected = None;

        assert_eq!(actual, expected);
    }
}
