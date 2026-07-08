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

Commits: [[3]]

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

## docs: record finalize ochid-loss bug (0.65.1)

Commits: [[4]]

The fc project hit a `finalize` squash that silently dropped a
described journal's message and its 6 `ochid:` trailers
(2026-06-08; recovered 2026-06-10 via op-log surgery). The bug
write-up and two doc-link fixes sat uncommitted in the working
copy; this cycle lands them, ports fc's AGENTS.md additions
back, and queues the fix as Todo #1.

- bugs.md gains the incident as Bugs #1: the failure mode
  (finalize's `--use-destination-message` assumption inverts
  when the journal is described on `@`), the cost (dangling
  code-side ochids), and the fix direction (detect source-only
  `ochid:` trailers; refuse or merge instead of dropping).
- fc additions ported back:
  - AGENTS.md: use-jj-not-git (jj Basics intro) and
    one-command-per-shell-invocation (Working Directory).
  - cycle-protocol.md `## ochid trailers`: path semantics,
    trailer-format example, `### Resolvability`,
    `### .vc-config.toml`.
  - cycle-protocol.md `### vc-x1 push wrapper`: push injects
    the `ochid:` trailers itself — don't hand-write them.
- Stale "active chores file" prose genericized: notes/README.md
  no longer names chores-10 as the active file (the
  refactor-tracking tables remain in chores-10.md, reachable
  via ARCHITECTURE.md); ARCHITECTURE.md's further-reading
  bullets simplified to match.

### As-built ladder

The planned `-0`/`-1`/close-out ladder collapsed to a single
commit once `vc-x1 push` (one app commit per invocation, ochid
trailers injected by the tool) became the commit vehicle:

- 0.65.1 — bug write-up, link fixes, fc port, Todo #1,
  close-out bookkeeping; one `vc-x1 push main`.

## fix: refuse ochid-dropping squash (0.65.2)

Commits: [[5]],[[6]],[[7]]

`finalize_exec` (`src/finalize.rs`) squashed with
`--use-destination-message` unconditionally, so a described
journal on `@` lost its message and `ochid:` trailers — this
broke the code↔session cross-links in the fc project (Bugs #1
in [bugs.md](../bugs.md)). Decision: refuse the squash when the
source message carries `ochid:` trailers the destination's
message lacks — the destination has its own ochids, so any
automatic message merge guesses wrong in some direction.

- The guard compares the two messages' column-0 `ochid:`
  trailers and errors when the source carries any the
  destination lacks, listing them with a by-hand remedy.
- It runs twice: in the parent's preflight (early, visible
  failure) and again in `finalize_exec` right before the
  squash, so a `jj describe` landing inside the `--delay`
  window is still caught.
- Adjacent hardening rode along: failure-marker surfacing
  moved to *after* the command's own output (with a
  historical banner) so a detached child's past failure
  isn't misread as the current run's, and the marker's
  `error=` value is flattened to keep one `key=value` per
  line.

### As-built ladder

Cycle used the Keep-separate shape — each rung pushed as its
own commit on `main` via `vc-x1 push`:

- 0.65.2-0 Preparation: backfill 0.65.1 Commits ref, bump
  version, pick up the entry, open this chores section.
- 0.65.2-1 ochid-trailer guard: `extract_ochids` /
  `ochids_at_risk` / `check_squash_keeps_ochids` + unit
  tests; wired into preflight and `finalize_exec`; README
  manual-test section; post-run marker surfacing;
  `support/gen-exmpl-1-3.sh` regenerator.
- 0.65.2 close-out: this bookkeeping commit.

### Outcome

The guard's three behaviors — synchronous refusal, normal
squash, and the detached-race re-check with a surfaced marker
— are demonstrated in the README's "Testing the ochid-trailer
guard" section, regenerable via `support/gen-exmpl-1-3.sh`.

## feat: reposition @ onto synced bookmark (0.66.0)

Commits: [[8]],[[9]],[[10]]

After `vc-x1 sync --no-check` fast-forwards a bookmark to the
updated remote, `@` was often left parented on the pre-fetch
tip. `sync` now runs a final apply-mode pass that repositions
`@` onto the freshly-synced bookmark, per-repo, replacing the
old `ensure_at_on_main` rebase step.

Rules (code repo: `xxx` = the synced `--bookmark`; `@-` =
parent of `@`):

- `xxx` a proper descendant of `@-`, `@` empty → `jj new xxx`.
- `xxx` a proper descendant of `@-`, `@` non-empty → rebase
  `@` onto `xxx`, gated by `--rebase` (else prompt on a TTY;
  skip + inform when declined or non-interactive).
