use std::path;
use std::collections::HashSet;
use crate::common;

use audiotags;
use log;
use regex;

#[derive(Debug)]
pub struct TrackMeta {
    re_split : regex::Regex,
    re_split_artist : regex::Regex,
    re_split_feat : regex::Regex,
    re_split_feat_end : regex::Regex,
    cache : HashSet<String>,
    pub artists : Vec<String>,
    pub features : Vec<String>,
    pub album : Option<String>,
    pub album_author : Option<String>,
    pub track_number : Option<usize>,
    pub title : Option<String>,
}

pub const DEFAULT_TITLE : &'static str = "Untitled";

macro_rules! impl_metadata {
    ($from:expr, $into:expr) => {
        if $from.is_none() {
            $from = Some($into);
        }
    }
}

impl TrackMeta {
    fn new() -> Self {
        Self {
            // i considered having '—' be separators, but i think they're used
            // too commonly in japanese text to make it reliable
            re_split : regex::Regex::new(r"\s-\s|\s–\s|\s::\s|\s~\s").unwrap(),
            re_split_artist : regex::Regex::new(r",\s|;\s|\sand\s|\svs\.?\s|\s[&+xX]\s|\x00").unwrap(),
            re_split_feat : regex::Regex::new(r"\s[fF]e?a?t\.?\s").unwrap(),
            re_split_feat_end : regex::Regex::new(r"[\(\[\{]\s*[fF]e?a?t\.?\s").unwrap(),
            cache : HashSet::new(),
            artists : Vec::new(),
            features : Vec::new(),
            album : None,
            album_author : None,
            track_number : None,
            title : None,
        }
    }

    fn add_artist(name : &str, cache : &mut HashSet<String>, collection : &mut Vec<String>) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        let name_lowercase = name.to_lowercase();
        if cache.contains(&name_lowercase) {
            return false;
        }
        cache.insert(name_lowercase);
        collection.push(name.to_string());
        true
    }

    fn from_artist(&mut self, artists : &str) {
        let mut artists_parts = self.re_split_feat.splitn(artists.trim(), 2);
        let new_artists = artists_parts.next().unwrap().trim();
        if let Some(features) = artists_parts.next() {
            // oops! more featured artists!
            let features = features.trim();
            for feature in self.re_split_artist.split(features) {
                Self::add_artist(feature, &mut self.cache, &mut self.features);
            }
        }
        for artist in self.re_split_artist.split(new_artists) {
            // TODO: remove this for fix it for titles like: cool Guy - cool Guy theme
            //if let Some(title) = &self.title {
            //    if title.contains(artist) {
            //        // strips visual noise by handling cases like:
            //        //   cool Guy, nobody - massive beat (nobody remix)
            //        // =>
            //        //   cool Guy - massive beat (nobody remix)
            //        continue;
            //    }
            //}
            Self::add_artist(artist, &mut self.cache, &mut self.artists);
        }
    }

    fn from_album(&mut self, album : &str) { impl_metadata!(self.album, album.trim().to_string()) }
    fn from_album_author(&mut self, album_author : &str) { impl_metadata!(self.album_author, album_author.trim().to_string()) }
    fn from_track_number(&mut self, track_number : usize) { impl_metadata!(self.track_number, track_number) }

    fn from_title(&mut self, title : &str) {
        let mut title_parts = self.re_split_feat_end.splitn(title.trim(), 2);
        let new_title = title_parts.next().unwrap().trim();
        if let Some(features) = title_parts.next() {
            let features = features.trim().trim_end_matches([')', ']', '}']);
            for feature in self.re_split_artist.split(features) {
                Self::add_artist(feature, &mut self.cache, &mut self.features);
            }
        }
        if self.title.is_none() {
            self.title = Some(new_title.to_owned());
        }
    }
}

