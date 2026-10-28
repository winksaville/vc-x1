//! A configured command list: a TOML array in the work side's
//! config, each element one invocation, run in order from the work
//! repo root, stopping at the first failure.
//!
//! - `[validate] full` and `fast` are run by `validate`,
//!   `[build] commands` by `build`, and the optional
//!   `[install] commands` by `install`, for every profile. Each caller
//!   passes its own label, which prefixes every message.
//! - Each command is printed before it runs, inherits the terminal,
//!   and runs from the root.
//! - The first non-zero exit stops the run, naming the command, its
//!   status, and where in the list it stopped.
//! - An empty or missing list is an error, not a silent pass: a
//!   list with nothing to run has done nothing.
//! - Elements are split on whitespace into a program and its
//!   arguments, with no shell in between, so a command's exit
//!   status is exactly the status the run sees.
//! - A caller may add environment variables to every command of
//!   the list, set on each child alone, never on vc-x1's own
//!   process: `build` passes the build's provenance this way.
//! - A caller may offer placeholders, `{name}` filled with a value
//!   in any word after the split, so a value with spaces stays one
//!   argument: `build` offers the chosen profile's properties,
//!   `{profile.cargo}` and the like, and `install` those and
//!   `{install.dir}`. A list-valued placeholder, such as
//!   `{profile.flags}`, standing alone as a word becomes its items, one
//!   word each. A `{name}` the caller does not offer is an error before
//!   anything runs, so a typo never runs as a literal path. A list
//!   offered none is taken as written, braces and all.

use std::path::Path;

use log::info;

/// Read `key`'s command list from the work side's config at `root`.
///
/// Errors, each prefixed by `label` and naming the key, when the
/// config is missing or unloadable, the key is absent, the value is
/// not a proper array, or the array is empty.
pub fn load(root: &Path, key: &str, label: &str) -> Result<Vec<String>, String> {
    let (cfg, commands) = read(root, key, label)?;
    let commands = commands.ok_or_else(|| {
        format!(
            "{label}: {}: no `{key}` table. Add one, a TOML array with one command per \
                 element (see `vc-x1 config -s work`)",
            cfg.path.display()
        )
    })?;
    nonempty(&cfg, key, label, commands)
}

/// Read `key`'s command list as [`load`] does, an absent key being
/// `None` rather than an error, for a list that replaces a built-in
/// step when present. A present list is checked as `load` checks it.
pub fn load_optional(root: &Path, key: &str, label: &str) -> Result<Option<Vec<String>>, String> {
    let (cfg, commands) = read(root, key, label)?;
    commands.map(|c| nonempty(&cfg, key, label, c)).transpose()
}

/// The config at `root` and `key`'s list in it, `None` when absent.
fn read(
    root: &Path,
    key: &str,
    label: &str,
) -> Result<(crate::config_md::VcConfig, Option<Vec<String>>), String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("{label}: {e}"))?
        .ok_or_else(|| format!("{label}: no config in '{}'", root.display()))?;
    let commands = crate::toml_simple::toml_get_list(&cfg.map, key)
        .map_err(|e| format!("{label}: {}: {e}", cfg.path.display()))?;
    Ok((cfg, commands))
}

/// `commands`, or an error when empty: nothing to run is not a pass.
fn nonempty(
    cfg: &crate::config_md::VcConfig,
    key: &str,
    label: &str,
    commands: Vec<String>,
) -> Result<Vec<String>, String> {
    if commands.is_empty() {
        return Err(format!(
            "{label}: {}: `{key}` is empty, nothing to run is not a pass",
            cfg.path.display()
        ));
    }
    Ok(commands)
}

/// Check every placeholder in `commands` is one of `names`, a string
/// placeholder, or one of `list_names`, a list-valued one standing
/// alone as a word, before anything runs. `key` names the list and
/// `label` prefixes the error.
pub fn check_placeholders(
    commands: &[String],
    names: &[&str],
    list_names: &[&str],
    key: &str,
    label: &str,
) -> Result<(), String> {
    let vars: Vec<(&str, &str)> = names.iter().map(|n| (*n, "")).collect();
    let lists: Vec<(&str, &[String])> = list_names.iter().map(|n| (*n, &[][..])).collect();
    for command in commands {
        expand(command, &vars, &lists).map_err(|e| format!("{label}: `{key}`: {e}"))?;
    }
    Ok(())
}

