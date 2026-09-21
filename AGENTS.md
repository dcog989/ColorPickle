# Agent Directives

## Project Specifics

- Name: ColorPickle
- Description: Linux screen color picker. Pick a pixel or drag-select a region from anywhere on screen, get the color in any common format, copy it. One glance, one click.
- Tech: Rust (single binary); `egui`/`eframe` GUI (`winit` via eframe); capture via `x11rb`, `ashpd`, `zbus`, `wayland-client`/`wayland-protocols`; clipboard via `arboard` (with the `wl-clipboard-rs` Wayland backend); `palette`; `lucide-icons`; `clap`, `serde`, `toml`, `directories`, `anyhow`, `thiserror`, `tracing`, `image`; Lefthook + Cocogitto.

See `README.md` for usage, configuration, install and distribution.

### Key Files

- `src/main.rs` — entry point, `clap` launch-mode dispatch
- `src/cli.rs` — `clap` args and `LaunchMode`
- `src/config.rs` — `Config` load/path via `directories` + `toml`
- `src/color/okhsl.rs` — Okhsl internal color model, backed by `palette::Okhsl`
- `src/capture/` — capture backends (KWin D-Bus, `ext-image-copy-capture`, X11, XDG portal) dispatched through a function-pointer `Backend` table in `capture/mod.rs`
- `src/ui/` — main window (color background), picker overlay, capture/session controller, theme
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
- Backends are a function-pointer `Backend` table in `capture/mod.rs`, not a trait; each returns a plain frame, so the picker overlay stays agnostic to X11 vs Wayland.
- Single-binary, two launch modes via `clap`: UI mode (default) and picker mode.
- No menus, no settings pane, no tabs — preserve the one-glance model.

### Architecture Notes

- Capture happens once, before the overlay is mapped — never per-move, so the always-on-top overlay cannot capture itself or re-trigger permission prompts.
- Backend preference order: KWin `org.kde.KWin.ScreenShot2` → `ext-image-copy-capture` → X11 `GetImage` → XDG desktop portal.
- Compositor-native pickers are deliberately not used (KWin `ColorPicker.pick()`, `hyprpicker`): they return a single pixel and expose no frame, so they cannot drive the magnifier or drag-average.
- KWin allowlists `ScreenShot2` callers via `X-KDE-DBUS-Restricted-Interfaces` in a `.desktop`: it canonicalises the entry's first `Exec` token and compares it to `/proc/<pid>/exe`, so `Exec` must be an absolute path (a bare `colorpickle` never matches). An AppImage runs from a temp mount and is denied, falling back to the portal.
- `ext-image-copy-capture` is a staging protocol; its backend opens its own Wayland connection rather than sharing `winit`'s, and composites per-output frames itself.
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
