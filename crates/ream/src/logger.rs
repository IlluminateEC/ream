use std::io::Write;

use colored::Colorize as _;
use log::{Level, kv::Key};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(action) = record.key_values().get(Key::from("action")) {
            let action_str = action.to_string();

            let action_colored = match record.level() {
                Level::Error => action_str.red().bold(),
                Level::Warn => action_str.yellow().bold(),
                Level::Info => action_str.green().bold(),
                Level::Debug => action_str.blue().bold(),
                Level::Trace => action_str.purple().bold(),
            };

            eprintln!("{:>12} {}", action_colored, record.args());

            return;
        }

        let label = match record.level() {
            Level::Error => "error".red().bold(),
            Level::Warn => "warning".yellow().bold(),
            Level::Info => "info".green().bold(),
            Level::Debug => "debug".blue().bold(),
            Level::Trace => "trace".purple().bold(),
        };

        eprintln!("{}: {}", label, record.args());
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}
