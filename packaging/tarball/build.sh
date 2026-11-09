#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

if ! command -v zstd >/dev/null 2>&1; then
    echo "error: zstd not installed" >&2
    exit 1
fi

version="$(awk -F'"' '/^version = / { print $2; exit }' Cargo.toml)"
name="colorpickle-$version-x86_64-linux"
staging="$root/target/tarball/$name"
outdir="$root/target/tarball"

cargo build --release --locked

rm -rf "$staging"
mkdir -p "$staging"

install -m755 target/release/colorpickle "$staging/colorpickle"
install -m644 packaging/colorpickle.desktop "$staging/colorpickle.desktop"
install -m644 assets/colorpickle.svg "$staging/colorpickle.svg"
install -m644 assets/colorpickle.png "$staging/colorpickle.png"
install -m644 LICENSE "$staging/LICENSE"
install -m755 packaging/tarball/install.sh "$staging/install.sh"

tar --zstd -C "$outdir" -cf "$outdir/$name.tar.zst" "$name"

echo "tarball written to $outdir/$name.tar.zst"
