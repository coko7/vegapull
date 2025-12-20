use anyhow::{bail, Context, Result};
use log::trace;
use regex::Regex;
use scraper::{ElementRef, Html};
use unicode_normalization::UnicodeNormalization;

use crate::{
    card::{Card, CardAttribute, CardCategory, CardColor, CardRarity},
    localizer::Localizer,
};

fn normalize_ascii(s: &str) -> String {
    // NFKC converts full-width digits/letters to ASCII equivalents
    s.nfkc().collect::<String>()
}

pub struct CardScraper {}

impl CardScraper {
    pub fn create_card(
        localizer: &Localizer,
        document: &Html,
        card_id: &str,
        pack_id: &str,
    ) -> Result<Card> {
        trace!("start create card: `{}`", card_id);
        let dl_elem = Self::get_dl_node(document, card_id.to_string())?;

        let id = Self::fetch_id(dl_elem)?;
        let pack_id = pack_id.to_string();
        let name = Self::fetch_name(dl_elem)?;
        let rarity = Self::fetch_rarity(localizer, dl_elem)?;
        let category = Self::fetch_category(localizer, dl_elem)?;
        let img_url = Self::fetch_img_url(dl_elem)?;
        let img_full_url = None;

        let colors = Self::fetch_colors(localizer, dl_elem)?;
        let cost = Self::fetch_cost(dl_elem)?;
        let attributes = Self::fetch_attributes(localizer, dl_elem)?;
        let power = Self::fetch_power(dl_elem)?;
        let counter = Self::fetch_counter(dl_elem)?;
        let types = Self::fetch_types(dl_elem)?;
        let effect = Self::fetch_effect(dl_elem)?;
        let trigger = Self::fetch_trigger(dl_elem)?;

        let card = Card {
            id,
            pack_id,
            name,
            rarity,
            category,
            img_url,
            img_full_url,
            colors,
            cost,
            attributes,
            power,
            counter,
            types,
            effect,
            trigger,
        };

        trace!("processed card: `{}`", card);
        Ok(card)
    }

    // element is top level <dl> tag
    pub fn fetch_id(element: ElementRef) -> Result<String> {
        trace!("fetching card.id...");
        let id = element
            .attr("id")
            .context("expected to find id attr on <dl>")?
            .to_string();

        trace!("fetched card.id: {}", id);
        Ok(id)
    }

    pub fn fetch_name(element: ElementRef) -> Result<String> {
        let sel = "dt>div.cardName";
        trace!("fetching card.name ({})...", sel);

        let name = Self::get_child_node(element, sel.to_string())?.inner_html();

        trace!("fetched card.name: {}", name);
        Ok(name)
    }

    pub fn fetch_rarity(localizer: &Localizer, element: ElementRef) -> Result<CardRarity> {
        let sel = "dt>div.infoCol>span:nth-child(2)";
        trace!("fetching card.rarity ({})...", sel);

        let raw_rarity = Self::get_child_node(element, sel.to_string())?.inner_html();

        trace!("fetched card.rarity: {}", raw_rarity);
        let rarity = CardRarity::parse(localizer, &raw_rarity)?;

        trace!("processed card.rarity");
        Ok(rarity)
    }

    pub fn fetch_category(localizer: &Localizer, element: ElementRef) -> Result<CardCategory> {
        let sel = "dt>div.infoCol>span:nth-child(3)";
        trace!("fetching card.category ({})...", sel);

        let raw_category = Self::get_child_node(element, sel.to_string())?.inner_html();

        trace!("fetched card.category: {}", raw_category);
        let category = CardCategory::parse(localizer, &raw_category)?;

        trace!("processed card.category");
        Ok(category)
    }

    pub fn fetch_img_url(element: ElementRef) -> Result<String> {
        let sel = "dd>div.frontCol>img";
        trace!("fetching card.img_url ({})...", sel);

        let img_elem = Self::get_child_node(element, sel.to_string())?;
        let img_url = img_elem
            .attr("data-src")
            .context("no data-src attr")?
            .to_string();

        trace!("fetched card.img_url: {}", img_url);
        Ok(img_url)
    }

