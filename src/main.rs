use argh::FromArgs;
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

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

/// Streams data from stdin through base64 encoding to the provided writer.
fn stream_from_stdin_to_writer<W: Write>(writer: W) -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Streams file contents through base64 encoding to the provided writer.
fn stream_from_file_to_writer<W: Write>(path: &Path, writer: W) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(&mut file, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Writes the OSC52 prefix escape sequence to stdout.
fn write_osc52_prefix() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b]52;c;")?;
    Ok(())
}

/// Writes the OSC52 suffix escape sequence to stdout and flushes.
fn write_osc52_suffix() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x07")?;
    stdout.flush()?;
    Ok(())
}

/// Streams data to clipboard using OSC52 escape sequences.
/// Reads from stdin if file is None, otherwise reads from the specified file.
fn stream_to_clipboard(file: Option<&Path>) -> io::Result<()> {
    write_osc52_prefix()?;
    match file {
        Some(path) => stream_from_file_to_writer(path, io::stdout())?,
        None => stream_from_stdin_to_writer(io::stdout())?,
    }
    write_osc52_suffix()
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    stream_to_clipboard(args.file.as_deref().map(Path::new))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn generate_osc52_sequence(data: &[u8]) -> String {
        let encoded = general_purpose::STANDARD.encode(data);
        format!("\x1b]52;c;{}\x07", encoded)
    }

    #[test]
    fn test_osc52_sequence_generation() {
        let test_cases = vec![
            (b"hello world" as &[u8], "aGVsbG8gd29ybGQ="),
            (b"", ""),
            (&[0x00, 0x01, 0x02, 0xFF], "AAEC/w=="),
            (b"a", "YQ=="),
            (b"ab", "YWI="),
            (b"abc", "YWJj"),
        ];

        for (input, expected_base64) in test_cases {
            let result = generate_osc52_sequence(input);
            let expected = format!("\x1b]52;c;{}\x07", expected_base64);
            assert_eq!(result, expected, "Failed for input: {:?}", input);

            // Verify structure
            assert!(result.starts_with("\x1b]52;c;"));
            assert!(result.ends_with("\x07"));

            // Verify base64 encoding is correct
            if !expected_base64.is_empty() {
                let base64_part = &result[7..result.len() - 1];
                let decoded = general_purpose::STANDARD
                    .decode(base64_part)
                    .expect("base64 decoding should succeed");
                assert_eq!(decoded, input);
            }
        }
    }

    #[test]
    fn test_stream_from_file_to_writer() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"test file content";
        temp_file.write_all(test_content)?;

        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path(), &mut output)?;

        let expected_base64 = general_purpose::STANDARD.encode(test_content);
        assert_eq!(
            String::from_utf8(output).expect("output should be valid UTF-8"),
            expected_base64
        );
        Ok(())
    }

    #[test]
    fn test_stream_from_file_to_writer_empty() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path(), &mut output)?;

        assert_eq!(
            String::from_utf8(output).expect("output should be valid UTF-8"),
            ""
        );
        Ok(())
    }

    #[test]
    fn test_stream_from_file_nonexistent() {
        let mut output = Vec::new();
        let result = stream_from_file_to_writer(Path::new("/nonexistent/file/path"), &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_from_file_large() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_content = vec![b'x'; 10000];
        temp_file.write_all(&large_content)?;

        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path(), &mut output)?;

        let expected_base64 = general_purpose::STANDARD.encode(&large_content);
        assert_eq!(
            String::from_utf8(output).expect("output should be valid UTF-8"),
            expected_base64
        );
        Ok(())
    }

    #[test]
    fn test_stream_from_stdin_to_writer() -> io::Result<()> {
        let test_input = b"stdin test data";
        let mut cursor = Cursor::new(test_input);

        let mut output = Vec::new();
        {
            let mut encoder = EncoderWriter::new(&mut output, &general_purpose::STANDARD);
            io::copy(&mut cursor, &mut encoder)?;
            encoder.finish()?;
        }

        let expected_base64 = general_purpose::STANDARD.encode(test_input);
        assert_eq!(
            String::from_utf8(output).expect("output should be valid UTF-8"),
            expected_base64
        );
        Ok(())
    }

    #[test]
    fn test_streaming_produces_same_result_as_original() -> io::Result<()> {
        let test_data = b"hello world streaming test";
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(test_data)?;

        let original_result = generate_osc52_sequence(test_data);

        let mut streamed_output = Vec::new();
        streamed_output.extend_from_slice(b"\x1b]52;c;");
        stream_from_file_to_writer(temp_file.path(), &mut streamed_output)?;
        streamed_output.extend_from_slice(b"\x07");

        assert_eq!(
            String::from_utf8(streamed_output).expect("output should be valid UTF-8"),
            original_result
        );
        Ok(())
    }
}
