use spotr::{args::{Opts, SubCommand}, auth::login};
use clap::Clap;

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Login => { login().await; },
        _ => {}
    }
}
