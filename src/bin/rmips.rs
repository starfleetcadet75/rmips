use anyhow::Result;
use clap::derive::Clap;
use human_panic::setup_panic;
use log::LevelFilter;
use rmips::emulator::Emulator;
use rmips::util::opts::Opts;
use simplelog::{CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::env;
use std::fs::File;

fn setup_logger(opts: &Opts) {
    let log_level = match opts.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 | _ => LevelFilter::Debug,
    };

    // Writes all logging to a file and prints logging at the given level to the terminal output
    let log_file = env::temp_dir().join("rmips.log");
    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            simplelog::Config::default(),
            TerminalMode::Stdout,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(log_file).expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to initialize logging");
}

fn main() -> Result<()> {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "starfleetcadet75 <starfleetcadet75@gmail.com>".into(),
        homepage: "github.com/starfleetcadet75/rmips".into(),
    });

    let opts = Opts::parse();
    setup_logger(&opts);

    let mut emulator = Emulator::new(opts)?;
    if let Err(err) = emulator.run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }

    Ok(())
}
