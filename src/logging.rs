use std::io::Write;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

/// Simple logger that routes by level:
/// - info and above → stdout (user-facing progress)
/// - debug/trace → stderr (only when verbose at that level)
/// - warn/error → stderr (always)
/// - log file captures all enabled levels
pub struct CliLogger {
    /// The verbose level for stderr output (None = no verbose)
    stderr_level: Option<LevelFilter>,
    log_file: Option<Mutex<std::fs::File>>,
}

impl CliLogger {
    pub fn new(stderr_level: Option<LevelFilter>, log_path: Option<&str>) -> Self {
        let log_file = log_path.map(|path| {
            Mutex::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("failed to open log file"),
            )
        });
        CliLogger {
            stderr_level,
            log_file,
        }
    }

    /// Initialize as the global logger.
    /// `verbose`: 0 = off, 1 = debug, 2+ = trace
    pub fn init(verbose: u8, log_path: Option<&str>) {
        let stderr_level = match verbose {
            0 => None,
            1 => Some(LevelFilter::Debug),
            _ => Some(LevelFilter::Trace),
        };
        let has_log_file = log_path.is_some();
        let logger = Box::new(CliLogger::new(stderr_level, log_path));

        // Max level: stderr_level if set, otherwise info
        // Log file captures whatever is enabled, not more
        let _ = has_log_file;
        let max = stderr_level.unwrap_or(LevelFilter::Info);
        log::set_max_level(max);
        log::set_boxed_logger(logger).expect("failed to set logger");
    }
}

impl Log for CliLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Always allow info and above
        if metadata.level() <= Level::Info {
            return true;
        }
        // Allow if stderr verbose is at this level
        if let Some(level) = self.stderr_level
            && metadata.level() <= level
        {
            return true;
        }
        false
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let msg = format!("{}", record.args());

        // Write to log file if configured — captures all levels
        if let Some(ref file) = self.log_file
            && let Ok(mut f) = file.lock()
        {
            let _ = writeln!(f, "[{:<5}] {}: {}", record.level(), record.target(), msg);
        }

        match record.level() {
            // User-facing progress → stdout
            Level::Info => println!("{msg}"),
            // Errors and warnings → stderr always
            Level::Error => eprintln!("error: {msg}"),
            Level::Warn => eprintln!("warn: {msg}"),
            // Debug/trace → stderr only if verbose allows this level
            Level::Debug | Level::Trace => {
                if let Some(level) = self.stderr_level
                    && record.level() <= level
                {
                    for line in msg.lines() {
                        eprintln!("  {line}");
                    }
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
