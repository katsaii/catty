use std::path;
use std::fs;
use std::env;
use std::collections::{ HashMap, HashSet };
use crate::common;

pub fn run(
    file_paths : &[String],
    _clean_dirs : bool,
    _clean_files : bool,
    yes : bool,
) -> common::Result<()> {
    sort_files(file_paths, yes)?;
    Ok(())
}

fn sort_files(file_paths : &[String], yes : bool) -> common::Result<()> {
    let mut collection_authors = HashMap::new();
    let mut file_meta_map = HashMap::new();
    let mut db = common::infer::Database::new();
    common::glob_foreach_many(file_paths, |file| {
        let file_meta = common::meta::parse(file)?;
        let file_location = if let Some(x) = db.add_file(file) { x } else {
            log::warn!("failed to load file, skipping: {}", file.display());
            return Ok(());
        };
        // register the authors of a collection
        if let Some(album) = &file_meta.album {
            if let Some(album_expect) = file.parent().and_then(|x| x.file_name()) {
                if album_expect.eq_ignore_ascii_case(album) {
                    if let Some(author) = &file_meta.album_author {
                        let authors = collection_authors
                                .entry(file_location.id_collection)
                                .or_insert_with(|| HashSet::new());
                        authors.insert(author.to_string());
                    }
                }
            }
        }
        // keep track of file metadata
        file_meta_map.insert(file_location.id, file_meta);
        Ok(())
    })?;
    let (mut collections, files) = db.complete();
    collections.retain(|x| x.has_files);
    collections.sort_by_key(|x| x.depth);
    let mut collection_moved = HashSet::new();
    let working_dir = env::current_dir().and_then(|x| fs::canonicalize(x))?;
    // move entire collections
    for collection in &collections {
        if let Some(id_parent) = &collection.id_parent {
            if collection_moved.contains(id_parent) {
                collection_moved.insert(collection.id);
                continue; // file has already been moved
            }
        }
        if working_dir.starts_with(collection.path.as_path()) {
            continue; // don't rename paths that contain the working directory
        }
        let author = if let Some(authors) = collection_authors.get(&collection.id) {
            if authors.len() == 1 {
                authors.iter().next().unwrap()
            } else {
                continue
            }
        } else {
            continue
        };
        let collection_name = collection.path.file_name().unwrap();
        let mut dest_path = path::PathBuf::new();
        dest_path.push(common::meta::get_category_name(author));
        dest_path.push(author);
        dest_path.push(collection_name);
        let src_path = get_rel_path(&working_dir, &collection.path);
        // confirm rename
        let unchanged = dest_path.as_os_str().eq_ignore_ascii_case(src_path.as_os_str());
        if unchanged {
            log::info!("file is unchanged, skipping: {}", src_path.display());
        } else {
            log::info!("moving from    '{}'\n         to => '{}'",
                    src_path.display(), dest_path.display());
            if yes || common::ask_confirm() {
                fs::create_dir_all(dest_path.parent().unwrap())?;
                fs::rename(src_path, dest_path)?;
                collection_moved.insert(collection.id);
            }
        }
    }
    // move individual files
    for file in &files {
        if collection_moved.contains(&file.id_collection) {
            continue; // file has already been moved
        }
        let file_meta = &file_meta_map[&file.id];
        // add author
        let mut dest_path = path::PathBuf::new();
        if let Some(author) = file_meta.get_author() {
            dest_path.push(common::meta::get_category_name(author));
            dest_path.push(author);
        } else {
            dest_path.push(common::meta::DEFAULT_CATEGORY);
            dest_path.push(".unknown");
        }
        // add album
        if let Some(album) = &file_meta.album {
            dest_path.push(album);
        }
        dest_path.push(file.path.file_name().unwrap());
        let src_path = get_rel_path(&working_dir, &file.path);
        // confirm rename
        let unchanged = dest_path.as_os_str().eq_ignore_ascii_case(src_path.as_os_str());
        if unchanged {
            log::info!("file is unchanged, skipping: {}", src_path.display());
        } else {
            log::info!("moving from    '{}'\n         to => '{}'",
                    src_path.display(), dest_path.display());
            if yes || common::ask_confirm() {
                fs::create_dir_all(dest_path.parent().unwrap())?;
                fs::rename(src_path, dest_path)?;
            }
        }
    }
    Ok(())
}

fn get_rel_path<'a>(cwd : &path::Path, file : &'a path::Path) -> &'a path::Path {
    file.strip_prefix(cwd).unwrap_or(&file)
}