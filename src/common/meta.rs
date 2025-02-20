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
    pub track_number : Option<(usize, String)>,
    pub title : Option<String>,
    pub file_name : Option<String>,
}

impl TrackMeta {
    pub fn get_author<'a>(&'a self) -> Option<&'a str> {
        if let Some(album_author) = &self.album_author {
            return Some(album_author);
        }
        if self.artists.len() > 1 {
            return Some(&self.artists[0]);
        }
        if !self.artists.is_empty() {
            let author = &self.artists[0];
            if !author.eq_ignore_ascii_case(DEFAULT_AUTHOR)
            && !author.eq_ignore_ascii_case(DEFAULT_AUTHOR_ID) {
                return Some(author);
            }
        }
        None
    }

}

pub const DEFAULT_CATEGORY : &'static str = ".other";
pub const DEFAULT_AUTHOR : &'static str = "unknown";
pub const DEFAULT_AUTHOR_ID : &'static str = "id"; // "in development"
pub const DEFAULT_TITLE : &'static str = "untitled";

pub fn get_category_name(author : &str) -> &'static str {
    if author.is_empty() {
        return DEFAULT_CATEGORY;
    }
    match author.chars().next().unwrap().to_ascii_uppercase() {
        'A'..='F' => "A-F",
        'G'..='K' => "G-K",
        'L'..='P' => "L-P",
        'Q'..='U' => "Q-U",
        'V'..='Z' => "V-Z",
        _ => DEFAULT_CATEGORY
    }
}

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
            // i considered having '—' be a separator, but i think they're used
            // too commonly in japanese text to make it reliable
            //
            // cautiously adding them as a fallback should be good enough
            re_split : regex::Regex::new(r"\s-\s|\s–\s").unwrap(),
            re_split_fallback : regex::Regex::new(r"\s-|-\s|\s–|–\s|\s—\s|\s::\s|\s~\s").unwrap(),
            // artist t+pazolite does this weird thing where instead of
            //
            //  artist1 & artist2 - songName
            //
            // it's one of
            //
            //  artist1 vs artist2 - songName
            //  artist1 - songName (vs artist2)
            //
            // not sure what to do about that case... except ignore it! yipee!
            re_split_artist : regex::Regex::new(r",\s|;\s|\sand\s|\s[&+xX]\s|\x00").unwrap(),
            re_split_feat : regex::Regex::new(r",?\s[fF]e?a?t\.?\s").unwrap(),
            re_split_feat_end : regex::Regex::new(r"[\(\[\{]\s*[fF]e?a?t\.?\s").unwrap(),
            cache : HashSet::new(),
            artists : Vec::new(),
            features : Vec::new(),
            album : None,
            album_author : None,
            track_number : None,
            title : None,
            file_name : None,
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
        let artists = artists.trim();
        if artists.is_empty() {
            return;
        }
        let mut artists_parts = self.re_split_feat.splitn(artists, 2);
        let new_artists = artists_parts.next().unwrap().trim();
        if let Some(features) = artists_parts.next() {
            // oops! more featured artists!
            let features = features.trim();
            for feature in self.re_split_artist.split(features) {
                Self::add_artist(feature, &mut self.cache, &mut self.features);
            }
        }
        for artist in self.re_split_artist.split(new_artists) {
            // TODO: remove this or fix it for titles like: cool Guy - cool Guy theme
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

    fn from_album(&mut self, album : &str) {
        let album = album.trim();
        if album.is_empty() {
            return;
        }
        impl_metadata!(self.album, album.to_string())
    }

    fn from_album_author(&mut self, album_author : &str) {
        let album_author = album_author.trim();
        if album_author.is_empty() {
            return;
        }
        impl_metadata!(self.album_author, album_author.to_string())
    }

    fn from_track_number(&mut self, track_number : usize) {
        // the space at the end of {:0>2} is necessary!
        impl_metadata!(self.track_number,
                (track_number, format!("{:0>2} ", track_number)))
    }

    fn from_file_name(&mut self, file_name : &str) {
        impl_metadata!(self.file_name, file_name.to_string())
    }

    fn from_title(&mut self, title : &str) {
        let title = title.trim();
        if title.is_empty() {
            return;
        }
        let mut title_parts = self.re_split_feat_end.splitn(title, 2);
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
    if let Some(file_name) = file_path.file_name().and_then(|x| x.to_str()) {
        meta.from_file_name(file_name);
    }
    // parse audio tags
    let dirty_tag = audiotags::Tag::new().read_from_path(file_path);
    let mut tag_artist = None;
    let mut tag_album = None;
    let mut tag_title = None;
    match dirty_tag {
        Ok(tag) => {
            tag_artist = tag.artist().map(String::from);
            tag_album = tag.album_title().map(String::from);
            tag_title = tag.title().map(String::from);
            // these tags can be added immediately, because the file stem is
            // unlikely to contain them
            tag.album_artist().map(|x| meta.from_album_author(x));
            tag.track_number().map(|x| meta.from_track_number(x as usize));
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
    if let Some(mut file_stem) = dirty_stem {
        if let Some((_, number_prefix)) = &meta.track_number {
            if file_stem.starts_with(number_prefix) {
                file_stem = file_stem[number_prefix.len()..]
                        .trim().trim_start_matches('-');
            }
        }
        let (artist, title) = stem_split(&meta, file_stem);
        if let Some(artist) = artist {
            stem_artist = Some(artist.to_string());
        }
        // try and infer album name from filepath
    'check:
        {
            if tag_album.is_none() {
                if let Some(dir_album) = file_path
                        .parent()
                        .and_then(|x| x.file_name())
                        .and_then(|x| x.to_str()) {
                    if let (Some(album), title) = stem_split(&meta, title) {
                        if album == dir_album {
                            stem_album = Some(album.to_string());
                            stem_title = Some(title.to_string());
                            break 'check;
                        }
                    }
                    if let Some(artist) = meta.album_author.as_ref()
                            .or(stem_artist.as_ref())
                            .or(tag_artist.as_ref()) {
                        if let Some(dir_artist) = file_path
                                .parent().unwrap().parent()
                                .and_then(|x| x.file_name())
                                .and_then(|x| x.to_str()) {
                            if artist.eq_ignore_ascii_case(dir_artist) {
                                stem_album = Some(dir_album.to_string());
                                stem_title = Some(title.to_string());
                                meta.from_album_author(dir_artist);
                                break 'check;
                            }
                        }
                    }
                }
            }
            stem_title = Some(title.to_string());
        }
    } else {
        log::warn!("failed to get stem for file '{}', skipping", file_path.display());
    }
    // patch the tag title, because Bandcamp actually gets this info wrong
    if let Some(title) = &tag_title {
        if let (Some(artist), title) = stem_split(&meta, title) {
            let mut trim_title = false;
            if let Some(expect_artist) = &tag_artist {
                trim_title = trim_title || expect_artist == artist;
            }
            if let Some(expect_artist) = &stem_artist {
                trim_title = trim_title || expect_artist == artist;
            }
            if trim_title {
                tag_title = Some(title.to_string());
            }
        }
    }
    // now apply metadata
    tag_album.as_ref().map(|x| meta.from_album(x));
    stem_album.as_ref().map(|x| meta.from_album(x));
    tag_title.as_ref().map(|x| meta.from_title(x));
    stem_title.as_ref().map(|x| meta.from_title(x));
    stem_artist.as_ref().map(|x| meta.from_artist(x)); // order is important here!
    tag_artist.as_ref().map(|x| meta.from_artist(x));
    Ok(meta)
}

fn stem_split<'a>(meta : &TrackMeta, stem : &'a str) -> (Option<&'a str>, &'a str) {
    let mut stem_parts = meta.re_split.splitn(stem, 2).map(|x| x.trim());
    let fst = stem_parts.next().unwrap();
    if let Some(snd) = stem_parts.next() {
        return (Some(fst), snd);
    }
    // probably has no author, but try the fallback just incase!
    let mut stem_parts = meta.re_split_fallback.splitn(fst, 2).map(|x| x.trim());
    let fst = stem_parts.next().unwrap();
    if let Some(snd) = stem_parts.next() {
        return (Some(fst), snd);
    }
    (None, fst)
}