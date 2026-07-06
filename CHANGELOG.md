# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-07-05

### Added
- `--tee` flag to mirror input through stdout while copying, so termcopy can
  sit inside a pipeline like tee(1) with the clipboard as a second destination
- `--output <stdout|tty>` flag to pin the escape-sequence destination explicitly
- Output cascade: the escape sequence goes to stdout when it is a terminal,
  otherwise to the controlling tty, otherwise falls back to stdout so remote
  copies over ssh without a tty still reach the local terminal

### Fixed
- Nix docker image no longer pulls the Rust toolchain into its closure,
  shrinking it from over 2GB to 17MB

### Changed
- Simplified code, tests, and build tooling; hardened the release pipeline
  and nix builds

## [0.1.0] - 2025-08-16

### Added
- Initial release of termcopy utility
- OSC52 escape sequence support for clipboard copying
- File input support for copying file contents to clipboard
- stdin piping support for copying piped input to clipboard
- Command line argument parsing using argh
- Base64 encoding for clipboard data
- Comprehensive testing suite with unit and integration tests
- GitHub Actions CI workflow for automated testing and building
- GitHub Actions release workflow
- Development environment with justfile for task automation
- Nix package support with flake configuration
- MIT License
- CI build badge in README
- coreutils package for sha256sum support

### Changed
- Enhanced nix package configuration with proper metadata
- Improved development workflow with linting and formatting tools
- Implement streaming base64 encoding for memory efficiency
- Remove unnecessary BufReader wrappers for improved performance

### Fixed
- Simplified nix configuration by removing unnecessary rust overlay
- Correct release workflow trigger conditions
- Add contents write permission to release workflow
