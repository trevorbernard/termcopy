use argh::FromArgs;
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

const OSC52_PREFIX: &str = "\x1b]52;c;";
const OSC52_SUFFIX: &str = "\x07";

#[derive(FromArgs)]
/// Copy data to clipboard using OSC52 escape sequences
struct Args {
    #[argh(switch, short = 'v', long = "version")]
    /// show version information
    version: bool,

    #[argh(positional)]
    /// file to copy (reads from stdin if not provided)
    file: Option<String>,
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

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // The base64 payload contains no newlines, so a bare (line-buffered)
    // StdoutLock would flush roughly once per KiB.
    let mut out = BufWriter::new(io::stdout().lock());
    match &args.file {
        Some(path) => copy_to_clipboard(File::open(path)?, &mut out),
        None => copy_to_clipboard(io::stdin().lock(), &mut out),
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
