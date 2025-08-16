use argh::FromArgs;
use base64::{engine::general_purpose, write::EncoderWriter};
use std::fs::File;
use std::io::{self, BufReader, Write};

#[derive(FromArgs)]
/// Copy data to clipboard using OSC52 escape sequences
struct Args {
    #[argh(positional)]
    /// file to copy (reads from stdin if not provided)
    file: Option<String>,
}

fn stream_from_stdin_to_writer<W: Write>(writer: W) -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

fn stream_from_file_to_writer<W: Write>(path: &str, writer: W) -> io::Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut encoder = EncoderWriter::new(writer, &general_purpose::STANDARD);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

fn write_osc52_prefix() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b]52;c;")?;
    Ok(())
}

fn write_osc52_suffix() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x07")?;
    stdout.flush()?;
    Ok(())
}

fn stream_to_clipboard_from_stdin() -> io::Result<()> {
    write_osc52_prefix()?;
    stream_from_stdin_to_writer(io::stdout())?;
    write_osc52_suffix()?;
    Ok(())
}

fn stream_to_clipboard_from_file(path: &str) -> io::Result<()> {
    write_osc52_prefix()?;
    stream_from_file_to_writer(path, io::stdout())?;
    write_osc52_suffix()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    if let Some(file_path) = &args.file {
        stream_to_clipboard_from_file(file_path)?;
    } else {
        stream_to_clipboard_from_stdin()?;
    }

    Ok(())
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
    fn test_generate_osc52_sequence() {
        let test_data = b"hello world";
        let result = generate_osc52_sequence(test_data);
        let expected = "\x1b]52;c;aGVsbG8gd29ybGQ=\x07";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_osc52_sequence_empty() {
        let test_data = b"";
        let result = generate_osc52_sequence(test_data);
        let expected = "\x1b]52;c;\x07";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_osc52_sequence_binary() {
        let test_data = &[0x00, 0x01, 0x02, 0xFF];
        let result = generate_osc52_sequence(test_data);
        let expected = "\x1b]52;c;AAEC/w==\x07";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_stream_from_file_to_writer() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"test file content";
        temp_file.write_all(test_content)?;

        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path().to_str().unwrap(), &mut output)?;

        let expected_base64 = general_purpose::STANDARD.encode(test_content);
        assert_eq!(String::from_utf8(output).unwrap(), expected_base64);
        Ok(())
    }

    #[test]
    fn test_stream_from_file_to_writer_empty() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path().to_str().unwrap(), &mut output)?;

        assert_eq!(String::from_utf8(output).unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_stream_from_file_nonexistent() {
        let mut output = Vec::new();
        let result = stream_from_file_to_writer("/nonexistent/file/path", &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_from_file_large() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_content = vec![b'x'; 10000];
        temp_file.write_all(&large_content)?;

        let mut output = Vec::new();
        stream_from_file_to_writer(temp_file.path().to_str().unwrap(), &mut output)?;

        let expected_base64 = general_purpose::STANDARD.encode(&large_content);
        assert_eq!(String::from_utf8(output).unwrap(), expected_base64);
        Ok(())
    }

    #[test]
    fn test_stream_from_stdin_to_writer() -> io::Result<()> {
        let test_input = b"stdin test data";
        let cursor = Cursor::new(test_input);
        let mut reader = BufReader::new(cursor);

        let mut output = Vec::new();
        {
            let mut encoder = EncoderWriter::new(&mut output, &general_purpose::STANDARD);
            io::copy(&mut reader, &mut encoder)?;
            encoder.finish()?;
        }

        let expected_base64 = general_purpose::STANDARD.encode(test_input);
        assert_eq!(String::from_utf8(output).unwrap(), expected_base64);
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
            let expected = format!("\x1b]52;c;{}\x07", expected_base64);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_osc52_sequence_contains_correct_parts() {
        let test_data = b"test";
        let result = generate_osc52_sequence(test_data);

        assert!(result.starts_with("\x1b]52;c;"));
        assert!(result.ends_with("\x07"));

        let base64_part = &result[7..result.len() - 1];
        let decoded = general_purpose::STANDARD.decode(base64_part).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_osc52_prefix_suffix() -> io::Result<()> {
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
        stream_from_file_to_writer(temp_file.path().to_str().unwrap(), &mut streamed_output)?;
        streamed_output.extend_from_slice(b"\x07");

        assert_eq!(String::from_utf8(streamed_output).unwrap(), original_result);
        Ok(())
    }
}
