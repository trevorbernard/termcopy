use argh::FromArgs;
use base64::{Engine as _, engine::general_purpose};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

#[derive(FromArgs)]
/// Copy data to clipboard using OSC52 escape sequences
struct Args {
    #[argh(positional)]
    /// file to copy (reads from stdin if not provided)
    file: Option<String>,
}

fn read_from_stdin() -> io::Result<Vec<u8>> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn read_from_file(path: &str) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn generate_osc52_sequence(data: &[u8]) -> String {
    let encoded = general_purpose::STANDARD.encode(data);
    format!("\x1b]52;c;{}\x07", encoded)
}

fn write_to_clipboard(data: &[u8]) -> io::Result<()> {
    let osc52 = generate_osc52_sequence(data);
    let mut stdout = io::stdout();
    stdout.write_all(osc52.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    let input_data = if let Some(file_path) = &args.file {
        read_from_file(file_path)?
    } else {
        read_from_stdin()?
    };

    write_to_clipboard(&input_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
    fn test_read_from_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"test file content";
        temp_file.write_all(test_content)?;

        let result = read_from_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(result, test_content);
        Ok(())
    }

    #[test]
    fn test_read_from_file_empty() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let result = read_from_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(result, b"");
        Ok(())
    }

    #[test]
    fn test_read_from_file_nonexistent() {
        let result = read_from_file("/nonexistent/file/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_from_file_large() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_content = vec![b'x'; 10000];
        temp_file.write_all(&large_content)?;

        let result = read_from_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(result, large_content);
        Ok(())
    }

    #[test]
    fn test_read_from_stdin_mock() -> io::Result<()> {
        use std::io::{BufReader, Read};

        let test_input = b"stdin test data";
        let mut cursor = Cursor::new(test_input);
        let mut reader = BufReader::new(&mut cursor);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        assert_eq!(buffer, test_input);
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
}
