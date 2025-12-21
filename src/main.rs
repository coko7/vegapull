use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::process::ExitCode;

use crate::cli::Cli;
use crate::config::initialize_configs;

mod card;
mod cli;
mod commands;
mod config;
mod localizer;
mod pack;
mod scraper;
mod storage;
mod utils;

fn main() -> ExitCode {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match process_args(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn process_args(args: Cli) -> Result<()> {
    info!("initialize config");
    initialize_configs()?;

    match args.command {
        cli::Commands::Pull {
            command,
            language,
            output_path,
            config_path,
            user_agent,
        } => match command {
            cli::PullSubCommands::All => {
                commands::pull_all(language, output_path, config_path, user_agent)
            }
            cli::PullSubCommands::Packs => {
                commands::pull_packs(language, output_path.as_deref(), user_agent)
            }
            cli::PullSubCommands::Cards { pack_id, mode } => commands::pull_cards(
                language,
                &pack_id.to_string_lossy(),
                output_path.as_deref(),
                mode,
                user_agent,
            ),
        },
        // cli::Commands::Diff { pack_files } => show_diffs(pack_files),
        cli::Commands::Config => commands::show_config(),
    }
}
