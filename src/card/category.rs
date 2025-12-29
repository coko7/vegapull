use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::localizer::Localizer;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum CardCategory {
    Leader,
    Character,
    Event,
    Stage,
    Don,
}

impl CardCategory {
    pub fn parse(localizer: &Localizer, value: &str) -> Result<CardCategory> {
        match localizer.match_category(value.trim()) {
            Some(key) => Ok(Self::from_str(&key)?),
            None => bail!("Failed to match category `{}`", value),
        }
    }

    pub fn from_str(value: &str) -> Result<CardCategory> {
        match value.to_lowercase().as_str() {
            "leader" => Ok(Self::Leader),
            "character" => Ok(Self::Character),
            "event" => Ok(Self::Event),
            "stage" => Ok(Self::Stage),
            "don" => Ok(Self::Don),
            _ => bail!("Unsupported category `{}`", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_leader_returns_ok() {
        assert_eq!(
            CardCategory::from_str("leADer").unwrap(),
            CardCategory::Leader
        );
    }

    #[test]
    fn from_str_character_returns_ok() {
        assert_eq!(
            CardCategory::from_str("chARActer").unwrap(),
            CardCategory::Character
        );
    }

    #[test]
    fn from_str_event_returns_ok() {
        assert_eq!(
            CardCategory::from_str("eVENt").unwrap(),
            CardCategory::Event
        );
    }

    #[test]
    fn from_str_stage_returns_ok() {
        assert_eq!(
            CardCategory::from_str("STage").unwrap(),
            CardCategory::Stage
        );
    }

    #[test]
    fn from_str_don_returns_ok() {
        assert_eq!(CardCategory::from_str("DON").unwrap(), CardCategory::Don);
    }

    #[test]
    fn from_str_invalid_returns_err() {
        assert!(CardCategory::from_str("not a valid category").is_err());
    }
}
