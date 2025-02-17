use std::path;
use std::collections::HashSet;
use crate::common;

use audiotags;
use log;
use regex;

#[derive(Debug)]
pub struct TrackMeta {
    re_split : regex::Regex,
    re_split_fallback : regex::Regex,
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
            //
            // cautiously adding them as a fallback should be good enough
            re_split : regex::Regex::new(r"\s-\s|\s–\s").unwrap(),
            re_split_fallback : regex::Regex::new(r"\s-|-\s|\s–|–\s|\s—\s|\s::\s|\s~\s").unwrap(),
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
    let mut stem_album = None as Option<String>;
    let mut stem_title = None;
    if let Some(file_stem) = dirty_stem {
        let mut stem_parts = meta.re_split
                //.splitn(file_stem, 3) // see below: albums disabled
                .splitn(file_stem, 2)
                .map(|x| x.trim())
                .collect::<Vec<_>>();
        if stem_parts.len() == 1 {
            // probably has no author, but try the fallback just incase!
            stem_parts = meta.re_split_fallback
                    .splitn(stem_parts[0], 2)
                    .map(|x| x.trim())
                    .collect::<Vec<_>>();
        }
        match stem_parts.as_slice() {
            // could only really be the track title
            [raw_title] => {
                stem_title = Some(raw_title.to_string());
            },
            [raw_artist, raw_title] => {
                stem_artist = Some(raw_artist.to_string());
                stem_title = Some(raw_title.to_string());
            },
            // TODO: fix album parsing, right now it fails on songs like:
            //
            //   Speedcore Front Ost Berlin - Speedcore Symphonia Part II - Kindesschlaf.mp3
            //
            // where "Speedcore Symphonia Part II" is interpreted as the album
            // name, when this isn't actually true
            /*
            [raw_artist, raw_album, raw_title] => {
                stem_artist = Some(raw_artist.to_string());
                stem_album = Some(raw_album.to_string());
                stem_title = Some(raw_title.to_string());
            },
            */
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