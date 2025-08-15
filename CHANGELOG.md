# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of termcopy utility
- OSC52 escape sequence support for clipboard copying
- File input support for copying file contents to clipboard
- stdin piping support for copying piped input to clipboard
- Command line argument parsing using argh
- Base64 encoding for clipboard data
- Comprehensive testing suite with unit and integration tests
- GitHub Actions CI workflow for automated testing and building
- Development environment with justfile for task automation
- Nix package support with flake configuration
- MIT License
- CI build badge in README

### Changed
- Enhanced nix package configuration with proper metadata
- Improved development workflow with linting and formatting tools

### Fixed
- Simplified nix configuration by removing unnecessary rust overlay
