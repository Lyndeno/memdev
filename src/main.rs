use memdev::memory::Memory;
use memdev::Result;
fn main() -> Result<()> {
    let mem = Memory::new()?;
    println!("{mem:?}");
    Ok(())
}
