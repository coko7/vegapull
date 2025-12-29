use anyhow::{bail, ensure, Context, Result};
use log::debug;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::pack::Pack;

pub fn show_diffs(pack_files: Option<Vec<PathBuf>>) -> Result<()> {
    if let Some(pack_files) = pack_files {
        ensure!(pack_files.len() == 2, "exactly two packs must be provided");

        let old_packs_path = pack_files.first().context("there should be a first")?;
        let new_packs_path = pack_files.last().context("there should be a last")?;

        ensure!(Path::exists(old_packs_path), "old_packs file not found");
        ensure!(Path::exists(new_packs_path), "new_packs file not found");

        let old_packs = fs::read_to_string(old_packs_path)?;
        let old_packs: Vec<Pack> = serde_json::from_str(&old_packs)?;
        let old_packs: HashSet<_> = old_packs.iter().collect();
        debug!(
            "successfully loaded {} packs from: `{}`",
            old_packs.len(),
            old_packs_path.display()
        );

        let new_packs = fs::read_to_string(new_packs_path)?;
        let new_packs: Vec<Pack> = serde_json::from_str(&new_packs)?;
        let new_packs: HashSet<_> = new_packs.iter().collect();
        debug!(
            "successfully loaded {} packs from: `{}`",
            new_packs.len(),
            new_packs_path.display()
        );

        let diff_packs: Vec<_> = old_packs.symmetric_difference(&new_packs).collect();
        debug!(
            "found {} diff(s) between both sets: {:#?}",
            diff_packs.len(),
            diff_packs
        );

        let diff_json = serde_json::to_string(&diff_packs)?;
        println!("{}", diff_json);
        return Ok(());
    }

    bail!("missing arguments")
}