- `xxx == @-` → already positioned, no-op.
- `xxx` not a descendant of `@-` → leave `@`, inform why.
- `.claude`: `@-` on `main` (ancestor-or-equal) → `jj new
  main`, else error. The prior `@` (session writes) is kept
  as a sibling head.

The pass runs after `run_plan` succeeds and *outside* the
`op_restore` revert region, so a reposition failure (e.g. a
session `@-` off main) is surfaced without rolling back the
successful fetch / fast-forward.

### Design notes

- Repo role is re-derived from each repo's `.vc-config.toml`
  `[workspace] path` (`/` = code, `/.claude` = session), since
  the `Vec<PathBuf>` reaching `sync_repos` has dropped the
  `Code`/`Bot` role.
- Scaffold and pass landed as one commit: an unconsumed
  `--rebase` field or helper trips `clippy -D warnings`
  (unused field / dead_code), so the flag and its consumer
  don't split cleanly.
- The non-empty-`@` rebase prompt is TTY-gated: no terminal
  and no `--rebase` → skip + inform rather than block on
  `read_line`.

### As-built ladder

Keep-separate shape — each rung pushed as its own commit on
`main` via `vc-x1 push`:

- 0.66.0-0 Preparation: backfill 0.65.2 Commits ref, bump
  version, pick up the entry, open this chores section.
- 0.66.0-1 reposition pass: `--rebase` flag; `reposition_at`
  dispatcher with `reposition_code` / `reposition_session`;
  `is_session_repo` / `confirm_rebase` / `at_is_empty`
  helpers; `ensure_at_on_main` removed; unit + integration
  tests.
- 0.66.0 close-out: this bookkeeping commit + README docs and
  examples.

### Outcome

`vc-x1 sync --no-check` leaves `@` freshly seated on the synced
bookmark: a clean code-repo `@` via `jj new`, a dirty one
rebased only with `--rebase` (or an interactive yes), and the
`.claude` journal restarted on `main` each sync. Documented in
the README `### sync` section.

## feat: single-mode sync + revert command (0.67.0)

Commits: [[11]],[[12]],[[13]],[[14]],[[15]],[[16]]

`vc-x1 sync` defaulted to `--check`, whose "verify only"
contract was a fiction (jj's fetch auto-fast-forwards tracked
bookmarks) and whose two-invocation verify-then-apply flow was
racy — the remote can move between the check run and the apply
run. Observed on test-repo-1: default sync reported "all
bookmarks up-to-date" while `@` stayed parented on the
pre-fetch tip. Sync became a single atomic operation with no
modes; failures stop for inspection instead of auto-reverting;
a new `revert` command undoes a sync explicitly.

- Diagnosis rode on TDD: the end-to-end `tests/cli_sync.rs`
  test drives the user's exact flow through the binary
  (`vc-x1 init` → `vc-x1 clone` trA/trB → change +
  `vc-x1 push` on trA → `vc-x1 sync` on trB), landed red
  (`#[ignore]`d) in -1, and went green when -2 flipped the
  default.
- `--check` survives hidden and deprecated solely for push's
  preflight shell-out; `--no-check` is rejected loudly.
  Removal is paired with the preflight rewire — see the
  Todo "sync follow-up: push preflight in-process; drop
  `--check`; revisit push auto-rollback".
- Stop-on-error inverts the old atomicity contract: instead
  of "either every repo advances or none do" (auto
  `jj op restore` on failure, evidence destroyed), a failure
  leaves state where it stopped; each repo's pre-sync op id
  is persisted to `.vc-x1/sync-state.toml` (cleared on
  success) and `vc-x1 revert` consumes it.
- Building the CLI fixture surfaced three latent defects:
  bugs.md 3–4 (init local bare remotes keep HEAD at
  `master`; clone session-remote derivation mismatch +
  relative-TARGET failure) and the `confirm_rebase` TTY
  hang under `cargo test` (fixed in -1 via `cfg!(test)`).

### As-built ladder

- 0.67.0-0 Preparation: backfill 0.66.0 Commits ref, bump
  version, write the In Progress block, open this section.
- 0.67.0-1 tests: two-clone peer-push coverage — in-process
  `sync_clone_ffs_main_after_peer_push` plus end-to-end
  `tests/cli_sync.rs`; default-mode test `#[ignore]`d (red);
  `confirm_rebase` TTY-hang fix.
- 0.67.0-2 sync single-mode: drop `--no-check`; default
  fetches, converges the bookmark, repositions `@`;
  `--check` hidden deprecated alias; default-mode test
  un-ignored (green).
- 0.67.0-3 stop-on-error: auto-revert removed; pre-sync op
  snapshot persisted per repo; failure report names repos +
  op ids and both undo routes.
- 0.67.0-4 `vc-x1 revert`: restore from the persisted
  snapshots, clear consumed state; shared `resolve_repos`
  with sync.
