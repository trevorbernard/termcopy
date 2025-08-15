use std::io::{self, Write};
use base64::{engine::general_purpose, Engine as _};

fn main() -> io::Result<()>{
    let data = general_purpose::STANDARD.encode(b"hello, world!");
    let osc52 = format!("\x1b]52;c;{}\x07", data);
    let mut stdout = io::stdout();
    stdout.write_all(osc52.as_bytes())?;
    stdout.flush()?;
    Ok(())
}
