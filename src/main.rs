mod common;
mod cmd_add;
mod cmd_rename;
mod cmd_sort;

use std::env;

use clap::{Parser, Subcommand};
use colog;

/// Music file manager.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wrapper around `yt-dlp` that attempts to download an audio file in
    /// the highest quality, with as much metadata as it can grab.
    ///
    /// Will not download video files.
    Add {
        /// URIs that point to the files to download.
        #[arg(required = true)]
        uris : Vec<String>,
        /// Indicates that the files are part of a playlist or album.
        #[arg(short, long, group = "media-type")]
        playlist : bool,
    },
    /// Renames all audio files in the working directory so they are in a
    /// consistent format.
    Rename {
        /// Paths of the files to format. Supports GLOB syntax.
        #[arg(required = true)]
        patterns : Vec<String>,
        /// (a)rtist name, (A)lbum name, track (n)umber, track (t)itle
        #[arg(short, long, default_value = "aAnt")]
        format : String,
        /// Include the artist name in the format. [default: enabled]
        #[arg(long)]
        _artist : bool,
        /// Exclude the artist name from the format.
        #[arg(long = "no-artist", overrides_with = "_artist")]
        no_artist : bool,
        /// Include the album name in the format. [default: disabled]
        #[arg(long, overrides_with = "_no_album")]
        album : bool,
        /// Exclude the album name from the format.
        #[arg(long = "no-album")]
        _no_album : bool,
        /// Include the track number in the format. [default: disabled]
        #[arg(long, overrides_with = "_no_number")]
        number : bool,
        /// Exclude the track number from the format.
        #[arg(long = "no-number")]
        _no_number : bool,
        /// Include the track title in the format. [default: enabled]
        #[arg(long)]
        _title : bool,
        /// Exclude the artist title from the format.
        #[arg(long = "no-title", overrides_with = "_title")]
        no_title : bool,
    },
    /// Organise audio files in the working directory into subfolders based on
    /// the artist name and album name.
    ///  - Albums without a primary artist are moved to a folder called `.VariousArtists`.
    ///  - Tracks without a known artist are moved to a folder called `.Unknown`.
    Sort {
        #[arg(required = true)]
        patterns : Vec<String>,
    },
}

fn main() {
    colog::init();
    if cfg!(feature = "dev_mode") {
        // update working directory to example/
        env::set_current_dir("example").expect("cannot update working dir");
    }
    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Add { uris, playlist } => cmd_add::run(uris, *playlist),
        Commands::Rename { patterns, format, no_artist, album, number, no_title, .. }
            => cmd_rename::run(&patterns, &format, !*no_artist, *album, *number, !*no_title),
        Commands::Sort { patterns } => cmd_sort::run(&patterns),
    };
    if let Err(msg) = result {
        log::error!("fatal error encountered:\n{}", msg);
    }
}