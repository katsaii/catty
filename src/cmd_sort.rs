use std::path;
use std::fs;
use std::collections::HashMap;
use crate::common;

pub fn run(
    file_paths : &[String],
    clean_dirs : bool,
    clean_files : bool,
) -> common::Result<()> {
    sort_files(file_paths)?;
    Ok(())
}

fn sort_files(file_paths : &[String]) -> common::Result<()> {
    let mut db = common::infer::Database::new();
    common::glob_foreach_many(file_paths, |file| {
        db.add_file(file);

        //let file_meta = common::meta::parse(file)?;
        //if file_meta.file_name.is_none() {
        //    log::warn!("file name contains invalid characters, skipping: {}", file.display());
        //    return Ok(());
        //}
        //let file_name = file_meta.file_name.as_ref().unwrap();
        //// add author
        //let mut dest_path = path::PathBuf::new();
        //if let Some(author) = file_meta.get_author() {
        //    dest_path.push(common::meta::get_category_name(author));
        //    dest_path.push(author);
        //} else {
        //    dest_path.push(common::meta::DEFAULT_CATEGORY);
        //    dest_path.push(".unknown");
        //}
        //// add album
        //if let Some(album) = &file_meta.album {
        //    dest_path.push(album);
        //}
        //println!("{}\t\t-\t{:?}", dest_path.display(), file_name);

        Ok(())
    })?;
    println!("{:#?}", db);
    let (collections, files) = db.complete();
    println!("col: {:#?}\nfil: {:#?}", collections, files);
    Ok(())
}

fn sort_file(file : &path::Path) -> common::Result<()> {
    let file_meta = common::meta::parse(file)?;
    let file_name = file_meta.file_name.unwrap();
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