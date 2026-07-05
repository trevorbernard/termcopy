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

# Type-check without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run linting (clippy and audit)
lint:
    cargo clippy --all-targets -- -D warnings
    cargo audit

# Clean build artifacts
clean:
    cargo clean
    @rm -f result result-*

# Install the binary locally
install:
    cargo install --path .

# Development workflow: format, lint, test
dev: fmt lint test

# Full CI workflow: prereq, format, lint, test, build
ci: prereq dev build

# Create a git tag for release (version must match Cargo.toml)
tag version:
    @actual="$(cargo pkgid | sed 's/.*[#@]//')"; \
      [ "$actual" = "{{version}}" ] || { echo "error: version {{version}} does not match Cargo.toml ($actual)" >&2; exit 1; }
    git tag -a v{{version}} -m "Release v{{version}}"
    git push origin v{{version}}

# Build release binaries for all supported platforms
build-release-all:
    nix build --system x86_64-linux -o result-x86_64-linux
    nix build --system aarch64-darwin -o result-aarch64-darwin
    cp result-x86_64-linux/bin/termcopy termcopy-x86_64-linux
    cp result-aarch64-darwin/bin/termcopy termcopy-aarch64-darwin
    sha256sum termcopy-* > checksums.sha256

# Prepare for release: format, test, build all platforms
release version: dev build-release-all
    @echo "Release {{version}} prepared. Run 'just tag {{version}}' to create and push the tag."
