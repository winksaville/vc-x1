# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

_None._

## In Progress

### feat: vc-x1 land

Picked from `## Todo` as **commit --land lands a closing already made** (agent, 2026-10-25),
rewritten as a command of its own (wink + agent, 2026-10-26).

#### Problem

`commit --land` lands only the closing it makes, so a cycle whose last commit is already made, a
closing pushed without `--land` or a single-step commit made with `--no-push`, is landed by
commands typed: `move-bookmark`, `push`, a `delete-bookmark` for each bookmark, and `install`. Met
at the Land of **feat: vc-x1 commit takes a revision** and **fix: the fixture builder drives
commit**, two cycles stacked and landing together.

#### Solution

- `vc-x1 land -s b|p --source <BM|REV> --onto <BM>`: `--onto` fast-forwarded to `--source` and
  pushed with the agent half as its partner, and `--source`'s bookmark deleted locally and on
  origin, `--skip local|remote|both` keeping what it names, `--dry-run` saying each step.
- `--source` resolves as `--closing`'s argument does. `--onto` is any local bookmark at its origin
  position that is an ancestor of `--source`, so a bushy tree lands anywhere, and one that is not
  is refused: `land` makes no merge.
- `--delete-intermediate-bookmarks` deletes the bookmarks on the commits strictly between, locally
  and on origin. Without it they are named as merged and kept.
- `commit --closing ... --land` lands through `land`, one code path.

#### Acceptance check

This series lands by `land` itself, from the work-repo with the closing at `@-`:

```
vc-x1-dev land -s b --source feat-vc-x1-land --onto main --delete-intermediate-bookmarks --dry-run
vc-x1-dev land -s b --source feat-vc-x1-land --onto main --delete-intermediate-bookmarks
vc-x1 install
```

The dry run lists the fast-forward and the three bookmarks it would delete,
`feat-vc-x1-commit-takes-a-revision`, `fix-the-fixture-builder-drives-commit`, and
`feat-vc-x1-land`. Afterwards `main` is at the closing in both repos and on origin, the three
bookmarks are gone locally and on origin, and `cargo test` passes with a test per rule.

#### Deliberation

- A command of its own: `land`, not `commit --land` with nothing to commit, wink's call
  (2026-10-26).
  - The commit is already made, so `commit` names the wrong act, and Land is the agent-files' word
    for this sequence.
  - `commit --closing ... --land` stays, as `commit` then `land`.
- `--source` and `--onto`, both required, named rather than positional (wink, 2026-10-26).
  - `--onto` over `--dest` and `--trunk`: `--trunk` assumes one line, and a bushy tree lands
    anywhere.
  - Named, as `merge` and `move-bookmark` name theirs: no write command takes two positionals, and
    their order is what a user would get wrong, landing the wrong way.
  - Long only, since `-s` is the scope throughout.
- Fast-forward only: an `--onto` that is not an ancestor of `--source` is refused, since a merge
  is `commit --closing`'s to make.
- `--delete-intermediate-bookmarks`, wink's idea (2026-10-26), spelled with `delete` to match
  `delete-bookmark`, and opt-in, since a bookmark in the range may be one the user means to keep.
