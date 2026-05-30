# Chores-13

Continuation of `chores-12.md` (closed at `0.62.x`). This
file covers the `0.63.0` cycle onward.

Subsection headers that record a release use the
trailing-version format from AGENTS.md: `## Description (X.Y.Z)`.
Reference numbering is file-local — see
[`AGENTS.md`](../../AGENTS.md#reference-numbering); chores-13
starts at `[1]`.

## docs: adopt AGENTS.md (0.63.0)

Commits: [[1]]

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

## docs: tighten after-finalize rule (0.63.1)

Commits: [[2]]

The stop-and-wait rule in `cycle-protocol.md` was titled
`### After finalize: stop and wait` and phrased as "final
words go in the approval prompt *before* executing finalize"
— wording that assumes finalize is a separate,
manually-invoked step and names only finalize as the
trigger. With `vc-x1 push` the push and `vc-x1 finalize` on
`.claude` are bundled as the tail stages, so there is no
"before finalize" gap left to speak in once the push is
invoked. After `0.63.0`'s push the bot emitted a closing
summary *after* `vc-x1 push` returned, violating the rule
(and likely riding into the `.claude` commit via the
`--delay 10` window).

- **Renamed to "After push or finalize"** — both triggers
  are named, in the title and the paragraph: the remote
  crossing is the push itself, and `vc-x1 finalize` on
  `.claude` is the detached step. This also covers a manual
  `jj git push` (push, no finalize) and a manual
  `vc-x1 finalize` (no wrapper).
- **Wrapper case spelled out** — `vc-x1 push` does both as
  tail stages (`push-app`, then `vc-x1 finalize` on
  `.claude`): closing words go *before* invoking the wrapper,
  and the bot does not purposely emit anything after it
  returns. The rule names the `vc-x1 finalize` operation, not
  the wrapper's internal `finalize-claude` stage label.

The bot thinks the original lapse was a compliance slip (the
rule was read and quoted, then broken), not a comprehension
gap; the doc change closes the one genuine ambiguity (the
wrapper bundles push + finalize, leaving no gap to speak in)
rather than expecting prose alone to prevent the slip. The
rule lives only in `cycle-protocol.md`; AGENTS.md already
mandates reading it before commit work, so no duplicate
restatement was added there.

## docs: codify merge-non-ff recipe (0.64.0)

# References

[1]: https://github.com/winksaville/vc-x1/commit/fdfa388817f4 "fdfa388817f4ec794038767df454ed5064c8ad90"
[2]: https://github.com/winksaville/vc-x1/commit/2cb596e45dd3 "2cb596e45dd3f895ff15f486e313cf9fb61f6621"
