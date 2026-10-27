# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

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

What was done:

- `vc-x1 land -s b|p [--source <BM|REV>] --onto <BOOKMARK>`, in `src/land.rs`: `--onto`
  fast-forwarded to `--source` and pushed, and the cycle's bookmark deleted locally and on origin,
  `--skip local|remote|both` keeping what it names, `--dry-run` making every check and saying each
  step, interleaved with the steps.
- `--source` resolves as `--closing`'s argument does, and left out is the one local bookmark
  besides `--onto`. `--onto` is any local bookmark at its origin position that is an ancestor of
  `--source`, so a bushy tree lands anywhere, and one that is not is refused: `land` makes no
  merge.
- The publish follows where the source is: at the work `@-` through `push`, which finishes the
  pair, and below it the trunk alone and the agent `main` by `squash-push`.
- `--delete-intermediate-bookmarks` deletes the bookmarks strictly between, locally and on origin.
  Without it they are named as merged and kept.
- `-s w` and `-s a` are refused with the reason, in the error and in `--help`.
- `commit --closing ... --land` lands through `land`, one code path.
- The README's `land` section explains it as the way to test a closing's release build before it
  is published, and its `### Write scopes` table gives each command's scopes and the rule behind
  them.

#### Acceptance check

This series lands by `land` itself, made by the binary that ships. A bootstrap, the release build
of the working copy installed off the PATH, makes the closing, pushed to its bookmark and not
landed. The committed closing's release build, the candidate, is then installed off the PATH,
tested, and after review lands the series and installs itself:

```
mkdir -p tmp/bootstrap tmp/candidate
vc-x1 install --dir tmp/bootstrap
tmp/bootstrap/vc-x1 commit -s b --closing feat-vc-x1-land --title "feat: vc-x1 land"
vc-x1 install --dir tmp/candidate
tmp/candidate/vc-x1 land -s b --onto main --dry-run
tmp/candidate/vc-x1 land -s b --source feat-vc-x1-land --onto main --delete-intermediate-bookmarks --dry-run
tmp/candidate/vc-x1 land -s b --source feat-vc-x1-land --onto main --delete-intermediate-bookmarks
tmp/candidate/vc-x1 install
cmp tmp/candidate/vc-x1 ~/.cargo/bin/vc-x1
```

Since a Land only fast-forwards, `main` becomes the closing itself, and a binary embeds only its
commit and whether the tree was clean, so the candidate is byte for byte the binary installed,
which `cmp` shows. The plain `vc-x1` is replaced only by what is on `main`. The first dry run is refused,
the series having three bookmarks besides `main`, and names them. The second lists the
fast-forward and the three bookmarks it would delete,
`feat-vc-x1-commit-takes-a-revision`, `fix-the-fixture-builder-drives-commit`, and
`feat-vc-x1-land`. Afterwards `main` is at the closing in both repos and on origin, the three
bookmarks are gone locally and on origin, and `cargo test` passes with a test per rule.

Result before the closing commit (2026-10-26, fwlaptop), on the closing's tree, `@-` the docs rung:
full validation passed, and both dry runs by `vc-x1 run`, the release build `0.85.11`, did as
stated, the first refused naming the three bookmarks and the second listing the fast-forward, the
push with its pair, and the three deletions. The candidate's dry runs, the Land, the install, and
the `cmp` run after the closing commit, their result reported then.

#### Deliberation

- A command of its own: `land`, not `commit --land` with nothing to commit, wink's call
  (2026-10-26).
  - The commit is already made, so `commit` names the wrong act, and Land is the agent-files' word
    for this sequence.
  - `commit --closing ... --land` stays, as `commit` then `land`.
- `--source` and `--onto`, named rather than positional (wink, 2026-10-26), `--source` later
  defaulted to the one local bookmark besides `--onto`, wink's call at a rung's review.
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
- A Land is always published, `--no-push` built and taken out (wink, 2026-10-26): `commit --land`
  already refuses a local Land, and a fast-forward lands exactly the source, so the source is what
  is tested before `land`.
- A release candidate is tested as the release build under a name other than the plain one, which
  is left to what is on `main` (wink + agent, 2026-10-26): `install --dir`, the release build
  off the PATH, run by its path, and for a period of use the same with a directory that lasts.
  The candidate is the committed source's build, since a binary embeds its commit and whether the
  tree was clean, so the one tested is byte for byte the one installed after the fast-forward.
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
- [feat: land defaults its source][l4] (done)
- [docs: the README and the v0.3.1 proposal name land][l5] (done)
- [feat: vc-x1 land closing][l6] (done)

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
- Inserted: **feat: land defaults its source**, from the same review, ahead
  of the docs rung so the README names the final flags.

##### feat: land defaults its source

`--source` defaults to the one local bookmark besides `--onto`, from wink's review of the rung
before (2026-10-26).

