# custom.md - the project layer

The project's own layer over the pinned agent-files (see
[AGENTS.md](AGENTS.md#custommd-the-project-layer)). Loaded last. On conflict, this file wins.

## Medium

The artifact is the `vc-x1` CLI, a Rust crate (manifest `Cargo.toml`, package name `vc-x1`).
Versioning specifics are in [versioning.md](agent-data/versioning.md). The validation commands
are not here: they are the work side's `[validate]` table in `.vc-config.md`, run by
`vc-x1 validate` and `vc-x1 validate --fast`, as the pinned checklists say.

**Single-name convention** (adopted at the 0.78.3 cycle): the package name is the binary's
name, `vc-x1` on `main`, with per-line dev names on long-lived branches, guarded by `build.rs`
on every cargo verb. This is the project's answer to versioning.md's
[Dev artifact name](agent-data/versioning.md#dev-artifact-name): the repo is striving to
release, so `main` builds install as plain `vc-x1`. The opening's bump sets the dev name with
the suffixed version, and the closing's bump sets the bare one under the dev name still. The
name itself is restored to `vc-x1` at the trapezoid recipe's step 2
([Trapezoid close-out recipe](agent-data/jj.md#trapezoid-close-out-recipe)), so the bookmark
stays `vc-x1-dev` until it is reshaped for landing (the restore was missed at two closings
before this line was written, 2026-08-22).

## Project conventions and overrides

Project-local conventions, and overrides of the pinned files. An override names the section it
supersedes. A project whose further layer holds these keeps this section empty.

_None._

## Dogfood log

This project's log lives in [notes/dogfood.md](notes/dogfood.md): dated entries on where the
pinned instructions chafed, failed, or got amended, the evidence base for the family's
convergence decisions ([Changing the agent-files](AGENTS.md#changing-the-agent-files)). In-flight
entries only: a resolved entry retires at the beat where it resolves, per
[Retiring Done entries](agent-data/notes.md#retiring-done-entries).
