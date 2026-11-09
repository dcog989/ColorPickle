# ColorPickle

Linux screen color picker. Pick a pixel or drag-select a region with a magnified picker, using any common color format. Launch to GUI or straight to picker.

![screenshot](assets/screen-1.webp)

Assign shortcut to launch the GUI or screen picker:

![screenshot](assets/screen-2.webp)

## Features

- Fullscreen picker overlay with a magnified view of the area under the cursor
- Click to pick a single pixel, or drag to select a region and pick its averaged color
- `Esc` or right-click cancels the picker
- Copy as HEX, RGB, HSL, HSV, Oklch, Oklab, Okhsl, P3, or CMYK
- Keyboard shortcuts `1`–`9` copy the respective format
- Input any supported format, or a CSS color name to set the color
- The main window is the current color, with contrast-adjusted controls
- Gradient Hue/Saturation/Lightness sliders
- APCA contrast scores of the current color on white and black
- History of the last 8 picks — click one to reuse it, hover and click × to remove it, or clear the list
- `+` button beside the input adds the current color to history
- Color-harmony swatches (complementary, split complementary, analogous, triadic, tetradic, rectangle) — click a swatch to use it
- Launch-mode, default-format and precision (Practical/Precise) selectors, saved to the config file
- About panel with the logo, version, log level, and the config, history, log and binary paths
- Screen capture on KDE Plasma (`ScreenShot2`), other Wayland compositors (`ext-image-copy-capture`) and X11, falling back to the slower XDG desktop portal

## Install

[Download latest release](https://github.com/dcog989/ColorPickle/releases/latest):

| Platform                 | Format     | Install                                                                   |
| ------------------------ | ---------- | ------------------------------------------------------------------------- |
| Arch / CachyOS           | `.tar.zst` | `tar --zstd -xf colorpickle-*.tar.zst && sudo ./colorpickle-*/install.sh` |
| Debian / Ubuntu          | `.deb`     | `sudo apt install ./colorpickle_*.deb`                                    |
| Fedora / RHEL / openSUSE | `.rpm`     | `sudo dnf install ./colorpickle-*.rpm`                                    |
| Linux - see NOTE         | AppImage   | `chmod +x ColorPickle-*.AppImage && ./ColorPickle-*.AppImage`             |

> [!NOTE]
> On KDE Plasma, the fast *ScreenShot2* capture requires the binary at a fixed path in `/usr`. The `.deb`, `.rpm` and tarball installs qualify; the AppImage runs from a temporary mount and falls back to a slower method such as the XDG desktop portal.

## Install from source

See [requirements](#requirements). Clone this repository then:

```sh
just install   # release build, then install to /usr
```

---

## Configuration

Settings live in `~/.config/colorpickle/config.toml` (or `$XDG_CONFIG_HOME/colorpickle/config.toml`). The app writes it whenever you change a setting; edit it with the app closed.

```toml
launch_mode = "ui_first"     # ui_first | picker_first
default_format = "hex"       # hex | rgb | hsl | hsv | oklch | oklab | okhsl | p3 | cmyk
precision = "practical"      # practical | precise
log_level = "info"           # error | warn | info | debug | trace
```

- `--launch-mode ui_first|picker_first` overrides `launch_mode` for one run
- `--log-level` overrides `log_level` for one run (see [Logs](#logs))
- Pick history is kept apart in `~/.local/state/colorpickle/state.toml` (or `$XDG_STATE_HOME/colorpickle/state.toml`), so picking never rewrites your settings

## Logs

Logs are written to `~/.local/state/colorpickle/log/`, one file per day named `colorpickle.log.<date>`; the 7 most recent are kept.

- `--log-level error|warn|info|debug|trace` sets the verbosity; default `info`
- `log_level` in `config.toml` does the same, and is overridden by the flag; the About panel has a log level dropdown that applies the change at once and saves it
- The same lines are mirrored to stderr, with color when stderr is a terminal

---

## Development

### Requirements

Rust 1.99, `just`, `pkg-config`, `lefthook`, Wayland and `libxkbcommon` development headers; `zstd` is needed for the tarball.

- Debian/Ubuntu: `sudo apt install just pkg-config libwayland-dev libxkbcommon-dev zstd`
- Arch: `sudo pacman -S --needed just pkgconf wayland libxkbcommon zstd`
- `lefthook` install from AUR or [release packages](https://github.com/evilmartians/lefthook/releases)

```sh
cargo install cargo-edit cocogitto cargo-outdated   # cargo set-version, cargo upgrade, cog, cargo outdated
lefthook install                                    # install the git hooks; lefthook must be on PATH
cargo install cargo-deb cargo-generate-rpm          # deb and rpm generators
```

Building the AppImage needs `linuxdeploy` on `PATH`, or its path in `LINUXDEPLOY`.

### Build

```sh
just build                               # release build
just check                               # clippy linter, formatting check, run tests
just lint                                # clippy only
just --list                              # all recipes

cargo build                              # debug build
cargo fetch && cargo outdated -w         # check for updates above semver range
cargo fmt --all -- --check               # formatting check
cargo fmt --all                          # format all files
cargo test --workspace                   # tests
cargo run                                # build and run
cargo upgrade && cargo update --verbose  # update Cargo.lock and dependencies to latest with semver ranges
cargo clean                              # clean build artifacts

just package                             # .deb, .rpm, AppImage and tarball into target/
cog bump --auto                          # bump version and push
```

---

## License

[GNU General Public License v3.0](LICENSE)
