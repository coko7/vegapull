use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::localizer::Localizer;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum CardColor {
    Red,
    Green,
    Blue,
    Purple,
    Black,
    Yellow,
}

impl CardColor {
    pub fn parse(localizer: &Localizer, value: &str) -> Result<CardColor> {
        match localizer.match_color(value.trim()) {
            Some(key) => Ok(Self::from_str(&key)?),
            None => bail!("Failed to match color `{}`", value),
        }
    }

    pub fn from_str(value: &str) -> Result<CardColor> {
        match value.to_lowercase().as_str() {
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            "black" => Ok(Self::Black),
            "yellow" => Ok(Self::Yellow),
            _ => bail!("Unsupported color `{}`", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_red_returns_ok() {
        assert_eq!(CardColor::from_str("rEd").unwrap(), CardColor::Red);
    }

    #[test]
    fn from_str_green_returns_ok() {
        assert_eq!(CardColor::from_str("grEEN").unwrap(), CardColor::Green);
    }

    #[test]
    fn from_str_blue_returns_ok() {
        assert_eq!(CardColor::from_str("BluE").unwrap(), CardColor::Blue);
    }

    #[test]
    fn from_str_purple_returns_ok() {
        assert_eq!(CardColor::from_str("purPLE").unwrap(), CardColor::Purple);
    }

    #[test]
    fn from_str_black_returns_ok() {
        assert_eq!(CardColor::from_str("bLAck").unwrap(), CardColor::Black);
    }

    #[test]
    fn from_str_yellow_returns_ok() {
        assert_eq!(CardColor::from_str("YeLLoW").unwrap(), CardColor::Yellow);
    }

    #[test]
    fn from_str_invalid_returns_err() {
        assert!(CardColor::from_str("not a valid color").is_err());
    }
}
