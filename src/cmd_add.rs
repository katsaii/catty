use crate::common;

use std::path;
use std::process;

use log;

pub fn run(uris : &[String], is_playlist : bool) -> common::Result<()> {
    assert!(!uris.is_empty());
    if let Some(ytdlp_path) = common::find_ytdlp_path() {
        log::info!("downloading files using installation: {}", ytdlp_path.display());
        let uris_n = uris.len();
        for (i, uri) in uris.iter().enumerate() {
            log::info!("task [{} / {}]", i, uris_n);
            fetch_uri(&ytdlp_path, uri, is_playlist)?;
        }
    } else {
        log::error!("an executable to `yt-dlp` is required for this command, aborting");
        log::info!("make sure `yt-dlp` or `youtube-dl` is in your PATH\n\
                    alternatively, add `yt-dlp = <path>` to your `catty.toml`");
    }
    Ok(())
}

fn fetch_uri(ytdlp_path : &path::Path, uri : &str, is_playlist : bool) -> common::Result<()> {
    let mut proc = process::Command::new(ytdlp_path);
    if is_playlist {
        proc.args([
            "--yes-playlist",
            "--parse-metadata", "%(track_number,playlist_index|)s:%(meta_track)s",
        ]);
    } else {
        proc.arg("--no-playlist");
    }
    proc.args([
        "--embed-metadata",  // grab as much metadata as we can get
        "--embed-thumbnail", // grab the thumbnail, too
        // skip video download, we don't need it
        // also try and find the best audio format
        "-f", "ba[ext=flac]/ba[ext=wav]/ba[ext=mp3]/ba",
    ]);
    // make sure the file path is descriptive
    let mut file_name = (if is_playlist { "%(playlist|Playlist)s/" } else { "" }).to_string();
    file_name.push_str("%(artist,creator,uploader,uploader_id|Unknown)s - %(title,track,fulltitle,webpage_url_basename|Unnamed)s.%(ext)s");
    proc.args(["-o", file_name.as_str()]);
    // submit command with the URI
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