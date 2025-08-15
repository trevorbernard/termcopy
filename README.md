# termcopy

[![CI](https://github.com/trevorbernard/termcopy/actions/workflows/ci.yml/badge.svg)](https://github.com/trevorbernard/termcopy/actions/workflows/ci.yml)

A utility program that enables clipboard copying using OSC52 escape sequences. Supports both file input and stdin piping for terminal-based clipboard operations.

## Usage

### Installation

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
