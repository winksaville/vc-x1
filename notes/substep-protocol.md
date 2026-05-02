# Substep protocol

When a single multi-step `X.Y.Z-N` cycle covers several
separable concerns and the per-step diff would be too large
to review in one pass, work each concern as its own substep
on its own jj `@`. The ladder collapses into a single
commit at close-out and ships through the normal
`vc-x1 push` flow.

This file is the protocol of record. CLAUDE.md points here.

## When to use

Use substeps when an `X.Y.Z-N` step splits naturally into
two or more units of work that:

- can be reviewed independently,
- want bisectable history during the cycle (so a regression
  surfaced by tests can be pinned to one substep),
- benefit from intermediate-test gates rather than one
  end-of-step gate.

Don't use substeps for trivial or single-concern steps. The
per-substep `cargo test --bins` cost is real; pay it only
when the review and bisection wins are real.

## Per-substep contract

For each substep:

1. `jj new -R .` — create a fresh empty `@`.
2. Do the substep's work on this `@`.
3. Run `cargo test --bins`. **Non-negotiable.** Build and
   clippy alone are not enough — past experience
   (0.41.1-6.5) had a regression introduced in an early
   substep that didn't surface until a later substep ran
   the full suite, by which point bisection cost was
   higher than it needed to be.
4. `jj describe -m "..." -m "..." -R .` — give the substep
   a working title and body. The title does **not** need a
   version suffix during the ladder; close-out collects
   every substep into one final commit which carries the
   `(X.Y.Z-N)` marker. Substep titles are scratch — they
   never reach the remote.

The ladder is local-only. Substep commits exist for review
and bisection during the cycle and disappear with the
close-out squash.

## Navigating the ladder

For revset basics (chid, cid, `@`, `@-`, `@+`, `..`, `::`,
prefix matching) see [`jj-revsets.md`](jj-revsets.md).

Common moves while a ladder is active:

- `jj log -r 'all()' -R .` — see the whole ladder at a
  glance
- `jj edit -r <prefix> -R .` — jump `@` to any commit in
  the ladder by chid prefix. Useful for bisection: re-run
  the failing test at each candidate without manual
  checkout juggling.
- `jj edit @-- -R .` — quick-jump back two substeps.
- `jj diff -r <chid> -R .` — review one substep's diff in
  isolation.

### Watch the set of substep commits

Below is an example of using a revset using the `::` operator
and chosing last commit pushed to the remote, lulqxovr. So
by using log and `lu::` we can immediately see all of the commits
associated with the substep:
```
wink@3900x 26-05-01T23:32:51.723Z:~/data/prgs/rust/vc-x1 (@xqpnxtyu)
$ jj log -r lu::
○  pynknoqu wink@saville.com 2026-05-01 16:16:23 33612c81
│  docs: CLAUDE.md substep-protocol pointer
@  xqpnxtyu wink@saville.com 2026-05-01 16:16:23 f40c49a7
│  docs: substep-protocol.md formal protocol
○  lzqmlpvy wink@saville.com 2026-05-01 16:16:23 8c722f8c
│  docs: jj-revsets.md review fixes
○  wmmxkuoz wink@saville.com 2026-05-01 11:28:46 substep-protocol-x1 65a6705f
│  chore: scratch — jj-revsets learning + substep test scaffolding
◆  lulqxovr wink@saville.com 2026-04-27 22:27:23 main 55eadc8e
│  docs: capture 0.41.1 init+clone redesign plan (0.42.0-4)
~
```

Substep work happens on whatever `@` is currently selected.
Use `jj edit -r <chid>` to switch to whichever substep you
want to work on — for example, `jj edit -r x` lands on the
substep whose chid starts with `x`, where it can be examined,
tested, or modified. Modifications to a selected substep
rewrite that commit in place, and descendants auto-rebase.

## Close-out/squash the substeps into a Step

When all substeps are done and `cargo test --bins` is green
on the latest:

```
jj squash --from "<base>..@-" --into @ -u -R .
```

Where `<base>` is the parent of the **first** substep —
typically the bookmark or commit the ladder branched from.
The `-u` flag (use destination message) tells jj to keep
`@`'s description and discard the source descriptions; the
intermediate titles served their purpose, and `vc-x1 push`
will overwrite `@`'s description anyway.

After squash:

- All substep diffs are folded into `@`.
- The intermediate substep commits become empty and are
  abandoned automatically.
- History is linear: `<base> → @`.

Then run `vc-x1 push <bookmark>` exactly as for any other
step. Push's `commit-app` stage finalizes `@` with the
close-out title/body and the proper `(X.Y.Z-N)` version
suffix.

### Validation status

The recipe was validated 2026-05-01 against scratch ladders
built by [`substep-test.sh`](substep-test.sh):

- 3-substep ladder (`base..@-` = {s1, s2}, `@ = s3`):
  squash produces `base → @` linear; intermediate commits
  abandoned; `@` retains its description.
- 5-substep ladder (`base..@-` = {s1..s4}, `@ = s5`): same
  outcome at scale.

The recipe is N-agnostic for N ≥ 2 substeps. For N = 1 the
close-out squash is a no-op (`base..@-` is empty when `@-`
is `<base>`); push the single commit directly.

## Recovery

If a substep goes wrong and you want to back out without
losing prior substeps:

- **Discard the current substep's work.** `jj abandon @ -R .`
  drops it and gives you a fresh empty `@` on the same
  parent. Earlier substeps are untouched.
- **Edit an earlier substep.** `jj edit -r <chid> -R .`,
  make corrections, then `jj edit -r <last-substep-chid>`
  to return. Descendants auto-rebase.
- **Discard the entire ladder.** `jj op log -R .` shows the
  op history; `jj op restore <op-id> -R .` reverts the repo
  to that point. This is a full undo — it removes *all*
  substep work after the chosen op. Use only when you want
  to start over. **Not** a close-out recipe; the squash
  above is.

## Worked example

A ladder of 3 substeps starting from bookmark `main` at
commit `abc1234`:

```
jj new -R .                                # substep 1
... edit files, cargo test --bins ...
jj describe -m "substep 1: ..." -R .

jj new -R .                                # substep 2
... edit files, cargo test --bins ...
jj describe -m "substep 2: ..." -R .

jj new -R .                                # substep 3
... edit files, cargo test --bins ...
jj describe -m "substep 3: ..." -R .
```

`jj log -r 'all()'` now shows:

```
@  yyy ... substep 3: ...
○  xxx ... substep 2: ...
○  www ... substep 1: ...
○  abc1234 main | <previous step's commit>
```

Close out:

```
jj squash --from "main..@-" --into @ -u -R .
```

Result:

```
@  yyy ... substep 3: ...   (now contains all substep diffs)
○  abc1234 main | <previous step's commit>
```

Then `vc-x1 push main --title "..." --body "..."` finalizes
the cycle.

**Note:** Above you see `-R .` being used to explicitly select
the current repo. For the bot the explicitness is desirable;
for others it's optional.

# References

- [`jj-revsets.md`](jj-revsets.md) — revset primitives
  (chid/cid, `@`/`@-`/`@+`, `..`/`::` ranges, prefix matching).
- [`substep-test.sh`](substep-test.sh) — script that
  scaffolds a 4-revision ladder under `/tmp/substep-test`
  for squash-recipe experiments.
- 0.41.1-6.5 cycle — first multi-substep usage. The
  per-substep `cargo test --bins` gate originated there
  after a regression introduced in an early substep wasn't
  caught until a later substep ran the full suite.