- `--source` left out is the one local bookmark besides `--onto`, every bookmark counted, not only
  the branch's, so a stray one asks for `--source` rather than being guessed past. None, or
  several, is refused with the names. A stack of cycles has several and names its tip.
- `land`'s `long_about` names it, and `--delete-intermediate-bookmarks`, which the rung before
  left out of it.
- `--no-push` was built here and taken out at review, wink's call (2026-10-26), with no Todo.
  The want was to land on `main` and test before origin sees it. A Land only fast-forwards, so
  `main` becomes the source commit itself, and installing and testing the source before `land` is
  that test: the closing made without `--land`, `vc-x1 install`, the testing, then `land`. A
  `main` that moved is refused, and the rebase it needs is a new tree, installed and tested again.
  A Land kept local was a state nothing else knows, which `land` again refused and `push` could
  not publish where it is, and `commit --land` already refuses `--no-push`.
- Filed: **build, install, and run take --dev**, from the question of whether a two-step Land is
  a `land-dev`, the `-dev` commands naming a build rather than a mode.
- Test: **the_source_defaults_to_the_one_other_bookmark**, landing without `--source`, then refused
  with none and with a stack's three.

##### docs: the README and the v0.3.1 proposal name land

The README's `land` section and `commit --land`'s pointer to it, and the v0.3.1 proposal's Land
naming `land`.

- The README explains `land` as the test-before-publish path: the closing made without `--land`,
  `vc-x1 install`, the testing, then `land`, the fast-forward landing the tree tested, and
  `commit --land` the one step when no test pause is wanted (wink, 2026-10-26).
- A `### Write scopes` section after `### Multi-repo queries`: each command's `-s` scopes in a
  table, with the rule that a scope is valid when what it publishes names nothing it leaves
  unpublished, the repo commands taking `a` and the cycle commands refusing or hemming it in
  (wink, 2026-10-26). The README-table bullet of **docs: each command's -s help names the scopes
  it takes** moves here, and **commit and land take the same scopes** gains one updating the
  table's rows.
- The `land` section: its examples, the test-before-publish sequence and why a fast-forward makes
  it exact, `--source` and its default, `--onto`, the publish by where the source is, `--skip` and
  `--delete-intermediate-bookmarks`, and `-s` linking the scope table. A Land is always published,
  `--dry-run` the look before it.
- `commit --land` says it is the closing then `land` in one step, linking the section, and the
  Usage list gains `land`.
- The v0.3.1 proposal's Land gains the test-before-publish sequence beside
  `commit --closing ... --land`.

##### feat: vc-x1 land closing

Closes the cycle: the record finalized and moved, the version the bare `0.85.11`.

- Close-out shape: trapezoid, the default, the closing's first parent `main` and its second the
  series' tip, so `git log --first-parent main` reads one commit for the three cycles.
- The closing is made by `commit --closing` without `--land`, by a bootstrap, the release build
  of the working copy at `tmp/bootstrap`, and pushed to `feat-vc-x1-land`. After its review the
  candidate, the committed closing's release build at `tmp/candidate`, lands it and installs
  itself, wink's call (2026-10-26): every rung was committed by the installed `0.85.9`, which has
  none of the series, so the closing and the Land are this cycle's code.
- The acceptance check and the README's `land` sequence moved from installing the closing before
  the Land, then from `vc-x1 run`, to `install --dir`: the plain `vc-x1` is replaced only by what
  is on `main`, and `cmp` shows the binary tested is the one installed.
- The cycle title is passed to `commit --closing`, which read the oldest opening above `main`,
  **feat: vc-x1 commit takes a revision**, the stack's first. Filed: **commit --closing reads the
  nearest opening's title**.
- Filed: **agent-files(proposal): a cycle is made by its own build**, and in **build, install,
  and run take --dev** the name and profile split, `install --dir` weighed as the soak.
- Nothing in the block goes to a `notes/` file, the README's `land` and `### Write scopes`
  sections holding the design.
- Re-parented after its push, wink's catch (2026-10-26): `commit --closing` merged `main` with the
  tip, though the cycle was opened above it at **fix: the fixture builder drives commit**, which
  hid the first cycle's closing and the fix from `main`'s first parents. `jj rebase -r` set the
  parents to the fix commit and the docs rung, the change id and the tree unchanged. Filed:
  **commit --closing merges onto the cycle's base when it was opened above the trunk**.
- `notes/README.md` names no commands, so it is unchanged, and no agent-file changed, so no size
  row is added.

# References

[l0]: #feat-vc-x1-land-opening
[l1]: #refactor-lands-plan-and-steps-move-out-of-commit
[l2]: #feat-vc-x1-land-lands-a-source-onto-a-bookmark
[l3]: #feat-land-deletes-the-intermediate-bookmarks
[l4]: #feat-land-defaults-its-source
[l5]: #docs-the-readme-and-the-v031-proposal-name-land
[l6]: #feat-vc-x1-land-closing