    pub fn fetch_colors(localizer: &Localizer, element: ElementRef) -> Result<Vec<CardColor>> {
        let sel = "dd>div.backCol div.color";
        trace!("fetching card.colors ({})...", sel);

        let raw_colors = Self::get_child_node(element, sel.to_string())?.inner_html();
        let raw_colors = Self::strip_html_tags(&raw_colors)?;
        trace!("fetched card.colors: {}", raw_colors);

        let raw_colors: Vec<&str> = raw_colors.split('/').collect();

        let mut colors = Vec::new();

        if raw_colors.len() == 1 && localizer.match_color(raw_colors[0]).is_none() {
            trace!("raw color is unseparatble, trying to parse character by character");
            let mut any = false;
            for ch in raw_colors[0].chars() {
                let char_color = ch.to_string();
                if let Some(raw_color) = localizer.match_color(&char_color) {
                    trace!("found color char: {} -> {}", char_color, raw_color);
                    let color = CardColor::from_str(&raw_color)?;
                    colors.push(color);
                    any = true;
                }
            }
            if any {
                colors.dedup();
                trace!("processed card.colors by character");
                return Ok(colors);
            }
        }

        for (index, raw_color) in raw_colors.iter().enumerate() {
            trace!("processing card.colors[{}]: {}", index, raw_color);
            let color = CardColor::parse(localizer, raw_color)?;
            colors.push(color);
        }

        trace!("processed card.colors");
        Ok(colors)
    }

    pub fn fetch_cost(element: ElementRef) -> Result<Option<i32>> {
        let sel = "dd>div.backCol>div.col2>div.cost";
        trace!("fetching card.cost ({})...", sel);

        let raw_cost = Self::get_child_node(element, sel.to_string())?.inner_html();
        let raw_cost = Self::strip_html_tags(&raw_cost)?;
        let raw_cost = normalize_ascii(&raw_cost)
            .replace([',', ' '], "")
            .trim()
            .to_string();
        trace!("fetched card.cost: {}", raw_cost);

        if raw_cost == "-" {
            trace!("card.cost unset");
            return Ok(None);
        }

        // Sanity check
        let digits: String = raw_cost.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            trace!("card.cost has no digits");
            return Ok(None);
        }

