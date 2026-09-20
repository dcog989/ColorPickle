# ColorPickle

Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.

## Features

- Fullscreen picker overlay with a magnified view of the area under the cursor
- Click to pick a single pixel, or drag to select a region and pick its averaged color
- Copy as HEX, RGB, HSL, Okhsl, Oklch, Oklab, CMYK, or CIELAB
- Keyboard shortcuts `1`–`8` copy the respective format
- Type a hex code, any supported format, or a CSS color name to set the color
- The main window is the current color, with contrast-adjusted controls
- Gradient Hue/Saturation/Lightness sliders
- History of the last 8 picks — click one to reuse it, or clear the list
- Colour-harmony swatches (complementary, split complementary, analogous, triadic, tetradic, rectangle) — click a swatch to use it
- Launch-mode and default-format selectors, saved to the config file
- Wayland capture via `ext-image-copy-capture`, falling back to the XDG desktop portal

## Usage

Run `colorpickle` for the main window, or `colorpickle --launch-mode picker_first` to go straight to the picker. In picker-first mode the main window opens with the picked color once you pick. The launch mode and other defaults can also be set in the config file.

In the main window, click the logo button to open the picker. **Click** copies a single pixel; **drag** selects a region and copies its averaged color. Either way the picker closes and the value is copied to the clipboard. Press `Esc` or right-click to dismiss without copying. `Esc` exits the main window.

Hover a format button to preview the current color in that format; click it, or press its number key, to copy.

The text field shows the current color in the default format. Type a replacement — `#ff0000`, `rgb(255, 0, 0)`, `oklch(0.7, 0.1, 120)`, `olive`, and so on — and press Enter to apply it.

Each pick is added to the history row; click a swatch to make it the current color, or the broom button to clear the history. The bottom row selects the launch mode and the default copy format; those changes are saved to the config file.

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

## License

GPL-3.0-only. See [LICENSE](LICENSE).
