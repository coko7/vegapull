use anyhow::Result;
use regex::Regex;
use scraper::ElementRef;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};

pub type PackId = String;

#[derive(Debug, Deserialize, Serialize)]
pub struct Pack {
    pub id: String,
    pub raw_title: String,
    pub title_parts: TitleParts,
}

impl Eq for Pack {}

impl PartialEq for Pack {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleParts {
    prefix: Option<String>,
    title: String,
    label: Option<String>,
}

impl Pack {
    pub fn new(element: ElementRef) -> Result<Self> {
        let raw_title = Self::flatten_title(&element.inner_html())?;
        let title_parts = Self::process_title_parts(&raw_title)?;

        Ok(Self {
            id: element.attr("value").unwrap().to_string(),
            raw_title,
            title_parts,
        })
    }

    fn process_title_parts(raw_title: &str) -> Result<TitleParts> {
        let mut processed_title: String = raw_title.to_string();

        let label = Self::get_label_from_title(raw_title)?;
        if let Some(ref label) = label {
            processed_title = Self::remove_label_from_title(&processed_title, label)?
                .trim()
                .to_string();
        }

        let prefix = Self::get_prefix_from_title(raw_title)?;
        if let Some(ref prefix) = prefix {
            processed_title = processed_title.replace(prefix, "");
        }

        let prefix = prefix.map(|val| val.trim().to_string());

        if processed_title.starts_with("-") {
            processed_title = processed_title[1..].to_string();
        }

        if processed_title.ends_with("-") {
            processed_title = processed_title[..processed_title.len() - 1].to_string();
        }

        let result = TitleParts {
            prefix,
            title: processed_title.to_string(),
            label,
        };

        Ok(result)
    }

    fn remove_label_from_title(title: &str, label: &str) -> Result<String> {
        let full_label = format!("[{}]", label);
        Ok(title.replace(&full_label, ""))
    }

    fn get_label_from_title(title: &str) -> Result<Option<String>> {
        let reg = Regex::new(r"\[(.*?)\]")?;
        if let Some(captured) = reg.captures(title) {
            if let Some(label) = captured.get(1) {
                return Ok(Some(label.as_str().to_string()));
            }
        }

        Ok(None)
    }

    fn get_prefix_from_title(title: &str) -> Result<Option<String>> {
        let reg = Regex::new(r"^(.*?)-.*?-")?;
        if let Some(captured) = reg.captures(title) {
            if let Some(prefix) = captured.get(1) {
                if prefix.is_empty() {
                    return Ok(None);
                }

                return Ok(Some(prefix.as_str().to_string()));
            }
        }

        Ok(None)
    }

    fn flatten_title(inner_html: &str) -> Result<String> {
        let reg = Regex::new(r"&lt;.*&gt;")?;
        let result = reg.replace_all(inner_html, "").to_string();
        Ok(result)
    }
}

impl fmt::Display for Pack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.id,
            self.title_parts.title,
            self.title_parts.label.as_ref().unwrap_or(&"".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_title_parts_returns_all_parts() {
        let raw_title = "PREFIX -TITLE- [LABEL]";
        let title_parts = Pack::process_title_parts(raw_title).unwrap();

        let exp_prefix = Some("PREFIX".to_string());
        let exp_title = "TITLE".to_string();
        let exp_label = Some("LABEL".to_string());

        assert_eq!(title_parts.prefix, exp_prefix);
        assert_eq!(title_parts.title, exp_title);
        assert_eq!(title_parts.label, exp_label);
    }

    #[test]
    fn process_title_parts_returns_without_prefix() {
        let raw_title = "-TITLE- [LABEL]";
        let title_parts = Pack::process_title_parts(raw_title).unwrap();

        let exp_prefix = None;
        let exp_title = "TITLE".to_string();
        let exp_label = Some("LABEL".to_string());

        assert_eq!(title_parts.prefix, exp_prefix);
        assert_eq!(title_parts.title, exp_title);
        assert_eq!(title_parts.label, exp_label);
    }

    #[test]
    fn process_title_parts_returns_without_label() {
        let raw_title = "PREFIX -TITLE-";
        let title_parts = Pack::process_title_parts(raw_title).unwrap();

        let exp_prefix = Some("PREFIX".to_string());
        let exp_title = "TITLE".to_string();
        let exp_label = None;

        assert_eq!(title_parts.prefix, exp_prefix);
        assert_eq!(title_parts.title, exp_title);
        assert_eq!(title_parts.label, exp_label);
    }

    #[test]
    fn process_title_parts_returns_only_title() {
        let raw_title = "-TITLE-";
        let title_parts = Pack::process_title_parts(raw_title).unwrap();

        let exp_prefix = None;
        let exp_title = "TITLE".to_string();
        let exp_label = None;

        assert_eq!(title_parts.prefix, exp_prefix);
        assert_eq!(title_parts.title, exp_title);
        assert_eq!(title_parts.label, exp_label);
    }

    #[test]
    fn get_label_from_title_returns_none() {
        let title = "Linux is dope";
        let label = None;

        let result = Pack::get_label_from_title(title);
        assert_eq!(result.unwrap(), label);
    }

    #[test]
    fn get_label_from_title_returns_some() {
        let title = "Linux is dope [Facts]";
        let label = Some("Facts".to_string());

        let result = Pack::get_label_from_title(title);
        assert_eq!(result.unwrap(), label);
    }

    #[test]
    fn get_label_from_title_multiple_matches_returns_first() {
        let title = "Linux is dope [Facts] and I love artichokes [Hmm...]";
        let label = Some("Facts".to_string());

        let result = Pack::get_label_from_title(title);
        assert_eq!(result.unwrap(), label);
    }

    #[test]
    fn flatten_title_removes_html_tags() {
        let original = "TITLE&lt;br class=\"test\"&gt; - gum is yummy - [1]";
        let flattened = "TITLE - gum is yummy - [1]";

        let result = Pack::flatten_title(original);
        assert_eq!(result.unwrap(), flattened);
    }

    #[test]
    fn flatten_title_stays_same() {
        let title = "TITLE - fzf is awesome - [2]";

        let result = Pack::flatten_title(title);
        assert_eq!(result.unwrap(), title);
    }
}
