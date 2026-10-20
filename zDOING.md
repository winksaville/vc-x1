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

### feat: vc-x1 commit, install, and run-dev

(wink + agent, 2026-10-20, opened 2026-10-20) The code **agent-files(proposal): v0.3.1** names: a
rung is committed locally before it is pushed, the closing is made as its trapezoid, the dev build
installs under its own root, and the promotion is a subcommand.

#### Problem

What the user reviews is not what is published. A rung's commit exists only once `vc-x1 push` has
made and published it, so a fix after the review of its description is a remote rewrite and a
squash-push in both repos. The closing is changed twice more after its review, at Land: the
package's plain name is squashed back into it, and the trapezoid recipe reshapes it into the merge.
The rename exists because a mid-cycle `cargo install` would otherwise replace the consumers'
binary, and Land's promotion itself names no command.

#### Solution

Provisional, the Todo's plan as filed:

- An `[install]` table in the config: `release`, the promotion,
  `["cargo install --path . --locked"]` here, `dev`, the dev install,
  `["cargo install --path . --locked --root tmp/dev"]` here, both lists like `[validate]`'s, and
  `dev-bin`, the dev install's binary, `"tmp/dev/bin/vc-x1"` here. `.vc-config.md`'s
  `[validate] full` ends with the dev install's line. The stale `vc-config.md` examples are fixed
  with the keys, fast's `["cargo test --bins"]`, full's `cargo test` without
  `-- --include-ignored`, and `test.fixtures`' `"../vc-x1-fixtures"` with its "skips with a notice"
  prose, and `vc-config-model.md` is regenerated.
- A scope resolver shared by `commit`, `squash`, `push`, `squash-push`, and `sync`: `-s`, required,
  one of `work|w`, `agent|a`, `both|b`, or `por|p`, a plain repo with no pair, and `-R` optional,
  the workspace by its root or either repo for `w|a|b`, the repo itself for `p`, the cwd without
  it. No `-s` is an error naming the four. `-s p` on a repo that is one side of a dual workspace is
  an error naming the side, and `-s w|a|b` on a directory with no workspace config is one.
- `vc-x1 commit -s b` makes the rung's commit pair from the two working copies, each stamped with
  the other's change id, with the given title and body. `-s w` and `-s a` are errors in a dual
  workspace, since a half is an unpaired commit, and `-s p` commits a plain repo with no trailer.
- `vc-x1 squash` folds each scoped working copy into its side of the pair, the review's fix into
  the work commit and the review's transcript into the agent commit, and refuses a side whose
  commit is published. With `--title` and `--body` it rewrites both descriptions and restamps the
  trailers, the one sanctioned re-describe of a trailer-carrying commit.
- `vc-x1 push` only publishes: the work bookmark `@-` sits on, and the agent-repo's `main`. Its
  agent half folds the agent `@` first, the journal's last lines, and its work half refuses a
  non-empty `@`, which is a `squash` or the next rung, the user's call. `--title`, `--body`, and the
  bookmark argument go, the bookmark inferred from `@-` and named only as a check.
- `vc-x1 squash-push` stays, wink's most-used command, `squash-push -s a` the agent half of a push
  on its own, and `-s w|b` the fold-and-publish that asks nothing.
- `vc-x1 commit -s b --closing` makes the work commit as the trapezoid merge, its first parent the
  fork point of the bookmark and `main`, its second the last rung. The " closing" and " dead-end
  closing" title suffixes are checked against the flag, never its trigger.
- `vc-x1 install` runs `[install] release` after checking that `@-` is `main` at `main@origin`,
  `@` is empty, and the version is bare, and `vc-x1 install --dev` runs `[install] dev` with no
  check, the mid-cycle refresh of the dev binary.
- `vc-x1 run-dev` runs `[install] dev-bin` with the given arguments from any directory of the
  workspace, and warns when the binary's `-V` is not the manifest's version.

#### Acceptance check

