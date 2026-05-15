# Chores-11

Continuation of `chores-10.md` (which is closed at `0.50.0` —
the Subcommand trait sweep). This file covers the `0.51.0`
cycle onward. Reference numbering is file-local — see
[`README.md`](../README.md#reference-numbering); chores-11
starts at `[1]`.

## chore: move chores under notes/chores/ (0.51.0)

Single-step. `notes/` was dominated by ten chores files plus
the freshly opened `chores-11.md`; the eleven files now move
under `notes/chores/`, leaving `notes/` for the live notes
(`todo.md`, `done.md`, `README.md`, design docs). Filenames
stay `chores-NN.md` — preserving them keeps section-anchor
slugs unchanged and reduces the cross-file ref update to a
single mechanical substitution.

Done.md hygiene rides along: the eleven `0.44.0`–`0.50.0`
entries that had been sitting in `todo.md > ## Done` migrate
into `done.md` (renumbered `[85]`–`[95]`), since the 0.50.0
close-out marks a natural batching point and keeps `todo.md`
focused on what's actually in flight.

### Substitutions applied

External callers (everything outside `notes/chores/`):

- `/notes/chores-NN.md` → `/notes/chores/chores-NN.md` —
  absolute-path refs in `notes/todo.md`, `notes/done.md`,
  `notes/README.md`, the sibling design notes, `CLAUDE.md`,
  `ARCHITECTURE.md`, the root `README.md`, and the
  GitHub-URL fragments in `src/clone.rs` /
  `src/config.rs` / `src/init.rs` / `src/init/params.rs`
  / `src/push.rs` / `src/scope.rs` / `src/subcommand.rs` /
  `src/sync.rs`.

Within-chores siblings (now under `notes/chores/`):

- `(../ARCHITECTURE.md` → `(../../ARCHITECTURE.md` (the
  root `ARCHITECTURE.md` is two `..` away now).
- `(../src/…` → `(../../src/…`.
- `(README.md` → `(../README.md` (sibling
  `notes/README.md`).
- `(forks-multi-user.md` → `(../forks-multi-user.md` and
  the `[1]: forks-multi-user.md` ref def in
  `chores-08.md` (similar fixups for
  `vc-x1-init.md` / `cargo-locked-issue.md`).

In-chores cross-refs to other chores files (now siblings
in the same `chores/` dir) — `[chores-NN.md](chores-NN.md#…)`
style — are unchanged.

### Why this shape

`/notes/chores/` (under notes/) rather than top-level
`/chores/` keeps the notes umbrella intact —
`todo.md` / `done.md` / `README.md` are conceptually
adjacent to chores and benefit from sharing the
directory. The `chores-` filename prefix is technically
redundant once the files live in `chores/` (could become
`01.md` / `02.md` / …), but the rename was rejected to
keep the diff focused: section-anchor slugs are
filename-independent, so the rename would buy no
ref-resolution simplification and would force renumbering
of every `[N]:` URL slug in `todo.md`, `done.md`, and the
src `.rs` GitHub fragments.

# References
