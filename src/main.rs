mod common;
mod cmd_add;

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
        #[arg(required = true)]
        uris : Vec<String>,
    },
    /// Organise audio files in the working directory into subfolders based on
    /// the artist name and album name.
    ///  - Albums without a primary artist are moved to a folder called `.VariousArtists`.
    ///  - Tracks without a known artist are moved to a folder called `.Unknown`.
    Sort,
    /// Renames all audio files in the working directory so they are in a
    /// consistent format.
    Rename,
}

fn main() {
    colog::init();
    if cfg!(feature = "dev_mode") {
        // update working directory to example/
        env::set_current_dir("example").expect("cannot update working dir");
    }
    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Add { uris } => cmd_add::run(uris),
        Commands::Sort => unimplemented!(),
        Commands::Rename => unimplemented!(),
    };
    if let Err(msg) = result {
        log::error!("fatal error encountered:\n{}", msg);
    }
}