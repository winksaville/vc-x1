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

Multi-commit cycles default to the merge-non-ff trapezoid,
but the recipe is split between `cycle-protocol.md` and
AGENTS.md with wrinkles re-derived each time (exercised
manually on `0.62.0`). Codify the full recipe and
standardize the flag spelling. Split out of Idea #1 (the
broader cycle-protocol.md codification).

- Recipe: `jj rebase -r <closeout> --onto <prev-closeout>
  --onto <work-tip>` (first `--onto` = trunk → first
  parent), then `jj new <merge>` to lift `@` above the
  merge, then `jj git push --bookmark main`.
- Standardize on `--onto` / `-o` (canonical), not the
  `-d` alias — update AGENTS.md's `-d` spellings (jj
  Basics + the post-amend `jj new` note) to `--onto`.
- `@` reverts to the work-tip's content until the
  `jj new` (the "post-amend `jj new`" gotcha already in
  AGENTS.md).
- Post-hoc caveat: if the cycle was already pushed
  keep-separate (as `0.62.0` was), the rebase needs
  `--ignore-immutable` and the push is a force-update of
  `main`; the standard recipe assumes the merge is set up
  before the close-out push.

### As-built ladder

- 0.64.0-0 Preparation — version bump, 0.63.1 `Commits:`
  backfill, item pickup. Also clarified the Preparation
  step itself: a Cargo.lock-update bullet, reworded the
  Todo "move into In Progress", and a Prose-form intro
  line at the top of `cycle-protocol.md`.
- 0.64.0-1 cycle-protocol.md — promote the recipe to a
  `### Merge non-ff recipe` subsection (rebase → `jj new`
  lift → push, with the parents broken out as sub-bullets
  and a post-hoc caveat); trim the shape bullet to point
  at it. Also reword `### Shape at close-out push`: the
  intro reframed (work done, shape chosen at push,
  post-publish change is a remote rewrite) and Merge
  non-ff tagged as the current default.
- 0.64.0-2 AGENTS.md — standardize `-d` → `--onto` in the
  post-amend note; add a jj Basics bullet for the
  `--onto`/`-o` spelling; sub-bullet the post-amend
  command list; reframe the note around jj's empty-`@`
  final form, dropping the merge-specific example and the
  HEAD-detach mechanism (the recipe now owns the merge
  sequence).
- 0.64.0-3 review round — make `### Merge non-ff recipe`
  step 2 self-contained (own the empty-`@` why inline) and
  drop the AGENTS.md post-amend `jj new` note entirely. The
  note's non-merge cases proved mis-categorized: `jj squash
  --into` leaves `@` clean on top, and `jj edit` lands `@`
  on the commit deliberately (you `jj new` when done, not
  to restore). One-directional ref (Shape → recipe), no
  circle.

# References

[1]: https://github.com/winksaville/vc-x1/commit/fdfa388817f4 "fdfa388817f4ec794038767df454ed5064c8ad90"
[2]: https://github.com/winksaville/vc-x1/commit/2cb596e45dd3 "2cb596e45dd3f895ff15f486e313cf9fb61f6621"
