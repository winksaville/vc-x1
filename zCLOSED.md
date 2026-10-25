# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### feat: vc-x1 commit takes a revision

Picked from `## Todo` as **commit takes a revision, the commit its description is for** (wink +
agent, 2026-10-25), retitled for the ladder.

#### Problem

`commit` describes only `@`, so a commit already made by hand, a draft below `@` with no
description, has no path to a described, trailer-stamped rung without `jj describe` typed. And
`--closing`'s argument is the merge's first parent while Land derives the cycle's bookmark from the
branch's head, so a second bookmark on the branch, which a user may make freely, has Land delete
the wrong one or refuse. Both met in the fixture `tmp/t-85-8.1-2ecac`, where line 2 of
`single-step.txt` sits in an undescribed `@-` under a second bookmark with an empty `@` above it.

#### Solution

What was done, the rung subsections holding each part's design:

- `commit` takes a positional REV, the commit its description and trailer are for, `@` by default,
  a draft below it described in place with its partner made, and one below `@-` kept local.
- `--bookmark` takes any bookmark on the work commit's line, so the cycle's bookmark follows a rung
  past a user's own sitting nearer.
- `--closing <BM|REV>` is required and names the tip, onto the trunk from `--land` or the new key
  `[cycle] trunk`, `main` when absent. One commit above the trunk is finalized in place and more
  get the closing merge, so the bare form went. A tip that is not `@-` is closed by a detour, and
  an empty `@` is never refused at a closing.
- The closing's bookmark is decided once by the argument's type, a bookmark named or a revision's
  current or nearest parent bookmark, several refused with the list. The closing's advance and the
  Land's deletion both use it, the branch's other bookmarks are named as merged, and a closing with
  none is kept local unless it lands.
- The v0.3.1 proposal names the new surface and holds the material the last cycle left for it.

#### Acceptance check

In the fixture `tmp/t-85-8.1-2ecac`, with the dev build, three lines land the work as it stands:

```
vc-x1-dev commit @- -s b --title "feat: add line-2" --body "Added line 2 to single-step.txt" --bookmark feat-vc-x1-single-step-commit
vc-x1-dev commit -s b --closing @- --land main
vc-x1-dev delete-bookmark -s w --bookmark single-step.txt-line-2 --skip remote
```

The second line lists both bookmarks at the tip and is rerun naming `feat-vc-x1-single-step-commit`,
which Land then deletes, naming the other as merged. Afterwards `main` is at the closing in both
repos and on origin, the rung carries its title and an `ochid:` trailer whose partner
`vc-x1 show -s both` resolves, both bookmarks are gone, and `cargo test` passes with a test per
rule.

Result (2026-10-25): passed in its replica, not yet run live.

- The test **the_fixture_lands_by_three_lines** runs the three lines on the fixture's shape with
  local remotes and asserts the end state, the revision form of the second line refused naming both
  bookmarks. `cargo test` passes, a test for each rule of the five feature rungs.
- The live fixture's first line was dry-run, the describe, the trailers, and the advance planned as
  in the test. The live run lands the fixture's `main` on GitHub, outside the waiver, and is
  wink's, its commands in the continuation notes.

#### Deliberation

- Runs under the v0.3.1 proposal's rules, wink's call (2026-10-25), the bend recorded here, Land
  inside it: `commit -s b --opening` makes the bookmark and the opening in one invocation and
  pushes after the description review, no rename to `-dev`, the closing by `commit --closing`.
  - The proposal's own text still names `commit --closing <REV>` and the bare form, which this
    cycle replaces, so the docs rung brings it to the new surface.
- The reviews and the per-push approvals are waived from **feat: commit --closing takes a bookmark
  or a revision** through the closing commit, wink's call (2026-10-25): "Permission granted to
  complete this cycle upto closing but not to land on main". Land is outside it and waits for
  wink's go, as does the choice of the closing's shape.
- REV is the commit the description is for, `@` by default, so today's behavior is the default
  and `@-` the one revision anyone will type for a draft made by hand.
  - It is unpublished, no remote bookmark reaches it, and carries no trailer yet, so the describe
    is a draft's rewrite and never a re-describe. A non-empty `@` above it is left as `push`
    leaves new work.
  - One commit only. Several drafts collapsing into one rung is **feat: vc-x1 runs a local
    ladder**.
