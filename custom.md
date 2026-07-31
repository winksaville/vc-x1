# custom.md - vc-x1's project layer

The one agent-editable instruction file (see [AGENTS.md](AGENTS.md#custommd-the-project-layer)).
Loaded after AGENTS.md; on conflict, this file wins.

## Medium and validation

The artifact is the `vc-x1` CLI, a Rust crate (manifest `Cargo.toml`, package name `vc-x1-dev`;
see [versioning.md](notes/versioning.md#dev-artifact-name)).

- **Full validation**
  - when: per-commit checklist step 4; skip-able for notes-only commits, mandatory at close-out
  - run as separate invocations, each exit status checked:
    1. `cargo fmt`
    2. `cargo clippy --all-targets -- -D warnings`
    3. `cargo test`
    4. `cargo install --path . --locked`
    5. (re-test if anything substantive changed)
- **Fast validation**
  - when: ladder checklist step 3
  - `cargo test --bins`
- **Pipelines hide failures**
  - never pipe a validating command into `tail`/`grep`
  - never `&&` after a piped stage: a pipeline's status is the last command's
  - `${PIPESTATUS[0]}`: the escape hatch when a pipe is genuinely wanted

## Project conventions and overrides

- **One home for a cycle's narrative** (adopted at 0.77.0-2)
  - overrides the per-commit chores build-up in
    [agent-data/notes.md](agent-data/notes.md#chores-section-content-no-edit-list-git-is-the-record)
    and [cycle-protocol.md](notes/cycle-protocol.md#chores-sections)
  - while a cycle runs, its ladder and narrative live only in `TODO.md > ## In Progress`
  - the chores section is created at close-out by moving the whole block in
  - rung `[[N]]` placeholders are backfilled in place as pushes make commits permanent
  - tracked by the `## Todo` entry "One home for a cycle's narrative", which fans the doc
    change out to the template family
- **Close-out shape default: trapezoid**
  - published by [the recipe](notes/cycle-protocol.md#trapezoid-close-out-recipe)
  - the refactor program pushes rungs 1:1 to the long-lived `refactor-vc-x1` bookmark, treated
    as permanent (merge-only onto `main`, never rebased)

## Dogfood log

Dated entries on where these instructions chafed, failed, or got amended; the evidence base
for promoting the local copy back to vc-x1-work-repo-template.

- 2026-07-30: adopted at 0.78.0-1
  - semantics-preserving restructure of AGENTS.md + cycle-protocol satellites
  - proposal snapshot frozen in the template as `AGENTS-vc-x1-f5-20260730.md`
- 2026-07-30: conventions clarified by the user at the 0.78.0-1 review
  - amends prose.md, code.md, cycle-protocol.md
  - title <=72 (was 50), with optional `(scope)`
  - docs wrap <=100, commit title/body at <=72
  - commit-body bullets are sentence fragments
  - the version-first bullet is spelled `vX.Y.Z-xxxx`
- 2026-07-30: version protocol defined (user + bot)
  - amends versioning.md (new Grammar and storage section) and cycle-protocol.md
  - one prose spelling `X.Y.Z-<dot-nested suffix>`, exactly one dash ever
  - `v`: a display-only prefix, never stored
  - per-medium storage: SemVer verbatim, PEP 440 remaps the `-` to `+`
  - stored versions identify, never order
  - `+` reserved for the remap
  - driven by a sibling repo's Python linter incident and a packaging-26.2 parser test
