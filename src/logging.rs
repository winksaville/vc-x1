use std::io::Write;
use std::sync::Mutex;

use log::{Level, Log, Metadata, Record};

/// Simple logger that routes by level:
/// - info and above → stdout (user-facing progress)
/// - debug → stderr (only when verbose)
/// - trace → stderr (only when verbose)
/// - warn/error → stderr (always)
pub struct CliLogger {
    verbose: bool,
    log_file: Option<Mutex<std::fs::File>>,
}

impl CliLogger {
    pub fn new(verbose: bool, log_path: Option<&str>) -> Self {
        let log_file = log_path.map(|path| {
            Mutex::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("failed to open log file"),
            )
        });
        CliLogger { verbose, log_file }
    }

    /// Initialize as the global logger.
    pub fn init(verbose: bool, log_path: Option<&str>) {
        let logger = Box::new(CliLogger::new(verbose, log_path));
        log::set_max_level(if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        });
        log::set_boxed_logger(logger).expect("failed to set logger");
    }
}

impl Log for CliLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.level() {
            Level::Error | Level::Warn => true,
            Level::Info => true,
            Level::Debug | Level::Trace => self.verbose,
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let msg = format!("{}", record.args());

        // Write to log file if configured
        if let Some(ref file) = self.log_file
            && let Ok(mut f) = file.lock()
        {
            let _ = writeln!(f, "[{}] {}: {}", record.level(), record.target(), msg);
        }

        match record.level() {
            // User-facing progress → stdout
            Level::Info => println!("{msg}"),
            // Errors and warnings → stderr always
            Level::Error => eprintln!("error: {msg}"),
            Level::Warn => eprintln!("warn: {msg}"),
            // Debug/trace → stderr, indented (only reached if verbose)
            Level::Debug | Level::Trace => {
                for line in msg.lines() {
                    eprintln!("  {line}");
                }
            }
        }
    }

    fn flush(&self) {
        use std::io;
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
        if let Some(ref file) = self.log_file
            && let Ok(mut f) = file.lock()
        {
            let _ = f.flush();
        }
    }
}