- `--closing` takes a bookmark or a revision, required with no default, both resolving to the
  tip as jj resolves them, a name that is both being the bookmark, and one node above the trunk
  finalized in place while more make the merge, so the bare form and the REV form become one.
  - The type decides what Land deletes: a bookmark given goes; a revision given means the current
    bookmark on it or else the nearest parent above the trunk, several at one height an error
    listing them, none meaning nothing deleted and said so. The bookmark form is the revision
    form's disambiguator, since it names the same commit.
  - Any commit on any line closes, bushy or linear, wink's call (2026-10-25): the closing is a new
    commit with the trunk and the tip as parents, made where the tip is, and `@` takes part only
    when `@-` is the tip, folding in with the new `@` set on the closing. Commits above a tip
    that is not `@-` stay as they are, their line unclosed. Two heads close one at a time, each
    landed before the next, since the second's first parent is the trunk as it then stands.
  - Without `--land` the first parent is the configured trunk, we think `[cycle] trunk`, `main`
    when absent, and nothing lands.
  - The closing's content is the record move when `[record]` is configured, plus `@`'s edits
    when `@-` is the tip. Today an empty `@` is refused as "nothing to commit" without a record.
- The cycle's bookmark is never derived from the branch: `--closing` names it or the nearest rule
  finds it, `--bookmark` moves it past a nearer one of the user's, and Land names the branch's
  other bookmarks as merged, theirs to keep or delete. **push takes a revision, and the remote's
  name for it** wants the same `--bookmark` widening.
- Four feature rungs in dependency order, REV first since the closing's tip resolution builds on
  it, the fixture test last since it needs them all, then the docs rung, which also carries the
  v0.3.1 material the last cycle's continuation notes held: the per-rung flow's steps 8 and 9
  merged now that `commit` pushes, Opening step 2 replaced by `commit --opening`, and wink's
  principle "warn rather than refuse", asked and unanswered, possibly for `agent-data/code.md`.
- The fixture is the acceptance target, so the last cycle's note to remove it is superseded until
  this cycle lands. After Land the fixture, its GitHub agent repo
  `winksaville/t-85-8.1-2ecac.agent-session`, the failed `tmp/t-85-8.1+2ecac`, and
  `tmp/prompt.txt` are wink's to remove.
- Grown at the rungs, each within the subject: the key `[cycle] trunk`, since the trunk could no
  longer be `--closing`'s argument; the detour, since a tip off `@-` had no working copy to close
  from; and the closing's bookmark decided once, since the advance at a closing merge refused the
  same two bookmarks the Land did.
- The opening's sync reproduced the last cycle's finding, "1 repo is up to date" with the work `@`
  dirty, filed as the Todo **sync says one repo when the work @ is dirty**. The Waiting entries'
  conditions were checked and neither is met.

#### Ladder

