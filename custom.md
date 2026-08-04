# custom.md - vc-x1's project layer

The one agent-editable instruction file (see [AGENTS.md](AGENTS.md#custommd-the-project-layer)).
Loaded after AGENTS.md; on conflict, this file wins.

## Medium and validation

A Rust CLI. Manifest `Cargo.toml`, package `vc-x1-dev`; the version-of-record is its `version`
field, with `Cargo.lock` its derived copy (`cargo update -p vc-x1-dev --offline` after a bump).
The installed dogfood binary is `vc-x1-dev`; see [Dev artifact name](agent-data/versioning.md).

- **Full validation**
  - when: per-commit checklist step 5; skip-able for notes-only commits, mandatory at close-out
  - four separate invocations, each exit status checked before the next:
    - `cargo fmt`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test`
    - `cargo install --path . --locked`
- **Fast validation**
  - when: ladder checklist step 3
  - `cargo test`

Never pipe a validating command into `tail` / `grep` and then `&&` the next one: the pipeline's
status is the last stage's, so the gate is decorative (found at the `0.77.0` close-out). Use
`${PIPESTATUS[0]}` when a pipe is genuinely wanted.

## Project conventions and overrides

- **Invoke `vc-x1-dev`, never `vc-x1`.** The dogfood binary is what this repo builds and what
  its changes must be exercised against; `vc-x1` is whatever happens to be installed.
- **Mailbox parameters**: member name `vc-x1`; the template repository is at
  `../vc-x1-template` (mailbox `messages/vc-x1.md` there, protocol in its `MESSAGES.md`).
  Provisional: an open proposal moves these to `.vc-config.toml`, on the grounds that they are
  environment configuration rather than instruction text and custom.md should be generic at
  birth (mailbox, 2026-08-03).
