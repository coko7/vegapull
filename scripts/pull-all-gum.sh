#!/usr/bin/env bash

VEGA_BIN=vegapull
LANGUAGES="🇬🇧 english\n🇯🇵 japanese"

language=$(echo -e "$LANGUAGES" | gum choose --header="Language:" | cut -d' ' -f2)
if [ -z "$language" ]; then
    exit 1
fi

default_data_dir="data/$language"
data_dir=$(gum input \
    --value="$default_data_dir" \
    --placeholder="Enter a path..." \
    --prompt="📦 TCG Download location> ")
if [ -z "$data_dir" ]; then
    exit 1
fi

echo "📦 TCG Download location: $data_dir"

if [ -d "$data_dir" ]; then
    if ! gum confirm "📁 Directory '$data_dir' is about to be wiped to hold new data, do you want to proceed?"; then
        exit 1
    fi

    rm -rf "$data_dir"
fi

mkdir "$data_dir"
echo -e "📁 Created directory: $data_dir\n"

if ! gum spin --title="VegaPull packs..." -- "$VEGA_BIN" --language "$language" pack > "$data_dir/packs.json"; then
    echo "Failed to pull list of packs using vegapull. Aborted" >&2
    exit 1
fi

pack_count=$(jq length "$data_dir"/packs.json)

echo -e "✅ Successfully found $pack_count packs!\n"

function pull_cards() {
    local index=1
    local packs
    packs=$(cat "$data_dir/packs.json")

    while read -r pack_id; do
        pack_title=$(echo "$packs" | jq --arg pack_id "$pack_id" -r '.[] | select(.id == $pack_id) | .title_parts.title')
        message="[$index/$pack_count] VagaPulling cards for: $pack_title ($pack_id)..."

        if ! gum spin --title="$message" -- \
            "$VEGA_BIN" --language "$language" card "$pack_id" > "$data_dir/cards_$pack_id.json"; then
            echo "Failed to pull cards using vegapull. Aborted" >&2
            return 1
        else
            echo "[$index/$pack_count] Succesfully VegaPulled cards for: $pack_title ($pack_id) ✅"
        fi

        ((index++))
    done < <( echo "$packs" | jq -r '.[].id')

    echo "✅ Successfully downloaded data for $index packs!"
}

if ! pull_cards; then
    exit 1
fi

function download_images() {
    local index=1
    local packs
    packs=$(cat "$data_dir/packs.json")

    while read -r pack_id; do
        local output_dir="$data_dir/images/$pack_id"
        pack_title=$(echo "$packs" | jq --arg pack_id "$pack_id" -r '.[] | select(.id == $pack_id) | .title_parts.title')
        echo "[$index/$pack_count] VagaPulling images for: $pack_title ($pack_id)..."

        if ! "$VEGA_BIN" --language "$language" image --output-dir="$output_dir" "$pack_id" -vv; then
            echo "Failed to pull images using vegapull. Aborted" >&2
            return 1
        else
            echo "[$index/$pack_count] Succesfully VegaPulled images for: $pack_title ($pack_id) ✅"
        fi

        ((index++))
    done < <( echo "$packs" | jq -r '.[].id')

    echo "✅ Successfully downloaded data for $index packs!"

}

if gum confirm "🖼️ Download card images as well?"; then
    download_images
fi

echo "✅ Successfully filled the punk records with latest data"
