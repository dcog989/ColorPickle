build:
    cargo build --release

lint:
    cargo clippy --workspace --all-targets -- -D warnings

# clippy type-checks too, so there's no separate `cargo check` step.
check: lint
    cargo fmt --all -- --check
    cargo test

package:
    packaging/build.sh

# Fixed /usr prefix: packaging/colorpickle.desktop pins Exec=/usr/bin/colorpickle, which the KWin D-Bus allowlist matches against.
install: build
    sudo install -Dm755 target/release/colorpickle /usr/bin/colorpickle
    sudo install -Dm644 packaging/colorpickle.desktop /usr/share/applications/colorpickle.desktop
    sudo install -Dm644 colorpickle.svg /usr/share/icons/hicolor/scalable/apps/colorpickle.svg
    sudo install -Dm644 assets/colorpickle.png /usr/share/icons/hicolor/256x256/apps/colorpickle.png
    sudo update-desktop-database /usr/share/applications