- 0.67.0 close-out: this bookkeeping commit + README rewrite
  (`### sync` single-mode + stop-on-error, new `### revert`).

### Outcome

Plain `vc-x1 sync` — the invocation a user reaches for — now
converges the bookmark *and* seats `@` on it in both repos, and
a failed sync ends with inspectable state plus an explicit,
scoped undo (`vc-x1 revert`). Verified on the real test-repo-1
clones: syncing t1B after a t1A push moved `main` and `@` for
the first time.

## docs: todo cleanup + trapezoid entries (0.67.1)

Commits: [[17]]

A session on the trapezoidal-commit workflow (work a code-repo
bookmark 1:1, land it as the merge non-ff close-out) turned
into a reshape of the push-related todos. Verified that
`vc-x1 push <bookmark>` applies the one bookmark name to both
repos — violating the bot-repo-linear-on-`main` invariant the
moment a feature bookmark is pushed. Single-commit cycle.

- New Todo #1 "push/sync: bookmark is code-repo-only; pin the
  bot repo to main" — the invariant fix; prereq for working
  the code repo on feature bookmarks.
- New Todo #2 "vc-x1 push: pause point between commit and
  publish stages" — the trapezoid close-out stays 1:1: commit
  stages run normally, pause for the merge rebase (chids
  survive rebase, so every ochid stays valid), resume via
  `--from bookmark-both`; retires the manual pre-commit
  workaround.
- "vc-x1 push: record uncovered code commits (N:1 code↔bot)"
  re-scoped to code worked outside vc-x1 (no bot pairings
  exist); the trapezoid close-out is explicitly out of scope.
- "vc-x1 push --squash" demoted to todo-backlog.md —
  after-publication squash is off the routine path now that
  merge non-ff is the routine shape; pre-publication squash
  needs no tooling.
- cycle-protocol.md push-wrapper improvements list synced to
  the reshaped todos.

### As-built ladder

- 0.67.1 single-commit cycle: todo reshape + backlog demotion
  + wrapper-list sync + this section.

## feat: pin bot repo to main (0.68.0)

# References

[1]: https://github.com/winksaville/vc-x1/commit/fdfa388817f4 "fdfa388817f4ec794038767df454ed5064c8ad90"
[2]: https://github.com/winksaville/vc-x1/commit/2cb596e45dd3 "2cb596e45dd3f895ff15f486e313cf9fb61f6621"
[3]: https://github.com/winksaville/vc-x1/commit/9a6839eb825d "9a6839eb825d3b8b9fce7be05d85f6f754514ed3"
[4]: https://github.com/winksaville/vc-x1/commit/28a0211a364a "28a0211a364aea03d19fc14a655275ba98c0498f"
[5]: https://github.com/winksaville/vc-x1/commit/61e6da2bd448 "61e6da2bd44872d805251ced3ecb3785a7b9dfdd"
[6]: https://github.com/winksaville/vc-x1/commit/e444d615142c "e444d615142c40ce2098008c0d18d46c299f35fe"
[7]: https://github.com/winksaville/vc-x1/commit/604d3b75f012 "604d3b75f01215b8ee82bc2cc9c7ebfe37f219cb"
[8]: https://github.com/winksaville/vc-x1/commit/766f3d4554a2 "766f3d4554a200f7bda8ac578479b6d9d917e290"
[9]: https://github.com/winksaville/vc-x1/commit/7d80bcc521c5 "7d80bcc521c5309e0a24a4dd1fe2974cd99dca2a"
[10]: https://github.com/winksaville/vc-x1/commit/1a6d0bd8941b "1a6d0bd8941b7698f49aae1292f04f83d709dcc9"
[11]: https://github.com/winksaville/vc-x1/commit/85ec8d4ce289 "85ec8d4ce289593e52ede1fbf426e08af56271c1"
[12]: https://github.com/winksaville/vc-x1/commit/261d53f43233 "261d53f43233173c266854a3a8d475d9d5dfac0a"
[13]: https://github.com/winksaville/vc-x1/commit/8cc79af9a87c "8cc79af9a87c655892eabe56478f8ac7631882d3"
[14]: https://github.com/winksaville/vc-x1/commit/98fc7df76bc3 "98fc7df76bc37058ebb746953b0efb20f7d4e4dd"
[15]: https://github.com/winksaville/vc-x1/commit/50e06379e4a9 "50e06379e4a9d2cd439cfbf21c585153279db554"
[16]: https://github.com/winksaville/vc-x1/commit/7f2f038ffc2c "7f2f038ffc2c065c9a4d5b468c25e6490fc7db3e"
[17]: https://github.com/winksaville/vc-x1/commit/01357c2bdec7 "01357c2bdec7ef10137a8b51351c77a3f14fc0ed"
