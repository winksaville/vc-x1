# custom-family.md - vc-x1's layer, as a member of the vc-x1 agent-file family

Read after [custom.md](custom.md), whose single convention entry points here. Present only in a
member repo. A project that is not a member does not carry this file, and nothing in it applies
to one.

**A member's `custom.md` differs from the payload by that one line, and everything of its own lives
here.** That is the family's convention and it buys a property worth having: `diff custom.md
<template>/work/custom.md` is one line for every member, so the file that can never be pinned still
converges, and a member's whole customization surface is this file. The medium below is here for
that reason rather than because it has anything to do with the family.

## Medium and validation

The artifact is the `vc-x1` CLI, a Rust crate (manifest `Cargo.toml`, package name `vc-x1`).
Versioning specifics are in [versioning.md](agent-data/versioning.md).

**What a version bump promises**: `X.Y.Z` is a scope signal to readers of the history (see
[Advancing X.Y.Z](agent-data/versioning.md#advancing-xyz-scope-decides)), not a compatibility
contract. The crate has no library target, but the built CLI has external consumers (the family's
repos run it), so a change that breaks an existing invocation or the `.vc-config.toml` schema is
called out in the commit and surfaced by the tool's own errors rather than promised by the
version. Revisit with a compat clause if a library crate ever splits out.

**Single-name convention** (adopted at the 0.78.3 cycle): the package name is the binary's
name, `vc-x1` on `main`, with per-line dev names on long-lived branches, guarded by `build.rs`
on every cargo verb. This is the project's answer to versioning.md's
[Dev artifact name](agent-data/versioning.md#dev-artifact-name): the repo is striving to
release, so `main` builds install as plain `vc-x1`.

- **Full validation**
  - when: per-commit checklist step 5 (skip-able for notes-only commits, mandatory at close-out)
  - run as separate invocations, each exit status checked:
    1. `cargo fmt`
    2. `cargo clippy --all-targets -- -D warnings`
    3. `cargo test`
    4. `cargo install --path . --locked`
    5. (re-test if anything substantive changed)
- **Fast validation**
  - when: ladder checklist step 3
  - `cargo test --bins`

## Membership

- **member name**: `vc-x1`
- **template repository**: `../vc-x1-template`

These two are environment rather than instruction, the same species as `.vc-config.toml`'s
`[repos]` paths, and they belong there instead. They cannot move yet: `vc-x1 config --validate`
rejects keys it does not know, so a config carrying them would fail its own validator. The
`[private]` table proposal tracks the fix.

## Messaging

Member notifications run through `../vc-x1-messages`, a sibling repo. Its `README.md` is the
protocol and governs, and our record file there is `vc-x1.md`, whose own header declares its
persistence policy. This file adds only the session behavior the protocol cannot know:

- **At acquaint, check `../vc-x1-messages/vc-x1.md`**: records without `read:` are unread, and
  records without an `outcome-*` field are open traffic.
- **An incoming request becomes a Todo or backlog entry, and the reply cites it** (wink,
  2026-08-12), so a commit has an entry to reference and the entry outlives the exchange.

Both bullets are family policy parked here until the 0816-proposal pins them into `agent-data`
and moves the member facts (the repo path, the file name) to config.

## Experimental agent-file rules

Rules this project already follows that belong in the pinned files, parked here only because
their convention cycle has not run yet. Each entry states the binding rule, names its
destination pinned file, and cites its dogfood evidence. An entry leaves this section the
moment it lands in its pinned file or is rejected, so a long-lived entry here is a process
failure. This section is itself an experiment: the pattern (a layer holding adopted-ahead
rules between convention cycles) gets proposed to the family alongside its contents.

_None._

## Dogfood log

This project's log lives in [notes/dogfood.md](notes/dogfood.md): dated entries on where the
pinned instructions chafed, failed, or got amended, the evidence base for the family's
convergence decisions ([Changing the agent-files](AGENTS.md#changing-the-agent-files)). It stays
a separate file because it predates this one and is long. In-flight entries only: a resolved
entry retires at the beat where it resolves, per
[Retiring Done entries](agent-data/notes.md#retiring-done-entries).
