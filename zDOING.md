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
- `vc-x1 commit` commits the work-repo's `@` with the given title and body and stamps its `ochid:`
  with the agent-repo `@`'s change id, which the agent-repo commit keeps when push commits it.
- `vc-x1 push` takes the title and body for the agent-repo commit from the work-repo's `@-`, so
  `--title` and `--body` go, compares the stamped `ochid:` with the agent-repo `@`'s change id,
  and on a mismatch restamps the unpublished commit and says so.
- `vc-x1 commit --closing` makes the commit as the trapezoid merge, its first parent the fork point
  of the bookmark and `main`, its second the last rung. The " closing" and " dead-end closing"
  title suffixes are checked against the flag, never its trigger.
- `vc-x1 install` runs `[install] release` after checking that `@-` is `main` at `main@origin`,
  `@` is empty, and the version is bare, and `vc-x1 install --dev` runs `[install] dev` with no
  check, the mid-cycle refresh of the dev binary.
- `vc-x1 run-dev` runs `[install] dev-bin` with the given arguments from any directory of the
  workspace, and warns when the binary's `-V` is not the manifest's version.

#### Acceptance check

In a scratch dual workspace made by the dev binary's `init`: `commit` makes a work-repo commit
whose `ochid:` names the agent-repo's `@`, and `push` publishes that commit unchanged, the
agent-repo commit's trailer naming it back. `commit --closing` on a bookmark of two rungs makes a
merge whose parents are the fork point and the second rung, and a `jj squash` into it keeps both.
`install` refuses, or warns, when `@-` is not `main` at `main@origin`, and `install --dev` leaves
`tmp/dev/bin/vc-x1` at the manifest's version. `run-dev -V` prints the manifest's version. In this
workspace, two full `validate` runs at different versions leave one binary under `tmp/dev`, its
`-V` the second version, and `tmp/dev/.crates2.json` naming it.

#### Deliberation

- Multi-step, eight rungs, a trapezoid the default, the agent's proposal and wink's go at the
  bookmark: opening, the install table, the local commit, push publishing it, the closing as the
  merge, install, run-dev, closing.
  - The install table is first, so every later rung's validation installs under `tmp/dev`.
  - The commit rungs come before install and run-dev, being the larger change, and the closing as
    the merge is its own rung, so the plain commit is reviewed before the merge builds on it.
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
- The description is written once, by `commit`: push reads it from `@-` for the agent-repo
  commit, so the reviewed text cannot be replaced by a mistyped `--body` at the push.
- The trailer is stamped at commit time from the agent-repo `@`'s change id, which a jj commit
  keeps, and push checks it: a mismatch comes from anything that commits or abandons the
  agent-repo's `@` between commit and push, `vc-x1 squash-push -R <agent-dir>` among them, and the
  restamp of an unpublished commit is allowed and announced.
- The closing is made as the merge at commit time: the merge's tree is the closing's tree, since
  the fork point is an ancestor of the last rung, so only the parents differ from a linear commit,
  and the reviewed merge is what lands. The flag, not the title, decides: a title is transcribed
  text, and the suffix check catches a mismatch.
- The record is kept here, in `zDOING.md`, per the trial this file describes, and the ladder's
  labels start at `[0]`, continuing the local experiment the last cycles began.

#### Ladder

- [feat: vc-x1 commit, install, and run-dev opening][0] (done)
- [feat: an install table in the config][1] (done)
- [feat: vc-x1 commit makes the rung's local commit][2]
- [feat: push publishes the local commit][3]
- [feat: vc-x1 commit --closing makes the trapezoid][4]
- [feat: vc-x1 install promotes main][5]
- [feat: vc-x1 run-dev runs the dev binary][6]
- [feat: vc-x1 commit, install, and run-dev closing][7]

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

##### feat: vc-x1 commit makes the rung's local commit

A rung becomes a local commit with the written title, body, and `ochid:` trailer before any push,
so the description review is of the commit itself.

##### feat: push publishes the local commit

Push takes the description from the local commit, checks its trailer against the agent-repo's `@`,
and publishes it as reviewed.

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
[2]: #feat-vc-x1-commit-makes-the-rungs-local-commit
[3]: #feat-push-publishes-the-local-commit
[4]: #feat-vc-x1-commit---closing-makes-the-trapezoid
[5]: #feat-vc-x1-install-promotes-main
[6]: #feat-vc-x1-run-dev-runs-the-dev-binary
[7]: #feat-vc-x1-commit-install-and-run-dev-closing
