# Chores-13

Continuation of `chores-12.md` (closed at `0.62.x`). This
file covers the `0.63.0` cycle onward.

Subsection headers that record a release use the
trailing-version format from AGENTS.md: `## Description (X.Y.Z)`.
Reference numbering is file-local — see
[`AGENTS.md`](../../AGENTS.md#reference-numbering); chores-13
starts at `[1]`.

## docs: adopt AGENTS.md (0.63.0)

The bot-instructions file is renamed `CLAUDE.md` → `AGENTS.md`
so Zed and the broader agent-tooling ecosystem (which now
default to `AGENTS.md`) read it natively. Claude Code still
auto-loads only `CLAUDE.md`, so a one-line `CLAUDE.md` import
shim (`@AGENTS.md`) keeps it working — both tools read one
canonical file.

- **Single canonical file** — `AGENTS.md` holds the content;
  `CLAUDE.md` is a one-line `@AGENTS.md` import for Claude
  Code's loader. Nothing human-facing points at the shim.
- **Live references repointed** — every `CLAUDE.md` mention
  and `#anchor` link in live docs (`README.md`,
  `ARCHITECTURE.md`, `notes/`, the `src/push.rs` pre-commit
  doc comment, and `AGENTS.md` itself) now names `AGENTS.md`,
  so links resolve in editors and on GitHub.
- **History left as written** — `CLAUDE.md` prose in
  `chores-01..12` and `done.md` records what was true at the
  time and is untouched; only the live navigational anchor
  links in the `chores-10/11/12` headers
  (`#reference-numbering`) were repointed so they don't
  dangle against the one-line `CLAUDE.md` shim.

### Why an import shim, not a symlink

A symlink `CLAUDE.md → AGENTS.md` also satisfies Claude
Code's loader, but the `@AGENTS.md` import was chosen
because it is explicit and visible in a diff and does not
depend on symlink semantics (git symlink handling,
cross-platform checkout). Separately, GitHub renders a
symlinked `.md` as a stub page rather than the target's
markdown, so a `CLAUDE.md#anchor` link through a symlink
would not resolve — moot here since live links now point
straight at `AGENTS.md`, but it reinforces keeping
`CLAUDE.md` a real file.