In a scratch dual workspace made by the dev binary's `init`: `commit -s b` makes a pair whose
trailers name each other, a `squash -s b` after a further edit in each repo folds both and leaves
the trailers valid, `push -s b` publishes the pair unchanged and refuses while the work `@` holds
an edit, and each of the five commands without `-s` exits non-zero naming the four scopes.
`commit -s b --closing` on a bookmark of two rungs makes a merge whose parents are the fork point
and the second rung, and a `squash -s b` into it keeps both. On a plain jj repo, `commit -s p` and
`push -s p` commit and publish with no trailer.
`install` refuses, or warns, when `@-` is not `main` at `main@origin`, and `install --dev` leaves
`tmp/dev/bin/vc-x1` at the manifest's version. `run-dev -V` prints the manifest's version. In this
workspace, two full `validate` runs at different versions leave one binary under `tmp/dev`, its
`-V` the second version, and `tmp/dev/.crates2.json` naming it.

#### Deliberation

- Multi-step, a trapezoid the default, the agent's proposal and wink's go at the bookmark, eight
  rungs then, thirteen since the pair, the read-first scope, push's halves, and squash-push's
  dry run (2026-10-20): opening, the install table, validate-desc with the resolver, the commit
  pair, push publishing only, squash, push's halves, sync and squash-push, squash-push's dry run,
  the closing as the merge, install, run-dev, closing.
  - The install table is first, so every later rung's validation installs under `tmp/dev`.
  - The resolver is ahead of the commands that take it, and the commit rungs come before install
    and run-dev, being the larger change. The closing as the merge is its own rung, so the plain
    pair is reviewed before the merge builds on it.
- No rename, a bend of Opening step 6 and Land step 1, wink's say at the opening's review
  (2026-10-20): the cycle dogfoods its own change, so the opening keeps the name `vc-x1`, moves
  the two validate lines to `--root tmp/dev`, and removes the `build.rs` guard, which would refuse
  the suffixed version under the plain name on every cargo verb. The bend covers this cycle's
  pushes and its Land, which restores no name, and nothing else: the rule's text is v0.3.1's to
  change.
  - What the opening's validate run left behind before the bend, a `vc-x1-dev 0.85.6-0` in
    `~/.cargo/bin`, is wink's to remove.
  - `README.md`'s single-name section describes the guard that is gone, and the install table
    rung rewrites it with the `[install]` keys it describes.
- The code runs ahead of the rules, as its own cycle: the rule text names commands, so the
  commands exist before the rules do. This cycle's own rungs are pushed by the installed
  `vc-x1 0.85.5`, with `--title` and `--body`, and the next cycle runs on what this one lands.
- One dev root, `tmp/dev`, not one per version, wink's suggestion weighed and set aside: the
  version is in the binary, `cargo install --path` always rebuilds and overwrites, and a directory
  per rung is the litter `versioning.md` Dev artifact name warns of.
  - The one stale case is a failed install leaving the previous rung's binary, which `run-dev`'s
    `-V` check names.
