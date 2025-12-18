use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::Context;
use env_logger::{Logger, Target, fmt::Formatter};
use log::{LevelFilter, Log, Metadata, Record};

#[derive(Debug)]
pub struct MultiLogger {
    loggers: Vec<Logger>,
}

impl MultiLogger {
    pub fn new(loggers: Vec<Logger>) -> Self {
        Self { loggers }
    }

    pub fn init(self) -> anyhow::Result<()> {
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }
}

impl Log for MultiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(metadata))
    }

    fn log(&self, record: &Record) {
        self.loggers
            .iter()
            .filter(|logger| logger.enabled(record.metadata()))
            .for_each(|logger| logger.log(record));
    }

    fn flush(&self) {
        self.loggers.iter().for_each(|logger| logger.flush());
    }
}

pub type FormatFn = fn(&mut Formatter, &Record<'_>) -> io::Result<()>;

#[derive(Debug, Clone, Copy)]
pub struct LoggerBuilder {
    formatter: FormatFn,
    level: LevelFilter,
}

impl LoggerBuilder {
    pub fn new(formatter: FormatFn, level: LevelFilter) -> Self {
        Self { formatter, level }
    }

    fn format(buf: &mut Formatter, record: &Record<'_>) -> io::Result<()> {
        writeln!(
            buf,
            "{} [{}] {} - {}",
            buf.timestamp(),
            record.level().to_string().to_uppercase(),
            record.target(),
            record.args()
        )
    }

    pub fn std_out_logger(&self) -> Logger {
        let env = env_logger::Env::new().default_filter_or(self.level.to_string());
        let mut builder = env_logger::Builder::from_env(env);
        builder.target(env_logger::Target::Stdout);
        builder.format_timestamp_millis();
        builder.format(self.formatter);
        builder.build()
    }

    pub fn std_err_logger(&self) -> Logger {
        let env = env_logger::Env::new().default_filter_or(self.level.to_string());
        let mut builder = env_logger::Builder::from_env(env);
        builder.target(env_logger::Target::Stderr);
        builder.format_timestamp_millis();
        builder.format(self.formatter);
        builder.build()
    }

    pub fn file_logger(&self, log_file_path: &Path) -> anyhow::Result<Logger> {
        if let Some(parent_directory) = log_file_path.parent() {
            fs::create_dir_all(parent_directory)
                .context("Failed to create log file parent directory")?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .context("Failed to open log file")?;

        let env = env_logger::Env::new().default_filter_or(self.level.to_string());
        let mut builder = env_logger::Builder::from_env(env);
        builder.target(Target::Pipe(Box::new(file)));
        builder.format_timestamp_millis();
        builder.format(self.formatter);
        Ok(builder.build())
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new(LoggerBuilder::format, LevelFilter::Off)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoggerOptions {
    pub level: LevelFilter,
    pub to_file: bool,
    pub to_stdout: bool,
    pub to_stderr: bool,
}

impl LoggerOptions {
    pub fn new(level: LevelFilter, to_file: bool, to_stdout: bool, to_stderr: bool) -> Self {
        Self {
            level,
            to_file,
            to_stdout,
            to_stderr,
        }
    }
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self {
            level: LevelFilter::Off,
            to_file: true,
            to_stdout: false,
            to_stderr: true,
        }
    }
}

pub struct EvLogger;

impl EvLogger {
    const LOG_FILE_NAME: &str = "out.log";

    pub fn init(project_dirs: &directories::ProjectDirs) -> anyhow::Result<()> {
        Self::init_with_options(project_dirs, LoggerOptions::default())
    }

    pub fn init_with_options(
        project_dirs: &directories::ProjectDirs,
        options: LoggerOptions,
    ) -> anyhow::Result<()> {
        let mut loggers = Vec::new();
        let logger_builder = LoggerBuilder::new(LoggerBuilder::format, options.level);

        if options.to_file {
            let log_file_path = project_dirs.config_local_dir().join(Self::LOG_FILE_NAME);
            loggers.push(logger_builder.file_logger(&log_file_path)?);
        }
        if options.to_stdout {
            loggers.push(logger_builder.std_out_logger());
        }
        if options.to_stderr {
            loggers.push(logger_builder.std_err_logger());
        }

        let multi_logger = MultiLogger::new(loggers);
        multi_logger.init()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{Level, Metadata};
    use tempfile::TempDir;

    #[test]
    fn test_multi_logger_new() {
        let logger_builder = LoggerBuilder::default();
        let std_err_logger = logger_builder.std_err_logger();
        let multi_logger = MultiLogger::new(vec![std_err_logger]);

        assert_eq!(multi_logger.loggers.len(), 1);
    }

    #[test]
    fn test_logger_builder_new() {
        let formatter = LoggerBuilder::format;
        let level = LevelFilter::Debug;
        let builder = LoggerBuilder::new(formatter, level);

        assert_eq!(builder.level, level);
    }

    #[test]
    fn test_logger_builder_std_out_logger() {
        let builder = LoggerBuilder::new(LoggerBuilder::format, LevelFilter::Info);
        let logger = builder.std_out_logger();

        assert!(
            logger.enabled(
                &Metadata::builder()
                    .level(Level::Info)
                    .target("test")
                    .build()
            )
        );
    }

    #[test]
    fn test_logger_builder_std_err_logger() {
        let builder = LoggerBuilder::new(LoggerBuilder::format, LevelFilter::Info);
        let logger = builder.std_err_logger();

        assert!(
            logger.enabled(
                &Metadata::builder()
                    .level(Level::Info)
                    .target("test")
                    .build()
            )
        );
    }

    #[test]
    fn test_logger_builder_file_logger_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_file_path = temp_dir.path().join("test.log");
        let builder = LoggerBuilder::new(LoggerBuilder::format, LevelFilter::Info);
        let logger = builder.file_logger(&log_file_path).unwrap();

        assert!(log_file_path.exists());
        assert!(
            logger.enabled(
                &Metadata::builder()
                    .level(Level::Info)
                    .target("test")
                    .build()
            )
        );
    }

    #[test]
    fn test_logger_builder_file_logger_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let log_file_path = temp_dir.path().join("subdir").join("test.log");
        let builder = LoggerBuilder::new(LoggerBuilder::format, LevelFilter::Info);
        let _logger = builder.file_logger(&log_file_path).unwrap();

        assert!(log_file_path.parent().unwrap().is_dir());
    }

    #[test]
    fn test_logger_builder_default() {
        let default_builder = LoggerBuilder::default();

        assert_eq!(default_builder.level, LevelFilter::Off);
    }

    #[test]
    fn test_logger_options_new() {
        let options = LoggerOptions::new(LevelFilter::Warn, true, false, true);

        assert_eq!(options.level, LevelFilter::Warn);
        assert!(options.to_file);
        assert!(!options.to_stdout);
        assert!(options.to_stderr);
    }

    #[test]
    fn test_logger_options_default() {
        let options = LoggerOptions::default();

        assert_eq!(options.level, LevelFilter::Off);
        assert!(options.to_file);
        assert!(!options.to_stdout);
        assert!(options.to_stderr);
    }
}
