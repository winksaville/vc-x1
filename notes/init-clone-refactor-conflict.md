# init-clone-refactor: recovery post-mortem + playbook

Drafted 2026-05-01 as a forward-looking brief while the
branch was still broken; rewritten 2026-05-02 after the
recovery completed. Documents what went wrong, the
playbook that fixed it, and the strategic decisions
still open.

## Status as of 2026-05-02 (post-recovery)

- `init-clone-refactor` bookmark is at `a8c1eefe`
  (chid `sumruqqomnzs`) — the published clean
  `0.41.1-6.5` commit.
- All 13 commits from fork point `ykxrlkkwqpyv`
  (`0.41.0`, cid `6747a27b`) through `0.41.1-6.5` are
  unconflicted. `cargo test --bins` passes 331/331 on
  the recovered tip.
- Recovery via `jj bookmark set init-clone-refactor
  -r init-clone-refactor@origin --allow-backwards
  -R .`. The published remote branch turned out to
  be the canonical-good version; local was holding a
  destructively-rewritten copy.
- 8 broken local-only chids still visible as orphans
  in `jj log` (no bookmark, GC-eligible). `jj abandon`
  cleanup is cosmetic, deferrable.

## Status as of 2026-05-01 (pre-recovery, historical)

- Branch existed locally with 8 commits all in
  conflict state: `0.41.1-0` (chore: open) → unnamed
  → `0.41.1-5` (init reshape) → `0.41.1-6.0..6.5`
  (POR baseline tests then progressive split of
  `create_local_repo`). Conflicts on the same files
  all the way down: `Cargo.toml`, `Cargo.lock`,
  `notes/todo.md`, `notes/chores-08.md` (3-sided),
  `src/config.rs`, `src/init.rs`, `src/clone.rs`.
- Original brief's hypothesis — "interrupted rebase
  or substep-style rewrite" — turned out to be wrong.
  See "Root cause" below.
- `@origin (ahead by at least 10 commits, behind by
  8 commits)` was what jj reported. The "ahead by
  *at least* 10" caveat (vs the precise "behind by
  8") was the tell that the remote held more commits
  than local — i.e. local had *lost* work, not just
  rewritten it. Worth noticing earlier next time.

## Why this matters (intent)

Refactor `init` and `clone` to share `init_one` /
`clone_one` primitives plus orchestrators
(`init_dual` / `clone_dual`). Eliminates duplicated
lifecycle logic between init and clone, and gives a
single chokepoint for `--scope`-related changes —
important because every subcommand picking up
`--scope` lands on the same `Scope` shape, and
reusable foundations mean those changes happen in
one place rather than parallel implementations.
Design captured in `notes/chores-07.md > ## init +
clone redesign (0.41.1)`.

## Root cause (post-mortem)

The bot thinks the local rewrites came from a
destructive `jj squash` (or equivalent operation)
that collapsed published commits `0.41.1-1`, `-2`,
`-3` into `-4`. Symptoms supporting this:

- Three chids existed on the remote (`mkwqykksykoo`
  `-1`, `plxsuynrtwks` `-2`, `xtrworrlkozn` `-3`)
  with no local counterpart at all.
- `uuzwzxzkszwq` (the local "unnamed" commit) shared
  the *same chid* as the remote `0.41.1-4` commit,
  but had its description stripped — consistent with
  a squash that pulled three commits' content into
  `-4`'s tree but lost the descriptions.
- Conflicts cascaded down `-5` and `-6.0..6.5`
  because those commits' parent tree no longer
  matched what they had been authored against — the
  rewritten `-4` diverged from what the descendants
  expected.

Net effect: 3 commits of real work disappeared from
local, `-4`'s description was lost, and every
descendant inherited a conflict against the changed
parent tree. The remote was untouched and held the
canonical-good version of all of it.

## Strategic decision (still deferred)

Keep `main` + the `0.42.0-X` series as-is. Now that
`init-clone-refactor` is recovered, we have a clean
branch from which to make the merge-direction call:

- rebase `0.41.1` on top of `0.42.0` as a `0.43.0`
  (or `0.42.1`), or
- vice versa (force-rewrite main — only if there's
  a reason that justifies rewriting published
  history),
- or some other topology.

Decision still deferred. The recovery removes the
"is the branch even salvageable" risk; the topology
question is now purely about scope and review
ergonomics, not technical feasibility.

## Recovery playbook (used 2026-05-02)

Reusable for similar "local rewrites broke a branch
that the remote holds in clean form" situations.
All read-only steps come first; mutation only after
verification.

