# Chores-09.md

General chores notes — design captures (forward-looking) and
post-implementation chore entries. Same shape as chores-01..08.md;
09 starts here because chores-07 (0.42.0 cycle, 600+ lines) and
chores-08 (0.41.1 cycle, 1500+ lines) are both already large; the
init-clone-refactor rebase landing is a natural new-file
boundary.

Subsection headers use the trailing-version format from CLAUDE.md
when they correspond to a release: `## Description (X.Y.Z)`.

## init-clone-refactor rebase landing (0.42.0-4.7)

Rebased main (0.42.0-0..-4.6) onto init-clone-refactor at its
0.41.1 close-out tip (`slvprlpw`). Chids preserved; ochid
trailers in the rebased commits stayed structurally valid (the
.claude pairings they point at had been WC-only during the icr
divergence; -4.7 itself re-establishes the .claude side).

### What needed real work

- **ttzwvpoq (0.42.0-1):** full `Scope` enum lifted wholesale
  from pre-rebase main; `#[allow(dead_code)]` on `is_bot_only`
  and `Single` / `parse_scope` since icr's bundle design keeps
  them dormant.
- **mvusyowm (0.42.0-2):** icr's bundle-flatten `InitArgs` +
  `ScopeOption`/`ScopeKind` already supersedes mvusyowm's
  parser-switch + `Option<Scope>` design; took icr wholesale
  across all 5 init.rs conflict regions. The cycle's narrative
  intent (custom `--scope` parser accepting paths) becomes
  dormant on the icr base; the `Scope::Single` variant survives
  for future use.
- **kyurxpnu (0.42.0-4.5):** CLAUDE.md substep references merged
  (lulqxovr's icr-side decomposition pointer + kyurxpnu's
  external `substep-protocol.md` / `jj-revsets.md` links) into
  a single continuation paragraph under `### Versioning`.
- **Reference renumbering** — cycle's `[72]`/`[73]`/`[74]`
  collided with icr's chores-08 anchors: `[72]→[76]`,
  `[73]→[77]`, `[74]→[78]`, plus self-refs in chores-07's
  -4.5 and -4.6 sections.

### Precautions taken (most unnecessary in retrospect)

- `main-2` local duplicate bookmark — never read.
- `gca-icr-main` common-ancestor marker — never referenced.
- `rslv-commit` cursor bookmark — handy as a navigation
  aid during cascade, not load-bearing (`@-` works).
- `../vc-x1-main` + `../vc-x1-icr` reference clones — useful
  for content lookup at specific chids; replaceable with
  `jj log -r <chid> --patch` against the local repo.
- Filesystem snapshots `vc-x1-20260505-1`, `vc-x1-20260506-1`
  — never restored from.
- `~/vc-x1-rebase-status.md` scratch file — useful for
  resuming across sessions, but ~200 lines was over-detailed;
  a 30-line cursor + decisions log would have sufficed.

### What was load-bearing

- `jj op restore` for local rewinds.
- origin's canonical pre-rebase main remaining intact until
  the explicit force-push at the end (the only fallback that
  mattered).
- jj's chid-preservation across rebase: ochid trailers stay
  valid pointers without manual fixup.
- Per-commit cargo cycle (fmt/clippy/test/install --locked)
  to verify each squash kept the chain buildable.

### Distilled recipe for next time

1. `jj rebase` of the divergent chain onto the new base.
2. Per conflicted commit: `jj new <commit>`, edit conflicts,
   cargo cycle, `jj squash`.
3. After cascade clears: cargo cycle at tip, `jj git push
   --bookmark main`.
4. `.claude` side: `vc-x1 push` to author the paired commit
   once main converges with origin.

Skip the duplicate bookmarks, reference clones, and
filesystem snapshots unless a specific reason emerges. The
op-log + origin are the safety net.

### Edits

- `notes/chores-09.md`: this file (new).
- `notes/todo.md`: ladder marker flips
  (-4.6 (current)→(done), add -4.7 (current)); add `-4.7`
  Done entry referenced as `[79]`; add `[79]:` ref target;
  move pre-0.42.0 Done entries to `notes/done.md`.
- `notes/done.md`: append migrated 0.40/0.41 Done entries.
- `Cargo.toml`: bump 0.42.0-4.6 → 0.42.0-4.7.
