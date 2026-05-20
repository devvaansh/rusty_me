use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let normalized = rusty_me::normalize_strict(input.trim_end())?;
    println!("{normalized}");

    Ok(())
}
