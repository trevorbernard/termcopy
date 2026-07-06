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

### Examples

Copy a file:
```bash
termcopy notes.txt
```

Copy the output of a command:
```bash
echo "hello world" | termcopy
git rev-parse HEAD | termcopy
pwd | termcopy
```

Copy from a remote machine to your **local** clipboard — the escape
sequence travels back over ssh and your local terminal interprets it:
```bash
ssh dev-box termcopy /etc/hostname
ssh dev-box 'grep ERROR /var/log/app.log | termcopy'
```

Copy from inside a script whose output is captured — the escape sequence
goes to your terminal, not into the capture, so the copy still happens and
the log stays clean:
```bash
./release.sh > release.log   # a termcopy call inside still reaches the clipboard
```

Copy *and* keep the data flowing with `--tee` — input passes through to
stdout unchanged, like `tee(1)` with the clipboard as the second
destination (requires a controlling terminal):
```bash
curl -s api/token | termcopy --tee | jq .    # view it formatted, copy the raw bytes
make 2>&1 | termcopy --tee | less            # page the build output, copy it too
./deploy.sh | termcopy --tee >> deploy.log   # log it and copy it in one run
```

Inside tmux, termcopy works with `set -g set-clipboard on` in your
`.tmux.conf` (tmux's default of `external` ignores OSC52 from applications).

### Output destination

By default the escape sequence goes to stdout when stdout is a terminal.
When stdout is redirected (`termcopy file > log`, command substitution, a
pipeline), it goes to the controlling terminal instead, so the copy still
happens and the capture stays clean; if there is no controlling terminal
(e.g. `ssh host termcopy` without a remote tty), it falls back to stdout so
the sequence reaches your local terminal. Use `--output stdout` or
`--output tty` to pin the destination explicitly:
```bash
termcopy --output stdout file.txt   # always stdout, e.g. for scripting
termcopy --output tty file.txt      # always the controlling terminal
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
