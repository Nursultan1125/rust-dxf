use std::fs::{File};
use std::io::{BufReader, Read};
mod lib;
fn main() -> std::io::Result<()> {
    let file = File::open("data/box12.sli")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let entities = lib::get_indexes(&contents);
    println!("{:?}", entities);
    println!("{:}", entities.len());
    println!("{:?}", entities.get(0).unwrap().node_id);
    Ok(())
}