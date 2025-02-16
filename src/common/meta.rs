use std::path;
use crate::common;

use audiotags;

enum Confidence {
    Poor,
    Average,
    Good,
}

type TrackField<T> = Option<(T, Confidence)>;

#[derive(Default)]
pub struct TrackMeta {
    pub artist : TrackField<String>,
    pub album : TrackField<String>,
    pub number : TrackField<usize>,
    pub title : TrackField<String>,
}

impl TrackMeta {
    pub fn from_stem(file_stem : &str) -> Self {
        let parts = file_stem.splitn(3, " - ").collect::<Vec<_>>();
        let mut artist = None;
        let mut album = None;
        let number = None;
        let mut title = None;
        match parts.as_slice() {
            // could only really be the track title
            [raw_title] => {
                title = Some((raw_title.to_string(), Confidence::Poor));
            },
            [raw_artist, raw_title] => {
                artist = Some((raw_artist.to_string(), Confidence::Average));
                title = Some((raw_title.to_string(), Confidence::Average));
            },
            [raw_artist, raw_album, raw_title] => {
                artist = Some((raw_artist.to_string(), Confidence::Average));
                album = Some((raw_album.to_string(), Confidence::Average));
                title = Some((raw_title.to_string(), Confidence::Average));
            },
            _ => unreachable!(),
        }
        Self { artist, album, number, title }
    }

    pub fn from_audiotags(file_path : &path::Path) -> common::Result<Self> {
        unimplemented!()
    }
}