//! Init's steps, numbered in the order a plan runs them.
//!
//! One list serves the dry run, which prints it whole, and the real
//! run, which prints each step as it starts, so the two cannot
//! number or name a step differently. A step a plan does not run is
//! left out rather than printed as skipped, so the numbers run
//! 1..=N with no gaps.

use std::path::PathBuf;

use log::info;

use super::{AgentPlan, InitParams, InitPlan, Provisioner};
use crate::options_flags::config::ConfigKind;

/// One step of an init run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    PrepareWork,
    ConfigWork,
    CommitWork,
    PrepareAgent,
    ConfigAgent,
    CommitAgent,
    CrossLink,
    PublishAgent,
    PublishWork,
    Symlink,
}

/// A plan's steps, each with its title.
pub(crate) struct Steps {
    list: Vec<(Step, String)>,
}

impl Steps {
    /// The steps `plan` runs, titled with what each will do to what.
    pub(crate) fn new(
        plan: &InitPlan,
        params: &InitParams,
        templates: Option<&(PathBuf, Option<PathBuf>)>,
        visibility: &str,
        create_symlink: bool,
    ) -> Self {
        let work_template = templates.map(|(w, _)| w);
        let agent_template = templates.and_then(|(_, a)| a.as_ref());
        let mut list = Vec::new();

        let work_dir = plan.project_dir.display();
        let prepare_work = if params.adopt {
            format!("Init the work repo (jj, colocated) in {work_dir}, adopting its content")
        } else {
            format!("Create {work_dir} and init the work repo (jj, colocated)")
        };
        list.push((
            Step::PrepareWork,
            with_template(prepare_work, work_template),
        ));
        let config_work = match (&plan.agent, &params.config) {
            (Some(_), _) | (None, None) => {
                "Write the work repo's .vc-config.md and .gitignore".into()
            }
            (None, Some(ConfigKind::None)) => {
                "Write the work repo's .gitignore (no .vc-config.md, per --config none)".into()
            }
            (None, Some(ConfigKind::Path(p))) => format!(
                "Copy {} as the work repo's config and write its .gitignore",
                p.display()
            ),
        };
        list.push((Step::ConfigWork, config_work));
        list.push((
            Step::CommitWork,
            "Commit the work repo's initial commit".into(),
        ));

        if let Some(agent) = &plan.agent {
            let prepare_agent = format!(
                "Create {} and init the agent repo (jj, colocated)",
                agent.path.display()
            );
            list.push((
                Step::PrepareAgent,
                with_template(prepare_agent, agent_template),
            ));
            list.push((
                Step::ConfigAgent,
                "Write the agent repo's .vc-config.md and .gitignore".into(),
            ));
            list.push((
                Step::CommitAgent,
                "Commit the agent repo's initial commit".into(),
            ));
            list.push((
                Step::CrossLink,
                "Cross-link the two initial commits with ochid trailers".into(),
            ));
            list.push((
                Step::PublishAgent,
                publish_title(plan, "agent", Some(agent), visibility),
            ));
        }
        list.push((
            Step::PublishWork,
            publish_title(plan, "work", None, visibility),
        ));
        if plan.agent.is_some() && create_symlink {
            list.push((Step::Symlink, "Create the Claude Code symlink".into()));
        }
        Self { list }
    }

    /// Print the whole list, for the dry run.
    pub(crate) fn print_all(&self) {
        for (i, (_, title)) in self.list.iter().enumerate() {
            info!("  {}. {title}", i + 1);
        }
    }

    /// Announce `step` as it starts, numbered by its place in the
    /// list. A step the list lacks is a bug in the caller, logged
    /// unnumbered rather than failing the run it narrates.
    pub(crate) fn begin(&self, step: Step) {
        match self.list.iter().position(|(s, _)| *s == step) {
            Some(i) => info!("Step {}: {}", i + 1, self.list[i].1),
            None => info!("Step: {step:?}"),
        }
    }

    /// The steps, in order: for tests.
    #[cfg(test)]
    pub(crate) fn steps(&self) -> Vec<Step> {
        self.list.iter().map(|(s, _)| *s).collect()
    }
}

/// `title`, with the template copy it includes when there is one.
fn with_template(title: String, template: Option<&PathBuf>) -> String {
    match template {
        Some(t) => format!("{title}, then copy template {}", t.display()),
        None => title,
    }
}

/// A publish step's title: set `main`, provision the remote the plan
/// names, and push. `agent` is the agent side's plan, `None` for the
/// work side.
fn publish_title(
    plan: &InitPlan,
    side: &str,
    agent: Option<&AgentPlan>,
    visibility: &str,
) -> String {
    let (url, slug, bare) = match agent {
        Some(a) => (a.url.as_str(), a.gh_slug.as_deref(), a.bare_path.as_ref()),
        None => (
            plan.work_url.as_str(),
            plan.gh_work_slug.as_deref(),
            plan.work_bare_path.as_ref(),
        ),
    };
    let remote = match (&plan.provisioner, slug, bare) {
        (Provisioner::GhCreate, Some(slug), _) => format!("create GitHub repo {slug} {visibility}"),
        (Provisioner::LocalBareInit, _, Some(bare)) => {
            format!("create bare repo {}", bare.display())
        }
        _ => "use the existing remote".to_string(),
    };
    format!("Set main on the {side} repo's initial commit, {remote}, and push to {url}")
}
