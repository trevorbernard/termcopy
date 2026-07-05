# termcopy

[![CI](https://github.com/trevorbernard/termcopy/actions/workflows/ci.yml/badge.svg)](https://github.com/trevorbernard/termcopy/actions/workflows/ci.yml)

A utility program that enables clipboard copying using OSC52 escape sequences. Supports both file input and stdin piping for terminal-based clipboard operations.

## Usage

### Installation

#### Prebuilt binaries

Download the archive for your platform from the [latest release](https://github.com/trevorbernard/termcopy/releases/latest):

| Platform | Asset |
|---|---|
| Linux x86_64 (static) | `termcopy-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 (static) | `termcopy-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `termcopy-<version>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `termcopy-<version>-x86_64-pc-windows-msvc.zip` |

On Linux and macOS:

```bash
VERSION=v0.2.0
TARGET=x86_64-unknown-linux-musl # or aarch64-unknown-linux-musl, aarch64-apple-darwin
curl -LO "https://github.com/trevorbernard/termcopy/releases/download/${VERSION}/termcopy-${VERSION}-${TARGET}.tar.gz"
curl -LO "https://github.com/trevorbernard/termcopy/releases/download/${VERSION}/checksums.sha256"
sha256sum -c --ignore-missing checksums.sha256
tar xzf "termcopy-${VERSION}-${TARGET}.tar.gz"
install -m 755 termcopy ~/.local/bin/
```

On macOS, use `shasum -a 256 -c --ignore-missing checksums.sha256` instead of `sha256sum`.

Release archives also carry signed build provenance; with the [GitHub CLI](https://cli.github.com/) you can verify an archive was built by this repository's release workflow:

```bash
gh attestation verify "termcopy-${VERSION}-${TARGET}.tar.gz" --repo trevorbernard/termcopy
```

On Windows, extract the `.zip` and place `termcopy.exe` somewhere on your `PATH`.

#### From source

```bash
just install
```

### Running the Program

Copy file contents to clipboard:
```bash
just run filename.txt
```

Copy from stdin:
```bash
echo "hello world" | just run
```

### Development

Build the project:
```bash
just build
```

Run tests and linting:
```bash
just dev
```

View all available commands:
```bash
just
```

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.