- [feat: vc-x1 commit takes a revision opening][0] (done)
- [feat: commit describes the commit REV names][1] (done)
- [feat: commit --bookmark takes any bookmark below REV][2] (done)
- [feat: commit --closing takes a bookmark or a revision][3] (done)
- [feat: commit --land deletes by the argument's type][4] (done)
- [test: the fixture lands by three lines][5] (done)
- [docs: the v0.3.1 proposal names commit's new surface][6] (done)
- [feat: vc-x1 commit takes a revision closing][7] (done)

##### feat: vc-x1 commit takes a revision opening

The cycle's setup commit: create and publish the bookmark, move the Todo entry into this block,
reset and file the continuation notes, and bump the version-of-record.

##### feat: commit describes the commit REV names

`commit` commits `@` and nothing else, so a draft below it stays undescribed. A positional REV,
`@` by default, names the commit that takes the description and the trailer, described in place
with no new commit when it is not `@`, its guards the draft's: unpublished and trailer-less.

* The draft's describe rewrites a commit that exists, where today's commit makes one.
  - REV is resolved once, `@` and no REV being today's path, and refused before anything is made:
    a revset naming other than one commit, a published commit, whose describe would be a remote
    rewrite, one carrying a trailer already, which is a pair's commit and `squash --title --body`
    re-describes, a conflicted one, and an empty one, since a rung changes the repo.
  - The work side is a describe in place, jj rebasing the descendants so `@` follows with its
    change id and its work, and the agent side is the partner commit as for `@`, its trailer naming
    the draft's change id, which the describe keeps.
  - `-s p` takes the same path with no trailer. `-s a` refuses REV, the agent repo alone having no
    work commit, and so do `--opening` and `--closing`, whose commits are made from `@`.
* The bookmark advance and the push assume the work commit is `@-`.
  - The line's bookmark is resolved at or below the draft rather than below `@-`, a bookmark on
    the draft being the line's, and the advance targets the draft's change id.
  - A draft at `@-` pushes as today. One below `@-` is committed and kept local with a line saying
    so, warned rather than refused, since push publishes `@-` and takes no revision until **push
    takes a revision, and the remote's name for it**.
* Deferred: a `--bookmark` that is not the nearest is still refused, the next rung's subject, so
  the fixture's cycle bookmark cannot yet move past the user's.

##### feat: commit --bookmark takes any bookmark below REV

`--bookmark` must name the nearest bookmark below the work commit, so the cycle's bookmark cannot
follow a rung past a user's own sitting nearer. Any bookmark below REV is accepted and advanced.

* The check was "among the nearest", push's own rule for the bookmark it publishes.
  - The named bookmark must be on the work commit's line, at or below `@-` for a commit of `@` and
    at or below the draft for a REV, and a bookmark off the line is refused naming the line's own,
    so a typo cannot move a branch elsewhere.
  - Push's check still holds afterwards, since the advance puts the named bookmark on the work
    commit, where it is the nearest by construction.
  - The default, the line's bookmark, is still the nearest alone, several refused, since a default
    that guessed among several would move the wrong one.
* `main` and `root()` are on every line, so they are nameable, as `main` was when nearest.

##### feat: commit --closing takes a bookmark or a revision

`--closing REV` names the merge's first parent and the bare form a single-step opening at `@-`,
two forms for one act. `--closing <BM|REV>`, required, names the tip, one node above the trunk
finalized in place and more merged, the closing made where the tip is, `@` folding in only when
`@-` is the tip, and an empty `@` never refused.

* The argument meant the trunk, and the tip was always `@-`.
  - It names the tip now, resolved to one commit, and the trunk is `--land`'s bookmark, else the
    new key `[cycle] trunk`, else `main`, read where it is now, so a moved trunk is still joined.
  - The commits between decide the shape, one finalized in place and more merged, so the bare form
    and the old "only the opening above REV" refusal go, and `--dead-end` with no REV goes with
    them, since the argument is now always there.
* The closing was made from `@`, so only `@-` could be a tip.
  - A tip that is not `@-` is closed by a detour: `@` moves onto it, the closing is made as for
    `@-`, and `@` returns, to the same commit when it held work and to a fresh empty one on its
    parent when it did not, both by change id, since a finalize rewrites the tip and gives the
    commits above it new commit ids. The move is undone with the rest on a failure.
  - The closing reads the config and the record from the tip's tree, since its working copy is the
    tip's, so a config edit sitting in the user's `@` is not seen. We think that is right, the
    closing carrying what the cycle carried.
  - Commits above a tip that is not `@-` stay as they are, their line unclosed.
* An empty `@` was refused at a closing merge with no `[record]`, though the merge is its content.
  - A closing is never refused for an empty `@`.
* Land reads the branch up to the tip rather than `@-`: the descent check and the bookmark it
  deletes, still the nearest below the tip, which the next rung replaces with the argument's type.

##### feat: commit --land deletes by the argument's type

Land deletes the bookmark at the branch's head and refuses two there, so a user's bookmark is
deleted in the cycle's place or stops the Land. `--land <TRUNK>` is the destination, any bookmark,
and the deletion follows `--closing`'s argument: a bookmark given, or the current or nearest
parent bookmark of a revision given, several listed, none said, the branch's others named as
merged.

* One pick serves two acts, the closing's advance and the Land's deletion.
  - The advance at a closing merge took the line's nearest bookmark by the rule every rung uses,
    which refuses two at the tip before Land is reached, so the closing's bookmark is decided once,
    by the argument's type, and both use it: the bookmark advances to the closing and Land deletes
    it.
  - A bookmark named is the one, the trunk excepted. A revision means the bookmarks at the nearest
    commits carrying any, in the range above the trunk, so a bookmark sitting on the trunk's commit
    is never taken, several refused with the list, since naming one settles it.
* A closing with no bookmark had its push publish it on the line's nearest, the trunk, a Land
  nobody asked for.
  - It is kept local with a line saying so, a warning and not a refusal, `--land` landing it.
* The branch's other bookmarks are left where they are and named after the Land as merged, so a
  user's own bookmark is never deleted unless named.
* `--land <TRUNK>` as the destination, any bookmark, came with the previous rung, which made the
  trunk the closing's first parent.

##### test: the fixture lands by three lines

The acceptance check run: the fixture's rung described, its cycle closed and landed, and the
second bookmark deleted, by the three lines and nothing typed in `jj`.

* The live fixture's remotes are on GitHub, and its second line lands its `main`, which the waiver
  of the reviews and pushes leaves out, "not to land on main", so the live run waits for wink.
  - The fixture's shape is replayed in a test with local remotes: an opening on the cycle's
    bookmark, a draft made by hand above it under a second, never-pushed bookmark, and an empty
    `@`. The three lines run as the acceptance check writes them, the revision form of the second
    refused naming both bookmarks and rerun with the cycle's, and the end state is asserted: `main`
    at the closing in both repos and on origin, the draft described with a trailer whose partner
    names it back, and `main` the only bookmark left.
  - The live fixture's first line was run with `--dry-run`, which writes nothing: the describe, the
    pair's trailers, and the advance past the user's bookmark were planned as in the test.
* The dry run's push line reports the bookmark where it is, at the opening, since a dry run moves
  nothing, where the real run advances it first: a report that disagrees with the run, filed as the
  Todo **commit's dry run reports the push as the advance leaves it**.

##### docs: the v0.3.1 proposal names commit's new surface

The proposal names `commit --closing <REV>` and a bare form this cycle retires, and the last
cycle's continuation notes held material for it never written in: steps 8 and 9 merged, Opening
step 2 as `commit --opening`, and "warn rather than refuse". The entry is brought to the surface
that exists.

* The proposal described the closing by its old argument, the trunk, and its bare form.
  - Its closing sections now name the tip, `--closing <BM|REV>`, onto the trunk from `--land` or
    `[cycle] trunk`, with no bare form, and Land as the closing's `--land` and its bookmark rule.
  - Any commit on any line closes, by the detour, a bullet of the commit-count section.
* The last cycle's continuation notes held three items never written in.
  - Steps 8 and 9 merge, `commit` pushing on the description review's go, `--no-push` keeping the
    look at the commit before a separate push.
  - Opening step 2 is `commit --opening`'s, named in the commit-count section.
  - "Warn rather than refuse" is a section of its own, marked open, asked and unanswered, with the
    uses this cycle made of it, so the answer has its examples beside it.
* `commit REV` for a draft made by hand joins the local-pair section, the sanctioned path where the
  rules would otherwise send an agent to `jj describe`.
* The two-paragraph body's pointer names the record where it is at that commit, zDOING.md on a
  rung and zCLOSED.md on the closing, written mid-cycle and set aside as a patch until this rung.

##### feat: vc-x1 commit takes a revision closing

Closing out the cycle.

* A closing made without `--land` has no one-command Land: `commit --land` lands only the closing
  it makes, so the Land of this cycle, its closing pushed under the waiver, is four commands typed,
  in the continuation notes.
  - Filed as the Todo **commit --land lands a closing already made**.
* The shape is the trapezoid, the only one `commit --closing` makes until **feat: commit --closing
  takes a shape, from the flag or the config**, and the choice is wink's before Land, the waiver
  leaving it out.
* Nothing beyond the README's `commit` section must outlive the cycle: the rules of REV, the tip,
  the trunk, and the closing's bookmark are written there.
* No agent-file changed, so `notes/agent-files-size.md` gains no row.

# References

[0]: #feat-vc-x1-commit-takes-a-revision-opening
[1]: #feat-commit-describes-the-commit-rev-names
[2]: #feat-commit---bookmark-takes-any-bookmark-below-rev
[3]: #feat-commit---closing-takes-a-bookmark-or-a-revision
[4]: #feat-commit---land-deletes-by-the-arguments-type
[5]: #test-the-fixture-lands-by-three-lines
[6]: #docs-the-v031-proposal-names-commits-new-surface
[7]: #feat-vc-x1-commit-takes-a-revision-closing
