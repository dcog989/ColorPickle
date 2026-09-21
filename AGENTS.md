# Agent Directives

## Project Specifics

- Name: ColorPickle
- Description: Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.
- Tech: Rust (single binary); `egui`/`eframe` GUI (`winit` via eframe); capture via `x11rb`, `ashpd`, `zbus`, `wayland-client`/`wayland-protocols`; clipboard via `arboard` (with the `wl-clipboard-rs` Wayland backend); `palette`; `lucide-icons`; `clap`, `serde`, `toml`, `directories`, `anyhow`, `thiserror`, `tracing`, `image`; Lefthook + Cocogitto.

See `.docs/HLD.md` for architecture, UI, capture backends, color pipeline, and distribution.

### Key Files

- `src/main.rs` — entry point, `clap` launch-mode dispatch
- `src/cli.rs` — `clap` args and `LaunchMode`
- `src/config.rs` — `Config` load/path via `directories` + `toml`
- `src/color/okhsl.rs` — Okhsl internal color model, backed by `palette::Okhsl`
- `src/color/mod.rs` — `ColorFormat` and copy-format output (CMYK is manual)
- `src/color/parse.rs` — typed color and CSS named-color parsing
- `src/color/harmony.rs` — color-harmony offsets and swatches
- `src/capture/` — capture backends (KWin D-Bus, `ext-image-copy-capture`, X11, XDG portal) dispatched through a function-pointer `Backend` table in `capture/mod.rs`
- `src/ui/main_window.rs` — main window (color background); split into `src/ui/main_window/` submodules (`toast`, `history`, `keys`, `slider_panel`, `settings_panel`)
- `src/ui/overlay.rs` — picker overlay: magnifier, click/drag pick, portal banner
- `src/ui/picker.rs` — capture/session controller between the main window and the overlay
- `src/ui/icons.rs` — lucide icon font setup and icon helpers
- `src/ui/theme.rs` — dynamic contrast theme derived from the current color
- `src/clipboard.rs` — clipboard writes via `arboard`

### Workflow

- Install: `cargo fetch`
- Dev: `cargo run`
- Test: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Build: `cargo build --release`

### Common Patterns

- Internal color state is always Okhsl; convert via sRGB as the interchange for all copy formats and the picker pipeline.
- Backends are a function-pointer `Backend` table in `capture/mod.rs`, not a trait; each returns a plain frame + rect, so the picker overlay stays agnostic to X11 vs Wayland.
- Single-binary, two launch modes via `clap`: UI mode (default) and picker mode.
- No menus, no settings pane, no tabs — preserve the one-glance model.

### File System Access

- Allowed: <project root> and all contained directories + files; `/tmp/*`.
- Read-Only: `.env*`, `.git/`.
- Disallowed: everything not listed in 'Allowed' unless user grants permission; .docs/ToDo.md
- Require confirmation: adding/removing dependencies, any operation outside project root.
- Do not delete files or make destructive changes without permission / confirmation.

---

## General Guidelines

### Code Changes

- For non-trivial work, propose an approach and confirm before implementing.
- Keep modifications minimal and scoped; prefer incremental improvements over rewrites. Ask before architectural changes.
- Use explicit types and named constants (no magic numbers).
- Return explicit error types; do not suppress exceptions.
- Follow standard repository linting and formatting configs.
- Decompose files over 400 lines if they mix concerns.
- Use clear naming over comments; reserve comments for complex workarounds or non-obvious issues — why, not what.
- Never run git mutations (commit, push, reset, rebase, amend) unless explicitly instructed.
- Do not create documentation files unless explicitly requested.

### Verification

- Do not run test, lint, format, or type-check commands; the user builds, tests, and lints manually.
- Run them only when the user explicitly asks.

### Author Environment

- CachyOS, KDE Plasma 6, Wayland, Btrfs.
- fish shell, Ghostty terminal, Fresh TUI editor, yay package manager, Bun npm manager, Firefox, and Zed code editor.

### Testing

- Do not create test files for trivial changes, or for behavior that is not reliably unit-testable in the test environment (e.g. UI layout/click mapping). Prefer no new files; only add a test when the logic is genuinely testable and worth guarding.

### Definition of Done

- Logic fully implemented.
- Existing docs updated if public interfaces changed.
- When required by the `Verification` rules, run the corresponding `Workflow` command.
- On completion of an update or fix, print a concise conventional commit message in a fenced code block.

### Communication Style

- Provide concise, actionable responses.
- Ask clarifying questions when requirements are ambiguous.
- Flag potential risks or edge cases proactively.
- Do not pretend to understand how the user feels.
- Never editorialise your answer. No "to be honest", "honestly", hedging, disclaimers, or meta-commentary — just answer.
