#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if ! cargo deb --version >/dev/null 2>&1; then
    echo "error: cargo-deb not installed (cargo install cargo-deb)" >&2
    exit 1
fi

if ! cargo generate-rpm --version >/dev/null 2>&1; then
    echo "error: cargo-generate-rpm not installed (cargo install cargo-generate-rpm)" >&2
    exit 1
fi

cargo build --release --locked
cargo deb --no-build
cargo generate-rpm
packaging/appimage/build.sh

echo "artifacts:"
find target/debian target/generate-rpm target/appimage -maxdepth 1 \
    \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) 2>/dev/null
