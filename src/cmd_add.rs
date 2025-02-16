use crate::common;

use std::fs;
use std::path;
use std::process;
use std::collections::HashMap;

use audiotags as atags;
use log;

pub fn run(uris : &[String]) -> common::Result<()> {
    assert!(!uris.is_empty());
    if let Some(ytdlp_path) = common::find_ytdlp_path() {
        let ffmpeg_path = common::find_ffmpeg_path();
        let ffmpeg_path_ref = ffmpeg_path.as_ref().map(|x| &**x);
        log::info!("downloading files using installation: {}", ytdlp_path.display());
        let uris_n = uris.len();
        for (i, uri) in uris.iter().enumerate() {
            log::info!("task [{} / {}]", i, uris_n);
            fetch_uri(&ytdlp_path, uri)?;
        }
    } else {
        log::error!("an executable to `yt-dlp` is required for this command, aborting");
        log::info!("make sure `yt-dlp` or `youtube-dl` is in your PATH\n\
                    alternatively, add `yt-dlp = <path>` to your `catty.toml`");
    }
    Ok(())
}

fn fetch_uri(ytdlp_path : &path::Path, uri : &str) -> common::Result<()> {
    let mut proc = process::Command::new(ytdlp_path);
    proc.args([
        "--parse-metadata", "%(release_year|):%(date)s",
        "--embed-metadata",  // grab as much metadata as we can get
        "--embed-thumbnail", // grab the thumbnail, too
        "-f", "bestaudio",   // skip video download, we don't need it
        // output in a specific format:
        "-o", "%(artist,creator,uploader,uploader_id|Unknown)s - %(title,track,fulltitle,webpage_url_basename|Unnamed)s.%(ext)s"
    ]);
    proc.arg(uri);
    proc.stdout(process::Stdio::inherit()); // keep writing output
    proc.stderr(process::Stdio::inherit());
    log::debug!("running process with args: {:?}", proc.get_args());
    let output = proc.output()?;
    if !output.status.success() {
        log::warn!("recieved non-zero exit code, see output window");
    }
    Ok(())
}

enum AudioFileType {
    Audio,
    Image(atags::MimeType),
    Unknown,
}

fn try_get_filetype(ext : &str) -> AudioFileType {
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        | "3gp" | "aa" | "aac" | "aax" | "act" | "aiff" | "alac" | "amr"
        | "ape" | "au" | "awb" | "dss" | "dvf" | "flac" | "gsm"
        | "iklax" | "ivs" | "m4a" | "m4b" | "m4p" | "mmf" | "movpkg"
        | "mp3" | "mpc" | "msv" | "nmf" | "ogg" | "opus" | "ra" | "raw"
        | "rf64" | "sln" | "tta" | "voc" | "vox" | "wav" | "wma" | "wv"
        | "webm" | "8svx" | "cda"
        => AudioFileType::Audio,
        "bmp" => AudioFileType::Image(atags::MimeType::Bmp),
        "gif" => AudioFileType::Image(atags::MimeType::Gif),
        "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp"
        => AudioFileType::Image(atags::MimeType::Jpeg),
        "apng" | "png" => AudioFileType::Image(atags::MimeType::Png),
        "tif" | "tiff" => AudioFileType::Image(atags::MimeType::Tiff),
        _ => AudioFileType::Unknown,
    }
}