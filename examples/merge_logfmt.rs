use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (document, errors) = rusty_me::parse_document_lossy(input.trim_end());

    for error in &errors {
        eprintln!("warning: {error}");
    }

    let merged = document.merge();
    let sorted = merged.sorted();
    println!("{sorted}");

    Ok(())
}
