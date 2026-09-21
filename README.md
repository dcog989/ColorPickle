# ColorPickle

Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.

## Features

- Fullscreen picker overlay with a magnified view of the area under the cursor
- Click to pick a single pixel, or drag to select a region and pick its averaged color
- Copy as HEX, RGB, HSL, Okhsl, Oklch, Oklab, CMYK, or CIELAB
- Keyboard shortcuts `1`–`8` copy the respective format
- Type a HEX code, any supported format, or a CSS color name to set the color
- The main window is the current color, with contrast-adjusted controls
- Gradient Hue/Saturation/Lightness sliders
- History of the last 8 picks — click one to reuse it, or clear the list
- Color-harmony swatches (complementary, split complementary, analogous, triadic, tetradic, rectangle) — click a swatch to use it
- Launch-mode and default-format selectors, saved to the config file
- Wayland capture via `ext-image-copy-capture`, falling back to the XDG desktop portal

## Install

Prebuilt `.deb`, `.rpm`, and AppImage packages are attached to each [release](https://github.com/dcog989/ColorPickle/releases).

Arch Linux (AUR):

```sh
yay -S colorpickle        # or paru -S colorpickle
```

Debian/Ubuntu (`.deb`):

```sh
sudo apt install ./colorpickle_*.deb
```

Fedora/RHEL/openSUSE (`.rpm`):

```sh
sudo dnf install ./colorpickle-*.rpm   # openSUSE: sudo zypper install ./colorpickle-*.rpm
```

AppImage (any distro):

```sh
chmod +x ColorPickle-*.AppImage
./ColorPickle-*.AppImage
```

If FUSE is unavailable, run it with `./ColorPickle-*.AppImage --appimage-extract-and-run`.

From source (requires a Rust toolchain), from a clone of this repository:

```sh
cargo install --path .
```

The AppImage needs a working X11 or Wayland session. The XDG desktop portal fallback additionally needs `xdg-desktop-portal` installed.

## Usage

Run `colorpickle` for the main window, or `colorpickle --launch-mode picker_first` to go straight to the picker. In picker-first mode the main window opens with the picked color once you pick. The launch mode and other defaults can also be set in the config file.

In the main window, click the pipette button to open the picker. **Click** copies a single pixel; **drag** selects a region and copies its averaged color. Either way the picker closes and the value is copied to the clipboard. Press `Esc` or right-click to dismiss without copying. `Esc` exits the main window.

Hover a format button to preview the current color in that format; click it, or press its number key, to copy.

The text field shows the current color in the default format. Type a replacement — `#ff0000`, `rgb(255, 0, 0)`, `oklch(0.7, 0.1, 120)`, `olive`, and so on — and press Enter to apply it.

Each pick is added to the history row; click a swatch to make it the current color, or the close button to clear the history. The bottom row selects the launch mode and the default copy format; those changes are saved to the config file.

## Configuration

ColorPickle reads `~/.config/colorpickle/config.toml`, and writes it back when you change the launch mode or default format in the UI. The keys are:

```toml
launch_mode = "ui_first"     # or "picker_first"
default_format = "hex"       # hex, rgb, hsl, okhsl, oklch, oklab, cmyk, cielab
theme = "system"             # system, light, or dark
```

## Build

Requires a Rust toolchain.

```sh
cargo build                              # debug build
cargo check                              # type-check only
cargo clippy --workspace --all-targets -- -D warnings
cargo fetch && cargo outdated -w         # check for updates above semver range
cargo fmt --check                        # formatting check
cargo fmt                                # format all files
cargo run                                # build and run
cargo test --workspace                   # tests
cargo build --release                    # release build
cargo upgrade && cargo update --verbose  # update Cargo.lock and dependencies to latest with semver ranges

cargo clean && rm -rf target/            # clean build artifacts
```

## Packaging

Release artifacts are built by `.github/workflows/release.yml` when a `v*` tag is pushed. To build them locally:

```sh
cargo install cargo-deb cargo-generate-rpm   # once
packaging/build.sh                           # .deb, .rpm and AppImage into target/
```

Individual artifacts:

- `.deb` — `cargo deb` (Debian/Ubuntu)
- `.rpm` — `cargo generate-rpm` (Fedora/RHEL/openSUSE)
- AppImage — `packaging/appimage/build.sh`, requires `linuxdeploy` on `PATH` (`LINUXDEPLOY=/path/to/linuxdeploy` to override)
- AUR — `packaging/aur/PKGBUILD`
- Source — `cargo install --path .`

## Known limitations

On KDE Plasma, the fast `org.kde.KWin.ScreenShot2` capture path is restricted by KWin to allow-listed executables (`X-KDE-DBUS-Restricted-Interfaces`), matched against the executable path. Package installs at `/usr/bin/colorpickle` (`.deb`, `.rpm`, AUR) qualify, but an AppImage runs from a temporary mount and is denied. AppImage users on Plasma therefore fall back to `ext-image-copy-capture` where the compositor supports it, or to the XDG desktop portal (shown as the "picking from a static screenshot" banner), which is slower.

## License

GPL-3.0-only. See [LICENSE](LICENSE).
