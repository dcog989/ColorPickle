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

# linuxdeploy resolves the desktop Exec against the AppDir and only accepts a
# bare name; the installed desktop file keeps an absolute Exec so KWin can
# authorise ScreenShot2 from a system install.
appimage_desktop="$outdir/colorpickle.desktop"
sed 's#^Exec=/usr/bin/#Exec=#' "$root/packaging/colorpickle.desktop" > "$appimage_desktop"

"$linuxdeploy" \
    --appdir "$appdir" \
    --executable "$root/target/release/colorpickle" \
    --desktop-file "$appimage_desktop" \
    --icon-file "$root/packaging/colorpickle.svg" \
    --icon-file "$root/assets/colorpickle.png" \
    --output appimage

find "$root" -maxdepth 1 -name 'ColorPickle*.AppImage' -exec mv {} "$outdir/" \;

echo "AppImage written to $outdir"
