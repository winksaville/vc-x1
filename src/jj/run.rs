//! The run record: what every operation a vc-x1 run commits says
//! about the run that made it.
//!
//! A run is one vc-x1 invocation. Its record is one operation
//! attribute, `args`, the key jj-cli keeps its own command line in,
//! so `jj op log` shows `args: vc-x1-dev 0.85.16+c594f1c06ba1 status
//! -s b` beside `args: jj diff --stat`, the first word naming the
//! program that ran. Ours carries the build after the program's name,
//! as the banner does, so an operation says which vc-x1 made it. Every
//! session the process opens shares the run, so a `-s b` run stamps
//! both repos with the same line.

use std::sync::OnceLock;

use jj_lib::transaction::Transaction;

/// The attribute the record is written under, jj-cli's own.
pub const ARGS_KEY: &str = "args";

/// One run's record, made once per process.
#[derive(Debug)]
pub struct Run {
    /// The program and its build, the banner's `<name> <version>`,
    /// then the command line's arguments, each a shell word, the body
    /// left out.
    pub args: String,
}

impl Run {
    /// A run of `program`, the banner's `<name> <version>`, its
    /// arguments this process's.
    fn new(program: &str) -> Self {
        let args: Vec<String> = std::env::args_os()
            .skip(1)
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let params = recorded_args(&args);
        Run {
            args: if params.is_empty() {
                program.to_string()
            } else {
                format!("{program} {params}")
            },
        }
    }
}

static RUN: OnceLock<Run> = OnceLock::new();

/// Begin this process's run as `program`, the banner's `<name>
/// <version>`, before any session commits an operation. Called once,
/// by `main`.
pub fn begin_run(program: &str) {
    RUN.get_or_init(|| Run::new(program));
}

/// This process's run: the one `begin_run` made, or where nothing
/// began one, a test, one made on first use with the program `?`.
pub fn run() -> &'static Run {
    RUN.get_or_init(|| Run::new("?"))
}

/// Stamp `tx` with this process's run, so the operation it commits
/// says which vc-x1 run made it. Every transaction a session starts
/// takes it: `RepoSession::start_tx` and the three the snapshot cycle
/// starts on its own.
pub fn tag_run(tx: &mut Transaction) {
    tx.set_attribute(ARGS_KEY.to_string(), run().args.clone());
}

/// The flag the record leaves out whole, flag and value: a commit's
/// body, which the commit carries and which would otherwise repeat on
/// every operation of the run. A `--title` is kept with its text, one
/// line that names the run.
const DROPPED_FLAG: &str = "--body";

/// A value longer than this, in characters, or holding a newline, is
/// kept as its length whatever flag it follows, so free text another
/// flag carries stays out. Long enough to keep a deep path.
const LONG_VALUE: usize = 100;

/// The arguments as the record keeps them: each a shell word,
/// [`DROPPED_FLAG`] and its value left out, in the `=` form too, and
/// any other value over [`LONG_VALUE`] or with a newline as its
/// length.
fn recorded_args(args: &[String]) -> String {
    let mut words = Vec::new();
    let mut drop_next = false;
    for arg in args {
        if std::mem::take(&mut drop_next) {
            continue;
        }
        if arg == DROPPED_FLAG {
            drop_next = true;
        } else if arg
            .split_once('=')
            .is_some_and(|(flag, _)| flag == DROPPED_FLAG)
        {
            // Dropped with its value.
        } else if arg.contains('\n') || arg.chars().count() > LONG_VALUE {
            words.push(format!("<{} chars>", arg.chars().count()));
        } else {
            words.push(shell_word(arg));
        }
    }
    words.join(" ")
}

/// `arg` as a shell word: bare when plain, single-quoted otherwise.
fn shell_word(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:@,+%".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_word_quotes_only_what_needs_it() {
        assert_eq!(shell_word("--title"), "--title");
        assert_eq!(shell_word("-R/a/b.c"), "-R/a/b.c");
        assert_eq!(shell_word("feat: two words"), "'feat: two words'");
        assert_eq!(shell_word("it's"), r"'it'\''s'");
        assert_eq!(shell_word(""), "''");
    }

    fn args(line: &[&str]) -> String {
        recorded_args(&line.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    /// A body is left out whole, flag and value, in the `=` form and
    /// as a path too, and a title keeps its text.
    #[test]
    fn recorded_args_drop_the_body_and_keep_the_title() {
        assert_eq!(
            args(&[
                "commit", "-s", "b", "--title", "feat: x", "--body", "A\n\nB."
            ]),
            "commit -s b --title 'feat: x'"
        );
        assert_eq!(
            args(&["squash", "--title=feat: x", "--body=b", "--body", "@b.md"]),
            "squash '--title=feat: x'"
        );
    }

    /// Text another flag carries is left out by its shape, a newline
    /// or a length over the limit, and a deep path is kept.
    #[test]
    fn recorded_args_leave_out_long_or_multiline_values() {
        let long = "x".repeat(LONG_VALUE + 1);
        let path = format!("/{}", "d/".repeat(40));
        assert_eq!(
            args(&["x", "--note", "a\nb", "--other", &long, "-R", &path]),
            format!(
                "x --note <3 chars> --other <{} chars> -R {path}",
                LONG_VALUE + 1
            )
        );
    }

    /// The record is the program, then the arguments, and the program
    /// alone when there are none.
    #[test]
    fn a_run_is_its_program_then_its_arguments() {
        let run = Run::new("vc-x1 0.1.0+abc");
        assert!(run.args.starts_with("vc-x1 0.1.0+abc"), "{}", run.args);
        assert_eq!(recorded_args(&[]), "");
    }
}
