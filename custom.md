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

- **One home for a cycle's narrative** (adopted at 0.77.0-2; narrowed 2026-08-02)
  - overrides the per-commit chores build-up in
    [agent-data/notes.md](agent-data/notes.md#chores-section-content-no-edit-list-git-is-the-record)
    and [cycle-protocol.md](notes/cycle-protocol.md#chores-sections)
  - while a cycle runs, its ladder and narrative live only in `TODO.md > ## In Progress`
  - the chores section is created at close-out by moving the whole block in, opening with the
    as-built ladder per the now-universal
    [chores commit references](agent-data/notes.md#chores-commit-references)
  - narrowed at the 2026-08-02 sync: the rung `[[N]]` backfill clause that lived here became
    the universal rule, so only the one-home timing above remains an override
  - tracked by the `## Todo` entry "One home for a cycle's narrative", which fans the doc
    change out to the template family
- **Mailbox parameters** (the acquaint-time check itself is the pinned AGENTS.md practice)
  - member name `vc-x1`; mailbox `../vc-x1-template/messages/vc-x1.md`
  - the message protocol is `../vc-x1-template/MESSAGES.md`
- **Cycle bookend titles: 0.78.0 is this repo's adoption boundary** (the rule itself is pinned
  in prose.md since the 20260802 snapshot)
  - retro-applied to `0.78.0-0` by a coordinated re-describe + force-push, safe because the
    ladder was unmerged branch-only history
  - earlier cycles' openings stay grandfathered, since they sit under shipped close-out refs
- **Version advancement is scope-based: minor for program/architecture cycles, patch for
  incremental ones** (adopted 2026-08-02, wink; iiac-perf's 2026-08-01 rule, worded for this
  repo)
  - minor when a refactor-program stage or an architectural change lands (the 0.7x.0 line)
  - patch for incremental cycles and interludes relative to the larger scope: docs, sweeps,
    small fixes (0.77.1-0.77.4 were already this shape)
  - first application: the source sweep targets 0.78.1
  - a tier-2 graduation candidate; on tomorrow's family agenda
- **Close-out shape default: trapezoid**
  - published by [the recipe](notes/cycle-protocol.md#trapezoid-close-out-recipe)
  - the refactor program pushes rungs 1:1 to the long-lived `refactor-vc-x1` bookmark, treated
    as permanent (merge-only onto `main`, never rebased)

## Dogfood log

Dated entries on where these instructions chafed, failed, or got amended; the evidence base
for promoting the local copy back to the template repository (vc-x1-template).

- 2026-08-02: synced to `AGENTS-vc-x1-f5-20260802-snapshot/`, the tier-1 graduation this
  session authored template-side
  - the 0802 snapshot is the 0730 one plus the graduation of the conventions the two repos
    dogfooded: write-to-full-width, cycle bookend titles, the checklist's close-the-records
    step, the mailbox check at acquaint
  - the 0730 amendments landed en route, this repo having run on the pre-amendment set since
    0.78.0-1: rule 0, hard-rules-first, generic pin lines, chores as-built ladder, chores
    `## Table of Contents`
  - cut as a new snapshot directory, not an in-place amendment: the template repo carries no
    commits, so amending 0730 in place would have destroyed the adoption record (wink's call)
  - notes/cycle-protocol.md amended to match the ladder form (Chores sections, Commits
    backfill)
  - the one-home override narrowed, its backfill clause having become the universal rule
  - bookends retro-applied to `0.78.0-0` by a coordinated re-describe + force-push of the
    unmerged ladder
  - two 0730 prose.md findings, both fixed in the 0802 snapshot: the "Conventional-commit
    shape" chores bullet still described the retired `Commits:` line, and the "Banned:"
    opening contradicted the transcription exception (now "the prohibition is on authoring")
  - tier 2 staged for iiac-perf's read (mailbox message): one-home, cycle-protocol.md into
    the byte-identical set, every-commit-belongs-to-a-cycle, scope-based version advancement
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
