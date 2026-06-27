use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (document, errors) = rusty_me::parse_document_lossy(input.trim_end());
    let normalized = document.encode();
    println!("{normalized}");

    if !errors.is_empty() {
        let mut stderr = io::stderr().lock();
        for error in errors {
            writeln!(stderr, "warning: {error}")?;
        }
    }

    Ok(())
}
