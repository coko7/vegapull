use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::localizer::Localizer;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum CardAttribute {
    Slash,
    Strike,
    Ranged,
    Special,
    Wisdom,
    Unknown, // For Imu Leader
}

impl CardAttribute {
    pub fn parse(localizer: &Localizer, value: &str) -> Result<CardAttribute> {
        match localizer.match_attribute(value.trim()) {
            Some(key) => Ok(Self::from_str(&key)?),
            None => bail!("Failed to match attribute `{}`", value),
        }
    }

    pub fn from_str(value: &str) -> Result<CardAttribute> {
        match value.to_lowercase().as_str() {
            "slash" => Ok(Self::Slash),
            "strike" => Ok(Self::Strike),
            "ranged" => Ok(Self::Ranged),
            "special" => Ok(Self::Special),
            "wisdom" => Ok(Self::Wisdom),
            "unknown" => Ok(Self::Unknown),
            _ => bail!("Unsupported attribute `{}`", value),
        }
    }

    pub fn from_icon_url(url: &str) -> Result<Vec<CardAttribute>> {
        let file = url
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("Invalid URL: {}", url))?;

        let stem = file
            .strip_prefix("ico_type")
            .ok_or_else(|| anyhow!("attribute icon: missing ico_type prefix in: {}", file))?;

        let value = stem
            .split('.')
            .next()
            .ok_or_else(|| anyhow!("attribute icon: missing file extension in: {}", file))?;

        match value {
            "01" => Ok(vec![Self::Strike]),
            "02" => Ok(vec![Self::Slash]),
            "03" => Ok(vec![Self::Special]),
            "04" => Ok(vec![Self::Ranged]),
            "05" => Ok(vec![Self::Wisdom]),
            "06" => Ok(vec![Self::Slash, Self::Strike]),
            "07" => Ok(vec![Self::Slash, Self::Special]),
            "08" => Ok(vec![Self::Strike, Self::Ranged]),
            "09" => Ok(vec![Self::Strike, Self::Special]),
            "10" => Ok(vec![Self::Strike, Self::Wisdom]),
            "11" => Ok(vec![Self::Slash, Self::Wisdom]),
            "12" => Ok(vec![Self::Unknown]), // Unknown for Imu
            _ => bail!("Unsupported attribute `{}`", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_slash_returns_ok() {
        assert_eq!(
            CardAttribute::from_str("SLash").unwrap(),
            CardAttribute::Slash
        );
    }

    #[test]
    fn from_str_strike_returns_ok() {
        assert_eq!(
            CardAttribute::from_str("strIKE").unwrap(),
            CardAttribute::Strike
        );
    }

    #[test]
    fn from_str_ranged_returns_ok() {
        assert_eq!(
            CardAttribute::from_str("rANged").unwrap(),
            CardAttribute::Ranged
        );
    }

    #[test]
    fn from_str_special_returns_ok() {
        assert_eq!(
            CardAttribute::from_str("spEciAl").unwrap(),
            CardAttribute::Special
        );
    }

    #[test]
    fn from_str_wisdom_returns_ok() {
        assert_eq!(
            CardAttribute::from_str("wiSDom").unwrap(),
            CardAttribute::Wisdom
        );
    }

    #[test]
    fn from_str_invalid_returns_err() {
        assert!(CardAttribute::from_str("not a valid value").is_err());
    }

    #[test]
    fn from_icon_url_single() {
        assert_eq!(
            CardAttribute::from_icon_url("/images/cardlist/attribute/ico_type02.png").unwrap(),
            vec![CardAttribute::Slash]
        );
    }

    #[test]
    fn from_icon_url_combo() {
        assert_eq!(
            CardAttribute::from_icon_url("/images/cardlist/attribute/ico_type07.png").unwrap(),
            vec![CardAttribute::Slash, CardAttribute::Special]
        );
    }

    #[test]
    fn from_icon_url_unknown() {
        assert_eq!(
            CardAttribute::from_icon_url("/images/cardlist/attribute/ico_type12.png").unwrap(),
            vec![CardAttribute::Unknown]
        );
    }
}
