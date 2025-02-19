use std::path;
use std::fs;
use crate::common;

pub fn run(
    file_paths : &[String],
    clean_dirs : bool,
    clean_files : bool,
) -> common::Result<()> {
    common::glob_foreach_many(file_paths, |file| {
        sort_file(file)
    })
}

fn sort_file(file : &path::Path) -> common::Result<()> {
    let file_meta = common::meta::parse(file)?;
    let file_name = file.file_name().and_then(|x| x.to_str()).unwrap();
    let mut new_file = file.to_path_buf();
    new_file.pop();
    // add author path
    let author = sanitise_file_name::sanitise(&file_meta.artists[0]);
    new_file.push(author);
    // add album (if it exists)
    if let Some(album) = &file_meta.album {
        let album = sanitise_file_name::sanitise(album);
        new_file.push(album);
    }
    // confirm rename
    log::info!("moving from    '{}'\n         to => '{}{}{}'",
            file.display(), new_file.display(), path::MAIN_SEPARATOR, file_name);
    if common::ask_confirm() {
        fs::create_dir_all(new_file.as_path())?;
        new_file.push(file_name);
        fs::rename(file, new_file)?;
    }
    Ok(())
}