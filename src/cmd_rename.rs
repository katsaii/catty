use crate::common;

pub fn run(
    format : &str,
    artist : bool,
    album : bool,
    number : bool,
    title : bool,
) -> common::Result<()> {
    let meta_path = std::path::Path::new("05 - 100 gecs, Fall Out Boy, Craig Owens, Nicole Dollanganger - hand crushed by a mallet (Remix) [feat. Fall Out Boy, Craig Owens, Nicole Dollanganger].mp3");
    let meta = common::meta::parse(meta_path);
    println!("{:?}", meta);
    Ok(())
}