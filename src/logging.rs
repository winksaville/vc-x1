use std::io::{IsTerminal, Write};
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

/// Simple logger that routes by level:
/// - info and above -> stdout (user-facing progress), wrapped as below
///   when stdout is a terminal, each line on its own
/// - debug/trace -> stderr (only when verbose at that level)
/// - warn/error -> stderr (always), wrapped to the terminal when stderr
///   is one, so a long message breaks between words with its later
///   lines indented, and left whole when it is not, so a log or a
///   `grep` still gets each message on its line
/// - log file captures all enabled levels, never wrapped
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
        // OK: default verbosity when -v/-vv absent
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

        let msg = named(&format!("{}", record.args()));

        // Write to log file if configured: captures all levels
        if let Some(ref file) = self.log_file
            && let Ok(mut f) = file.lock()
        {
            let _ = writeln!(f, "[{:<5}] {}: {}", record.level(), record.target(), msg);
        }

        match record.level() {
            // User-facing progress -> stdout
            // Written, not `println!`, so a reader that closes early, a
            // `| head`, ends the output quietly rather than panicking.
            Level::Info => {
                let _ = writeln!(std::io::stdout(), "{}", for_stdout(&msg));
            }
            // Errors and warnings -> stderr always
            Level::Error => eprintln!("{}", for_stderr("error: ", &msg)),
            Level::Warn => eprintln!("{}", for_stderr("warn: ", &msg)),
            // Debug/trace -> stderr only if verbose allows this level
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

/// Name this invocation was launched under: `argv[0]`'s basename,
/// falling back to the compiled bin name when argv is empty or
/// unreadable. Runtime on purpose: the binary's on-disk name is
/// the manifest's per-line package name, and a copy or rename
/// afterwards still self-reports the name it actually runs as,
/// which no compile-time constant can know.
pub(crate) fn invoked_name() -> &'static str {
    static NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    NAME.get_or_init(|| {
        std::env::args_os()
            .next()
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| env!("CARGO_BIN_NAME").to_string()) // OK: no argv, the built name
    })
}

/// `msg` with each command it names run as the invocation was, so a
/// hint read under `vc-x1-dev` says `vc-x1-dev`: every `` `vc-x1 ``,
/// a command in backticks, takes the invoked name. One place rather
/// than every message, the messages written with the binary's name.
fn named(msg: &str) -> String {
    rename_commands(msg, invoked_name())
}

/// `msg` with every `` `vc-x1 `` made `` `<name> ``, unchanged when
/// `name` is the binary's own.
fn rename_commands(msg: &str, name: &str) -> String {
    let own = env!("CARGO_BIN_NAME");
    if name == own {
        return msg.to_string();
    }
    msg.replace(&format!("`{own} "), &format!("`{name} "))
}

/// The indent of a wrapped message's later lines, so the first line,
/// with its `error:` or `warn:`, stands out.
const HANG: &str = "  ";

/// `prefix` and `msg` as stderr shows them: wrapped to the terminal's
/// width when stderr is a terminal, else as one line per line of the
/// message.
fn for_stderr(prefix: &str, msg: &str) -> String {
    let line = format!("{prefix}{msg}");
    match width_of(std::io::stderr()) {
        Some(width) => wrap(&line, width, true),
        None => line,
    }
}

/// An info message as stdout shows it: each of its lines wrapped to the
/// terminal's width when stdout is a terminal, its own lines left at
/// their own indent, since an info message of several lines is a
/// listing, a description, or a ladder, and else as it is.
fn for_stdout(msg: &str) -> String {
    match width_of(std::io::stdout()) {
        Some(width) => wrap(msg, width, false),
        None => msg.to_string(),
    }
}

/// The width to wrap a stream at: its terminal's, the width the user
/// chose, or `None` when it is not a terminal.
fn width_of<S: IsTerminal + std::os::fd::AsFd>(stream: S) -> Option<usize> {
    if !stream.is_terminal() {
        return None;
    }
    terminal_size::terminal_size_of(stream).map(|(terminal_size::Width(w), _)| usize::from(w))
}

