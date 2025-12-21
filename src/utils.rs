use anyhow::Result;
use chrono::Local;
use std::{env::current_dir, path::PathBuf};

pub fn get_default_data_dirname() -> String {
    let timestamp = Local::now().format("%y%m%d_%H%M%S").to_string();
    format!("data-{}", timestamp)
}

pub fn get_default_data_dir() -> Result<PathBuf> {
    let dir_name = get_default_data_dirname();
    Ok(current_dir()?.join(dir_name))
}