/// Split `command` on whitespace and fill each word's placeholders,
/// `{name}` for each `(name, value)` of `vars`, and for each
/// `(name, items)` of `lists` a word that is `{name}` alone becomes the
/// items, one word each, none for an empty list. A word is split before
/// it is filled, so a value with spaces stays one word. With neither
/// the words are as written. A `{name}`, a name of letters, digits,
/// `.`, `-`, and `_`, offered by neither is an error naming the ones
/// offered, a list inside a longer word is an error, since its items
/// have no word to share, and any other brace is literal.
pub fn expand(
    command: &str,
    vars: &[(&str, &str)],
    lists: &[(&str, &[String])],
) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for word in command.split_whitespace() {
        let alone = word.strip_prefix('{').and_then(|w| w.strip_suffix('}'));
        if let Some((_, items)) = alone.and_then(|name| lists.iter().find(|(n, _)| *n == name)) {
            out.extend(items.iter().cloned());
        } else if vars.is_empty() && lists.is_empty() {
            out.push(word.to_string());
        } else {
            out.push(fill(word, vars, lists)?);
        }
    }
    Ok(out)
}

/// Fill one word's placeholders: see [`expand`].
fn fill(word: &str, vars: &[(&str, &str)], lists: &[(&str, &[String])]) -> Result<String, String> {
    let is_name = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_');
    let mut out = String::new();
    let mut rest = word;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        // OK: no character past the name is a name running to the word's end
        let name_len = after.find(|c: char| !is_name(c)).unwrap_or(after.len());
        let name = &after[..name_len];
        if name.is_empty() || !after[name_len..].starts_with('}') {
            out.push('{');
            rest = after;
            continue;
        }
        if lists.iter().any(|(n, _)| *n == name) {
            return Err(format!(
                "`{{{name}}}` is a list, which fills a word of its own, so it stands alone \
                 rather than inside `{word}`"
            ));
        }
        let Some((_, value)) = vars.iter().find(|(n, _)| *n == name) else {
            let offered: Vec<String> = vars
                .iter()
                .map(|(n, _)| *n)
                .chain(lists.iter().map(|(n, _)| *n))
                .map(|n| format!("{{{n}}}"))
                .collect();
            return Err(format!(
                "`{{{name}}}` is not a placeholder here, which offers {}",
                offered.join(", ")
            ));
        };
        out.push_str(value);
        rest = &after[name_len + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Run `commands` in order from `root`, printing each before it
/// runs, stopping at the first failure. `key` names the list in the
/// messages, `label` prefixes them, `env` is added to each command's
/// environment, and `vars` and `lists` are the placeholders offered
/// (see [`expand`]).
pub fn run(
    root: &Path,
    key: &str,
    commands: &[String],
    env: &[(&str, &str)],
    vars: &[(&str, &str)],
    lists: &[(&str, &[String])],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = commands.len();
    for (i, command) in commands.iter().enumerate() {
        // The command as it runs, its placeholders filled, so the log
        // shows the paths it used.
        let shown = if vars.is_empty() && lists.is_empty() {
            command.clone()
        } else {
            expand(command, vars, lists)
                .map_err(|e| format!("{label}: `{command}`: {e}"))?
                .join(" ")
        };
        let command = &shown;
        info!("{label}: [{}/{n}] {command}", i + 1);
        let status = run_one(root, &commands[i], env, vars, lists, label)?;
        if !status.success() {
            return Err(format!(
                "{label}: `{command}` failed ({}), stopping at {}/{n} of {key}",
                describe_status(status),
                i + 1,
            )
            .into());
        }
    }
    info!("{label}: {n} command(s) passed ({key})");
    Ok(())
}

/// Run one command line from `root`, inheriting stdio and adding
/// `env` to its environment, and return its exit status. The line
/// is split on whitespace and its placeholders filled from `vars` and
/// `lists` (see [`expand`]), the first word the program, so no shell
/// sits between vc-x1 and the command's own status.
pub fn run_one(
    root: &Path,
    command: &str,
    env: &[(&str, &str)],
    vars: &[(&str, &str)],
    lists: &[(&str, &[String])],
    label: &str,
) -> Result<std::process::ExitStatus, String> {
    let words = expand(command, vars, lists).map_err(|e| format!("{label}: `{command}`: {e}"))?;
    let Some((program, args)) = words.split_first() else {
        return Err(format!("{label}: an empty command element"));
    };
    // Register entry 5 (clippy.toml): a configured command list,
    // `[validate]`'s, `[build]`'s, or `[install]`'s, is run as the
    // user would type it, which is the whole job of the commands that
    // run one.
    #[allow(clippy::disallowed_methods)]
    let mut cmd = std::process::Command::new(program);
    cmd.args(args).current_dir(root).envs(env.iter().copied());
    cmd.status().map_err(|e| {
        format!(
            "{label}: cannot run `{command}` from '{}': {e}",
            root.display()
        )
    })
}

/// Render an exit status for the failure message: the code when
/// there is one, else the raw status (a signal on Unix).
fn describe_status(status: std::process::ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => format!("{status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Unique tempdir for a one-file workspace.
    fn workspace(tag: &str, config: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("vc-x1-cmdlist-{tag}-{ts}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".vc-config.md"), config).unwrap();
        root
    }

    /// A missing list and an empty list are errors naming the key
    /// and carrying the caller's label.
    #[test]
    fn missing_or_empty_list_errors() {
        let root = workspace("missing", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = load(&root, "validate.full", "validate").unwrap_err();
        assert!(e.starts_with("validate: "), "{e}");
        assert!(e.contains("no `validate.full` table"), "{e}");
        let root2 = workspace(
            "empty",
            "```toml\n[repos]\nwork = \".\"\n[build]\ndev = []\n```\n",
        );
        let e = load(&root2, "build.dev", "build").unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("`build.dev` is empty"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// An optional list that is absent is `None`, a present one is
    /// read, and an empty one is the same error as `load`'s.
    #[test]
    fn an_optional_list_may_be_absent() {
        let root = workspace(
            "optional",
            "```toml\n[repos]\nwork = \".\"\n[install]\nrelease = [\"sh a.sh\"]\ndev = []\n```\n",
        );
        assert_eq!(
            load_optional(&root, "install.none", "install").unwrap(),
            None
        );
        assert_eq!(
            load_optional(&root, "install.release", "install").unwrap(),
            Some(vec!["sh a.sh".to_string()])
        );
        let e = load_optional(&root, "install.dev", "install-dev").unwrap_err();
        assert!(e.starts_with("install-dev: "), "{e}");
        assert!(e.contains("`install.dev` is empty"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// Placeholders fill inside a word, a value with spaces stays one
    /// word, a list offered none is as written, any brace that is not
    /// a placeholder is literal, and an unknown one is an error naming
    /// the ones offered.
    #[test]
    fn a_list_fills_words_of_its_own() {
        let flags = vec!["--profile".to_string(), "release-lto".to_string()];
        let none: Vec<String> = vec![];
        let vars = [("profile.bin", "target/x/app")];
        let lists = [
            ("profile.flags", flags.as_slice()),
            ("profile.none", none.as_slice()),
        ];
        assert_eq!(
            expand("cargo build {profile.flags} --locked", &vars, &lists).unwrap(),
            ["cargo", "build", "--profile", "release-lto", "--locked"]
        );
        assert_eq!(
            expand("cargo build {profile.none} --locked", &vars, &lists).unwrap(),
            ["cargo", "build", "--locked"]
        );
        assert_eq!(
            expand("cp {profile.bin} x", &vars, &lists).unwrap(),
            ["cp", "target/x/app", "x"]
        );
        let e = expand("cargo x{profile.flags}", &vars, &lists).unwrap_err();
        assert!(
            e.contains("is a list, which fills a word of its own"),
            "{e}"
        );
        let e = expand("cp {profile.dir} x", &vars, &lists).unwrap_err();
        assert!(
            e.contains("{profile.bin}, {profile.flags}, {profile.none}"),
            "{e}"
        );
    }

    /// Split and fill with string placeholders alone.
    fn words(command: &str, vars: &[(&str, &str)]) -> Result<Vec<String>, String> {
        expand(command, vars, &[])
    }

    #[test]
    fn placeholders_fill_words() {
        let vars = [
            ("build.release-bin", "/a b/app"),
            ("install.release-dir", "/bin"),
        ];
        assert_eq!(
            words(
                "cp {build.release-bin} {install.release-dir}/app.new",
                &vars
            )
            .unwrap(),
            ["cp", "/a b/app", "/bin/app.new"]
        );
        assert_eq!(words("echo {x}", &[]).unwrap(), ["echo", "{x}"]);
        assert_eq!(
            words("echo {} {a b} x{ }y {install.release-dir}{", &vars).unwrap(),
            ["echo", "{}", "{a", "b}", "x{", "}y", "/bin{"]
        );
        let e = words("cp {build.dev-bin} x", &vars).unwrap_err();
        assert!(e.contains("`{build.dev-bin}` is not a placeholder"), "{e}");
        assert!(
            e.contains("{build.release-bin}, {install.release-dir}"),
            "{e}"
        );
        let e = check_placeholders(
            &["cp {install.release-dir} x".to_string()],
            &["install.dir"],
            &[],
            "install.commands",
            "install",
        )
        .unwrap_err();
        assert!(
            e.starts_with("install: `install.commands`: `{install.release-dir}`"),
            "{e}"
        );
    }

    /// A list name is accepted standing alone and refused inside a
    /// longer word, and an unknown name is refused naming both kinds.
    #[test]
    fn check_placeholders_takes_list_names() {
        let check = |command: &str| {
            check_placeholders(
                &[command.to_string()],
                &["profile.cargo"],
                &["profile.flags"],
                "build.commands",
                "build",
            )
        };
        check("cargo build {profile.flags} --profile {profile.cargo}").unwrap();
        let e = check("cargo x{profile.flags}").unwrap_err();
        assert!(e.contains("is a list"), "{e}");
        let e = check("cargo {profile.bin}").unwrap_err();
        assert!(e.contains("{profile.cargo}, {profile.flags}"), "{e}");
    }

    /// Commands run in order and the first failure stops the run,
    /// naming the command, its exit status, and the list.
    #[test]
    fn stops_at_first_failure() {
        let root = workspace("stop", "```toml\n[repos]\nwork = \".\"\n```\n");
        let commands = vec![
            "true".to_string(),
            "false".to_string(),
            "touch never.txt".to_string(),
        ];
        let e = run(&root, "validate.fast", &commands, &[], &[], &[], "validate")
            .unwrap_err()
            .to_string();
        assert!(e.contains("failed (exit 1)"), "{e}");
        assert!(e.contains("2/3 of validate.fast"), "{e}");
        assert!(!root.join("never.txt").exists(), "the run did not stop");
        std::fs::remove_dir_all(&root).ok();
    }

    /// All passing runs every command from the root.
    #[test]
    fn all_pass_runs_from_root() {
        let root = workspace("pass", "```toml\n[repos]\nwork = \".\"\n```\n");
        let commands = vec!["touch ran.txt".to_string(), "true".to_string()];
        run(&root, "validate.full", &commands, &[], &[], &[], "validate").unwrap();
        assert!(root.join("ran.txt").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    /// The caller's variables reach each command's environment.
    #[test]
    fn env_reaches_the_commands() {
        let root = workspace("env", "```toml\n[repos]\nwork = \".\"\n```\n");
        // printenv exits 0 when the variable is set and 1 when not.
        let commands = vec!["printenv VC_X1_TEST_CMDLIST".to_string()];
        run(
            &root,
            "build.commands",
            &commands,
            &[("VC_X1_TEST_CMDLIST", "x")],
            &[],
            &[],
            "build",
        )
        .unwrap();
        let e = run(&root, "build.commands", &commands, &[], &[], &[], "install").unwrap_err();
        assert!(e.to_string().contains("failed (exit 1)"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// An unrunnable program is an error naming the command.
    #[test]
    fn unrunnable_program_errors() {
        let root = workspace("norun", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = run_one(&root, "no-such-program-xyz --flag", &[], &[], &[], "build").unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("cannot run `no-such-program-xyz --flag`"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }
}
