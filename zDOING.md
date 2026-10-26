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
- [feat: vc-x1 land lands a source onto a bookmark][l2]
- [feat: land deletes the intermediate bookmarks][l3]
- [docs: the README and the v0.3.1 proposal name land][l4]
- [feat: vc-x1 land closing][l5]

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

##### feat: land deletes the intermediate bookmarks

`--delete-intermediate-bookmarks`: the bookmarks strictly between `--onto` and `--source` deleted
locally and on origin, the dry run listing them.

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
[l4]: #docs-the-readme-and-the-v031-proposal-name-land
[l5]: #feat-vc-x1-land-closing
