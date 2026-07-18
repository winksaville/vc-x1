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

Commits: [[18]],[[19]],[[20]],[[21]],[[22]],[[23]]

`vc-x1 push <bookmark>` applied the one bookmark name to both
repos (preflight tracking check, `bookmark-both`,
`finalize --push`), and sync's classify/fetch used the passed
bookmark for every repo — but the bot repo is a linear journal
on `main` by design. Pushing a feature bookmark would create
and push that bookmark in the bot repo, leave the bot `main`
behind, and wedge the next sync's `reposition_session`. Prereq
for the trapezoidal-commit workflow (branch the code repo; bot
stays on `main`).

Two adjacent sync warts surfaced (and were fixed) mid-cycle
while dogfooding: the unconditional `jj new main` on the
session repo, and the three-lines-to-say-nothing clean-case
output.

### As-built ladder

- 0.68.0-0 prep: backfill Commits:, bump version, pick up
  todo, open chores section
- 0.68.0-1 sync: session repo pins `main` — tracking
  preflight + classify/act use a per-repo bookmark; tests
- 0.68.0-2 sync: `reposition_session` no-ops when `@-` is
  already the `main` tip — previously it always `jj new
  main`ed (empty `@`: chid/op churn; non-empty `@`: live
  session writes stranded on a sibling head); tests
- 0.68.0-3 sync: quiet output — clean case prints one
  summary line (`UP_TO_DATE_MSG` const shared with main.rs's
  long_about); per-repo "@ already on" no-op lines demoted
  to debug
- 0.68.0-4 push: every stage's session-repo side uses
  `main`, never the passed bookmark
  - preflight verifies the session repo tracks `main`
  - bookmark stage sets app → `<bookmark>`, session →
    `main`; renamed `bookmark-both` → `bookmark-set` (no
    legacy alias — nothing external invokes the old name)
  - finalize-claude pushes `main`
  - completion sanity checks the session repo's `main`
  - `PushState.bookmark` holds the code-repo bookmark only
  - test: a feature-bookmark push advances the session
    repo's `main` and creates no `feature` bookmark there
- 0.68.0 close-out: this section; README sync/push updates;
  wrapper planned-list sync

## docs: diagnose silent session-push loss (0.68.1)

Commits: [[24]]

Bugs #1 reported the session repo "sometimes not pushed"
(tprobe: GitHub bot repo 8 commits behind while both local
repos and the GitHub code repo agreed). Diagnosed 2026-07-14;
this cycle records the diagnosis and queues the fix design as
Todo #1. Single-commit cycle.

- Root cause: push's `finalize-claude` stage delegates the
  session push to a detached (`setsid`) child that sleeps 10s
  first; a sandboxed Bash command kills its pid namespace at
  command exit, so a bot-run push loses the child every time —
  before its first `jj` command.
- Evidence: tprobe app repo has 8 `push bookmark main` ops;
  its `.claude` has zero squash/push ops since clone;
  reproduced with a bare `setsid` child in this project's
  sandbox.
- Silent by construction:
  - SIGKILL leaves no failure marker
  - `~/.cache/vc-x1` is outside the sandbox write allowlist
    and `write_failure_marker` swallows errors by design
  - `verify_completion_sanity` checks local state only
- Fix design (Todo #1): push does everything inline (squash
  micro-tail + `jj git push` in-process); preflight errors
  when `.claude main` is ahead of `main@origin`; `finalize`
  becomes the user's zero-arg empty-@ tidy-up, run after the
  bot goes quiet. The empty-@ goal is self-referential for
  the bot — finalizing is itself session data — so only the
  user can capture the full tail.

### As-built ladder

