//! The run record: what every operation a vc-x1 run commits says
//! about the run that made it.
//!
//! A run is one vc-x1 invocation. Its record is written as operation
//! attributes, jj's string-to-string map, the one jj-cli keeps its
//! own `args` in, so `jj op log` shows it with no template of ours.
//! This module is the typed side of that map: the keys are
//! [`RunAttribute`], the id a [`RunId`], so the strings are made and
//! read in one place. Every session the process opens shares the
//! run, so a `-s b` run stamps both repos with one id.

use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use jj_lib::transaction::Transaction;

/// The run record's format, written with every record, so a later
/// vc-x1 reading an older operation knows what it is reading.
pub const RUN_FORMAT: u32 = 1;

/// The keys of the run record in an operation's attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunAttribute {
    /// The run's [`RunId`].
    Id,
    /// The run's subcommand, as clap names it: `commit`, `squash-push`.
    Command,
    /// The run's command line, each argument a shell word, for people.
    Args,
    /// The record's format, [`RUN_FORMAT`].
    Format,
}

impl RunAttribute {
    /// The attribute's key, as the op store keeps it.
    pub fn key(self) -> &'static str {
        match self {
            Self::Id => "vc-x1-run",
            Self::Command => "vc-x1-command",
            Self::Args => "vc-x1-args",
            Self::Format => "vc-x1-run-format",
        }
    }
}

/// A run's id: 128 bits in the UUID v7 layout, 48 bits of Unix
/// milliseconds, the version 7, 12 random bits, the variant `0b10`,
/// and 62 random bits. So ids sort by when their runs began, and
/// stay unique beyond the one op log they are written in. Written as
/// 32 lowercase hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunId(pub u128);

impl RunId {
    /// A new id for a run beginning now.
    pub fn now() -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0); // OK: a clock before 1970 sorts first, its random bits keep it unique
        Self::from_parts(ms, rand::random::<u128>())
    }

    /// The id for a run begun at `ms` Unix milliseconds, its random
    /// fields taken from `random`.
    fn from_parts(ms: u128, random: u128) -> Self {
        let timestamp = (ms & 0xffff_ffff_ffff) << 80;
        let version = 0x7 << 76;
        let rand_a = ((random >> 64) & 0xfff) << 64;
        let variant = 0b10 << 62;
        let rand_b = random & ((1 << 62) - 1);
        Self(timestamp | version | rand_a | variant | rand_b)
    }
}

impl fmt::Display for RunId {
    /// 32 lowercase hex digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl FromStr for RunId {
    type Err = String;

    /// Exactly 32 hex digits, so a value that is not ours never reads
    /// as a run.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 32 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("not a run id, 32 hex digits: {s:?}"));
        }
        u128::from_str_radix(s, 16)
            .map(Self)
            .map_err(|e| format!("not a run id: {s:?}: {e}"))
    }
}

/// One run's record, made once per process.
#[derive(Debug)]
pub struct Run {
    /// The run's id.
    pub id: RunId,
    /// The subcommand, as clap names it.
    pub command: String,
    /// The command line, each argument a shell word.
    pub args: String,
}

impl Run {
    /// A run of `command` beginning now, its command line this
    /// process's.
    fn new(command: &str) -> Self {
        Run {
            id: RunId::now(),
            command: command.to_string(),
            args: std::env::args_os()
                .map(|arg| shell_word(&arg.to_string_lossy()))
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// The record as the op store keeps it, key and value.
    fn attributes(&self) -> [(RunAttribute, String); 4] {
        [
            (RunAttribute::Id, self.id.to_string()),
            (RunAttribute::Command, self.command.clone()),
            (RunAttribute::Args, self.args.clone()),
            (RunAttribute::Format, RUN_FORMAT.to_string()),
        ]
    }
}

static RUN: OnceLock<Run> = OnceLock::new();

/// Begin this process's run as `command`, the subcommand clap parsed,
/// before any session commits an operation. Called once, by `main`.
pub fn begin_run(command: &str) {
    RUN.get_or_init(|| Run::new(command));
}

/// This process's run: the one `begin_run` made, or where nothing
/// began one, a test, one made on first use with the command `?`.
pub fn run() -> &'static Run {
    RUN.get_or_init(|| Run::new("?"))
}

/// Stamp `tx` with this process's run, so the operation it commits
/// says which vc-x1 run made it. Every transaction a session starts
/// takes it: `RepoSession::start_tx` and the three the snapshot cycle
/// starts on its own.
pub fn tag_run(tx: &mut Transaction) {
    for (attribute, value) in run().attributes() {
        tx.set_attribute(attribute.key().to_string(), value);
    }
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

    /// The id has the UUID v7 layout: the milliseconds on top, then
    /// the version and the variant where the layout puts them, and
    /// random bits everywhere else.
    #[test]
    fn run_id_has_the_uuid_v7_layout() {
        let ms = 0x0123_4567_89ab;
        let id = RunId::from_parts(ms, u128::MAX);
        assert_eq!(id.0 >> 80, ms);
        assert_eq!((id.0 >> 76) & 0xf, 0x7);
        assert_eq!((id.0 >> 62) & 0b11, 0b10);
        assert_eq!(id.to_string(), "0123456789ab7fffbfffffffffffffff");
        let zero = RunId::from_parts(ms, 0);
        assert_eq!(zero.to_string(), "0123456789ab70008000000000000000");
    }

    /// Ids sort by when their runs began, whatever their random bits.
    #[test]
    fn run_ids_sort_by_time() {
        assert!(RunId::from_parts(1, u128::MAX) < RunId::from_parts(2, 0));
    }

    /// An id reads back as itself, and a value that is not 32 hex
    /// digits is no id.
    #[test]
    fn run_id_round_trips_and_refuses_other_values() {
        let id = RunId::now();
        assert_eq!(id.to_string().parse::<RunId>(), Ok(id));
        for other in ["", "97d669fcc2f1e9c9", &format!("{id}0"), &"g".repeat(32)] {
            assert!(other.parse::<RunId>().is_err(), "{other:?}");
        }
    }
}
