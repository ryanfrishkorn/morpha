mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::setup()?;
    Ok(())
}
