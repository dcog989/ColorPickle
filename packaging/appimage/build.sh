#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

linuxdeploy="${LINUXDEPLOY:-linuxdeploy}"
appdir="$root/target/appimage/ColorPickle.AppDir"
outdir="$root/target/appimage"

if ! command -v "$linuxdeploy" >/dev/null 2>&1; then
    echo "error: '$linuxdeploy' not found; install linuxdeploy or set LINUXDEPLOY" >&2
    exit 1
fi

cargo build --release --locked

rm -rf "$appdir"
mkdir -p "$outdir"

"$linuxdeploy" \
    --appdir "$appdir" \
    --executable "$root/target/release/colorpickle" \
    --desktop-file "$root/packaging/colorpickle.desktop" \
    --icon-file "$root/packaging/colorpickle.svg" \
    --icon-file "$root/assets/colorpickle.png" \
    --output appimage

find "$root" -maxdepth 1 -name 'ColorPickle*.AppImage' -exec mv {} "$outdir/" \;

echo "AppImage written to $outdir"
