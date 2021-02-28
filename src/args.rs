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

    /// Login to the Spotify API using an OAuth2.0 PKCE Flow.
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    Login,
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    /// Show the currently playing song/podcast.
    Current,
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    /// Skip to the next song.
    Skip,
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    /// Skip to the previous song.
    Back,
    #[clap(version = "0.1", author = "Avery Wagar <ajmw.subs@gmail.com>")]
    /// Print out some shorthand aliases for common commands for Bash/ZSH
    Alias
}