/// The words of `line`, a span in backticks kept whole as one word, so a
/// command the message names is never broken across lines.
fn words(line: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in line.chars() {
        if c == '`' {
            quoted = !quoted;
        }
        if c == ' ' && !quoted {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// `text` wrapped at `width` columns: each of its lines broken between
/// words, a broken line's rest indented by [`HANG`] beyond the line's
/// own indent. With `indent_lines` the message's own lines after its
/// first are indented by [`HANG`] too, so an error's detail sits under
/// its first line. A word wider than the line, a long path or command,
/// gets a line of its own and is not split, and a blank line stays
/// blank.
pub(crate) fn wrap(text: &str, width: usize, indent_lines: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    for (n, source) in text.lines().enumerate() {
        if source.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        let own = source.len() - source.trim_start().len();
        let indent = if n == 0 || !indent_lines {
            source[..own].to_string()
        } else {
            format!("{HANG}{}", &source[..own])
        };
        let hang = format!("{HANG}{indent}");
        let mut line = indent.clone();
        let mut fresh = true;
        for word in words(source.trim_start()) {
            let needed = if fresh {
                word.chars().count()
            } else {
                word.chars().count() + 1
            };
            if !fresh && line.chars().count() + needed > width {
                out.push(std::mem::take(&mut line));
                line = hang.clone();
                fresh = true;
            }
            if !fresh {
                line.push(' ');
            }
            line.push_str(&word);
            fresh = false;
        }
        out.push(line);
    }
    let mut wrapped = out.join("\n");
    if text.ends_with('\n') {
        wrapped.push('\n');
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A long message breaks between words, its later lines indented,
    /// none wider than the width unless one word is.
    /// A command a message names takes the invoked name, and only a
    /// command, in backticks: a file named for the binary is left.
    #[test]
    fn a_hint_names_the_command_run() {
        let own = env!("CARGO_BIN_NAME");
        let msg = format!("`{own} push -s b` publishes it, as {own}.md says");
        assert_eq!(rename_commands(&msg, own), msg);
        assert_eq!(
            rename_commands(&msg, "vc-x1-dev"),
            format!("`vc-x1-dev push -s b` publishes it, as {own}.md says")
        );
    }

    #[test]
    fn a_long_message_wraps_between_words() {
        let msg = "error: cycle plan: /home/wink/tmp/t17 has no TODO.md or zTOC.md, and a \
                   cycle runs on its record";
        let wrapped = wrap(msg, 40, true);
        assert_eq!(
            wrapped,
            "error: cycle plan: /home/wink/tmp/t17\n  \
             has no TODO.md or zTOC.md, and a cycle\n  \
             runs on its record"
        );
        assert!(
            wrapped.lines().all(|l| l.chars().count() <= 40),
            "{wrapped}"
        );
        assert_eq!(wrap("short", 40, true), "short", "a short one is as it was");
    }

    /// A command in backticks is one word, moved whole to the next line,
    /// and a word wider than the line gets a line to itself.
    #[test]
    fn a_command_is_never_split() {
        let wrapped = wrap(
            "error: publish it first: `vc-x1 push -s b -r main` then land",
            30,
            true,
        );
        assert_eq!(
            wrapped,
            "error: publish it first:\n  `vc-x1 push -s b -r main`\n  then land"
        );
        let long = wrap(
            "error: at /a/very/long/path/that/does/not/fit/anywhere ok",
            20,
            true,
        );
        assert_eq!(
            long,
            "error: at\n  /a/very/long/path/that/does/not/fit/anywhere\n  ok"
        );
    }

    /// A message of several lines keeps them, each later one indented,
    /// its own indent kept beyond the hang.
    #[test]
    fn a_messages_own_lines_and_indent_are_kept() {
        let msg = "error: refused. The ways on:\n  - a merge at the Land\n  - a rebase";
        assert_eq!(
            wrap(msg, 80, true),
            "error: refused. The ways on:\n    - a merge at the Land\n    - a rebase"
        );
    }

    /// An info message's own lines stay at their indent, a long one
    /// breaking under itself, and blank lines and a final newline stay.
    #[test]
    fn an_info_message_keeps_its_lines() {
        let msg = "ladder:\n  - 0 (done) feat: a rung whose title is long enough to wrap\n\nnext\n";
        assert_eq!(
            wrap(msg, 40, false),
            "ladder:\n  - 0 (done) feat: a rung whose title is\n    long enough to wrap\n\nnext\n"
        );
    }
}
