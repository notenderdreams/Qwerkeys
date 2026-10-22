# List available commands
default:
    @just --list

# Run in development mode
dev *args:
    cargo run -- {{args}}

# Run release build
run *args:
    cargo run --release -- {{args}}

# Type-check without building binary
check:
    cargo check

# Build debug binary
build:
    cargo build

# Build optimized release binary
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run linter
lint:
    cargo clippy --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run all CI checks (format, clippy, tests)
ci: fmt-check lint test

# Regenerate AppIcon.icns and app-icon.png from assets/app-icon.svg
icon:
    ./scripts/generate-icon.swift

# Build release and bundle into Qwerkeys.app and DMG
bundle version="v0.0.0-dev": build-release
    ./scripts/bundle.sh {{version}}

# Clean build target, bundle outputs, and generated icons
clean:
    cargo clean
    rm -rf Qwerkeys.app dmg-staging *.dmg packaging/AppIcon.iconset assets/AppIcon.icns assets/app-icon.png

# Install git hooks into .git/hooks/
setup-hooks:
    ./scripts/setup-hooks.sh
