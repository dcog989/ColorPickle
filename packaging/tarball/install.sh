#!/usr/bin/env sh
set -eu

here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

if [ "$(id -u)" -ne 0 ]; then
    echo "error: run with sudo" >&2
    exit 1
fi

# Fixed /usr prefix: the desktop file pins Exec=/usr/bin/colorpickle, which the
# KWin D-Bus allowlist matches against for ScreenShot2.
install -Dm755 "$here/colorpickle" /usr/bin/colorpickle
install -Dm644 "$here/colorpickle.desktop" /usr/share/applications/colorpickle.desktop
install -Dm644 "$here/colorpickle.svg" /usr/share/icons/hicolor/scalable/apps/colorpickle.svg
install -Dm644 "$here/colorpickle.png" /usr/share/icons/hicolor/256x256/apps/colorpickle.png

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications
fi
