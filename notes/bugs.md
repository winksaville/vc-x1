# Bugs

Known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` …) the same way
as `## Todo` in `TODO.md`; run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. **`init --repo local` bare remotes keep HEAD at
   `refs/heads/master`.** The only branch pushed is `main`,
   so a later `jj git clone` of that bare repo has no default
   branch to auto-track and `vc-x1 clone` fails its
   `verify_tracking` check ("bookmark 'main' has non-tracking
   remote 'main@origin'"). Found building `tests/cli_sync.rs`
   (worked around there with `git symbolic-ref HEAD
   refs/heads/main`).
   - **Fix direction:** init's local bare provisioning sets
     HEAD to `refs/heads/main` at creation.

2. **`clone` session-remote derivation mismatches init's
   local naming; relative TARGET breaks the session clone.**
   Two related defects, both found building
   `tests/cli_sync.rs`:
   - `derive_session_url` maps `<x>.git` →
     `<x>.claude.git`, but `init --repo local` names the
     session remote `remote-claude.git` — so a dual clone of
     a locally-init'd project's `remote-code.git` looks for
     `remote-code.claude.git` and fails.
   - A relative local-path TARGET fails on the session side
     regardless: `clone_dual` runs the session `jj git
     clone` with the just-cloned code repo as cwd, so the
     relative source no longer resolves. Workaround: pass an
     absolute TARGET.

3. **`push` `bookmark-set` races the git index lock.**
   `jj bookmark set` on the colocated work repo failed twice
   (the 0.69.0-3 and 0.69.0-4 pushes, same stage) with
   "Failed to reset Git HEAD state … could not acquire lock
   for `.git/index` … after 1 attempt(s)"; the lockfile was
   already gone on inspection seconds later.
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

# References
