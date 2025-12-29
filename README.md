# 👒 vega (pull)

![build](https://github.com/coko7/vegapull/actions/workflows/rust.yml/badge.svg)

A CLI utility to retrieve data for the One Piece Trading Cards Game (TCG).

It goes directly goes against the [onepiece-cardgame.com](https://en.onepiece-cardgame.com) website and attempts to scrap information such as packs, cards and images.

> [!WARNING]
> Copyright disclaimer:
> - **Data** downloaded using this tool is copyrighted by ©Eiichiro Oda/Shueisha, Toei Animation, Bandai Namco Entertainment Inc.
> - **Source code** for this tool is available under the GNU Affero General Public License 3.0 or later. See [LICENSE](LICENSE) for more details.

> [!IMPORTANT]
> ✨ [v1.0.0](https://crates.io/crates/vegapull/1.0.0) is out now! The entire tool has been reworked to be user-friendly and ***blazingly fast 🚀***
>
> Changes:
> - rename from ~~vegapull~~ to **vega** as it fits better when combining with subcommands
> - rename subcommands and reorder them
> - build support for parallel downloads for json and images directly into the tool
> - rework and improve the interactive mode
> - add new options (like setting the user agent for downloading)
> - bug fixes

![demo](https://github.com/user-attachments/assets/c236f123-e519-40fd-b9bd-00e7ba50ef6b)

<!-- Old demo: ![demo](https://github.com/user-attachments/assets/6ac89611-08b5-4caa-ba83-a696929a2e37) -->

## Installation

The easiest way to install is through [crates.io](https://crates.io/crates/vegapull):
```sh
cargo install vegapull
```

The other option is to build from source:
```sh
git clone https://github.com/coko7/vegapull.git
cd vegapull
cargo build --release
```

## How to use?

To download all data from One Piece TCG, it's recommended to use the interactive mode:
```console
$ vega pull all
```

You can restrict the download further by using the other subcommands:
- `vega pull packs`: downloads the list of packs and stops
- `vega pull cards 569301`: download all cards in pack 569301 (json only)
- `vega pull cards 569302 --with-images`: download all cards in pack 569302 along with all images

See more commands with `vega help`

## Helper Scripts

If the out-of-the box **vega** command is not enough for your use case, then you can use helper scripts to further refine and automate the data download.

Previously, vega did not natively support parallel downloads and the interactive mode was ugly.
As a result, it was encouraged to make helper scripts that would call the vega cli and provide a friendier UX.

But since [v1.0.0](https://crates.io/crates/vegapull/1.0.0), vega now supports all of this natively so I would recommend against using helper scripts unless you really know what you are doing.

> [!WARNING]
> The helper scripts were made to work with older version and it's likely the 1.0.0 release has totally broken them.
> I did not have time/motivation to update them but feel free to do so if that is something you care about.
> They are in the [scripts](./scripts) sub-directory.

### Bash

```console
// ⚠️ broken in v1.0.0, fix it yourself or use an older version of vega
$ bash scripts/pull-all.sh
// the `gum` one is more complete but requires some additional tooling to install in your shell:
$ bash scripts/pull-all-gum.sh
```

### Go

> [!NOTE]
> Requires [Go](https://go.dev/) to be installed.

```console
// ⚠️ broken in v1.0.0, fix it yourself or use an older version of vega
$ go run scripts/pull.go
```

### Python

You can find a Python helper script on this repository: https://github.com/buhbbl/punk-records

## Where can I find prefetched datasets?

> [!WARNING]
> Keep in mind that data downloaded by **vega** is copyrighted data (see copyright notice at the top of this file).

There are currently two Git repositories with JSON data:
- [buhbbl/punk-records](https://github.com/buhbbl/punk-records) (all languages)
- [Coko7/vegapull-records](https://github.com/Coko7/vegapull-records) (english/japanese only)
