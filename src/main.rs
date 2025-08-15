use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use base64::{engine::general_purpose, Engine as _};

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
    let args: Vec<String> = env::args().collect();

    let input_data = if args.len() > 1 {
        read_from_file(&args[1])?
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
