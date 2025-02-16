use crate::common;

use std::path;
use std::process;

use log;

pub fn run(uris : &[String]) -> common::Result<()> {
    assert!(!uris.is_empty());
    if let Some(ytdlp_path) = common::find_ytdlp_path() {
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
        "--embed-metadata",  // grab as much metadata as we can get
        "--embed-thumbnail", // grab the thumbnail, too
        // skip video download, we don't need it
        // also try and find the best audio format
        "-f", "ba[ext=flac]/ba[ext=wav]/ba[ext=mp3]/ba",
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