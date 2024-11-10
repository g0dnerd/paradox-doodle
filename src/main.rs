use clap::Parser;
use doodle::{framework::run, scene::Scene, Cli};
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args = Cli::parse();
    log::info!("Args: {:?}", args);
    run::<Scene>("scene", args);
}
