# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0](https://github.com/coko7/vegapull/compare/v1.2.3...v1.3.0) - 2026-08-21

### Added

- add color aliases and enhance attribute handling in localizer. ([#21](https://github.com/coko7/vegapull/pull/21))

### Other

- *(deps)* bump openssl from 0.10.79 to 0.10.81 ([#23](https://github.com/coko7/vegapull/pull/23))
- *(deps)* bump crossbeam-epoch and h2

## [1.2.3](https://github.com/coko7/vegapull/compare/v1.2.2...v1.2.3) - 2026-07-22

### Fixed

- *(strip_html_tags)* flatten html instead of removing tags and their content

## [1.2.2](https://github.com/coko7/vegapull/compare/v1.2.1...v1.2.2) - 2026-05-07

### Dependencies

- *(deps)* bump openssl from 0.10.73 to 0.10.79
- *(deps)* bump bytes from 1.10.1 to 1.11.1
- *(deps)* bump rustls-webpki from 0.103.7 to 0.103.13
- *(deps)* bump rand from 0.8.5 to 0.8.6

## [1.2.1](https://github.com/coko7/vegapull/compare/v1.2.0...v1.2.1) - 2026-03-24

### Fixed

- handle empty block_number gracefully + skip failed cards (#10)

## [1.2.0](https://github.com/coko7/vegapull/compare/v1.1.0...v1.2.0) - 2026-01-20

### Added

- add support for block_number card field

## [1.1.0](https://github.com/coko7/vegapull/compare/v1.0.0...v1.1.0) - 2025-12-31

### Added

- add vega.meta.toml file + remove locale subdir from downloaded data

## [1.0.0](https://github.com/coko7/vegapull/releases/tag/v1.0.0) - 2025-12-29

Initial tagged release, including the v1 rework of the CLI (interactive mode, subcommands,
parallel downloads) built up over the project's pre-1.0 history.

### Added

- scrape list of booster packs
- fetch card colors, life/cost, attributes, power and counter
- add img dl + logs + refactor json file
- *(localizer)* support other locales via per-language toml files (colors, attributes, categories, rarities)
- add en + jp locales
- add a CLI around the wrapper (#1)
- add bash script to quickly download all data
- add support for asia-en lang + fix pull_all script
- add interactive mode to download all data (including images)
- batch download images with subcommand
- add new cmd `diff` to find diffs between two packs.json
- add native support for fast parallel image downloads
- handle user config dir and auto create locale toml files
- support scraping for japanese, chinese, thai & french (#7)

### Changed

- *(breaking)* move locales to config dir + add config command + new `-c` argument to specify config path
- V1 rework (#8)

### Fixed

- fix update code for fetching images
- *(cli)* add missing languages in from_str
- *(pack)* properly compute title parts (prefix, title, label) + change data format for pack
- fix color fetching bug with some cards + minor code improvements
- trim spaces when parsing color (#3)
- replace xdg crate with directories to fix windows install
- fix various warnings and improve code using clippy
- fix storage.get_path so that rust-version 1.75.0 is supported
- fix scraping of other languages (japanese, chinese, thai & french) (#7)