1. **Back up first.** Cheap insurance.
   - `cp -a . ../<dir>-<date>-N` — full snapshot,
     includes `.jj/op` history. Or `rsync -a
     --exclude=target/ ./ ../<dir>/` if `target/`
     bloat matters.
   - `git clone <remote> ../<dir>-clone-<date>-N`
     — canonical published state, jj-free, browsable
     in gitk without local-rewrite confusion.
   - `jj op log -R . > ../<dir>-op-log-<date>-N.txt`
     — text-readable op log, independent of jj.

2. **Verify remote tip is unconflicted (read-only):**
   ```
   jj git fetch -R .
   jj log -R . -r '<fork-chid>::<branch>@origin' \
     -T '... if(conflict, "CONFLICT", "ok") ...'
   ```
   Every line must read `[ok]`. If any are
   `[CONFLICT]`, the remote isn't a safe target and
   this playbook doesn't apply.

3. **Verify local-only commits are all chid-rewrites
   of remote commits (read-only):**
   ```
   jj log -R . -r '<branch>@origin..<branch>' \
     -T 'change_id.short() ++ ...'
   ```
   Every chid here must also appear on the remote.
   If any chid is unique to local, the reset would
   lose work — investigate before proceeding.

4. **Reset bookmark to remote:**
   ```
   jj bookmark set <branch> -r <branch>@origin \
     --allow-backwards -R .
   ```
   The `--allow-backwards` is required because jj
   guards against accidental backwards moves.

5. **Sanity test:**
   ```
   jj new <branch> -R .
   cargo test --bins      # or full cargo clean ; cargo test for paranoia
   ```

6. **Optional cleanup:** `jj abandon` the orphan
   local-only chids if you don't want them in
   `jj log`. Cosmetic; jj's GC eventually prunes
   them anyway.

## Diagnostic technique (reusable)

Two questions to ask when a branch is mysteriously
conflicted:

- **Are the same chids on the remote in clean form?**
  `jj log -r '<fork>::<branch>@origin'` with the
  conflict-status template tells you whether the
  remote has a working version of every commit.
- **Is local missing any chids the remote has?**
  Compare chid lists between
  `<fork>::<branch>@origin` and
  `<fork>::<branch>` — gaps indicate commits the
  local branch *lost* (squash, abandon, or similar).

If both yield "yes, remote clean + local missing
chids," the reset playbook above is the shortest
path. If only the first yields yes (no missing
chids, local just rewrote into broken state), you
can shrink step 1 of the playbook (work-loss risk
is gone) and just reset.

The fork point itself can be derived without
hand-scanning parent columns:
```
jj log -r 'heads(::<branch-A> & ::<branch-B>)'
```
Returns the latest commit that's an ancestor of
both — the divergence point.

## JJ-CONFLICT-README (the thing that shows in gitk)

When jj has a conflicted commit, it serializes the
conflict state into a git-side tree containing:

- `JJ-CONFLICT-README` — text file explaining the
  format.
- `.jjconflict-base-N/` — the base side(s) of the
  three-way merge.
- `.jjconflict-side-N/` — each conflict input.

Plain git tools (gitk, GitHub UI, etc.) display
this as real files. **Recoverable** — `jj` itself
knows the commit is conflicted and `jj resolve`
walks the inputs. Don't `git checkout` a conflicted
commit and start editing files directly; jj's
invariants get confused. The README itself includes
the recovery hint (`jj abandon` if you accidentally
end up in this state).

## Open questions

Merge direction and chores-NN ordering moved to
`notes/todo.md > ## Todo` (top entries) for
discoverability. Remaining technical question:

- **Conflict surface against 0.42.0.** Now that
  `init-clone-refactor` is clean, what's the actual
  diff vs current `main`? `0.42.0-1/2/3` rewrote
  `init.rs` / `scope.rs` / `common.rs`; `0.41.1-5/
  -6.x` also touched those. Real conflicts will
  surface only at the rebase step.

## References

- `notes/chores-07.md > ## init + clone redesign
  (0.41.1)` — full design captured in 0.42.0-4
  (docs-only commit on main).
- `notes/jj-revsets.md` — revset primitives used in
  the diagnostic and playbook above (`..` exclusive
  range, `::` inclusive DAG range, `heads()`,
  template `if(conflict, ...)`).
- `notes/substep-protocol.md` — close-out squash
  recipe for substep ladders. Note: this branch's
  `-6.0..6.5` ladder was authored before the
  protocol was formalized; the protocol postdates
  it.
