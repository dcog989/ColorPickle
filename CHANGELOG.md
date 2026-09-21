# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## v0.2.3 - 2026-09-21
#### Bug Fixes
- (**ci**) attach package files to the release - (9cfc299) - dcog989
#### Build system
- ignore merge commits and push after bump - (bed28b6) - dcog989

- - -

## v0.2.2 - 2026-09-21
#### Build system
- ignore merge commits in cog - (b777fd8) - dcog989

- - -

## v0.2.1 - 2026-09-21
#### Bug Fixes
- (**packaging**) use a bare Exec for the AppImage desktop file - (3955d07) - dcog989
#### Documentation
- screenshot - (245bb9a) - dcog989
#### Build system
- (**deps**) bump actions/download-artifact from 4 to 8 - (76277c8) - dependabot[bot]
- (**deps**) bump actions/checkout from 4 to 7 - (eb88596) - dependabot[bot]
- (**deps**) bump actions/upload-artifact from 4 to 7 - (fe3935e) - dependabot[bot]
- (**deps**) bump softprops/action-gh-release from 2 to 3 - (9299b47) - dependabot[bot]

- - -

## v0.2.0 - 2026-09-21
#### Miscellaneous Chores
- (**version**) v0.1.0 - (f127dbe) - dcog989

- - -

