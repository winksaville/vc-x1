//! A retired positional: every argument is a flag, the command names
//! aside, so a command that took a positional keeps it as a hidden
//! argument and refuses it with the flag that replaced it, rather
//! than with clap's generic unexpected-argument error. A stale doc or
//! a habit is told its own fix.

/// The refusal for the old positional `value` on `command`, naming
/// `flag`, the spelling that replaced it, with the value in place.
pub fn refusal(command: &str, value: &str, flag: &str) -> String {
    format!("{command}: the positional is retired, every argument is a flag: `{flag} {value}`")
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
}
