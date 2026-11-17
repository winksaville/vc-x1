//! `cargo xtask completions [NAME...]`: print a bash function that
//! completes `cargo xtask` and each NAME, a shell alias for it, by
//! asking the xtask of the repo the shell is in.
//!
//! The arguments are clap's, whose completion engine answers when
//! `COMPLETE` is set, as `vc-x1`'s does, and `run`'s arguments are the
//! binary's to complete ([`super::run::forward_completion`]).

use std::process::ExitCode;

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Print a bash function that completes `cargo xtask` and its aliases",
    long_about = "\
        Print a bash function that completes `cargo xtask`, and each NAME, a\n\
        shell alias for it such as `cx`. It holds no task names: on each tab\n\
        it asks the xtask of the repo the shell is in, so one copy in a\n\
        shell's startup file serves every repo whose xtask answers this way.\n\n\
        `cargo xtask <tab>` reaches it through cargo's own bash completion,\n\
        which hands a subcommand it does not know to the completion\n\
        registered for `cargo-<name>`, and needs bash-completion 2.12 or\n\
        later. A NAME is completed directly.\n\n\
        Save it once and source that file, the function being the same in\n\
        every repo:\n\
        \x20 cargo xtask completions cx > ~/.config/cargo-xtask-completion.bash"
)]
pub struct Args {
    /// A shell alias for `cargo xtask`, completed as it is
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,
}

/// Print the function.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    println!("{}", completion_script(&args.names).trim_end());
    Ok(ExitCode::SUCCESS)
}

/// The bash function `completions` prints, registered for
/// `cargo-xtask`, the name cargo's completion hands `cargo xtask` to,
/// and for each of `names`.
///
/// It is clap's own registration script with one change: where that
/// calls one binary by name, this runs `cargo xtask`, so the answer is
/// the xtask's of the repo the shell is in. A repo with no xtask, or
/// one that fails, completes nothing.
fn completion_script(names: &[String]) -> String {
    let mut registered = vec!["cargo-xtask".to_string()];
    registered.extend(names.iter().cloned());
    format!(
        r#"# Completion for `cargo xtask`, printed by `cargo xtask completions`.
# It asks the xtask of the repo the shell is in, on each tab.
_cargo_xtask_complete() {{
    local IFS=$''
    local _CLAP_COMPLETE_INDEX=${{COMP_CWORD}}
    local _CLAP_COMPLETE_COMP_TYPE=${{COMP_TYPE}}
    if compopt +o nospace 2> /dev/null; then
        local _CLAP_COMPLETE_SPACE=false
    else
        local _CLAP_COMPLETE_SPACE=true
    fi
    local words=("${{COMP_WORDS[@]}}")
    if [[ "${{BASH_VERSINFO[0]}}" -ge 4 ]]; then
        words[COMP_CWORD]="$2"
    fi
    COMPREPLY=( $(         _CLAP_IFS="$IFS"         _CLAP_COMPLETE_INDEX="$_CLAP_COMPLETE_INDEX"         _CLAP_COMPLETE_COMP_TYPE="$_CLAP_COMPLETE_COMP_TYPE"         _CLAP_COMPLETE_SPACE="$_CLAP_COMPLETE_SPACE"         COMPLETE="bash"         cargo xtask -- "${{words[@]}}" 2> /dev/null     ) )
    if [[ $? != 0 ]]; then
        unset COMPREPLY
    elif [[ $_CLAP_COMPLETE_SPACE == false ]] && [[ "${{COMPREPLY-}}" =~ [=/:]$ ]]; then
        compopt -o nospace
    fi
}}
if [[ "${{BASH_VERSINFO[0]}}" -eq 4 && "${{BASH_VERSINFO[1]}}" -ge 4 || "${{BASH_VERSINFO[0]}}" -gt 4 ]]; then
    complete -o nospace -o bashdefault -o nosort -F _cargo_xtask_complete {names}
else
    complete -o nospace -o bashdefault -F _cargo_xtask_complete {names}
fi
"#,
        names = registered.join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The function asks `cargo xtask`, not a binary by name, and is
    /// registered for cargo's hand-off name and each alias given.
    #[test]
    fn the_completion_function_asks_the_repos_xtask() {
        let script = completion_script(&["cx".to_string()]);
        assert!(
            script.contains("cargo xtask -- \"${words[@]}\" 2> /dev/null"),
            "{script}"
        );
        // Registered twice over, as clap's script is: unsorted where
        // bash can, 4.4 and later, so the tasks keep their order.
        assert!(
            script.contains("-o nosort -F _cargo_xtask_complete cargo-xtask cx\n"),
            "{script}"
        );
        assert!(
            script.ends_with("-F _cargo_xtask_complete cargo-xtask cx\nfi\n"),
            "{script}"
        );
        assert!(completion_script(&[]).ends_with("_cargo_xtask_complete cargo-xtask\nfi\n"));
    }
}
