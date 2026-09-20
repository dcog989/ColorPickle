# ColorPickle

Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.

## Features

- Fullscreen picker overlay with a magnified view of the area under the cursor
- Click to pick a single pixel, or drag to select a region and pick its averaged color
- Copy as HEX, RGB, HSL, Okhsl, Oklch, Oklab, or CMYK
- Okhsl-based internal color model
- Whole-window color background with contrast-adjusted controls
- Gradient Hue/Saturation/Lightness sliders
- Wayland capture via xdg-desktop-portal with a static-frame fallback
- Esc / right-click to dismiss the picker without copying

## Status

Early scaffold. The main window uses the current color as its background, with an eyedrop launcher, gradient Hue/Saturation/Lightness sliders, the Okhsl color pipeline, seven copy formats, and a working clipboard. The picker overlay (magnifier, click-to-copy a pixel, drag-to-average a region, Esc/right-click dismiss) runs over a Wayland xdg-desktop-portal capture.

Still pending: the X11 (`xcap`) and KDE `ScreenShot2` capture backends, alpha/checkerboard, the color input field and parser, CIELAB plus `1`–`8` shortcuts, history swatches, copy-on-pick, the launch-mode/default-format controls, and a follow-system theme UI.

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

## Stack

| Layer | Crate |
|---|---|
| GUI | `egui` + `eframe` |
| Windowing | `winit` |
| Capture | `xcap`, `ashpd` |
| Clipboard | `arboard` (`wl-clipboard-rs` backend on Wayland) |
| Color | `palette` (Okhsl, Oklab, HSL) |
| Config | `serde`, `toml`, `directories` |
| CLI | `clap` |

## License

GPL-3.0-only. See [LICENSE](LICENSE).
