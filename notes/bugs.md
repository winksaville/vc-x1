# Bugs

Known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` …) the same way
as `## Todo` in `TODO.md`; run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. **`push` `bookmark-set` races the git index lock.**
   `jj bookmark set` on the colocated work repo failed twice
   (the 0.69.0-3 and 0.69.0-4 pushes, same stage) with
   "Failed to reset Git HEAD state … could not acquire lock
   for `.git/index` … after 1 attempt(s)"; the lockfile was
   already gone on inspection seconds later. Seen again at
   the 0.75.0-2 push (which also triggered #3) and the
   0.77.0-0 push (2026-07-28) — four occurrences, always at
   `bookmark-set`, always transient.
   - **Cost:** push aborts mid-flow (rollback restored both
     repos cleanly both times); recovery is a `--restart`
     rerun, which succeeded both times.
   - The bot thinks a git-aware watcher (shell prompt,
     editor) briefly re-reads the repo after the commit
     stages touch `.git`, holding the index lock exactly
     when `bookmark-set` resets git HEAD; jj gives up after
     a single attempt.
   - **Fix direction** (two options, possibly combined):
     - retry with short backoff around the bookmark-set
       stage (or all jj invocations that reset git HEAD
       state)
     - use jj-lib in-process instead of spawning `jj`
       commands (suggested 2026-07-15) — the lock
       contention is external, so this alone doesn't
       remove the race, but the retry loop becomes ours
       (catch the lock error, back off, retry) with real
       error types instead of stderr parsing; this is the
       refactor program's
       [jj-lib migration stage](refactor-20260716.md#stage-jj-lib-migration)

2. **stdout output panics on a closed pipe (EPIPE).**
   `vc-x1 bot-session <file> | head` panics once `head`
   closes the pipe: the logger's `println!` aborts with
   "failed printing to stdout: Broken pipe". Repo-wide
   behavior of the `info!` → `println!` path, but
   bot-session (0.70.0-2) is the first subcommand whose
   output routinely feeds a pager/filter. Found during
   0.70.0-2 verification.
   - **Cost:** ugly panic + backtrace hint instead of the
     Unix-conventional silent exit; output before the pipe
     closed is intact.
   - **Fix direction:** handle EPIPE in the logger (write
     via `writeln!` to a locked stdout and exit 0 on
     `BrokenPipe`), or reset SIGPIPE to default on unix at
     startup.

3. **`push` resume-after-rollback replays from the wrong
   stage.** Observed at the 0.75.0-2 push (2026-07-23): the
   `push` `bookmark-set` git-index-lock race fired, the error path
   `op restore`d both repos — undoing `commit-work` /
   `commit-bot` — but the state file still said
   `stage = bookmark-set`. The rerun resumed there,
   *skipping the commit stages*: bookmark-set pinned the
   bookmarks to `@-` (the **previous** cycle commit),
   push-work no-op re-pushed it, and squash-push-bot
   squashed the accumulated session data into the
   already-published previous bot commit and republished
   it. The completion sanity check caught the chid
   mismatch and warned, but after the damage.
   - **Cost:** no data loss (work `@` kept the uncommitted
     changes; the bot chid is rebase-stable so the ochid
     pairing survived) — but session data landed under the
     previous commit's title, and the published bot commit
     was rewritten in place.
   - The state file and the op-restore rollback disagree
     about where the run stopped: rollback rewinds the
     *repos* to pre-commit, but not the *state* to the
     `message` stage. Any `Err` between `commit-work` and
     the stage save has the same shape.
   - **Fix direction:** on rollback, rewind (or delete) the
     state file in the same breath — or retire the state
     file entirely and derive resume from repo reality,
     which is the refactor program's
     [stateless push stage](refactor-20260716.md#stage-stateless-push);
     this incident is its strongest evidence yet.
   - **Second occurrence, 0.77.0-0 push (2026-07-28)** — the
     same #1 lock race, the same shape, caught before any
     damage this time. The rollback was clean (both repos
     back to pre-commit, `@` holding the uncommitted changes,
     nothing published — the failure was two stages before
     `push-work`), and the state file still read
     `stage=bookmark-set`. A plain rerun would have set both
     bookmarks to the *previous* cycle's commit and squashed
     this session's data into the already-published bot
     commit; `--restart` is the safe rerun until the fix
     lands. Noted while opening the very cycle that fixes it.

4. **`push` `commit-work` commits an empty `@`, minting a
   duplicate stamped commit.** Observed at the 0.76.0-1 push
   (2026-07-27): the work commit had been made by hand before
   invoking `vc-x1 push`, so `@` was empty — `commit-bot`
   skips a clean repo, but `commit-work` committed the empty
   `@` anyway with the supplied `--title`/`--body`. The
   result: an empty duplicate of the real commit on top of
   it, the ochid trailer stamped on the duplicate (push
   stamps only the topmost commit), the bookmark pushed at
   the duplicate, and the bot commit's `ochid:` pointing at
   the duplicate instead of the real commit.
   - **Cost:** no data loss, but published history needed a
     dual-repo repair — describe + abandon + sideways
     force-push on both sides (`--ignore-immutable`).
   - **Fix direction:** `commit-work` skips an empty `@`
     like `commit-bot` does, and the ochid stamp then lands
     on the real topmost commit; alternatively error loudly
     when `@` is empty and no commit is needed. Fold into
     the refactor program's
     [split push.rs + stateless push stage](refactor-20260716.md#stage-split-pushrs),
     which rebuilds this code path.

# References