pub fn parse(file_path : &path::Path) -> common::Result<TrackMeta> {
    let mut meta = TrackMeta::new();
    // parse audio tags
    let dirty_tag = audiotags::Tag::new().read_from_path(file_path);
    let mut tag_artist = None;
    let mut tag_album = None;
    let mut tag_album_author = None;
    let mut tag_number = None;
    let mut tag_title = None;
    match dirty_tag {
        Ok(tag) => {
            tag_artist = tag.artist().map(String::from);
            tag_album = tag.album_title().map(String::from);
            tag_album_author = tag.album_artist().map(String::from);
            tag_number = tag.track_number().map(|x| x as usize);
            tag_title = tag.title().map(String::from);
        }
        Err(audiotags::Error::IOError(err)) => return Err(Box::new(err)),
        Err(err) => {
            log::warn!(
                "failed to get metadata for file '{}'\nreason = {}",
                file_path.display(), err
            );
        }
    }
    // parse from file stem
    let dirty_stem = file_path.file_stem().and_then(|x| x.to_str());
    let mut stem_artist = None;
    let mut stem_album = None;
    let mut stem_title = None;
    if let Some(file_stem) = dirty_stem {
        let stem_parts = meta.re_split.splitn(file_stem, 3).map(|x| x.trim()).collect::<Vec<_>>();
        match stem_parts.as_slice() {
            // could only really be the track title
            [raw_title] => {
                stem_title = Some(raw_title.to_string());
            },
            [raw_artist, raw_title] => {
                stem_artist = Some(raw_artist.to_string());
                stem_title = Some(raw_title.to_string());
            },
            [raw_artist, raw_album, raw_title] => {
                stem_artist = Some(raw_artist.to_string());
                stem_album = Some(raw_album.to_string());
                stem_title = Some(raw_title.to_string());
            },
            _ => unreachable!(),
        }
    } else {
        log::warn!("failed to get stem for file '{}', skipping", file_path.display());
    }
    // now apply metadata
    stem_album.as_ref().map(|x| meta.from_album(x));
    tag_album.as_ref().map(|x| meta.from_album(x));
    tag_album_author.as_ref().map(|x| meta.from_album_author(x));
    tag_number.map(|x| meta.from_track_number(x));
    stem_title.as_ref().map(|x| meta.from_title(x));
    tag_title.as_ref().map(|x| meta.from_title(x));
    stem_artist.as_ref().map(|x| meta.from_artist(x));
    tag_artist.as_ref().map(|x| meta.from_artist(x));
    Ok(meta)
}

#[derive(Debug)]
pub struct TrackMeta_ {
    pub artists : Vec<String>,
    pub features : Vec<String>,
    pub album : Option<String>,
    pub number : Option<usize>,
    pub title : String,
}

pub fn parse_(file_path : &path::Path) -> common::Result<TrackMeta_> {
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
                "failed to get metadata for file '{}'\nreason = {}",
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
    Ok(TrackMeta_ { artists, features, album, number, title : title.unwrap_or("Undefined".to_owned()) })
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
    let splitter = regex::Regex::new(r", |; | and | [&+xX] |\x00").unwrap();
    let splitter_feat = regex::Regex::new(r"[\(\[\{]\s*[fF]e?a?t\.?\s").unwrap();
    let splitter_feat_2 = regex::Regex::new(r"\s[fF]e?a?t\.?\s").unwrap();
    // parse features
    if let Some(in_title) = &in_title {
        let mut split = splitter_feat.splitn(in_title, 2);
        let new_title = split.next().unwrap();
        if let Some(in_feature) = split.next() {
            title = Some(new_title.trim().to_string());
            let in_feature = in_feature.trim().trim_end_matches([')', ']', '}']);
            for feature in splitter.split(in_feature) {
                let feature = feature.trim().to_string();
                if !feature.is_empty() {
                    features.push(feature);
                }
            }
        }
    }
    // parse artists
    if let Some(in_artist) = &in_artist {
        let mut split = splitter_feat_2.splitn(in_artist, 2);
        let in_artist = split.next().unwrap();
        if let Some(in_feature) = split.next() {
            // OOPS! more featured artists, i lied
            for feature in splitter.split(in_feature) {
                let feature = feature.trim().to_string();
                if !feature.is_empty() {
                    features.push(feature);
                }
            }
        }
        for artist in splitter.split(in_artist) {
            let artist = artist.trim().to_string();
            if !artist.is_empty() && !features.contains(&artist) {
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