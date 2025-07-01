use std::io::{self, Write};

/// Prompts the user for a yes/no confirmation.
pub fn confirm(prompt: &str) -> Result<bool, io::Error> {
    let mut input = String::new();
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase() == "y")
}
