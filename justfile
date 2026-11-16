# Verify the repo: the same lint, format check and tests CI runs.
default: check

# Lint, format check and tests.
check: lint format-check test

# Clippy, warnings as errors.
lint:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# Rewrite formatting.
format:
    cargo fmt --all

# Fail if anything needs formatting.
format-check:
    cargo fmt --all -- --check

# Test suite.
test:
    cargo test --locked --workspace

# Release build.
build:
    cargo build --release

# Debug build and run.
run:
    cargo run

# Remove build artifacts.
clean:
    cargo clean

# .deb, .rpm, AppImage and tarball into target/.
package:
    packaging/build.sh

# Fixed /usr prefix: packaging/colorpickle.desktop pins Exec=/usr/bin/colorpickle, which the KWin D-Bus allowlist matches against.
install: build
    sudo install -Dm755 target/release/colorpickle /usr/bin/colorpickle
    sudo install -Dm644 packaging/colorpickle.desktop /usr/share/applications/colorpickle.desktop
    sudo install -Dm644 assets/colorpickle.svg /usr/share/icons/hicolor/scalable/apps/colorpickle.svg
    sudo install -Dm644 assets/colorpickle.png /usr/share/icons/hicolor/256x256/apps/colorpickle.png
    sudo update-desktop-database /usr/share/applications

# Refresh dependencies to the latest allowed versions.
update:
    cargo upgrade && cargo update --verbose

# Bump the version from conventional commits and tag it.
release:
    cog bump --auto

# Preview dependency updates above the semver ranges.
outdated:
    cargo outdated -w
