use std::io::{self, Write};

use super::TuiResult;

pub fn print_header(msg: &str) {
    println!("{}", msg);
}

/// print prompt (no newline)
pub fn print_prompt(prompt: &str) -> TuiResult<()> {
    print!("{}", prompt);
    io::stdout().lock().flush()?;
    Ok(())
}

pub fn readline() -> TuiResult<(usize, String)> {
    let mut raw_line = String::new();

    let bytes_read = io::stdin().read_line(&mut raw_line)?;
    Ok((bytes_read, raw_line.trim().to_string()))
}
pub fn readline_with_prompt(prompt: &str) -> TuiResult<String> {
    print_prompt(prompt)?;
    Ok(readline()?.1)
}
