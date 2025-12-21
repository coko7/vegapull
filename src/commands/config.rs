use anyhow::{ensure, Result};
use std::fs;

use crate::config;

pub fn show_config() -> Result<()> {
    let config_dir = config::get_config_dir()?;
    ensure!(
        config_dir.exists(),
        format!("config directory not found: {}", config_dir.display())
    );

    let entries = fs::read_dir(&config_dir)?;
    println!("config directory: {}", config_dir.display());

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        println!("- {}", file_name.to_string_lossy());
    }

    Ok(())
}
