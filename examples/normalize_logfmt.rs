use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let document = rusty_me::parse_document_strict(input.trim_end())?;
    let normalized = document.encode();
    println!("{normalized}");

    Ok(())
}
