use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::localizer::Localizer;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum CardRarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    SuperRare = 3,
    SecretRare = 4,
    Leader = 5,
    Special = 6,
    TreasureRare = 7,
    Promo = 8,
}

impl CardRarity {
    pub fn parse(localizer: &Localizer, value: &str) -> Result<CardRarity> {
        match localizer.match_rarity(value) {
            Some(key) => Ok(Self::from_str(&key)?),
            None => bail!("Failed to match rarity `{}`", value),
        }
    }

    pub fn from_str(value: &str) -> Result<CardRarity> {
        match value.to_lowercase().as_str() {
            "common" => Ok(Self::Common),
            "uncommon" => Ok(Self::Uncommon),
            "rare" => Ok(Self::Rare),
            "super_rare" => Ok(Self::SuperRare),
            "secret_rare" => Ok(Self::SecretRare),
            "leader" => Ok(Self::Leader),
            "special" => Ok(Self::Special),
            "treasure_rare" => Ok(Self::TreasureRare), // Supposedly added in OP07
            "promo" => Ok(Self::Promo),                // Promo cards (Ultra rare)
            _ => bail!("Unsupported rarity `{}`", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_common_returns_ok() {
        assert_eq!(CardRarity::from_str("coMMOn").unwrap(), CardRarity::Common);
    }

    #[test]
    fn from_str_uncommon_returns_ok() {
        assert_eq!(
            CardRarity::from_str("UNcomMon").unwrap(),
            CardRarity::Uncommon
        );
    }

    #[test]
    fn from_str_rare_returns_ok() {
        assert_eq!(CardRarity::from_str("RARE").unwrap(), CardRarity::Rare);
    }

    #[test]
    fn from_str_super_rare_returns_ok() {
        assert_eq!(
            CardRarity::from_str("suPER_RAre").unwrap(),
            CardRarity::SuperRare
        );
    }

    #[test]
    fn from_str_secret_rare_returns_ok() {
        assert_eq!(
            CardRarity::from_str("secRET_RarE").unwrap(),
            CardRarity::SecretRare
        );
    }

    #[test]
    fn from_str_leader_returns_ok() {
        assert_eq!(CardRarity::from_str("leADEr").unwrap(), CardRarity::Leader);
    }

    #[test]
    fn from_str_special_returns_ok() {
        assert_eq!(
            CardRarity::from_str("spEcIaL").unwrap(),
            CardRarity::Special
        );
    }

    #[test]
    fn from_str_treasure_rare_returns_ok() {
        assert_eq!(
            CardRarity::from_str("trEasUre_Rare").unwrap(),
            CardRarity::TreasureRare
        )
    }

    #[test]
    fn from_str_promo_returns_ok() {
        assert_eq!(CardRarity::from_str("pROmO").unwrap(), CardRarity::Promo)
    }

    #[test]
    fn from_str_invalid_returns_err() {
        assert!(CardRarity::from_str("not a valid rarity").is_err())
    }
}
