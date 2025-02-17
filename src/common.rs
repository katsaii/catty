pub mod meta;

use std::fs;
use std::io::{stdout, Write};
use std::path;

use which::which;
use toml;
use log;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const CONFIG_PATH : &'static str = "catty.toml";

pub fn find_config(key : &str) -> Option<String> {
    let file = fs::read_to_string(CONFIG_PATH).ok()?;
    let value = file.parse::<toml::Table>().ok()?;
    let toml_value = value.get(key)?.as_str()?;
    return Some(toml_value.to_owned());
}

pub fn ask_confirm() -> bool {
    log::warn!("do you accept? [Y/n]");
    let mut input = String::new();
    stdout().flush().unwrap();
    loop {
        std::io::stdin().read_line(&mut input).unwrap();
        match input.as_str().trim() {
            "y" | "Y" => return true,
            "n" | "N" => return false,
            "" => continue,
            otherwise => {
                log::error!("invalid input '{}', assuming (n)o", otherwise);
                return false;
            },
        }
    }
}

pub fn find_ytdlp_path() -> Option<path::PathBuf> {
    if let Some(ytdlp_config_path) = find_config("yt-dlp") {
        match fs::exists(&ytdlp_config_path) {
            Ok(exists) => if exists {
                return Some(ytdlp_config_path.into())
            },
            Err(msg) => log::error!("{}", msg),
        }
        log::warn!("installation does not exist at: {}\n\
                    looking for installation in PATH", ytdlp_config_path);
    }
    if let Ok(ytdlp_path) = which("yt-dlp") {
        return Some(ytdlp_path);
    }
    log::warn!("cannot find executable to `yt-dlp`, falling back to `youtube-dl`...");
    if let Ok(youtubedl_path) = which("youtube-dl") {
        return Some(youtubedl_path);
    }
    return None;
}

pub fn _find_ffmpeg_path() -> Option<path::PathBuf> {
    if let Some(ffmpeg_config_path) = find_config("ffmpeg") {
        match fs::exists(&ffmpeg_config_path) {
            Ok(exists) => if exists {
                return Some(ffmpeg_config_path.into())
            },
            Err(msg) => log::error!("{}", msg),
        }
        log::warn!("installation does not exist at: {}\n\
                    looking for installation in PATH", ffmpeg_config_path);
    }
    if let Ok(ytdlp_path) = which("ffmpeg") {
        return Some(ytdlp_path);
    }
    log::warn!("cannot find executable to `ffmpeg`, some behaviour may be degraded");
    return None;
}

pub fn ext_is_audio_file(ext : &str) -> bool {
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        | "3gp" | "aa" | "aac" | "aax" | "act" | "aiff" | "alac" | "amr"
        | "ape" | "au" | "awb" | "dss" | "dvf" | "flac" | "gsm"
        | "iklax" | "ivs" | "m4a" | "m4b" | "m4p" | "mmf" | "movpkg"
        | "mp3" | "mpc" | "msv" | "nmf" | "ogg" | "opus" | "ra" | "raw"
        | "rf64" | "sln" | "tta" | "voc" | "vox" | "wav" | "wma" | "wv"
        | "webm" | "8svx" | "cda"
        => true,
        _ => false,
    }
}