- The install stays `vc-x1 install`, after, the Land's last act.
- Opened on the unlanded series: the bookmark is created at **fix: the fixture builder drives
  commit**, not at `main`, and the Land takes all three cycles, wink's call (2026-10-26).
  - Bends [Opening](AGENTS.md#opening)'s bookmark-on-`main`, for this cycle's opening only.
  - Created locally at the opening's start and published by the opening's push, since publishing
    it alone would publish the fix commit, kept local by wink's "just commit".

#### Ladder

- [feat: vc-x1 land opening][l0] (done)
- [refactor: land's plan and steps move out of commit][l1] (done)
- [feat: vc-x1 land lands a source onto a bookmark][l2] (done)
- [feat: land deletes the intermediate bookmarks][l3] (done)
- [feat: land keeps a land local and defaults its source][l4]
- [docs: the README and the v0.3.1 proposal name land][l5]
- [feat: vc-x1 land closing][l6]

##### feat: vc-x1 land opening

Opens the cycle: the Todo moved here and rewritten, the version bumped.

##### refactor: land's plan and steps move out of commit

`plan_land` and `land_after` move from `src/commit.rs` into `src/land.rs`, taking the tip and the
trunk as arguments rather than `CommitParams`, and `commit --land` calls them. Only the dry run
changes.

- `land::plan` takes the repo, a label, the trunk, the tip, the cycle's bookmark, and `--skip`.
  `commit`'s `plan_land` keeps reading its params: `--land`'s trunk or the configured one, and no
  `--skip` alongside `--opening`.
- `land::run` takes the push's scope and the target commit, `@-` from `commit`, where `land_after`
  read `commit`'s `Publish` and hard-coded `@-`. `Publish::scope` now serves both of `commit`'s
  pushes.
- `LandPlan` carries the label, so `land`'s messages name the command that ran it, the dry run's
  lines `commit --land: [dry-run]` where they said `commit: [dry-run]`.
- The test **a_conflicted_closing_is_not_landed** moves with the code it tests.
- `land::run`'s dry run is interleaved, wink's call over a plan of steps (2026-10-26): it was an
  `if dry` branch ahead of the steps that said them apart from the code doing them and skipped the
  conflict check. Now the check runs in both modes and each step is its line and then the step
  behind `if !dry`, so a dry run refuses a conflicted closing, tested by
  **a_dry_run_refuses_a_conflicted_closing**, and says the push as its own line.
  - Filed: **Dry runs interleave: commit, push, and sync** and **agent-files(proposal): dry runs
    interleave**.
- `jj::rev_exists`'s doc comment loses "spawn error" and "either path", left from when jj was
  also spawned: every revset here resolves in jj-lib.

##### feat: vc-x1 land lands a source onto a bookmark

The `land` subcommand: `-s b|p`, `--source`, `--onto`, `--skip`, `--dry-run`, every check before
anything moves.

- `land -s b|p --source <BM|REV> --onto <BOOKMARK>`, both required, with `--skip` and `--dry-run`.
  `-s w` and `-s a` are refused. A work-only Land is safe only when the target's pair is
  finished, and otherwise publishes trailers naming agent commits that are not on origin, which
  `-s b` finishes first, so nothing is lost. The agent repo alone has no cycle to land. Kept
  strict, wink's call (2026-10-26), until a work-only Land is needed.
  The reason is in the error and in `--help`, and the README's in the docs rung.
- `-h` names only the scopes a command takes, so clap's `[possible values]` agrees with it, and
  `--help` adds why the others are refused. `merge` had the same wording, all four forms listed
  and then two refused, and is fixed with it, wink's call (2026-10-26).
  `--source` resolves to exactly one commit, and `--onto` must be a local bookmark, not already at
  it, then `land::plan`'s checks: at its origin position and an ancestor of the source.
- The tip and the cycle's bookmark resolve by `--closing`'s rule, moved from `commit` into
  `land::resolve_tip` and `land::cycle_bookmark`, the label and the flag passed in, so `commit`'s
  messages are unchanged and both commands name the same bookmark.
- The publish follows where the target is, since `push` publishes the work `@-` alone, setting the
  bookmark there and finishing its pair:
  - The work `@-`, the closing's case and `land --source @-`: the trunk set there and `push`, as
    `commit --land` did, which commits a pending agent partner or restamps the trailer.
  - Below it, whose pair is finished: the trunk set and pushed alone, and the agent `main`
    published by `squash-push`, the journal folded into the agent `@-`.
- `run` names its target in messages, "the closing" from `commit` and `'<source>'` from `land`.
- Filed: **lookup matches a line to its write by content**, in `## Waiting` on the shared-repo
  Todo, from the scope discussion's question of what a trailer is worth across contributors.
- Tests: a cycle at the work head lands, a source below it lands with the commit above left where
  it is, `-s p` lands a plain repo, `-s w` and `-s a` are refused, a dry run moves nothing, and
  every refusal comes before anything moves.

##### feat: land deletes the intermediate bookmarks

`--delete-intermediate-bookmarks`: the bookmarks strictly between `--onto` and `--source` deleted
locally and on origin, the dry run listing them.

- `land::plan` takes `delete_intermediate` and splits the branch's other bookmarks in two: those
  strictly between the trunk and the tip, `(trunk..tip) ~ tip`, and those at the tip beside the
  cycle's. With the flag the first are deleted after the cycle's, each named "between", and
  without it both are named as merged and kept, as before. A bookmark at the tip besides the
  cycle's is kept either way, since the user may have put it there.
- `--skip` applies to them as to the cycle's bookmark, and `--skip both` deletes none.
- `commit --land` passes no flag: a closing's Land deletes the cycle's bookmark alone, as before.
- On this series the dry run deletes `feat-vc-x1-land`, `feat-vc-x1-commit-takes-a-revision`, and
  `fix-the-fixture-builder-drives-commit`, the acceptance check's three.
- Tests: **the_intermediate_bookmarks_are_deleted_when_asked**, its dry run deleting nothing and
  the bookmark at the tip kept, and **the_intermediate_bookmarks_are_kept_unless_asked**, without
  the flag and with `--skip remote`.
- Filed: **docs: each command's -s help names the scopes it takes**, the scope table the docs rung
  was to carry moved into it, its own cycle after the Land, wink's call (2026-10-26).
- Filed: **commit and land take the same scopes**, from the review's question of why `commit`
  takes `-s a` and `land` refuses `-s w`, wink's call (2026-10-26).
- Inserted: **feat: land keeps a land local and defaults its source**, from the same review, ahead
  of the docs rung so the README names the final flags.

##### feat: land keeps a land local and defaults its source

`--push` and `--no-push`, and `--source` defaulting to the one local bookmark besides `--onto`.

##### docs: the README and the v0.3.1 proposal name land

The README's `land` section and `commit --land`'s pointer to it, and the v0.3.1 proposal's Land
naming `land`.

##### feat: vc-x1 land closing

Closing out the cycle.

# References

[l0]: #feat-vc-x1-land-opening
[l1]: #refactor-lands-plan-and-steps-move-out-of-commit
[l2]: #feat-vc-x1-land-lands-a-source-onto-a-bookmark
[l3]: #feat-land-deletes-the-intermediate-bookmarks
[l4]: #feat-land-keeps-a-land-local-and-defaults-its-source
[l5]: #docs-the-readme-and-the-v031-proposal-name-land
[l6]: #feat-vc-x1-land-closing
