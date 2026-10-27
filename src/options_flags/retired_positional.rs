//! A retired positional: every argument is a flag, the command names
//! aside, so a command that took a positional keeps it as a hidden
//! argument and refuses it with the flag that replaced it, rather
//! than with clap's generic unexpected-argument error. A stale doc or
//! a habit is told its own fix.

/// The refusal for the old positional `value` on `command`, naming
/// `flag`, the spelling that replaced it, with the value in place.
pub fn refusal(command: &str, value: &str, flag: &str) -> String {
    message(command, &format!("{flag} {value}"))
}

/// The read commands' retired `[REVISION] [COMMITS]` pair, refused
/// naming `-r` and `-n` with the values in place, either alone, and
/// nothing to refuse when neither was given. The count is kept as
/// typed, so one that is no number is still told its flag.
pub fn refuse_rev_count(
    command: &str,
    rev: Option<&str>,
    count: Option<&str>,
) -> Result<(), String> {
    let form = match (rev, count) {
        (None, None) => return Ok(()),
        (Some(r), None) => format!("-r {r}"),
        (None, Some(n)) => format!("-n {n}"),
        (Some(r), Some(n)) => format!("-r {r} -n {n}"),
    };
    Err(message(command, &form))
}

/// A retired pair of positionals on `command`, refused naming `first`
/// for the first word and `second` for the second, either alone, and
/// nothing to refuse when neither was given.
pub fn refuse_two(
    command: &str,
    retired: &[String],
    first: &str,
    second: &str,
) -> Result<(), String> {
    let form = match retired {
        [] => return Ok(()),
        [a] => format!("{first} {a}"),
        [a, b, ..] => format!("{first} {a} {second} {b}"),
    };
    Err(message(command, &form))
}

/// The one wording: the command, the rule, and the flag form to type.
fn message(command: &str, form: &str) -> String {
    format!("{command}: the positional is retired, every argument is a flag: `{form}`")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The refusal names the command and the flag form to type.
    #[test]
    fn refusal_names_the_flag_form() {
        assert_eq!(
            refusal("push", "@-", "-r"),
            "push: the positional is retired, every argument is a flag: `-r @-`"
        );
    }

    /// The pair names `-r`, `-n`, or both, and neither is no refusal.
    #[test]
    fn the_rev_count_pair_names_its_flags() {
        assert_eq!(refuse_rev_count("list", None, None), Ok(()));
        let err = refuse_rev_count("list", Some("@.."), Some("5")).unwrap_err();
        assert!(err.ends_with("`-r @.. -n 5`"), "{err}");
        let err = refuse_rev_count("chid", Some("@-"), None).unwrap_err();
        assert!(err.ends_with("`-r @-`"), "{err}");
    }
}
