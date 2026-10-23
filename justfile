# List available commands
default:
    @just --list

# Run in development mode
dev *args:
    cargo run -- {{args}}

# Run and auto-rebuild on file changes (requires `cargo install cargo-watch`)
watch *args:
    cargo watch -x "run -- {{args}}"

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

# Bundle into Qwerkeys.app and DMG (a `-debug` version suffix uses the DEBUG binary)
bundle version="v0.0.0-dev":
    #!/usr/bin/env sh
    case "{{version}}" in
        *-debug) cargo build ;;
        *) cargo build --release ;;
    esac
    ./scripts/bundle.sh {{version}}

# Reset Input Monitoring & Accessibility grants for com.notenderdreams.qwerkeys (use before `just install` to re-test onboarding)
reset-permissions:
    tccutil reset ListenEvent com.notenderdreams.qwerkeys || sudo tccutil reset ListenEvent com.notenderdreams.qwerkeys
    tccutil reset Accessibility com.notenderdreams.qwerkeys || sudo tccutil reset Accessibility com.notenderdreams.qwerkeys
    @echo "✓ Reset Input Monitoring & Accessibility for com.notenderdreams.qwerkeys"

# Clean, build release, bundle, and install Qwerkeys.app into /Applications (pair with `just reset-permissions` to re-test onboarding)
install version="v0.0.0-dev": (bundle version)
    @rm -rf /Applications/Qwerkeys.app
    @cp -R Qwerkeys.app /Applications/Qwerkeys.app
    @version="{{version}}"; echo "✓ Installed /Applications/Qwerkeys.app"; \
        echo "  DMG kept in repo: Qwerkeys-${version#v}.dmg"    

# Reset privacy grants, then install the DEBUG build (bundled .app always shows onboarding)
install-debug: reset-permissions (install "v0.0.0-debug")

# Clean build target, bundle outputs, and generated icons
clean:
    cargo clean
    rm -rf Qwerkeys.app dmg-staging *.dmg packaging/AppIcon.iconset assets/AppIcon.icns assets/app-icon.png

# Install git hooks into .git/hooks/
setup-hooks:
    ./scripts/setup-hooks.sh
