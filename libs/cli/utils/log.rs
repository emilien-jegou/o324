use colored::*;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub struct SimpleLogger {
    level: LevelFilter,
}

impl SimpleLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(SimpleLogger { level });
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let (icon, color) = match record.level() {
            Level::Error => ("❌", Color::Red),
            Level::Warn => ("⚠️", Color::Yellow),
            Level::Info => ("ℹ️", Color::Green),
            Level::Debug => ("🐞", Color::Blue),
            Level::Trace => ("🔍", Color::Magenta),
        };

        let message = format!("{} {} - {}", icon, record.target(), record.args());

        println!("{}", message.color(color).bold());
    }

    fn flush(&self) {}
}
