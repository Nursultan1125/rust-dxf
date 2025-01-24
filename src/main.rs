use std::fs::{File};
use std::io::{BufReader, Read};
mod lib;
fn main() -> std::io::Result<()> {
    let file = File::open("data/karimova01.sli")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let entities = lib::sli_to_json(&contents, None);
    println!("{:?}", entities);
    println!("{:}", entities.len());
    Ok(())
}