- `install --dev` exists from the start, wink's vote (2026-10-20): fast is `cargo test --bins`
  alone, so only a full run refreshes `tmp/dev`, and a refresh on demand needs no full run.
  - The dev install's line is written twice, in `[install] dev` and at the end of `[validate]
    full`, until the installed `vc-x1` has the subcommand and full's last entry can be
    `vc-x1 install --dev`.
- The guard moves from `build.rs` into `vc-x1 install`: a build script sees every cargo verb but
  not an install's root, so once the dev build carries the plain name it cannot tell a dev install
  from the promotion, while `vc-x1 install` sees the bookmark, the remote, and the version.
  - A hand-typed `cargo install --path .` stays unguarded, a slip the sanctioned path never makes.
  - Open: whether a failed check refuses or warns.
- `commit` makes both repos' commits, wink's direction (2026-10-20) that every rung be local like
  the closing, taken one step further: the plan had commit make the work commit and push make the
  agent commit, with the trailer stamped from a predicted change id and checked at push. With the
  pair made together both ids are known, nothing is predicted, and push carries no description at
  all, so the reviewed text cannot be replaced on the way out.
  - The review's transcript is in the agent `@` when push runs, so push's agent half folds it into
    the pair's agent commit first, a squash into an unpublished commit, which keeps its change id.
    A `squash-push -s a` between commit and push publishes the agent half early, and push finds it
    done.
- `squash` exists because a rung is continued, wink's point: a review's fix must fold into the
  pair, and `jj squash` by hand does one repo and knows nothing of the other. It refuses a
  published side, and when the agent half went out early it folds the work side alone and says
  the transcript goes with the next rung, where it would land today.
  - A description fix is a re-describe, which the rule forbids by hand on a trailer-carrying
    commit, and the pair carries trailers from the start. `squash --title --body` is the way, so
    the rule's text becomes never by hand and never on a published commit, v0.3.1's to write.
- The two working copies mean different things, which is where push and squash-push part: the
  agent `@` is a journal, never empty while a session runs, and folding it is never a decision, so
  push's agent half always folds it. The work `@` is a decision, a fix or the next rung, so push
  refuses it and `squash-push -s w|b` is the user saying fold it and go.
- `-s` is required and `-R` optional, wink's rule (2026-10-20), after `--por` was weighed as the
  alternative for a plain repo and retired into the scope `p`: a write command never guesses which
  repos it acts on, `sync`'s "neither flag means the repo I am in" being fine for a look and not
  for a commit, where a silent half is an unpaired commit. `-R` is then only ever where, and `-s`
  which. The short names `a`, `w`, `b`, `p` are wink's, `squash-push -s a` replacing today's
  `squash-push -R .agent-session/`.
  - The same rule for every command that touches a repo, the read commands' defaults, the validate
    family, and clone's and init's `--por`, is the sweep **every repo command takes -s, required**
    in `TODO.md`, kept out of this cycle so the ladder stays on its subject.
  - To check at the push rung: the lookup recognizes push invocations in transcripts by their
    command text, and the fixture's recorded lines are the old form, so the new form must be
    recognized without the old one being lost.
- A read-only command takes the scope first, wink's suggestion at the write rung's review
  (2026-10-20): `validate-desc -s b` shows the resolver working, with nothing published if the
  resolution is wrong, and it is a check that would be run anyway. The write rung, already
  written, was split by path into the resolver half, which the validate-desc rung builds on, and
  the `sync` and `squash-push` half, a local commit on `wip-sync-squash-push` that follows later.
  The resolver lost its `write` name on the way.
- `-s w` is the pair in two steps, wink's thought at the commit rung's review (2026-10-20): the
  refusal of `w` as "unpaired" was wrong, since the agent `@`'s id is known at that moment just as
  for `b`, so the work commit's trailer is stamped either way and the agent half is only deferred
  to push. Today's push already completes it, the bugs.md #4 path, and push's rung gains a case
  rather than a rewrite: fold the journal when the agent `@-` names the work commit, else commit
  it as the partner. `-s a` alone stays refused, the one direction where unpaired is real.
- Push's halves are a rung of their own, wink's reading of the squash rung's push dry run
  (2026-10-20): the halves are independent, since each trailer is fixed before push runs, so
  `push -s b` is `push -s a`, mirroring the `commit -s w` before it, then `push -s w`. Its own
  rung rather than a fold into the squash rung, since it rewrites push's structure and the squash
  rung was reviewed, and the squash rung went out by the old push.
- `squash-push --dry-run` is a rung inserted after the stash, wink's pick (2026-10-20), when
  `squash-push --dry-run -s a` was refused as an unknown flag: the other write commands all had
  one by then.
  - Its work and description reviews are waived, wink's delegation at the insert: the rung runs
    through `commit -s w` without stopping, and the push still takes its own approval. The
    waiver covers this rung and nothing else.
- The pair's commands come before the stash, wink's vote after the validate-desc push
  (2026-10-20), `commit`, then `squash`, then `squash-push`, with `push` moved between the first
  two on the agent's reason: handed a pair, today's push publishes the work half as-is but commits
  the agent journal as a second agent commit naming the same work commit, so the first rung made
  by `commit -s b` cannot go out cleanly until push folds the journal into the pair. With push
  fixed next, every later rung is dogfooded end to end, squash's own included.
- The closing is made as the merge at commit time: the merge's tree is the closing's tree, since
  the fork point is an ancestor of the last rung, so only the parents differ from a linear commit,
  and the reviewed merge is what lands. The flag, not the title, decides: a title is transcribed
  text, and the suffix check catches a mismatch.
- The record is kept here, in `zDOING.md`, per the trial this file describes, and the ladder's
  labels start at `[0]`, continuing the local experiment the last cycles began.

#### Ladder

- [feat: vc-x1 commit, install, and run-dev opening][0] (done)
- [feat: an install table in the config][1] (done)
- [feat: validate-desc takes -s, the first scope][2] (done)
- [feat: vc-x1 commit makes the rung's commit pair][3] (done)
- [feat: vc-x1 push only publishes the pair][4] (done)
- [feat: vc-x1 squash folds work into the pair][5] (done)
- [fix: push runs the agent half, then the work half][6] (done)
- [feat: sync and squash-push take -s, required][7] (done)
- [feat: squash-push takes --dry-run][8] (done)
- [feat: vc-x1 commit --closing makes the trapezoid][9]
- [feat: vc-x1 install promotes main][10]
- [feat: vc-x1 run-dev runs the dev binary][11]
- [feat: vc-x1 commit, install, and run-dev closing][12]

##### feat: vc-x1 commit, install, and run-dev opening

The cycle's setup commit: create and publish the bookmark at `main`, move the Todo entry into this
block, and bump the version-of-record. Under the bend above it keeps the package name, installs
under `--root tmp/dev` from both validate lists, and removes the `build.rs` guard, so the cycle
runs on the convention it introduces. It also carries the Todo's own split, made before the
opening: the code entry cut from **agent-files(proposal): v0.3.1** and placed first.

##### feat: an install table in the config

The config gains `[install] release`, `dev`, and `dev-bin`, the schema's stale examples are fixed
and the model regenerated, `README.md`'s single-name section becomes the dev root's, and two
validate runs show one binary left under `tmp/dev`.

* Nothing reads the new keys yet.
  - The schema names the subcommands that will, `install`, `install --dev`, and `run-dev`, so its
    prose is ahead of the binary until their rungs land. On this rung's tree the keys are valid
    config and nothing more: the dev binary's `validate-config` finds no problem, since its schema
    is built from the same file, while the installed `0.85.5` reports the three keys and their
    links as unknown until Land.
* The two-run check passed on the cycle's own rungs.
  - The opening's full run installed `0.85.6-0` under `tmp/dev`, this rung's installed `0.85.6-1`,
    and after it `tmp/dev/bin` holds one `vc-x1`, its `-V` the second version, with
    `.crates.toml` and `.crates2.json` rewritten to name it alone. Nothing was deleted between.
* The README still described the guard and the old fixture path.
  - Its Build and install section now says the name never changes and where a cycle's builds go,
    and its Acceptance fixtures section says `.fixtures`, that the tests are ignored without
    `--include-ignored`, and that an absent fixture panics naming the build command.

##### feat: validate-desc takes -s, the first scope

The scope leaf and the resolver, and their first user, a read-only command: `-s w|a|b` required,
whose commits are checked, one table per side under its label, `-s p` refused, and `-R` optional,
where.

* The scope leaf knew sides, not a plain repo.
  - `RepoScope` is a `Scope` or `Por`, parsed from the four forms long or short, and `parse_scope`
    learned the short forms too, so the read commands accept `-s a` ahead of the sweep. The comma
    forms stay with `parse_scope`, for the read commands that predate `both`.
* Nothing resolved "which repos" for a required scope.
  - `resolve_scoped_repos` is that: the sides from the workspace the start is in, by its root or
    either repo, and for `p` the start itself, refused when it is a side. A repo under a
    workspace's tree that is neither side is plain, so a scratch repo in `tmp/` is reachable.
* `validate-desc` checked one repo, `-R`'s, against its partner.
  - It checks each scoped side against its other side, `b` being both directions, and refuses
    `p` at params time with the reason. The init test that ran it once per side runs `-s b` once
    per workspace.
* The rule was tried where a wrong resolution costs nothing.
  - The write half, `sync` and `squash-push` on the same resolver, was written first and set
    aside on the local bookmark `wip-sync-squash-push`, the next rung, so this one publishes
    nothing by the new rule before the rule has been seen working on a read.
  - One impurity of the path split: `common.rs` carries the two messages that now say
    `squash-push -s a`, one rung before squash-push takes it.

##### feat: vc-x1 commit makes the rung's commit pair

A rung becomes a pair of local commits, one per repo, each stamped with the other's change id,
before any push, so the description review is of the commit itself and the lookup works on an
unpushed rung.

* The pair's two ids were going to be one known and one predicted.
  - Both are read from the working copies before either commit is made, since a jj commit keeps
    `@`'s change id, so the work commit's trailer names the agent `@` and the agent commit's the
    work `@`, each written once. A test checks the ids survive the commits and that validate-desc
    passes the pair in both directions with nothing pushed.
* The pair can be made in two steps.
  - `-s w` makes the work commit now, its trailer naming the agent `@` all the same, and leaves
    the agent half to push, which commits the journal as the partner, wink's thought at this
    rung's review: it works with the installed push today, and the review's own transcript lands
    in the rung's agent commit. `-s b` makes both now, `-s p` a plain repo's commit, and only
    `-s a` is refused, an agent commit made first being the one unpaired case. `--title` and
    `--body` are required and may not be blank, `--body @PATH` reads a file, and the editor path
    push has is left out until it is asked for.
* The work `@` is a decision, the agent `@` a journal.
  - An empty work `@` is nothing to commit and refuses before anything is touched, while the agent
    `@` is committed as it stands. A failure after the first commit restores both repos from `jj
    op` snapshots, push's rule inside its mutation window.
* The resolver called a single-repo workspace a side.
  - `FixturePor` lays out a work config with no agent, and `-s p` on it was refused as "the work
    side of the dual workspace". A side is one only when the config declares an agent, so a
    single-repo workspace is plain for `p`, with the resolver's test covering it.
* This rung is the first made by its own command.
  - `tmp/dev/bin/vc-x1 commit -s w` made it, the body from `tmp/body.md`, and the installed
    `vc-x1 0.85.5 push` published it, committing the agent journal as the partner, since its work
    half was already there. `-s b` waits for the push rung: handed both commits, today's push
    makes the journal a second agent commit.

##### feat: vc-x1 push only publishes the pair

Push carries no description: its agent half folds the journal into the pair's agent commit when
`@-` already names the work commit, else commits the journal as the partner with the work
commit's description, its work half refuses unsquashed work, and it publishes the pair as
reviewed. The transcript's push lines keep being recognized by the lookup in both the old form
and the new.

* Push wrote descriptions and ran gates of its own.
  - The review gate, the message stage, the editor, `--title`, `--body`, and `--no-squash-push`
    are gone, and the retired flags are errors naming themselves. What remains is `-s`, `-R`, the
    bookmark as a check, `--dry-run`, and `--step` with `--yes` for its gates. A rerun stays safe,
    since every stage still checks its precondition.
* The pair can arrive in three states.
  - A `plan` stage reads them before anything is touched: the agent `@-` names the work commit
    (`commit -s b`) and the journal folds into it at squash-push, the agent `@` is the change the
    work commit names (`commit -s w`) and the journal is committed as the partner with the work
    commit's own text, or the agent repo changed since the commit and the unpublished work commit
    is restamped to name the agent `@`, said out loud, a published one refusing. Each state has a
    test, the restamp's keeping the work change id.
* The refusals had to come before any mutation.
  - Reading the agent `@` snapshots it, so a journal file the size limit refuses stops the plan
    with the file named and no rollback, the m-10-0 case now a CLI test of `commit -s b` for
    either side and of `push -s b` for the journal.
* The halves and the plain repo.
  - `-s w` publishes the work commit alone, `-s a` completes the agent half after it, the by-hand
    remainder of a run that stopped past push-work, and `-s p` publishes a plain repo's `@-` on
    its bookmark with no trailer asked for. The bookmark is the line's unless named, and a name
    that is not among the nearest bookmarks below `@-` is refused naming them.
* The lookup knew push lines by two fixed prefixes.
  - It knows `commit` as a third verb, since the body now rides on it, and the binary by its
    basename after any path, since this cycle's rungs run `tmp/dev/bin/vc-x1`. The fixture's
    recorded lines still read as before.
* The pair's two paths and the two-repo snapshots were written twice.
  - `pair.rs` holds `Pair`, `resolve_pair`, and `Snapshots` with `take` and `restore`, the pieces
    `commit` and `push` both had, wink's question at this rung's review. The description helper
    went to `desc_helpers`, where the trailer's shape lives, and the one-off pieces, the body's
    `@PATH`, the partner check, the preflights, stay where they are until a third copy appears.
* Two tests and a script still spoke the old push.
  - The sync CLI test's peer push is `commit -s w` then `push -s w`, and `with_trailer` in
    `desc_helpers` is the one place a trailer is appended, fixing the double blank line commit
    left. `support/fixtures/build-dr-1.py` still drives `push --title --body` and is the
    **dr-1 is built in Rust** cycle's to retire, so a rebuild of the fixture after Land needs it
    changed first.

##### feat: vc-x1 squash folds work into the pair

A review's fix folds into the pair in both repos at once, refusing a published side, and a
description fix goes the same way with the trailers restamped.

The first rung under the two-paragraph body, wink's (2026-10-20), so this subsection is the full
account its body points at.

* A rung could not be continued once committed.
  - `vc-x1 squash -s b` folds the work `@` into the work commit and the journal into the agent
    commit, each keeping its change id, so both trailers stay right untouched. `-s w` and `-s a`
    fold one side, `-s p` a plain repo's `@` into its `@-`.
* The agent side can be committed, pending, or changed.
  - A partner committed by `commit -s b` takes the journal. A pending one, after `commit -s w`,
    is the journal already, push's to commit with the work commit's text, so squash leaves it,
    and a changed one is push's to restamp. A test covers the first two.
* A description fix was a re-describe the rules forbid by hand.
  - `--title` with `--body` rewrites the descriptions and keeps each trailer, the sanctioned
    re-describe, never of a published commit. A test checks the pair still validates in both
    directions.
* A published side is a remote rewrite.
  - It is refused before anything moves, as is a work `@-` with no trailer. A failure after the
    first mutation restores both repos from the shared snapshots.
* The pair's state was read by push alone.
  - `pair.rs` gained `work_commit`, `partner`, `refuse_conflicts`, and `is_published`, which push
    and squash share, and `desc_helpers::read_body` is the `@PATH` reading commit and squash
    share. The third copy of the conflict check, squash's, is what moved it.
* The lookup did not know squash.
  - `squash` is a verb beside `commit`, `push`, and `squash-push`, since it carries a body.
* Push's plan said the agent half twice, wink's question at this rung's push dry run.
  - The decision line and the summary line are one: what is published, then what the agent half
    does and why, the restamp still a warning. Folded into this rung by its own `squash -s b`,
    the first real fold, after the commit.
* Commit and squash mutated with no preview, wink's point at this rung's review.
  - Both take the shared `--dry-run` leaf: `commit --dry-run` prints the descriptions it would
    write, trailers included, so a description is reviewed before anything is committed, and
    `squash --dry-run` prints what it would fold and re-describe. A test of each checks nothing
    moved.

##### fix: push runs the agent half, then the work half

Push bundled its two halves into one mutation window and logged them out of order, though each
commit's trailer is fixed before push runs, so neither half needs the other. `push -s b` is now
`push -s a` then `push -s w`, each with its own rollback, in the order the pair is completed.

* The halves shared one mutation window and one rollback.
  - The agent half completes the partner by the state found, then squash-pushes `main`. The work
    half restamps if its trailer does not name the partner, then sets and pushes the bookmark.
    Each takes a one-repo `pair::Snapshot` of the repo it touches and restores only that one, since
    the other may have moved on, a journal writing to it.
* The restamp was decided from the agent side, before the agent half ran.
  - It is the work half's question, "does my trailer name the partner?", asked after the agent
    half, so it holds for every order, `-s a` then `-s w` included, which a test now covers. A
    published work commit that would need it is refused before the agent half publishes.
* The plan was a stage of its own with a line of its own.
  - Each half's first line says what it does and why, `push agent:` then `push work:`, in the
    order they run, and the work bookmark is resolved first, so a run that cannot publish the work
    half stops before the agent half publishes anything.
  - In a dry run the intent lines carry `[dry-run]` like the step lines, wink's catch at this
    rung's push dry run, where "push work: publish ..." read as done. Folded in by `squash -s b`.

##### feat: sync and squash-push take -s, required

The two write commands this cycle changes rather than builds take the resolver: `-s` required,
`-R` optional, `squash-push` acting on each resolved repo, work before agent, and the README and
the messages saying `-s a`. Written before the validate-desc rung and held on
`wip-sync-squash-push` until it.

* Sync and squash-push guessed which repos they acted on.
  - Both take `-s`, required, and `-R`, optional, by `resolve_scoped_repos`. A bare `sync` was
    the repo it ran in, and a bare `squash-push` the cwd's repo, so the workspace's other repo was
    silently left out. `squash-push` acts on each resolved repo, work before agent, so `-s b` is
    the fold-and-publish of both that asks nothing, and `-s a` is wink's daily
    `squash-push -R .agent-session/`.
* The README, three messages, and the support script named `-R <bot-repo>`.
  - They say `-s a`, the README's sync section states the rule once for every scoped command,
    and its squash-push examples carry the scope.
* The stash was written against the resolver's first name.
  - Brought onto the pushed tip by a rebase and a squash into `@`, with no conflict, its two uses
    of `resolve_write_repos` renamed, and its bookmark deleted once empty.
* After Land, wink's `squash-push -R .agent-session/` and the agent-files' `sync --scope=both`
  are the old forms.
  - The first fails naming `--scope`, the second still parses, and both are v0.3.1's text to
    change, its "The scope flags in the rules" item.
* `-s p` on a side is refused for sync and squash-push too.
  - On those two it would act as the side's own scope, but on commit, push, and squash it would
    bypass the pair, so the one rule stays, wink's call at this rung's review (2026-10-20): `p`
    is a repo with no partner, and the error names the side's scope.

##### feat: squash-push takes --dry-run

Squash-push was the one write command with no preview: `--dry-run` prints what it found and what
it would do in each resolved repo, and changes nothing.

* `squash-push --dry-run -s a` was refused as an unknown flag.
  - It takes the shared `DryRunFlag` leaf and stops after its precheck, which has already printed
    the verdict line, naming each step it would take: the squash, or that the source is empty and
    it would skip it, the bookmark set, and the push. With `-s b` it says so for each repo, work
    first. A test checks the tail stays in `@` and `main` does not move.
* Push's agent half builds squash-push's params directly.
  - It passes `dry_run: false`, since push's own dry run stops before it.

##### feat: vc-x1 commit --closing makes the trapezoid

The closing is made as the merge of the trunk and the ladder when it is committed, so no reshape
follows its review.

##### feat: vc-x1 install promotes main

The promotion is a subcommand that checks it is installing pushed `main` at a bare version, the
check the `build.rs` guard the opening removed could not make, and `--dev` is the dev install on
demand.

##### feat: vc-x1 run-dev runs the dev binary

The dev binary, off the PATH, is run through the config, with a warning when it is not the
manifest's version.

##### feat: vc-x1 commit, install, and run-dev closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-commit-install-and-run-dev-opening
[1]: #feat-an-install-table-in-the-config
[2]: #feat-validate-desc-takes--s-the-first-scope
[3]: #feat-vc-x1-commit-makes-the-rungs-commit-pair
[4]: #feat-vc-x1-push-only-publishes-the-pair
[5]: #feat-vc-x1-squash-folds-work-into-the-pair
[6]: #fix-push-runs-the-agent-half-then-the-work-half
[7]: #feat-sync-and-squash-push-take--s-required
[8]: #feat-squash-push-takes---dry-run
[9]: #feat-vc-x1-commit---closing-makes-the-trapezoid
[10]: #feat-vc-x1-install-promotes-main
[11]: #feat-vc-x1-run-dev-runs-the-dev-binary
[12]: #feat-vc-x1-commit-install-and-run-dev-closing
