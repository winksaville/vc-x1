//! `cargo xtask <name> [ARGS]...`: a task the workspace declares in
//! `[workspace.metadata.xtask.tasks]`, its commands run in order as
//! `validate`'s are, stopping at the first failure, the arguments
//! after the name appended to the last command.
//!
//! A declared task is how a project adds its own task and leaves
//! `xtask/` verbatim, so a newer copy of `xtask/` replaces it whole. A
//! name a built-in task has is refused, never run, since the built-in
//! would take it.

use std::process::ExitCode;

use clap::Command;

use crate::provenance::Provenance;
use crate::util::{command_at, root, status};
use crate::workspace::Workspace;

/// Run the declared task `words[0]`, the rest its arguments. `builtin`
/// names the built-in tasks, which a declared one may not shadow.
pub fn run(words: &[String], builtin: &[String]) -> Result<ExitCode, String> {
    let Some((name, args)) = words.split_first() else {
        return Err("no task named".to_string());
    };
    let root = root();
    let ws = Workspace::read(&root)?;
    refuse_shadows(&ws, builtin)?;
    let Some(lists) = ws.tasks.get(name) else {
        return Err(format!(
            "no task `{name}`: the built-in tasks are {}, and the workspace's {}",
            builtin.join(", "),
            match ws.tasks.is_empty() {
                true => "none, [workspace.metadata.xtask.tasks] declaring them".to_string(),
                false => ws.tasks.keys().cloned().collect::<Vec<_>>().join(", "),
            }
        ));
    };
    let lists = with_args(lists, args);
    let provenance = Provenance::read(&root);
    for (i, words) in lists.iter().enumerate() {
        let what = format!("`{}`", words.join(" "));
        eprintln!("{name}: {what}");
        let cmd = command_at(&root, provenance.as_ref(), words);
        status(cmd, &what)
            .map_err(|e| format!("{name}: {e}, command {} of {}", i + 1, lists.len()))?;
    }
    Ok(ExitCode::SUCCESS)
}

/// The task's commands with `args` appended to the last.
fn with_args(lists: &[Vec<String>], args: &[String]) -> Vec<Vec<String>> {
    let mut lists = lists.to_vec();
    if let Some(last) = lists.last_mut() {
        last.extend(args.iter().cloned());
    }
    lists
}

/// Refuse a workspace whose declared task takes a built-in's name.
fn refuse_shadows(ws: &Workspace, builtin: &[String]) -> Result<(), String> {
    let shadows: Vec<&String> = ws.tasks.keys().filter(|n| builtin.contains(n)).collect();
    match shadows.as_slice() {
        [] => Ok(()),
        names => Err(format!(
            "[workspace.metadata.xtask.tasks] declares {}, a built-in task's name, which the \
             built-in would take: rename it",
            names
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// `cmd` with a subcommand for each task the workspace declares, so
/// completion and help offer them beside the built-ins. A workspace
/// that cannot be read adds none, quietly, completion being no place
/// for its error.
pub fn with_declared(cmd: Command) -> Command {
    let Ok(ws) = Workspace::read(&root()) else {
        return cmd;
    };
    let builtin: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    ws.tasks
        .iter()
        .filter(|(name, _)| !builtin.contains(name))
        .fold(cmd, |cmd, (name, lists)| {
            let about = format!(
                "The workspace's task: {}",
                lists
                    .iter()
                    .map(|w| format!("`{}`", w.join(" ")))
                    .collect::<Vec<_>>()
                    .join(", then ")
            );
            cmd.subcommand(
                Command::new(name.clone()).about(about).arg(
                    clap::Arg::new("args")
                        .num_args(0..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn ws(tasks: &[(&str, &[&[&str]])]) -> Workspace {
        Workspace {
            target_dir: PathBuf::from("/w/target"),
            bins: Vec::new(),
            default_bin: None,
            default_run: None,
            validate: None,
            validate_fast: None,
            tasks: tasks
                .iter()
                .map(|(name, lists)| {
                    let lists = lists
                        .iter()
                        .map(|w| w.iter().map(|s| s.to_string()).collect())
                        .collect();
                    (name.to_string(), lists)
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    /// The arguments go to the last command, and a declared task with
    /// a built-in's name is refused naming it.
    #[test]
    fn a_declared_task_takes_its_arguments_last_and_shadows_nothing() {
        let lists = vec![
            vec!["cargo".to_string(), "fmt".to_string()],
            vec!["cargo".to_string(), "doc".to_string()],
        ];
        assert_eq!(
            with_args(&lists, &["--open".to_string()]),
            [vec!["cargo", "fmt"], vec!["cargo", "doc", "--open"]]
        );
        let builtin = vec!["build".to_string(), "run".to_string()];
        assert!(refuse_shadows(&ws(&[("docs", &[&["cargo", "doc"]])]), &builtin).is_ok());
        let e = refuse_shadows(&ws(&[("build", &[&["make"]])]), &builtin).unwrap_err();
        assert!(
            e.contains("declares `build`, a built-in task's name"),
            "{e}"
        );
    }
}
