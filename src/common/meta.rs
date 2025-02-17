use std::path;
use crate::common;

use audiotags;
use log;
use regex;

#[derive(Debug)]
pub struct TrackMeta {
    pub artists : Vec<String>,
    pub features : Vec<String>,
    pub album : Option<String>,
    pub number : Option<usize>,
    pub title : String,
}

pub fn parse(file_path : &path::Path) -> common::Result<TrackMeta> {
    let mut artist = None;
    let mut album = None;
    let mut number = None;
    let mut title = None;
    // audiotags
    match parse_from_audiotags(file_path) {
        Ok((tag_artist, tag_album, tag_number, tag_title)) => {
            artist = tag_artist;
            album = tag_album;
            number = tag_number;
            title = tag_title;
        },
        Err(audiotags::Error::IOError(err)) => return Err(Box::new(err)),
        Err(err) => {
            log::warn!(
                "failed to get metadata for file '{}', skipping:\n{}",
                file_path.display(), err
            );
        }
    }
    // from filename
    if artist.is_none() || album.is_none() || title.is_none() {
        if let Some(file_stem) = file_path.file_stem().and_then(|x| x.to_str()) {
            let (stem_artist, stem_album, stem_title) = parse_from_stem(file_stem);
            if artist.is_none() { artist = stem_artist; }
            if album.is_none() { album = stem_album; }
            if title.is_none() { title = stem_title; }
        } else {
            log::warn!("failed to get stem for file '{}', skipping", file_path.display());
        }
    }
    // parse current metadata into contributing artists and featured artists
    let (mut artists, features, title) = parse_artist_info(artist, title);
    if artists.is_empty() {
        artists.push("Unknown".to_owned());
    }
    Ok(TrackMeta { artists, features, album, number, title : title.unwrap_or("Undefined".to_owned()) })
}

fn trim_str(x : &str) -> Option<String> {
    match x.trim() {
        "" => None,
        otherwise => Some(otherwise.to_string()),
    }
}

fn parse_from_audiotags(file_path : &path::Path) -> audiotags::Result<(
    Option<String>,
    Option<String>,
    Option<usize>,
    Option<String>,
)> {
    let tag = audiotags::Tag::new().read_from_path(file_path)?;
    let tag_artist = tag.artist().and_then(trim_str);
    let tag_album = tag.album_title().and_then(trim_str);
    let tag_number = tag.track_number().map(|x| x as usize);
    let tag_title = tag.title().and_then(trim_str);
    Ok((tag_artist, tag_album, tag_number, tag_title))
}

fn parse_from_stem(file_stem : &str) -> (
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let parts = file_stem.splitn(3, " - ").map(|x| x.trim()).collect::<Vec<_>>();
    let mut artist = None;
    let mut album = None;
    let title;
    match parts.as_slice() {
        // could only really be the track title
        [raw_title] => {
            title = trim_str(raw_title);
        },
        [raw_artist, raw_title] => {
            artist = trim_str(raw_artist);
            title = trim_str(raw_title);
        },
        [raw_artist, raw_album, raw_title] => {
            artist = trim_str(raw_artist);
            album = trim_str(raw_album);
            title = trim_str(raw_title);
        },
        _ => unreachable!(),
    }
    (artist, album, title)
}

fn parse_artist_info(in_artist : Option<String>, in_title : Option<String>) -> (
    Vec<String>,
    Vec<String>,
    Option<String>,
) {
    let mut artists = vec![];
    let mut features = vec![];
    let mut title = None;
    // TODO: move this regex somewhere else so its not being compiled every time
    let splitter = regex::Regex::new(r", |; | [&+xX] |\x00").unwrap();
    let splitter_feat = regex::Regex::new(r"[\(\[\{]\s*[fF]eat\.?\s").unwrap();
    // parse features
    if let Some(in_title) = &in_title {
        let mut split = splitter_feat.splitn(in_title, 2);
        let new_title = split.next();
        let in_feature = split.next();
        if let (Some(new_title), Some(in_feature)) = (new_title, in_feature) {
            title = Some(new_title.trim().to_string());
            let in_feature = in_feature.trim().trim_end_matches([')', ']', '}']);
            for feature in splitter.split(in_feature) {
                features.push(feature.trim().to_string());
            }
        }
    }
    // parse artists
    if let Some(in_artist) = &in_artist {
        for artist in splitter.split(in_artist) {
            let artist = artist.trim().to_string();
            if !features.contains(&artist) {
                artists.push(artist);
            }
        }
    }
    // no features, keep the same title
    if title.is_none() {
        title = in_title;
    }
    (artists, features, title)
}