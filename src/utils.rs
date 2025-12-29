use anyhow::Result;
use chrono::Local;
use std::{env::current_dir, path::PathBuf};

use crate::cli::LanguageCode;

pub fn get_default_data_dirname(language: LanguageCode) -> String {
    let timestamp = Local::now().format("%y%m%d_%H%M").to_string();
    format!("data-{timestamp}-{language}")
}

pub fn get_default_data_dir(language: LanguageCode) -> Result<PathBuf> {
    let dir_name = get_default_data_dirname(language);
    Ok(current_dir()?.join(dir_name))
}
