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

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    let input_data = if let Some(file_path) = &args.file {
        read_from_file(file_path)?
    } else {
        read_from_stdin()?
    };

    let data = general_purpose::STANDARD.encode(&input_data);
    let osc52 = format!("\x1b]52;c;{}\x07", data);
    let mut stdout = io::stdout();
    stdout.write_all(osc52.as_bytes())?;
    stdout.flush()?;
    Ok(())
}
