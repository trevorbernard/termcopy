# List the available recipes
default:
  @just --list --justfile {{justfile()}}

# Check that all prerequisites are installed
prereq:
    @echo "Checking prerequisites..."
    @just --version
    @cargo --version
    @cargo fmt --version
    @cargo audit --version
    @cargo clippy --version
    @cargo nextest --version

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the project
run *args:
    cargo run -- {{args}}

# Run tests
test:
    cargo nextest run

# Check code formatting
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run linting (clippy and audit)
lint:
    cargo clippy
    cargo audit

# Clean build artifacts
clean:
    cargo clean
    @rm result >/dev/null 2>&1 || true

# Install the binary locally
install:
    cargo install --path .

# Show cargo version and project info
info:
    cargo --version
    @echo "Project: $(grep '^name' Cargo.toml | cut -d '"' -f 2)"
    @echo "Version: $(grep '^version' Cargo.toml | cut -d '"' -f 2)"

# Development workflow: format, lint, test
dev: fmt lint test

# Full CI workflow: prereq, clean, format, lint, test, build
ci: prereq clean fmt lint test build

# Create a git tag for release
tag version:
    git tag -a v{{version}} -m "Release v{{version}}"
    git push origin v{{version}}

# Build release binaries for all supported platforms
build-release-all:
    nix build --system x86_64-linux
    cp result/bin/termcopy termcopy-x86_64-linux
    nix build --system aarch64-darwin
    cp result/bin/termcopy termcopy-aarch64-darwin
    sha256sum termcopy-* > checksums.sha256

# Prepare for release: format, test, build all platforms
release version: fmt lint test build-release-all
    @echo "Release {{version}} prepared. Run 'just tag {{version}}' to create and push the tag."
