# Agent Directives

## Project Specifics

- Name: ColorPickle
- Description: Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.
- Tech: Rust (single binary); `egui`/`eframe` GUI (`winit` via eframe); capture via `x11rb`, `ashpd`, `zbus`, `wayland-client`/`wayland-protocols`; clipboard via `arboard` (with the `wl-clipboard-rs` Wayland backend); `palette`; `lucide-icons`; `clap`, `serde`, `toml`, `directories`, `anyhow`, `thiserror`, `tracing`, `image`; Lefthook + Cocogitto.

See `README.md` for usage, configuration, install and distribution.

### Key Files

- `src/main.rs` — entry point; clipboard-helper dispatch, then `clap` launch-mode dispatch
- `src/cli.rs` — `clap` args and `LaunchMode`
- `src/config.rs` — `Config` load/path via `directories` + `toml`
- `src/state.rs` — `State` (pick history) load/path under `$XDG_STATE_HOME`
- `src/persistence.rs` — atomic toml load/save shared by `Config` and `State`
- `src/logging.rs` — `tracing` subscriber: daily-rotating file log under `$XDG_STATE_HOME/colorpickle/log` plus stderr; level from `RUST_LOG`, `--log-level` or `config.toml`
- `src/color/mod.rs` — `ColorFormat` (hex, rgb, hsl, hsv, oklch, oklab, okhsl, p3, cmyk) and `Precision`
- `src/color/parse.rs` — input parsing: hex, `rgb`/`hsl`/`hsv`/`oklch`/`oklab`/`okhsl`/`cmyk`, `color(display-p3 …)`, CSS color names
- `src/color/okhsl.rs` — Okhsl internal color model, backed by `palette::Okhsl`
- `src/color/harmony.rs` — harmony swatch math; `src/color/apca.rs` — APCA contrast
- `src/capture/` — capture backends (KWin D-Bus, `ext-image-copy-capture`, X11, XDG portal) dispatched through a function-pointer `Backend` table in `capture/mod.rs`
- `src/ui/` — main window (color background), picker overlay, capture/session controller, theme; `main_window/` holds the history, about, settings, slider and toast panels
- `src/clipboard.rs` — clipboard writes via `arboard`, handed to a re-exec'd helper process

### Workflow

- Install: `just install` (needs `pkg-config`, Wayland and `libxkbcommon` development headers)
- Dev: `cargo run`
- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all`
- Check lint, format and tests: `just check`
- Build: `cargo build --release`
- Package: `just package` (needs `cargo-deb`, `cargo-generate-rpm`, `linuxdeploy`)

### Common Patterns

- Internal color state is always Okhsl; convert via sRGB as the interchange for all copy formats and the picker pipeline.
- Backends are a function-pointer `Backend` table in `capture/mod.rs`, not a trait; each returns a plain frame, so the picker overlay stays agnostic to X11 vs Wayland.
- Single-binary, two launch modes via `clap`: UI mode (default) and picker mode.
- No menus, no settings pane, no tabs — preserve the one-glance model.
- User data is split so picking never rewrites settings: `$XDG_CONFIG_HOME/colorpickle/config.toml` (`launch_mode`, `default_format`, `precision`, `log_level`), history in `$XDG_STATE_HOME/colorpickle/state.toml`, logs in `$XDG_STATE_HOME/colorpickle/log`.

### Architecture Notes

- Capture happens once, before the overlay is mapped — never per-move, so the always-on-top overlay cannot capture itself or re-trigger permission prompts.
- The clipboard is owned by a re-exec'd copy of the binary: `clipboard::set_text` spawns `current_exe()` with `COLORPICKLE_CLIPBOARD_SERVE=1` and pipes the text to its stdin, so the selection outlives the window. `main` checks that variable before `clap` and logging, so the helper opens no window and writes no log; if the spawn fails, an in-process `arboard::Clipboard` is used.
- Backend preference order: KWin `org.kde.KWin.ScreenShot2` → `ext-image-copy-capture` → X11 `GetImage` → XDG desktop portal.
- Compositor-native pickers are deliberately not used (KWin `ColorPicker.pick()`, `hyprpicker`): they return a single pixel and expose no frame, so they cannot drive the magnifier or drag-average.
- KWin allowlists `ScreenShot2` callers via `X-KDE-DBUS-Restricted-Interfaces` in a `.desktop`: it canonicalises the entry's first `Exec` token and compares it to `/proc/<pid>/exe`, so `Exec` must be an absolute path (a bare `colorpickle` never matches). An AppImage runs from a temp mount and is denied, falling back to the portal.
- `ext-image-copy-capture` is a staging protocol; its backend opens its own Wayland connection rather than sharing `winit`'s, and captures only the output containing the cursor (falling back to the output at the origin).
- Capture is single-output: KWin uses `CaptureActiveScreen`, X11 captures the cursor's monitor, and the `ext-image-copy-capture` backend picks the cursor's output; there is no multi-output stitching.
- CMYK output is computed manually; `palette`'s Okhsl reports hue in degrees (0–360), not 0–1 turns.
- Distribution: AppImage is primary, with `.deb`, `.rpm`, AUR and source installs alongside.

### File System Access

- Allowed: <project root> and all contained directories + files; `/tmp/*`.
- Read-Only: `.env*`, `.git/`.
- Disallowed: everything not listed in 'Allowed' unless user grants permission; .docs/ToDo.md
- Require confirmation: adding/removing dependencies, any operation outside project root.
- Do not delete files or make destructive changes without permission / confirmation.

---

## General Guidelines

### Code Changes

- For non-trivial work, propose an approach and confirm before implementing, including architectural changes.
- Keep modifications minimal and scoped; prefer incremental improvements over rewrites.
- Use explicit types and named constants (no magic numbers).
- Return explicit error types; do not suppress exceptions.
- Follow standard repository linting and formatting configs.
- Decompose files over 400 lines if they mix concerns.
- Name things clearly; reserve comments for the non-obvious why (workarounds, edge cases).
- Never run git mutations (commit, push, reset, rebase, amend) unless explicitly instructed.
- Do not create documentation files unless explicitly requested.
- Adhere to <https://develop.kde.org/hig/>.

### Author Environment

- Ryzen 5800X3D, Radeon RX 7900 XT, Display (AW3225QF); CachyOS, KDE, KWin / Wayland, Btrfs.
- fish shell, Ghostty terminal, Fresh TUI editor, yay package manager, bun npm manager, Firefox, Zed code editor.

### Testing

- Add tests only when the logic is genuinely testable and worth guarding. No test files for trivial changes or behavior the environment cannot test reliably (e.g. UI layout/click mapping).

### Definition of Done

- Logic fully implemented.
- Existing docs updated if public interfaces changed.
- Run the corresponding `Workflow` command when `Verification` requires it.

### Commit Messages

- On completion of an update or fix, print a conventional commit message in a fenced code block.
- Add a body only when the "why" isn't obvious.
- Never hard-wrap: one physical line per paragraph, however long.

### Communication Style

- Format: short declarative sentences, one idea per line.
- Instructions: numbered steps, one action each (exact command, path, or code); no prose recap.
- Explanations: essential context only. End question answers with `TL;DR:`, one sentence; omit for status updates and completed work.
- Tone: no preamble, restating, hedging, disclaimers, emotional framing, or closing offers. Keep every fact, command, flag, path, default, caveat.
- Edge Cases: state risks explicitly; ask when requirements are ambiguous.
