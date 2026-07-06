use argh::{FromArgValue, FromArgs};
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Read, Write};

const OSC52_PREFIX: &str = "\x1b]52;c;";
const OSC52_SUFFIX: &str = "\x07";

#[cfg(windows)]
const TTY_PATH: &str = "CONOUT$";
#[cfg(not(windows))]
const TTY_PATH: &str = "/dev/tty";

#[derive(FromArgs)]
/// Copy data to clipboard using OSC52 escape sequences
struct Args {
    #[argh(switch, short = 'v', long = "version")]
    /// show version information
    version: bool,

    #[argh(option, short = 'o', long = "output")]
    /// where to write the escape sequence: "stdout" or "tty" (default:
    /// stdout if it is a terminal, otherwise the controlling tty,
    /// otherwise stdout)
    output: Option<OutputTarget>,

    #[argh(switch, short = 't', long = "tee")]
    /// pass input through to stdout while copying; the escape sequence
    /// goes to the controlling tty (requires one)
    tee: bool,

    #[argh(positional)]
    /// file to copy (reads from stdin if not provided)
    file: Option<String>,
}

/// Mirrors everything read from `reader` to `tee`.
struct TeeReader<R, W> {
    reader: R,
    tee: W,
}

impl<R: Read, W: Write> Read for TeeReader<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.tee.write_all(&buf[..n])?;
        Ok(n)
    }
}

enum OutputTarget {
    Stdout,
    Tty,
}

impl FromArgValue for OutputTarget {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "stdout" => Ok(Self::Stdout),
            "tty" => Ok(Self::Tty),
            _ => Err(format!("expected \"stdout\" or \"tty\", got \"{value}\"")),
        }
    }
}

fn open_tty() -> io::Result<File> {
    File::options()
        .write(true)
        .open(TTY_PATH)
        .map_err(|e| io::Error::new(e.kind(), format!("cannot open {TTY_PATH}: {e}")))
}

fn clipboard_writer(target: Option<OutputTarget>) -> io::Result<Box<dyn Write>> {
    match target {
        Some(OutputTarget::Stdout) => Ok(Box::new(io::stdout().lock())),
        Some(OutputTarget::Tty) => Ok(Box::new(open_tty()?)),
        None => {
            let stdout = io::stdout().lock();
            if stdout.is_terminal() {
                return Ok(Box::new(stdout));
            }
            // stdout is redirected, so send the escape sequence to the
            // controlling terminal instead of polluting the capture. If there
            // is no controlling terminal, the pipe may still lead to one
            // (e.g. `ssh host termcopy` without a remote tty), so fall back.
            match open_tty() {
                Ok(tty) => Ok(Box::new(tty)),
                Err(_) => Ok(Box::new(stdout)),
            }
        }
    }
}

fn base64_encode_stream(mut reader: impl Read, writer: impl Write) -> io::Result<()> {
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

fn copy_to_clipboard(source: impl Read, mut dest: impl Write) -> io::Result<()> {
    dest.write_all(OSC52_PREFIX.as_bytes())?;
    base64_encode_stream(source, &mut dest)?;
    dest.write_all(OSC52_SUFFIX.as_bytes())?;
    dest.flush()
}

fn run(source: impl Read, escape: impl Write, tee: bool) -> io::Result<()> {
    if tee {
        let mut stdout = io::stdout().lock();
        let source = TeeReader {
            reader: source,
            tee: &mut stdout,
        };
        copy_to_clipboard(source, escape)?;
        stdout.flush()
    } else {
        copy_to_clipboard(source, escape)
    }
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let escape_dest = if args.tee {
        // In tee mode stdout carries the data, so the escape sequence must
        // go to the controlling terminal — there is no stdout fallback.
        if matches!(args.output, Some(OutputTarget::Stdout)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--output stdout cannot be combined with --tee: stdout carries the data",
            ));
        }
        Box::new(open_tty()?) as Box<dyn Write>
    } else {
        clipboard_writer(args.output)?
    };

    // The base64 payload contains no newlines, so an unbuffered writer
    // would flush roughly once per KiB.
    let mut escape = BufWriter::new(escape_dest);
    match &args.file {
        Some(path) => run(File::open(path)?, &mut escape, args.tee),
        None => run(io::stdin().lock(), &mut escape, args.tee),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::io::Cursor;

    fn osc52(data: &[u8]) -> String {
        let mut out = Vec::new();
        copy_to_clipboard(Cursor::new(data), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn test_osc52_sequence_format() {
        let cases: &[(&[u8], &str)] = &[
            (b"", "\x1b]52;c;\x07"),
            (b"a", "\x1b]52;c;YQ==\x07"),
            (b"ab", "\x1b]52;c;YWI=\x07"),
            (b"abc", "\x1b]52;c;YWJj\x07"),
            (b"hello world", "\x1b]52;c;aGVsbG8gd29ybGQ=\x07"),
            (&[0x00, 0x01, 0x02, 0xFF], "\x1b]52;c;AAEC/w==\x07"),
        ];

        for (input, expected) in cases {
            assert_eq!(osc52(input), *expected);
        }
    }

    #[test]
    fn test_tee_mirrors_input_and_copies() {
        let mut mirrored = Vec::new();
        let mut escape = Vec::new();
        let source = TeeReader {
            reader: Cursor::new(b"hello world"),
            tee: &mut mirrored,
        };
        copy_to_clipboard(source, &mut escape).unwrap();

        assert_eq!(mirrored, b"hello world");
        assert_eq!(escape, b"\x1b]52;c;aGVsbG8gd29ybGQ=\x07");
    }

    #[test]
    fn test_streaming_matches_batch_encoding() {
        let data = vec![b'x'; 10000];
        let expected = format!(
            "{}{}{}",
            OSC52_PREFIX,
            general_purpose::STANDARD.encode(&data),
            OSC52_SUFFIX
        );

        assert_eq!(osc52(&data), expected);
    }
}