## v0.1.0 - 2026-09-21
#### Features
- (**capture**) add an ext-image-copy-capture Wayland backend on its own connection - (70eff9c) - dcog989
- (**capture**) implement the X11 xcap backend - (91097bb) - dcog989
- (**capture**) add KWin ScreenShot2 capture backend - (919c49f) - dcog989
- (**color**) add input field and colour parser - (cfeefe6) - dcog989
- (**color**) add CIELAB format, format shortcuts and copy icon - (dc2d71a) - dcog989
- (**packaging**) add keywords and a picker-first desktop action - (f2a49ce) - dcog989
- (**packaging**) add deb, rpm, AppImage and AUR distribution with release CI - (0df902c) - dcog989
- (**ui**) show APCA contrast against white and black - (e875e4d) - dcog989
- (**ui**) pick theme foreground with APCA contrast - (510c609) - dcog989
- (**ui**) use a pipette icon for the picker launcher instead of the logo - (cc8a4dd) - dcog989
- (**ui**) refined sliders - (dee4d15) - dcog989
- (**ui**) round widget, window, menu, and swatch corners - (204bcf6) - dcog989
- (**ui**) add colour-harmony palette row with clickable swatches - (26057f6) - dcog989
- (**ui**) add logo icon, ui refined - (75eaad3) - dcog989
- (**ui**) open the main window with the picked colour in picker-first mode - (e3cb5d7) - dcog989
- (**ui**) multiple refinements - (799d417) - dcog989
- (**ui**) pipette icon, ui spacing, ++ - (d267afa) - dcog989
- (**ui**) add pick history, bottom-row selectors and ESC-to-exit - (3b0a948) - dcog989
- (**ui**) move sliders to a full-height right-hand panel - (94ce58d) - dcog989
#### Bug Fixes
- (**capture**) import the x11rb Connection trait in the X11 backend - (a5f029f) - dcog989
- (**capture**) allocate shm buffers with memfd instead of a world-readable temp file - (34a0d3a) - dcog989
- (**capture**) reject empty frames at the capture boundary - (dff5885) - dcog989
- (**capture**) align monitor captures to logical geometry and output transform - (1928cf7) - dcog989
- (**capture**) wait for ext-image frames against a real wall-clock deadline - (bb888cb) - dcog989
- (**capture**) show the static-screenshot banner only for the portal fallback - (e883d0e) - dcog989
- (**capture**) stop leaving portal screenshots on disk - (c59b6ad) - dcog989
- (**capture**) stop the portal connection tying itself to a throwaway runtime - (5588524) - dcog989
- (**clipboard**) keep one clipboard instance alive across copies - (e895b98) - dcog989
- (**color**) avoid negative zero in decimal color components - (0140ae7) - dcog989
- (**color**) drop trailing zeros from decimal color formats - (4b5ef16) - dcog989
- (**color**) format HSL and Oklch hues as positive degrees - (f84e597) - dcog989
- (**config**) fall back to defaults and write the config atomically - (ffaeb28) - dcog989
- (**deps**) enable arboard wayland-data-control and drop unused wl-clipboard-rs - (b7f1551) - dcog989
- (**input**) only re-parse edited field and disambiguate percent parsing - (20bf74a) - dcog989
- (**logging**) default to info level in release builds - (1e66b41) - dcog989
- (**overlay**) drop the unconditional repaint and fit oversized frames to the GPU texture limit - (b947178) - dcog989
- (**overlay**) align the picker reticle with the sampled pixel - (8abe5c1) - dcog989
- (**overlay**) drop the no-op picker fullscreen re-request - (e06bf44) - dcog989
- (**overlay**) give the picker its own app id so it stops restoring the main window size - (a9e6414) - dcog989
- (**overlay**) magnify relative to logical screen size - (248f99f) - dcog989
- (**overlay**) magnify a square pixel region instead of stretching - (c62bb00) - dcog989
- (**overlay**) build selection fill from a const colour constructor - (994e783) - dcog989
- (**packaging**) use an absolute Exec so KWin authorizes ScreenShot2 - (4721a83) - dcog989
- (**packaging**) follow Arch Rust guidelines in the AUR PKGBUILD - (2823a0e) - dcog989
- (**packaging**) declare winit runtime X11/EGL libraries - (0bc9240) - dcog989
- (**packaging**) use the ColorPickle source directory in the AUR PKGBUILD - (bac8071) - dcog989
- (**picker**) hide the main window before capturing the frozen frame - (1af4b2d) - dcog989
- (**picker**) don't time out while the portal permission prompt is open - (ec13e9b) - dcog989
- (**picker**) size the overlay to the captured desktop bounds - (9461d70) - dcog989
- (**ui**) set the main window app_id to match the desktop file - (168a8e6) - dcog989
- (**ui**) assert format shortcut keys match the format list - (c616d96) - dcog989
- (**ui**) discard label responses in on_hover_ui closures - (c77de87) - dcog989
- (**ui**) don't let Escape close the window while editing the color field - (d84fc31) - dcog989
- (**ui**) toast font size - (8897384) - dcog989
- (**ui**) stabilise foreground contrast and apply the theme before the first frame - (fa1a61d) - dcog989
- (**ui**) derive foreground lightness from the background instead of a fixed shift - (2af8714) - dcog989
- (**ui**) keep the copy buttons on one row and set a minimum window width - (c2b0ec2) - dcog989
- (**ui**) drop the "Parsed color" toast on input blur - (38de72f) - dcog989
- (**ui**) theme popup and tooltip surfaces so text stays readable - (38bd910) - dcog989
- (**ui**) draw an eyedropper icon for the picker launcher - (98f4f2d) - dcog989
- (**ui**) use egui 0.36 Panel API for the slider panel - (07fa318) - dcog989
#### Performance Improvements
- (**capture**) swizzle frames in place and move them instead of cloning - (bdef4bf) - dcog989
- (**overlay**) compute the picked average only on release - (943a208) - dcog989
- (**slider**) sample gradients at 64 stops instead of per pixel row - (1be470a) - dcog989
- (**ui**) cache theme, min-size, and formatted strings; lazy tooltips - (f120197) - dcog989
#### Documentation
- (**agents**) absorb HLD rationale, drop HLD reference - (08ed14f) - dcog989
- (**agents**) fix capture backend description and dependency list - (97dd973) - dcog989
- (**agents**) correct capture backend description and key files list - (1d193ad) - dcog989
- (**readme**) add install section - (a0b526b) - dcog989
- (**readme**) make README user-facing - (4952848) - dcog989
- colour -> color - (5bcdce8) - dcog989
- note the KWin ScreenShot2 AppImage capture limitation - (d1fd7b2) - dcog989
- drop alpha from the design - (999556b) - dcog989
#### Build system
- (**deps**) enable only the png codec for image - (18da7e7) - dcog989
- (**profile**) add release LTO/strip and optimize dependencies in dev - (3368c5f) - dcog989
- declare rust-version and disable publishing - (06791e2) - dcog989
#### Continuous Integration
- build the release once and package all artifacts in one job - (8290fcc) - dcog989
- build with --locked before packaging deb/rpm - (80a5b29) - dcog989
- add fmt/clippy/test workflow and dependabot config - (2fbbec4) - dcog989
- scope the write token to the release job - (26dc5ab) - dcog989
#### Refactoring
- (**capture**) capture the cursor's single output - (d74a043) - dcog989
- (**capture**) drop desktop-rect plumbing and size overlay via fullscreen - (f3d2fc0) - dcog989
- (**capture**) replace xcap with an x11rb root-window grab - (405d93c) - dcog989
- (**capture**) unify the backend composite helpers - (99a7cb0) - dcog989
- (**capture**) use delegate_noop for event-less Wayland interfaces - (531b6be) - dcog989
- (**capture**) store required Wayland managers as non-optional - (393be4f) - dcog989
- (**capture**) decode portal screenshot URIs with url::Url - (9cad6b8) - dcog989
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>(**capture**) replace CaptureBackend trait with capture() and backend table - (c2978d9) - dcog989
- (**color**) implement PartialEq for Okhsl - (0d5abb0) - dcog989
- (**config**) derive Default for Config - (9efcb19) - dcog989
- (**deps**) drop direct egui and tokio, block the portal future on pollster - (edcd273) - dcog989
- (**picker**) remove the colour value readout from the magnifier - (cafb08f) - dcog989
- (**ui**) sample slider gradient per stop - (69c20bc) - dcog989
- (**ui**) decompose main_window into toast, history, keys and panel modules - (4e835f8) - dcog989
- (**ui**) render icons with the lucide-icons font instead of custom drawing - (3f87356) - dcog989
- (**ui**) recreate the settings/palette/broom icons with original geometry - (96df6bb) - dcog989
- (**ui**) collapse nested if in the picker overlay - (ae0bd98) - dcog989
- (**ui**) add icon_slot and move Lucide icon data to ui::icons - (279ab3e) - dcog989
- (**ui**) derive slider gradients from a channel helper - (8f655a4) - dcog989
- (**ui**) drop redundant MAGNIFIER_PIXEL_SIZE constant - (125161f) - dcog989
- (**ui**) use Rc<Cell> for the picker outcome - (d85ad9b) - dcog989
- (**ui**) compute relative luminance via palette Xyz - (045de03) - dcog989
- (**ui**) extract pure pixel math into ui::pixels - (b009d3c) - dcog989
- (**ui**) split MainWindow::ui into panel methods; encapsulate Toast and History - (a07963a) - dcog989
- centralize color formatting and clipboard copy - (f410d2b) - dcog989
- fix clippy lints in capture backends and picker overlay - (007fe4a) - dcog989
- own LaunchMode in config and drop unused ColorFormat ValueEnum - (fea7e5a) - dcog989
#### Miscellaneous Chores
- (**hooks**) enforce conventional commit messages with cog verify - (1063522) - dcog989
- (**logging**) install a tracing subscriber and instrument the picker path - (9a91b3b) - dcog989
- (**release**) sync the version across Cargo.toml, PKGBUILD and .SRCINFO on bump - (ac80acb) - dcog989
- (**ui**) remove saturation tooltip - (3280da0) - dcog989
- fmt - (23f15b9) - dcog989
- format - (ce4a39b) - dcog989
- fmt - (c7674cc) - dcog989
- fmt - (d93c192) - dcog989
- remove winit - (e406927) - dcog989
- format - (01e0865) - dcog989
- updates - (7dbcaab) - dcog989
- launch - (8a42df3) - dcog989

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).