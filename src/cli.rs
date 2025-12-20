use anyhow::Result;
use clap::{command, Parser, Subcommand, ValueEnum};
use inquire_derive::Selectable;
use std::{
    ffi::OsString,
    fmt::{self},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Parser)]
#[command(
    name = "vega",
    about = "Scrape cards data from the official One Piece TCG website",
    long_about = None
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum CardDownloadMode {
    #[value(name = "image", alias = "img", alias = "i")]
    ImageOnly = 1,
    #[value(name = "data", alias = "d")]
    DataOnly = 2,
    #[value(name = "all", alias = "a")]
    All = 3,
}

#[derive(Debug, Subcommand)]
pub enum PullSubCommands {
    /// Download the complete dataset for a given language
    #[command(name = "all", alias = "records")]
    All,
    /// Download the list of existing packs
    #[command(name = "packs", alias = "pack")]
    Packs {
        /// Save output directly to <OUTPUT_FILE>
        #[arg(short, long = "out")]
        output_file: Option<PathBuf>,
    },
    /// Download all cards for a given pack
    #[command(name = "cards", alias = "card")]
    Cards {
        /// ID of the pack
        pack_id: OsString,

        /// Save downloaded data to <OUTPUT_PATH> instead of stdin
        #[arg(short, long = "output-path")]
        output_path: Option<PathBuf>,

        #[arg(short, long, value_enum, default_value_t = CardDownloadMode::DataOnly)]
        mode: CardDownloadMode,
    },
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download datasets from the official site
    #[command(name = "pull", alias = "p", alias = "fetch", alias = "punk")]
    Pull {
        #[command(subcommand)]
        command: PullSubCommands,

        /// Dataset to use
        #[arg(short, long, alias = "lang", value_name = "LANGUAGE", default_value_t = LanguageCode::English, value_enum)]
        language: LanguageCode,

        /// Path to the config directory (where locales are stored)
        #[arg(short = 'c', long = "config-dir")]
        config_directory_path: Option<PathBuf>,

        /// Send User-Agent <NAME> to server
        #[arg(short = 'A', long = "user-agent", value_name = "NAME")]
        user_agent: Option<String>,
    },
    /// Compare datasets
    #[command(name = "diff", alias = "df")]
    Diff {
        /// Output differences between two packs.json files
        #[arg(short, long = "packs", num_args = 2, value_names = ["FILE1", "FILE2"])]
        pack_files: Option<Vec<PathBuf>>,
    },
    /// Output current configuration
    #[command(name = "config", alias = "conf")]
    Config,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Selectable)]
pub enum LanguageCode {
    #[value(name = "english", alias = "en")]
    English,
    #[value(name = "japanese", alias = "jp")]
    Japanese,
    #[value(name = "french", alias = "fr")]
    French,
    #[value(name = "chinese-hongkong", alias = "zh_hk", alias = "zh_HK")]
    ChineseHongKong,
    #[value(name = "chinese-simplified", alias = "zh_cn", alias = "zh_CN")]
    ChineseSimplified,
    #[value(name = "chinese-taiwan", alias = "zh_tw", alias = "zh_TW")]
    ChineseTaiwan,
    #[value(name = "english-asia", alias = "en-asia")]
    EnglishAsia,
    #[value(name = "thai", alias = "th")]
    Thai,
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageCode::ChineseHongKong => write!(f, "chinese-hong-kong"),
            LanguageCode::ChineseSimplified => write!(f, "chinese-simplified"),
            LanguageCode::ChineseTaiwan => write!(f, "chinese-taiwan"),
            LanguageCode::English => write!(f, "english"),
            LanguageCode::EnglishAsia => write!(f, "english-asia"),
            LanguageCode::Japanese => write!(f, "japanese"),
            LanguageCode::Thai => write!(f, "thai"),
            LanguageCode::French => write!(f, "french"),
        }
    }
}

impl LanguageCode {
    pub fn to_path(self) -> PathBuf {
        let path = self.to_string();
        PathBuf::from(path)
    }
}

impl FromStr for LanguageCode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "chinese-hongkong" => Ok(LanguageCode::ChineseHongKong),
            "chinese-simplified" => Ok(LanguageCode::ChineseSimplified),
            "chinese-taiwan" => Ok(LanguageCode::ChineseTaiwan),
            "english" => Ok(LanguageCode::English),
            "english-asia" => Ok(LanguageCode::EnglishAsia),
            "japanese" => Ok(LanguageCode::Japanese),
            "thai" => Ok(LanguageCode::Thai),
            "french" => Ok(LanguageCode::French),
            _ => Err(()),
        }
    }
}
