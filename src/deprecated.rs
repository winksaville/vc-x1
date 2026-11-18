//! What vc-x1 handed to the project at 0.85.25: its `validate`,
//! `build`, `run`, and `install`, and the `[validate]`, `[build]`,
//! `[install]`, and `[profile.*]` config tables they read.
//!
//! The work's tasks are the project's own, `cargo xtask <task>` in a
//! Rust workspace that carries xtask, and whatever `CONTRIBUTING.md >
//! ## Tasks` names elsewhere. The commands still run and the tables
//! still load, each with a warning naming what replaced it, since
//! every project on the machine shares one `vc-x1` and a refusal would
//! stop their sessions mid-cycle. A later release refuses them, once
//! each has migrated.

use log::warn;

/// The warning for the deprecated task command `old`, `validate`,
/// `build`, `run`, or `install`, each now the project's task of the
/// same name, `cx` being the shell alias `cargo xtask completions`
/// suggests.
pub fn task(old: &str) -> String {
    format!("Deprecated `vc-x1 {old}`: use `cargo xtask {old}` or `cx {old}` instead")
}

/// Warn that the task command `old` is deprecated, before it runs.
pub fn warn_task(old: &str) {
    warn!("{}", task(old));
}

/// The deprecated tables, each by the prefix its keys carry, its
/// label, and the warning's tail, what to use instead.
const TABLES: &[(&str, &str, &str)] = &[
    (
        "validate.",
        "[validate]",
        "use `cargo xtask validate` instead, its lists in Cargo.toml's \
         `[workspace.metadata.xtask]`",
    ),
    ("build.", "[build]", "use `cargo xtask build` instead"),
    ("install.", "[install]", "use `cargo xtask install` instead"),
    (
        "profile.",
        "[profile.*]",
        "use Cargo's own `[profile.<name>]` in Cargo.toml instead, chosen by `cargo xtask \
         build --profile NAME`",
    ),
];

/// The label of the deprecated table `key` is in, `None` when it is
/// in none.
pub fn table_of(key: &str) -> Option<&'static str> {
    TABLES
        .iter()
        .find(|(prefix, _, _)| key.starts_with(prefix))
        .map(|(_, label, _)| *label)
}

/// The warning for the deprecated table `label`, one of
/// [`table_of`]'s, naming what to use instead.
pub fn table(label: &str) -> String {
    let instead = TABLES
        .iter()
        .find(|(_, l, _)| *l == label)
        .map_or("", |(_, _, r)| *r);
    format!("Deprecated `{label}`: {instead}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A task's warning names it and its xtask task, in full and by
    /// the alias.
    #[test]
    fn a_task_names_its_replacement() {
        assert_eq!(
            task("install"),
            "Deprecated `vc-x1 install`: use `cargo xtask install` or `cx install` instead"
        );
    }

    /// Each table's keys find it, any other key none, and each
    /// table's warning names what replaced it.
    #[test]
    fn a_table_is_found_by_its_keys() {
        assert_eq!(table_of("validate.full"), Some("[validate]"));
        assert_eq!(table_of("build.commands"), Some("[build]"));
        assert_eq!(table_of("install.dir"), Some("[install]"));
        assert_eq!(table_of("profile.dev.suffix"), Some("[profile.*]"));
        assert_eq!(table_of("version.file"), None);
        assert_eq!(table_of("repos.work"), None);
        assert_eq!(
            table("[build]"),
            "Deprecated `[build]`: use `cargo xtask build` instead"
        );
        assert!(table("[validate]").contains("`[workspace.metadata.xtask]`"));
        assert!(table("[profile.*]").starts_with("Deprecated `[profile.*]`: use Cargo's own"));
    }
}
