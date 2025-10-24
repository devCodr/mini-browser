#!/bin/bash
# scripts/make-icon.sh
# Genera build/icon.icns a partir de icon.png (necesita macOS)
# Usa: ./scripts/make-icon.sh [source.png]

set -euo pipefail
SRC=${1:-"$(dirname "$0")/../icon.png"}
OUT_DIR="$(dirname "$0")/../build"
ICONSET="$OUT_DIR/icon.iconset"
ICNS="$OUT_DIR/icon.icns"

mkdir -p "$ICONSET"
mkdir -p "$OUT_DIR"

if [ ! -f "$SRC" ]; then
  echo "Source icon not found: $SRC"
  exit 1
fi

# Sizes required for iconutil
sizes=(16 32 64 128 256 512 1024)
for size in "${sizes[@]}"; do
  sips -z $size $size "$SRC" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  # also create @2x where applicable
  sips -z $((size*2)) $((size*2)) "$SRC" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null || true
done

# Create icns
iconutil -c icns "$ICONSET" -o "$ICNS"

# Cleanup iconset
rm -rf "$ICONSET"

echo "Created $ICNS"
