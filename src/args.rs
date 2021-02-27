use clap::{Clap};

#[derive(Clap)]
#[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand
}

#[derive(Clap)]
pub enum SubCommand {
    /// Plays a specific song or resume latest.
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Play,
    /// Pauses playback.
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Pause,
    /// Switch between play/pause
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Toggle,

    /// Save the song that is currently playing to your configured Playlist
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Save,

    /// Login to the spotify api
    /// 1. Opens the Spotify OAuth URL in a browser or prints it to screen.
    /// 2. 
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Login
}