- 0.68.1 single-commit cycle: bugs.md diagnosis rewrite,
  Todo #1 fix design, Done entry, 0.68.0 `Commits:`
  backfill, Done 0.51.0–0.65.2 batch retired to done.md
  (refs re-slotted to [96]..[112] — todo.md's numbers
  collided with done.md's namespace), version bump + this
  section.

## feat: inline session push + squash-push (0.69.0)

Commits: [[25]],[[26]],[[27]],[[28]],[[29]],[[30]]

Bugs #1 (silent session-push loss): push's session publish ran in
a detached, delayed child that a sandboxed bot run killed at
command exit — every bot-run push, silently, while every local
check passed (tprobe's bot repo sat 8 commits behind). This cycle
makes the session publish in-process and visible, renames the
mechanism, and adds a backstop that catches any future loss.

- Inline session push: push's last stage (`squash-push-bot`)
  squashes the session tail and pushes `main` in-process — a
  failure is a visible push failure. No detach, no delay, no
  failure markers.
- `finalize` → `squash-push`: mechanism-named, repo-generic,
  zero-ceremony (bare invocation squashes `@ → @-` and pushes
  `main` in `.`). `--detach`/`--delay`/`--exec`/`--push` and the
  failure-marker machinery retired; no deprecated alias (the flag
  surface changed incompatibly).
- At-rest publish invariant + backstop: between commands the bot
  repo's `main == main@origin` — the bookmark only moves inside a
  push / squash-push run, which publishes it in the same
  invocation. New read-only `vc-x1 validate-bot` checks it; push
  preflight errors on a mismatch (decided: no automatic fixing);
  squash-push reports and proceeds (publishing is its job),
  suppressed when run as push's stage where the mismatch is the
  normal mid-push state.
- Content-agnostic principle (decided 2026-07-15): vc-x1 assumes
  nothing about a repo's contents beyond `.jj` and
  `.vc-config.toml`. Push preflight's hardcoded cargo
  fmt/clippy/test removed — preflight is version-control checks
  only; projects run their own checks before pushing.
- Work/bot vocabulary: push stages renamed (`commit-work`,
  `commit-bot`, `push-work`, `squash-push-bot`), prose sweep
  across docs and code, README rewritten for the new world (new
  Terminology section; testing walkthroughs re-validated against
  live fixture runs); cycle-protocol's stop-and-wait rewritten
  succinct and directive-scoped, plus the user-side Flush recipe.
- Crate temporarily `vc-x1-dev` during the cycle (dual-install
  window for another bot instance), renamed back to `vc-x1` at
  -3 when the window closed.
- Dogfood: every cycle push ran the WIP binary. Two transient
  `.git/index.lock` races at `bookmark-set` (the -3 and -4
  pushes) were recovered by rollback + `--restart` and recorded
  as a new bug.

### As-built ladder

- 0.69.0-0 prep: version bump, Todo #1 pickup, chores section
- 0.69.0-1 inline session push (`squash-push-bot` in-process)
- 0.69.0-2 `finalize` → `squash-push` rename; detach retired;
  protocol docs rewritten for the inline world
- 0.69.0-3 published backstop: `validate-bot`, erroring
  preflight, squash-push report; cargo preflight removed;
  crate renamed back to `vc-x1`
- 0.69.0-4 work/bot terminology + stage-name sweep; README
  rewrite; mismatch report suppressed mid-push; "finalize"
  scrubbed from `*.rs` and cycle-protocol.md
- 0.69.0 close-out: Bugs #1 pruned as fixed, index.lock race
  recorded, notes retired

## docs: shared protocol sync + jj refactor plan

Commits: [[31]],[[32]]

Two docs threads landed together (first title under the new
no-version-suffix convention — see versioning.md, adopted this
commit).

- Shared protocol sync: adopt the vc-template-x1 shared set —
  AGENTS.md, cycle-protocol.md, versioning.md (new),
  jj-tips.md (new, supersedes jj-revsets.md) — with vc-x1's
  0.69.0 corrections applied and ratified template-side
  (inline squash-push flow restored, current stage names,
  preflight described as state-checks-only, roadmap material
  de-projected from the wrapper section). The per-file
  what/why is
  [notes-sync-20260716.md](../notes-sync-20260716.md); the
  goal is a byte-identical shared set across iiac-perf,
  vc-x1, vc-template-x1.
- jj refactor program: the `bookmark-set` index-lock race
  investigation (bugs.md entry 3) grew into a staged program
  — typed jj facade → jj-lib in-process, ending subprocess
  spawning — absorbing eight existing Todos; plan and design
  in [refactor-20260716.md](../refactor-20260716.md) (the
  first dated plan file, one `##` section per stage for
  stable anchors). todo.md restructured around it; the
  TODO.md-move entry added as Todo #1.
- [[32]] (`docs: converge shared protocol doc set`) is the
  follow-up review round: the four fixes agreed across the
  three sessions, converging the shared set byte-identical
  (agreed sha256s recorded in its commit body).

## docs: move todo.md to root TODO.md

Commits: [[33]],[[34]],[[35]],[[36]]

The todo list is the project's live state and the routine
acquaint read; root-level uppercase puts it in the
conventional root-file family (README, LICENSE, AGENTS.md,
ARCHITECTURE.md) — the same "easy for everyone to find"
argument that put AGENTS.md at the root. Siblings
(`todo-backlog.md`, `bugs.md`, `done.md`) stay in `notes/` —
TODO.md is the entry point, the README→docs/ shape.

- Decision (2026-07-16): shared AGENTS.md keeps hard paths
  (greppable) rather than naming the file abstractly.
- The AGENTS.md "File reads" section (and cycle-protocol.md's
  two mentions) are part of the shared byte-identical set, so
  this is a three-project change (vc-x1, vc-template-x1,
  iiac-perf) applied identically; vc-x1 went first, breaking
  byte-identity until the other two apply the same change.
- Historical files (chores-NN.md, done.md, dated
  manifests/audits) keep their `notes/todo.md` mentions.
- The no-arg `validate-todo` / `fix-todo` default
  (`TODO_FILE`) had to follow the move — a behavior change
  the original Todo entry's "code behavior is unaffected"
  parenthetical missed (it held only for doc strings).
- Surfaced in review: the validate/fix wrapper layer has no
  tests (the analyze cores do) — noted as a bullet on the
  validate-numbering Todo.

### As-built ladder

- [[33]] 0.69.2-0 docs: open TODO.md move cycle
- [[34]] 0.69.2-1 docs: move notes/todo.md to TODO.md
- [[35]] 0.69.2-2 refactor: TODO.md as validate/fix-todo default
- [[36]] 0.69.2 docs: move todo.md to root TODO.md (close-out)

## feat: bot-session transcript viewer

Commits: [[37]],[[38]],[[39]],[[40]],[[41]]

Display one Claude Code session transcript
(`.claude/<uuid>.jsonl`) as a readable conversation — first step
toward seeing all bot sessions and linking prompts to the
changes they produced. v1 is file-path in, conversation view
out; index view, session discovery, and cross-file references
(sidechain `agent-*.jsonl`, compaction chains) come in later
cycles.

- Own tolerant reader over any external crate: the transcript
  format is undocumented and churns continuously, so every
  full-schema crate permanently trails (surveyed: cct parses
  today's files but silently drops newer fields, stale since
  2026-05; weavr models the right tolerant posture). The
  long-term goal — prompt ↔ commit linking via ochids — is
  vc-x1-specific anyway.
- Two-layer parse: serde_json as JSON-text → `Value` only (no
  derive anywhere); hand-written extraction into our own
  structs. Unknown fields ride along in the retained raw
  `Value`; unknown entry/block types land in `Other` variants;
  a live session's truncated last line is a warning, never a
  failure. `FileTranscript` is one file's parse — a *session*
  can span files; assembly is a later layer. Whole-file
  in-memory by design (largest observed ~8 MB); streaming via
  `BufRead` is a drop-in escape hatch.
- Renamed `show-session` → `bot-session` mid-cycle: unique
  `b<tab>` completion prefix, display-family naming (list /
  desc / chid / show name what they display), and "bot"
  matches the project's bot-repo terminology. The pushed
  -0/-1 titles keep the old stem.
- Output is eight composable items — headers, user, assistant,
  tool, thinking, results, meta, summary — each `--<item>` /
  `--no-<item>` (last-wins), `--all` / `--none` bulk bases
  (aliases `--no-none` / `--no-all`). Defaults resolve
  git-style, most specific wins: CLI > workspace
  `.vc-config.toml` > user `~/.config/vc-x1/config.toml` >
  built-in (`headers,user,assistant,tool,summary`), both
  configs via `[bot-session].items` (comma-separated string —
  toml_simple has no arrays).
- `--lines SPEC` slices the rendered output: `N` first / `-N`
  last / `I,C` from 0-based Index `I` / `I,-C` ending at `I`;
  `0` = summary only; elision markers at cut points; a sliced
  summary leads with "K of M lines shown" so it never claims
  more than was displayed.
- Plain-text output — the `===` header delimiters carry the
  structure, no ANSI. Headers carry the full UTC date-time;
  the `Z` is claimed only when the source timestamp ends in
  `Z` (observed always across all ~56k lines, but
  undocumented — we think Claude Code writes
  `Date.toISOString()`); any other shape passes through
  verbatim.
- v1 uses no version-control code; when later cycles link
  prompts to commits (chids, ochid trailers), they go through
  jj-lib in-process per the typed jj facade Todo #1 — no new
  `run("jj", …)` sites.
- Deferred / recorded along the way: `--raw` mode (Todo #12,
  source-line units); the As-built-ladder `[[N]]` ref
  convention (Todo #11, shared-doc sync); the repo-wide
  `info!`/`println!` EPIPE panic (Bugs #4), surfaced by the
  first long-output subcommand.

### As-built ladder

- [[37]] 0.70.0-0 chore: open show-session cycle
- [[38]] 0.70.0-1 feat: transcript parse + typed layer for show-session
- [[39]] 0.70.0-2 feat: bot-session command + conversation renderer
- [[40]] 0.70.0-3 feat: bot-session item flags + config defaults
- [[41]] 0.70.0 feat: bot-session transcript viewer (close-out)

## feat: bot-session --result-lines knob

Commits: [[42]]

The [result]-body cap becomes user-controllable:
`--result-lines N` (default 10, `0` = unlimited) in the
Output-range help group, threaded params → render →
push_result. Before this the cap was hardwired — even `--all`
could not show a full tool result. First of two lightweight
single-commit cycles from the --raw Todo; `--raw` itself is
0.70.2.

## feat: bot-session --fields + --raw explorer

Commits: [[43]]

`--raw` reframed at review from a cat-like verbatim dump
(barely beats jq) to *schema exploration*: the parser keeps
every field Anthropic writes while the typed layer consumes a
known subset — the difference is the unexplored surface, so
format changes surface themselves instead of being discovered
by accident.

- `--fields`: aggregated inventory — every dotted leaf path
  per entry type ( `[]` marks array elements) with count,
  value kinds, and up to 3 short samples.
- `--unknown`: inventory minus `KNOWN_PATHS` (the extractor's
  consumed set, subtree semantics — e.g.
  `message.content[].input` covers everything beneath it).
  First real run: 132 unknown paths in one 485-line file
  (message.model, stop_reason, thinking signatures,
  content[].caller.*, requestId, userType, …).
- `--raw [--lines]`: pretty-printed source lines for content
  drill-down; unparseable lines pass through verbatim; no
  summary, no markers. Conflicts with `--fields`.
- `--lines` unified to *source JSONL lines* in **every** view
  (matches jq/editors and the malformed-line warnings), the
  conversation view included — it renders the entries whose
  source lines fall in the slice, with source-line elision
  markers and a "--lines selected K of L source lines"
  summary clause. Surfaced across two review rounds:
  --fields first ignored --lines entirely, then the
  conversation view was still slicing *rendered* lines —
  which selected different regions depending on the item
  flags. One unit everywhere is stable and predictable;
  rendered-line slicing (`| head`-style) is what pipes are
  for. `apply_lines` and the old rendered-slice summary died
  in the unification; `parse_file` lost its last caller and
  was removed (each view reads the text itself).
- `--per-line` (review round): one fields section per source
  line (`=== Index N: <type> [time] ===` + path/kind/value
  rows), malformed lines in place; composes with `--unknown`
  for a line-by-line walk of just the unmodeled surface.
- Deferred: drift-over-time baseline (first-seen dates across
  the repo's dated transcripts) — discovery/index cycle
  territory.

## feat: bot-session --col-width knob

Commits: [[44]]

The field views' (`--fields`/`--unknown`/`--per-line`)
first-column pad — the dotted-path column, previously a
hardwired `{:<44}` at two sites — becomes `--col-width N`
(Output default 68) and its default widens 44 → 68.

- The pad is a *minimum* width (`{:<width$}`), so a longer
  path overflows and pushes the type column out of alignment;
  the knob is about where the type/value columns settle for
  the common case, not truncation.
- Single fixed width shared by both the aggregated
  (`--fields`/`--unknown`) and the `--per-line` view — they
  read from the same knob so the two never diverge.
- `--col-width 12` (test) confirms the override; the default
  68 is asserted separately.

### Choosing 68

We picked 68 from measured key-path lengths, not a guess. A
real 55,642-line `--per-line` dump of one session gave, over
its ~52k rendered rows:

- median path 19 chars; 85th percentile 48 (the old pad);
  95th 61.
- 68 aligns the type column for ~99% of rows — every
  structural key including the relative-path
  `snapshot.trackedFileBackups.<rel>.backupFileName` (68) and
  the `message.usage.iterations[]…ephemeral_*_input_tokens`
  family (67), the 1,524-row knee of the distribution.
- The only keys past 68 are ~216 outliers of one shape —
  `snapshot.trackedFileBackups.<absolute path>.*`, where the
  embedded absolute path (`/home/.../plans/*.md`) reaches
  91–98 chars. An absolute path is unbounded, so no fixed
  width should chase it; those overflow by design.
- 73 would also catch the `toolUseResult.usage.iterations[]…`
  variants but costs 5 columns of whitespace on every
  short-key row — not worth ~216 more aligned rows.

### Config-hierarchy scope (deferred)

At review the knob's resolution was raised: `[bot-session].items`
resolves CLI > workspace `.vc-config.toml` > user config >
built-in, but `--col-width` (and the existing `--result-lines`)
are plain clap defaults with no config layer. Bringing both
scalar args under that hierarchy is a config-plumbing change
(args → `Option`, new config keys, resolution in the op) folded
into Todo #12 (config discoverability) rather than done here, so
this cycle stays a single focused knob.

## feat: config discoverability + scalar hierarchy

Commits:

vc-x1 had two config homes — the typed user config
(`~/.config/vc-x1/config.toml`) and the untyped workspace
`.vc-config.toml` (read key-by-key) — with no way to discover
settable keys or catch typos (unknown keys are silently
ignored), and the bot-session scalar knobs (`--result-lines`,
`--col-width`) had no config layer while `items` did. This cycle
adds a code-declared **schema registry** as the single source of
truth that a `config` command, init's commented defaults, and a
`--validate` check all derive from, so they cannot drift.

### As-built ladder
- 0.71.0-0 chore: open config cycle — version-of-record bump, In
  Progress ladder, chores section opened, --col-width Commits
  backfill
- 0.71.0-1 feat: bot-session scalar config keys — --result-lines
  / --col-width now resolve CLI > workspace > user > built-in
  like [bot-session].items; args → Option, workspace_items() →
  workspace_bot_session(), resolution moved into the op
- 0.71.0-2 feat: config schema registry — src/config_schema.rs,
  the keystone: one ConfigKey table (schema()) over all 13
  settable keys, the single source of truth the print/init/
  validate surfaces derive from; drift tests pin the numeric/
  string defaults to their source consts (COL_WIDTH,
  RESULT_LINE_CAP, DEFAULT_STATE_*)
- 0.71.0-3 feat: config print command — vc-x1 config [--home]
  prints the annotated schema (sshd_config style, grouped by
  home then TOML section); the first consumer of schema()

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
[18]: https://github.com/winksaville/vc-x1/commit/5e0f61e1e8ce "5e0f61e1e8ce2a075fc0bd51494ddd716477a30e"
[19]: https://github.com/winksaville/vc-x1/commit/c1bff242430c "c1bff242430c0f602fbf360b03f609f17e443c06"
[20]: https://github.com/winksaville/vc-x1/commit/93cdfe355632 "93cdfe355632ee5ee27610cddd9f0e38903db863"
[21]: https://github.com/winksaville/vc-x1/commit/208a0a06ac81 "208a0a06ac81ba26fec97f323dcbf6d8a6602505"
[22]: https://github.com/winksaville/vc-x1/commit/d56ea6b1455a "d56ea6b1455a344fe35b68c4de9a8596dfc5e692"
[23]: https://github.com/winksaville/vc-x1/commit/0a83adea1491 "0a83adea1491eec57b66f10efaafdaa105d7a42f"
[24]: https://github.com/winksaville/vc-x1/commit/bf59d5c1860e "bf59d5c1860e265ad78cc5f705f3672c36fc3b75"
[25]: https://github.com/winksaville/vc-x1/commit/0bd73998ca22 "0bd73998ca224879406ea41fe79e9305652b8f8a"
[26]: https://github.com/winksaville/vc-x1/commit/ebd7465c724f "ebd7465c724f32c2034f1a66657d079ddf5cfc23"
[27]: https://github.com/winksaville/vc-x1/commit/6bb848b7c7bd "6bb848b7c7bd036d57fee9386cefd3e1d44aaa60"
[28]: https://github.com/winksaville/vc-x1/commit/c1844659350b "c1844659350b00b2d04f6259493ad3a686b3d163"
[29]: https://github.com/winksaville/vc-x1/commit/d2fa36840c89 "d2fa36840c8915ade0dd4eeab6a59701acc1710e"
[30]: https://github.com/winksaville/vc-x1/commit/c169225e1f2b "c169225e1f2bcacc34fc02966695a05090f13297"
[31]: https://github.com/winksaville/vc-x1/commit/be71ef70e5b7 "be71ef70e5b7d26bda8477ce841a2e446772b21c"
[32]: https://github.com/winksaville/vc-x1/commit/66bc1f2cfda8 "66bc1f2cfda8732226a3e7afc42ab9b7e6c83f45"
[33]: https://github.com/winksaville/vc-x1/commit/638244e41ca4 "638244e41ca40aeeafd98dd365046ee0c90173c2"
[34]: https://github.com/winksaville/vc-x1/commit/0268d454d5b7 "0268d454d5b772268bfe90eda2aa7e93629bc783"
[35]: https://github.com/winksaville/vc-x1/commit/9cb62219a8ea "9cb62219a8ea4342c87f0a961dfbb4d5e11c6d9c"
[36]: https://github.com/winksaville/vc-x1/commit/48886d3e38b6 "48886d3e38b61bc01c2d0613a27e9e0b8740fd2e"
[37]: https://github.com/winksaville/vc-x1/commit/a6266e6ed0a0 "a6266e6ed0a0fea051e71c75958034d66d0fc603"
[38]: https://github.com/winksaville/vc-x1/commit/300eb35136cc "300eb35136cc6035e03713d9cca5ee0c05aed635"
[39]: https://github.com/winksaville/vc-x1/commit/1ccba615836d "1ccba615836d67ec5dec5bd7dc1958d5cb842106"
[40]: https://github.com/winksaville/vc-x1/commit/13080d695ae3 "13080d695ae3dce926f006bfc0665e759538a3f1"
[41]: https://github.com/winksaville/vc-x1/commit/2d591fc32e98 "2d591fc32e987f20c024032b878aea903cd339f1"
[42]: https://github.com/winksaville/vc-x1/commit/81638962f044 "81638962f044f69978dc62574140e1c7d6444fcd"
[43]: https://github.com/winksaville/vc-x1/commit/8363696c8b21 "8363696c8b2185d93dd3603919caed09baff60fe"
[44]: https://github.com/winksaville/vc-x1/commit/4edb63643923 "4edb63643923408d3576c225b5bdc7be83c579cd"
