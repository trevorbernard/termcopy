use argh::FromArgs;
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, Read, Write};

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

fn base64_encode_stream<R: Read + ?Sized, W: Write>(reader: &mut R, writer: W) -> io::Result<()> {
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

fn copy_to_clipboard(source: &mut dyn Read, dest: &mut dyn Write) -> io::Result<()> {
    dest.write_all(OSC52_PREFIX.as_bytes())?;
    base64_encode_stream(source, &mut *dest)?;
    dest.write_all(OSC52_SUFFIX.as_bytes())?;
    dest.flush()
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    match &args.file {
        Some(path) => copy_to_clipboard(&mut File::open(path)?, &mut out),
        None => copy_to_clipboard(&mut io::stdin().lock(), &mut out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    fn generate_osc52_sequence(data: &[u8]) -> String {
        let encoded = general_purpose::STANDARD.encode(data);
        format!("{}{}{}", OSC52_PREFIX, encoded, OSC52_SUFFIX)
    }

    #[test]
    fn test_generate_osc52_sequence() {
        let result = generate_osc52_sequence(b"hello world");
        assert_eq!(result, "\x1b]52;c;aGVsbG8gd29ybGQ=\x07");
    }

    #[test]
    fn test_generate_osc52_sequence_empty() {
        let result = generate_osc52_sequence(b"");
        assert_eq!(result, "\x1b]52;c;\x07");
    }

    #[test]
    fn test_generate_osc52_sequence_binary() {
        let result = generate_osc52_sequence(&[0x00, 0x01, 0x02, 0xFF]);
        assert_eq!(result, "\x1b]52;c;AAEC/w==\x07");
    }

    #[test]
    fn test_base64_encode_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"test file content";
        temp_file.write_all(test_content)?;

        let mut output = Vec::new();
        base64_encode_stream(&mut File::open(temp_file.path())?, &mut output)?;

        let expected = general_purpose::STANDARD.encode(test_content);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_base64_encode_empty_file() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut output = Vec::new();
        base64_encode_stream(&mut File::open(temp_file.path())?, &mut output)?;

        assert_eq!(String::from_utf8(output).unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_base64_encode_large_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_content = vec![b'x'; 10000];
        temp_file.write_all(&large_content)?;

        let mut output = Vec::new();
        base64_encode_stream(&mut File::open(temp_file.path())?, &mut output)?;

        let expected = general_purpose::STANDARD.encode(&large_content);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_base64_encode_cursor() -> io::Result<()> {
        let test_input = b"stdin test data";
        let mut cursor = Cursor::new(test_input);

        let mut output = Vec::new();
        base64_encode_stream(&mut cursor, &mut output)?;

        let expected = general_purpose::STANDARD.encode(test_input);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_osc52_sequence_format() {
        let test_cases = vec![
            (b"a".as_slice(), "YQ=="),
            (b"ab".as_slice(), "YWI="),
            (b"abc".as_slice(), "YWJj"),
            (b"hello".as_slice(), "aGVsbG8="),
        ];

        for (input, expected_base64) in test_cases {
            let result = generate_osc52_sequence(input);
            let expected = format!("{}{}{}", OSC52_PREFIX, expected_base64, OSC52_SUFFIX);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_osc52_sequence_contains_correct_parts() {
        let test_data = b"test";
        let result = generate_osc52_sequence(test_data);

        assert!(result.starts_with(OSC52_PREFIX));
        assert!(result.ends_with(OSC52_SUFFIX));

        let base64_part = &result[OSC52_PREFIX.len()..result.len() - OSC52_SUFFIX.len()];
        let decoded = general_purpose::STANDARD.decode(base64_part).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_streaming_produces_same_result_as_batch() -> io::Result<()> {
        let test_data = b"hello world streaming test";
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(test_data)?;

        let expected = generate_osc52_sequence(test_data);

        let mut streamed_output = Vec::new();
        copy_to_clipboard(&mut File::open(temp_file.path())?, &mut streamed_output)?;

        assert_eq!(String::from_utf8(streamed_output).unwrap(), expected);
        Ok(())
    }
}
