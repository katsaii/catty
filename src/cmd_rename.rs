use std::fmt::Write;
use std::fs;
use std::path;
use crate::common;

use sanitise_file_name;
use log;

pub fn run(
    patterns : &[String],
    format : &str,
    artist : bool,
    album : bool,
    number : bool,
    title : bool,
) -> common::Result<()> {
    common::glob_foreach_many(patterns, |file| {
        rename_file(file, format, artist, album, number, title)
    })
}

fn rename_file(
    file : &path::Path,
    format : &str,
    artist : bool,
    album : bool,
    number : bool,
    title : bool,
) -> common::Result<()> {
    let file_meta = common::meta::parse(file)?;
    // build new stem
    let mut new_stem = String::new();
    let mut first = true;
    for fmt_chr in format.chars() {
        match fmt_chr {
            'a' if artist => {
                if !first { new_stem.push_str(" - "); }
                new_stem.push_str(file_meta.artists.join(", ").as_str());
                first = false;
            },
            'A' if album => {
                if let Some(album) = &file_meta.album {
                    if !first { new_stem.push_str(" - "); }
                    new_stem.push_str(album);
                    first = false;
                }
            },
            'n' if number => {
                if let Some(number) = &file_meta.number {
                    if !first { new_stem.push_str(" - "); }
                    new_stem.push_str(&number.to_string());
                    new_stem.push_str(" ");
                    first = true;
                }
            },
            't' if title => {
                let title = &file_meta.title;
                if !first { new_stem.push_str(" - "); }
                new_stem.push_str(title);
                if artist && !file_meta.features.is_empty() {
                    new_stem.push_str(" [feat. ");
                    new_stem.push_str(file_meta.features.join(", ").as_str());
                    new_stem.push_str("]");
                }
                first = false;
            },
            _ => (),
        }
    }
    if let Some(ext) = file.extension().and_then(|x| x.to_str()) {
        new_stem.push('.');
        new_stem.push_str(ext);
    }
    let new_stem = sanitise_file_name::sanitise(&new_stem);
    let new_file = file.with_file_name(new_stem);
    if file == new_file {
        log::info!("file '{}' is unchanged, skipping", file.display());
    } else {
        // confirm rename
        log::info!("renaming from    {}\n           to => {}", file.display(), new_file.display());
        if common::ask_confirm() {
            fs::rename(file, new_file)?;
        }
    }
    Ok(())
}