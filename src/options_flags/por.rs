//! `-s/--scope both|por`: what `init` and `clone` create, a dual
//! workspace or a plain single repo, with the retired `--por` kept
//! hidden to be refused. See [options_flags](README.md) for shared
//! architecture.

use clap::Args;

use super::scope::{RepoScope, repo_scope_parser};

/// The creation scope leaf: see
/// [Consuming an OF](README.md#consuming-an-of).
///
/// Workspace topology selector at creation time, spelled `-s` as on
/// every write command:
///
/// - `both`, the default: dual workspace (work + agent repo).
/// - `por`: plain single repo, no agent repo.
/// - `work` and `agent` alone are refused, a workspace being created
///   whole, and `--por`, the old spelling, is refused naming
///   `--scope por`.
#[derive(Args, Debug, Clone, Default)]
pub struct PorFlag {
    /// What to create, `b|both` a dual workspace (work + agent repo)
    /// or `p|por` a plain single repo [default: both]
    #[arg(
        short = 's',
        long = "scope",
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["both", "por"])
    )]
    pub scope: Option<RepoScope>,

    /// Retired: `--por`, now `--scope por`, refused naming it.
    #[arg(long = "por", hide = true)]
    pub retired_por: bool,
}

impl PorFlag {
    /// True for a plain single repo, `-s por`. Total, `check` having
    /// refused what it cannot mean.
    pub fn is_por(&self) -> bool {
        matches!(self.scope, Some(RepoScope::Por))
    }

    /// Refuse the retired `--por` on `command`, naming `--scope por`,
    /// and a scope of one side alone.
    pub fn check(&self, command: &str) -> Result<(), String> {
        if self.retired_por {
            return Err(format!(
                "{command}: --por is retired, the scope is a flag on every write command: \
                 `--scope por`"
            ));
        }
        match &self.scope {
            Some(RepoScope::Sides(sides)) if sides.0.len() == 1 => Err(format!(
                "{command} -s: a workspace is created whole, so the scope is `both` or `por`, \
                 not one side"
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        p: PorFlag,
    }

    /// `-s por` is a plain repo, `-s both` and none a dual workspace,
    /// one side alone and `--por` refused naming the fix.
    #[test]
    fn the_scope_selects_the_topology() {
        let p = T::try_parse_from(["t", "-s", "por"]).unwrap().p;
        assert!(p.is_por() && p.check("init").is_ok());
        let p = T::try_parse_from(["t", "-s", "b"]).unwrap().p;
        assert!(!p.is_por() && p.check("init").is_ok());
        let p = T::try_parse_from(["t"]).unwrap().p;
        assert!(!p.is_por() && p.check("init").is_ok());
        let p = T::try_parse_from(["t", "-s", "work"]).unwrap().p;
        assert!(p.check("init").unwrap_err().contains("`both` or `por`"));
        let p = T::try_parse_from(["t", "--por"]).unwrap().p;
        assert!(p.check("clone").unwrap_err().ends_with("`--scope por`"));
    }
}
