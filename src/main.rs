use argh::{FromArgValue, FromArgs};
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const OSC52_PREFIX: &str = "\x1b]52;c;";
const OSC52_SUFFIX: &str = "\x07";

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
    file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
enum Command {
    Version,
    Copy { input: Input, mode: Mode },
}

#[derive(Debug, PartialEq)]
enum Input {
    Stdin,
    File(PathBuf),
}

#[derive(Debug, PartialEq)]
enum Mode {
    /// `None` picks the destination automatically.
    Copy(Option<OutputTarget>),
    /// Input is mirrored to stdout; the escape sequence always goes to the tty.
    Tee,
}

impl TryFrom<Args> for Command {
    type Error = &'static str;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        if args.version {
            return Ok(Self::Version);
        }
        let mode = match (args.tee, args.output) {
            (true, Some(OutputTarget::Stdout)) => {
                return Err(
                    "--output stdout cannot be combined with --tee: stdout carries the data",
                )
            }
            (true, _) => Mode::Tee,
            (false, target) => Mode::Copy(target),
        };
        let input = args.file.map_or(Input::Stdin, Input::File);
        Ok(Self::Copy { input, mode })
    }
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

fn open_tty() -> io::Result<File> {
    File::options()
        .write(true)
        .open(TTY_PATH)
        .map_err(|e| io::Error::new(e.kind(), format!("cannot open {TTY_PATH}: {e}")))
}

fn escape_writer(mode: &Mode) -> io::Result<Box<dyn Write>> {
    match mode {
        Mode::Copy(Some(OutputTarget::Stdout)) => Ok(Box::new(io::stdout().lock())),
        Mode::Copy(Some(OutputTarget::Tty)) | Mode::Tee => Ok(Box::new(open_tty()?)),
        Mode::Copy(None) => {
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

/// Builds the complete escape sequence in memory. Nothing reaches the
/// terminal until the input has been fully read, so a read error can't leave
/// an unterminated OSC that swallows later output, and tee'd data can't be
/// interleaved into the payload when stdout and the tty are the same device.
fn osc52_sequence(mut source: impl Read) -> io::Result<Vec<u8>> {
    let mut encoder =
        EncoderWriter::new(OSC52_PREFIX.as_bytes().to_vec(), &general_purpose::STANDARD);
    io::copy(&mut source, &mut encoder)?;
    let mut seq = encoder.finish()?;
    seq.extend_from_slice(OSC52_SUFFIX.as_bytes());
    Ok(seq)
}

fn copy(source: impl Read, mode: Mode) -> io::Result<()> {
    let mut escape = escape_writer(&mode)?;
    let seq = match mode {
        Mode::Tee => {
            let mut stdout = io::stdout().lock();
            let seq = osc52_sequence(TeeReader {
                reader: source,
                tee: &mut stdout,
            })?;
            stdout.flush()?;
            seq
        }
        Mode::Copy(_) => osc52_sequence(source)?,
    };
    escape.write_all(&seq)?;
    escape.flush()
}

fn run(command: Command) -> io::Result<()> {
    match command {
        Command::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Copy {
            input: Input::Stdin,
            mode,
        } => copy(io::stdin().lock(), mode),
        Command::Copy {
            input: Input::File(path),
            mode,
        } => {
            let file = File::open(&path)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
            copy(file, mode)
        }
    }
}

fn main() -> ExitCode {
    let args: Args = argh::from_env();
    let result = Command::try_from(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        .and_then(run);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {e}", env!("CARGO_PKG_NAME"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::io::Cursor;

    fn osc52(data: &[u8]) -> String {
        String::from_utf8(osc52_sequence(Cursor::new(data)).unwrap()).unwrap()
    }

    fn parse(args: &[&str]) -> Result<Command, &'static str> {
        Command::try_from(Args::from_args(&["termcopy"], args).unwrap())
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
        let source = TeeReader {
            reader: Cursor::new(b"hello world"),
            tee: &mut mirrored,
        };
        let escape = osc52_sequence(source).unwrap();

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

    #[test]
    fn test_read_error_yields_no_sequence() {
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("boom"))
            }
        }
        assert!(osc52_sequence(Failing).is_err());
    }

    #[test]
    fn test_parse_args() {
        let file = |p: &str| Input::File(PathBuf::from(p));
        let copy = |input, mode| Ok(Command::Copy { input, mode });

        assert_eq!(
            parse(&["-v", "--tee", "-o", "stdout"]),
            Ok(Command::Version)
        );
        assert_eq!(parse(&[]), copy(Input::Stdin, Mode::Copy(None)));
        assert_eq!(
            parse(&["-o", "tty", "f"]),
            copy(file("f"), Mode::Copy(Some(OutputTarget::Tty)))
        );
        assert_eq!(parse(&["--tee"]), copy(Input::Stdin, Mode::Tee));
        assert_eq!(
            parse(&["--tee", "-o", "tty"]),
            copy(Input::Stdin, Mode::Tee)
        );
        assert!(parse(&["--tee", "-o", "stdout"]).is_err());
    }
}
