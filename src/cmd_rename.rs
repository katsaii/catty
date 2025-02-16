use crate::common;

pub fn run(
    format : &str,
    artist : bool,
    album : bool,
    number : bool,
    title : bool,
) -> common::Result<()> {
    println!("fmt {} a {} A {} n {} t {}", format, artist, album, number, title);
    Ok(())
}