        match digits.parse::<i32>() {
            Ok(val) => {
                trace!("processed card.cost");
                Ok(Some(val))
            }
            Err(e) => bail!("failed to parse card.cost value `{}`: {}", raw_cost, e),
        }
    }

    pub fn fetch_attributes(
        localizer: &Localizer,
        element: ElementRef,
    ) -> Result<Vec<CardAttribute>> {
        let sel = "dd>div.backCol>div.col2>div.attribute>img";
        trace!("fetching card.attributes ({})...", sel);

        if let Ok(attr_img) = Self::get_child_node(element, sel.to_string()) {
            if let Some(url) = attr_img.attr("src") {
                if let Ok(attributes) = CardAttribute::from_icon_url(url) {
                    trace!("processed card.attributes");
                    return Ok(attributes);
                }
                trace!("failed to parse card.attributes from icon url {}", url);
            }
            trace!("Falling back to processing alt attribute");

            let raw_attributes = attr_img.attr("alt").context("no alt attr")?.to_string();
            trace!("fetched card.attributes: {}", raw_attributes);

            if raw_attributes.is_empty() {
                trace!("card.attributes empty");
                return Ok(Vec::new());
            }

            let raw_attributes: Vec<&str> = raw_attributes.split('/').collect();

            let mut attributes = Vec::new();
            for (index, raw_attribute) in raw_attributes.iter().enumerate() {
                trace!("processing card.attributes[{}]: {}", index, raw_attribute);
                let attribute = CardAttribute::parse(localizer, raw_attribute)?;
                attributes.push(attribute);
            }

            trace!("processed card.attributes");
            return Ok(attributes);
        }

        trace!("card.attributes no img found");
        Ok(Vec::new())
    }

    pub fn fetch_power(element: ElementRef) -> Result<Option<i32>> {
        let sel = "dd>div.backCol>div.col2>div.power";
        trace!("fetching card.power ({})...", sel);

        let raw_power = Self::get_child_node(element, sel.to_string())?.inner_html();
        let raw_power = Self::strip_html_tags(&raw_power)?;
        let raw_power = normalize_ascii(&raw_power)
            .replace([',', ' '], "")
            .trim()
            .to_string();
        trace!("fetched card.power: {}", raw_power);

        if raw_power == "-" || raw_power.is_empty() {
            trace!("card.power unset");
            return Ok(None);
        }

        // Sanity check
        let digits: String = raw_power.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            trace!("card.power has no digits");
            return Ok(None);
        }

        match digits.parse::<i32>() {
            Ok(val) => {
                trace!("processed card.power");
                Ok(Some(val))
            }
            Err(e) => bail!("failed to parse card.power value `{}`: {}", raw_power, e),
        }
    }

    pub fn fetch_counter(element: ElementRef) -> Result<Option<i32>> {
        let sel = "dd>div.backCol>div.col2>div.counter";
        trace!("fetching card.counter ({})...", sel);

        let raw_counter = Self::get_child_node(element, sel.to_string())?.inner_html();
        let raw_counter = Self::strip_html_tags(&raw_counter)?;
        let raw_counter = normalize_ascii(&raw_counter)
            .replace([',', ' '], "")
            .trim()
            .to_string();
        trace!("fetched card.counter: {}", raw_counter);

        if raw_counter == "-" || raw_counter.is_empty() {
            trace!("card.counter unset");
            return Ok(None);
        }

        // Extract only digits from the string (some french cards have extra text, e.g., カウンター1000 in EB01-018_p1)
        let digits: String = raw_counter.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            trace!("card.counter has no digits");
            return Ok(None);
        }

        match digits.parse::<i32>() {
            Ok(val) => {
                trace!("processed card.counter");
                Ok(Some(val))
            }
            Err(e) => bail!(
                "failed to parse card.counter value `{}`: {}",
                raw_counter,
                e
            ),
        }
    }

    pub fn fetch_types(element: ElementRef) -> Result<Vec<String>> {
        let sel = "dd>div.backCol>div.feature";
        trace!("fetching card.types ({})...", sel);

        let types = Self::get_child_node(element, sel.to_string())?.inner_html();
        let types = Self::strip_html_tags(&types)?;
        trace!("fetched card.types: {}", types);

        let types: Vec<String> = types.split('/').map(str::to_owned).collect();

        trace!("processed card.types");
        Ok(types)
    }

    pub fn fetch_effect(element: ElementRef) -> Result<String> {
        let sel = "dd>div.backCol>div.text";
        trace!("fetching card.effect ({})...", sel);

        let effect = Self::get_child_node(element, sel.to_string())?.inner_html();
        let effect = Self::strip_html_tags(&effect)?;
        trace!("fetched card.effect: {}", effect);

        Ok(effect)
    }

    pub fn fetch_trigger(element: ElementRef) -> Result<Option<String>> {
        let sel = "dd>div.backCol>div.trigger";
        trace!("fetching card.trigger ({})...", sel);

        if let Ok(trigger_div) = Self::get_child_node(element, sel.to_string()) {
            let trigger = trigger_div.inner_html();
            let trigger = Self::strip_html_tags(&trigger)?;
            trace!("fetched card.trigger: {}", trigger);

            return Ok(Some(trigger));
        }

        trace!("card.trigger no html found");
        Ok(None)
    }

    fn strip_html_tags(value: &str) -> Result<String> {
        let reg = Regex::new(r"<[^>]*>.*?</[^>]*>")?;
        let result = reg.replace_all(value, "").trim().to_string();
        Ok(result)
    }

    fn get_child_node(element: ElementRef, selector: String) -> Result<ElementRef> {
        let node_sel = scraper::Selector::parse(&selector).unwrap();
        let results: Vec<_> = element.select(&node_sel).collect();

        match results.len() {
            0 => bail!("Expected `{}` but got nothing", selector),
            1 => Ok(*results.first().unwrap()),
            _ => bail!("Expected single `{}` but got many", selector),
        }
    }

    pub fn get_dl_node(document: &Html, card_id: String) -> Result<ElementRef<'_>> {
        let dl_sel = format!("dl#{}", card_id);
        let dl_sel = scraper::Selector::parse(&dl_sel).unwrap();
        let dl_elem = document.select(&dl_sel).next().unwrap();
        Ok(dl_elem)
    }
}
