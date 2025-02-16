use std::fs;
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

pub fn pause() {
    use std::io::Read;
    println!("waiting for user input...");
    std::io::stdin().read(&mut [0]).unwrap();
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

pub fn find_ffmpeg_path() -> Option<path::PathBuf> {
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