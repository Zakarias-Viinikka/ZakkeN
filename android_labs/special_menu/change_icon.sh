#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RES_DIR="$SCRIPT_DIR/app/src/main/res"

if [ ! -d "$RES_DIR" ]; then
    echo "Error: '$RES_DIR' not found. Run this from your Android project root." >&2
    exit 1
fi

ALLOWED="png, jpg, jpeg, webp, svg"
INPUT_PATH="${1:-}"

if [ -z "$INPUT_PATH" ]; then
    echo "============================================="
    echo " Android App Icon Changer"
    echo " Allowed formats: $ALLOWED"
    echo "============================================="
    echo ""
    read -r -p "Enter path to icon image: " INPUT_PATH
fi

# Expand ~ and strip surrounding quotes/whitespace
INPUT_PATH="$(eval echo "$INPUT_PATH")"
INPUT_PATH="${INPUT_PATH#[\"\']}"
INPUT_PATH="${INPUT_PATH%[\"\']}"
INPUT_PATH="${INPUT_PATH#"${INPUT_PATH%%[![:space:]]*}"}"
INPUT_PATH="${INPUT_PATH%"${INPUT_PATH##*[![:space:]]}"}"

if [ -z "$INPUT_PATH" ]; then
    echo "Error: No file path provided." >&2
    exit 1
fi

if [ ! -f "$INPUT_PATH" ]; then
    echo "Error: File not found: '$INPUT_PATH'" >&2
    exit 1
fi

EXT="${INPUT_PATH##*.}"
EXT="$(echo "$EXT" | tr '[:upper:]' '[:lower:]')"

case "$EXT" in
    png|jpg|jpeg|webp|svg) ;;
    *)
        echo "Error: Unsupported format '.$EXT'." >&2
        echo "Allowed formats: $ALLOWED" >&2
        exit 1
        ;;
esac

resize_to_png() {
    local src="$1"
    local size="$2"
    local out="$3"
    mkdir -p "$(dirname "$out")"

    if command -v magick &>/dev/null; then
        magick "$src" -resize "${size}x${size}" "$out"
    elif command -v convert &>/dev/null; then
        convert "$src" -resize "${size}x${size}" "$out"
    elif command -v ffmpeg &>/dev/null; then
        ffmpeg -y -i "$src" -vf "scale=${size}:${size}" "$out" 2>/dev/null
    elif python3 -c "from PIL import Image" &>/dev/null; then
        python3 -c "from PIL import Image; img=Image.open('$src').convert('RGBA'); img.resize(($size, $size), Image.Resampling.LANCZOS).save('$out')"
    else
        echo "Error: No image resizing tool found." >&2
        echo "Please install ImageMagick: sudo apt install imagemagick" >&2
        exit 1
    fi
}

echo "Generating icons from '$INPUT_PATH'..."

# Densities and corresponding sizes (px)
declare -A SIZES=(
    ["mipmap-mdpi"]=48
    ["mipmap-hdpi"]=72
    ["mipmap-xhdpi"]=96
    ["mipmap-xxhdpi"]=144
    ["mipmap-xxxhdpi"]=192
)

for folder in "${!SIZES[@]}"; do
    size="${SIZES[$folder]}"
    target_dir="$RES_DIR/$folder"
    mkdir -p "$target_dir"
    
    # Generate both regular and round icons
    resize_to_png "$INPUT_PATH" "$size" "$target_dir/ic_launcher.png"
    cp "$target_dir/ic_launcher.png" "$target_dir/ic_launcher_round.png"
    echo "  ✓ $folder (${size}x${size})"
done

# Remove any XML adaptive icons in anydpi-v26 so Android 8+ doesn't override your PNGs
if [ -d "$RES_DIR/mipmap-anydpi-v26" ]; then
    rm -f "$RES_DIR/mipmap-anydpi-v26/ic_launcher.xml" 2>/dev/null || true
    rm -f "$RES_DIR/mipmap-anydpi-v26/ic_launcher_round.xml" 2>/dev/null || true
fi

echo "Done! Icons successfully updated."
