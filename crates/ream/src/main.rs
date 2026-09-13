#![allow(clippy::expect_used)]

pub(crate) mod logger;

use pound::Parse;

use crate::logger::Logger;

#[derive(Parse, Debug)]
enum Command {
    New,

    Run {
        args: Vec<String>,
    },
    Build,
    Test,

    // Dependencies
    /// Add a dependency to the current project
    Add,
    /// Remove a dependency from the current project
    Remove,
    /// Update dependencies to their latest versions
    Update,

    Lsp,
}

#[derive(Parse, Debug)]
struct Cli {
    #[pound(flag, long, short, count)]
    verbose: u8,

    #[pound(flag, long, short)]
    quiet: bool,

    #[pound(subcommand)]
    command: Command,
}

fn main() {
    log::set_logger(&Logger).expect("failed to set up logger");
    log::set_max_level(log::LevelFilter::Info);

    let args = Cli::parse();

    if args.quiet && args.verbose > 0 {
        log::warn!(
            action = "Awa";
            "--quiet combined with --verbose makes no sense. --verbose will take priority."
        );
    }

    if args.quiet {
        log::set_max_level(log::LevelFilter::Warn);
    }

    match args.verbose {
        0 => {}

        1 => log::set_max_level(log::LevelFilter::Debug),

        2 => log::set_max_level(log::LevelFilter::Trace),

        _ => log::warn!("more than two verbose flags has no meaning"),
    }

    #[allow(clippy::todo)]
    match args.command {
        Command::New => todo!(),
        Command::Run { args } => println!("{args:?}"),
        Command::Build => todo!(),
        Command::Test => todo!(),
        Command::Add => todo!(),
        Command::Remove => todo!(),
        Command::Update => todo!(),
        Command::Lsp => ream_lsp::start_lsp(),
    }
}
