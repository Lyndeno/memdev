use memdev::memory::Memory;
fn main() {
    let mem = Memory::new().unwrap();
    println!("{:?}", mem);
}
