use std::fmt;

use serde::{Deserialize, Serialize};

use super::{CardAttribute, CardCategory, CardColor, CardRarity};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Card {
    pub id: String,
    pub pack_id: String,
    pub name: String,
    pub rarity: CardRarity,
    pub category: CardCategory,
    // pub number: i32,
    // #[serde(skip_serializing)]
    // pub copyright: String,

    // Images
    pub img_url: String,
    pub img_full_url: Option<String>,
    // pub illustration: CardIllustration,
    // pub illustrator_name: String,

    // Gameplay
    pub cost: Option<i32>, // Only Character, Event, and Stage (called life for Leader)
    pub attributes: Vec<CardAttribute>, // Only Leader and Character
    pub power: Option<i32>, // Only Leader and Character
    pub counter: Option<i32>, // Only Character
    pub colors: Vec<CardColor>,
    pub block_number: i32,

    pub types: Vec<String>,
    pub effect: String,
    pub trigger: Option<String>,
    // pub notes: String,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. `{}`", self.id, self.name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CardIllustration {
    Comic,
    Animation,
    Original,
    Other,
}
