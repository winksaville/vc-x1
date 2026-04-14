# Chores-04

## Audit `unwrap`/`unwrap_or` usage (0.32.0)

Survey every `unwrap*` site in `src/` (non-test). Classify each, then
annotate with a trailing `// OK: …` comment that justifies why the
call is acceptable. This is a documentation pass — no behavioral
changes — and a convention we can extend to future code.

### Comment convention

- `// OK: <specific reason>` — when there's a real precondition,
  invariant, or domain reason worth capturing
- `// OK: obvious` — when the default is self-evident from context
  (e.g. `desc.lines().next().unwrap_or("")` — empty desc → empty title)

Bare `// OK` is avoided because it reads like a truncated comment.
Abbreviations like `SE` are avoided because they require a decoder
ring for anyone reading the code out of context.

Tests are left alone. `#[cfg(test)]` `.unwrap()` panics on failure,
which is the correct test behavior.

### Documentation home

Dev-facing conventions live in `notes/README.md` (alongside existing
"Versioning during development" and "Todo format" sections). User-facing
`/README.md` gets a small `## Contributing` section pointing at
`notes/`. `CLAUDE.md` adds a one-line reference so the bot sees the
same convention.

- `notes/README.md` — new `## Code Conventions` section with the
  `// OK: …` rule and examples
- `/README.md` — new `## Contributing` section with link to `notes/`
- `CLAUDE.md` — one-line reference to `notes/README.md#code-conventions`

### Library `.unwrap()` (one site)

`src/desc_helpers.rs:157` — inside a `match matches.len()` with arm
`1 =>`, so `matches.len() == 1` is proven. Refactor to block form,
add `#[allow(clippy::unwrap_used)]` so we can enable the project-wide
lint later without this site firing, and an `// OK: …` comment.

```rust
1 => {
    #[allow(clippy::unwrap_used)]
    // OK: `1 =>` arm guarantees matches.len() == 1
    Ok(TitleMatch::One(matches.into_iter().next().unwrap()))
}
```

### Library `.unwrap_or*` sites

All receive a trailing `// OK: …` comment. Inventory (15 sites):

| File:line | Comment |
|---|---|
| `fix_desc.rs:116` | `// OK: obvious` |
| `fix_desc.rs:218` | `// OK: obvious` |
| `fix_desc.rs:268` | `// OK: "?" placeholder when fix couldn't derive ochid` |
| `fix_desc.rs:284` | `// OK: obvious` |
| `validate_desc.rs:112` | `// OK: obvious` |
| `logging.rs:48` | `// OK: default verbosity when not set` |
| `common.rs:124` | `// OK: CLI default revision` |
| `common.rs:267` | `// OK: no ochid trailer → empty string` |
| `common.rs:268` | `// OK: obvious` |
| `common.rs:289` | `// OK: obvious` |
| `common.rs:308` | `// OK: obvious` |
| `common.rs:358` | `// OK: no --ancestors limit → unbounded` |
| `desc_helpers.rs:104` | `// OK: default true when flag absent` |
| `desc_helpers.rs:133` | `// OK: obvious` |
| `desc_helpers.rs:147` | `// OK: obvious` |
| `finalize.rs:137` | `// OK: default to @ when no squash spec` |
| `clone.rs:34` | `// OK: repo name may not end in .git` |
| `clone.rs:41` | `// OK: repo name may not contain /` |
| `show.rs:140` | `// OK: obvious` |
| `show.rs:144` | `// OK: obvious` |
| `show.rs:230` | `// OK: invalid timestamp → epoch fallback for display` |
| `show.rs:232` | `// OK: invalid tz offset → UTC fallback for display` |
| `symlink.rs:44` | `// OK: read_link after symlink_metadata said it's a symlink; empty path on rare race` |

(Inventory shows 23 sites once fully enumerated — the earlier count of
15 missed some clone/show duplicates. Final count confirmed during edits.)

### `symlink.rs:44` decision

`std::fs::read_link(path).unwrap_or_default()` — reachable only if
`path.symlink_metadata()` just said `is_symlink() == true`. A TOCTOU
race (symlink removed between metadata and read_link calls) could
fire it; falling back to empty `PathBuf` means the caller's subsequent
comparison against the expected target will fail and the symlink gets
recreated. That is acceptable behavior. Keep the default, document
with `// OK: …`.

### Test code

~54 `.unwrap()` calls in `#[cfg(test)]` modules left as-is. Tests
panicking on setup failure is the correct behavior and idiomatic Rust.

### Version

Single-step bump to `0.32.0`. Mechanical doc-only change, no behavior
difference, one commit.

## Make `finalize` failures visible (0.33.0)

`finalize --detach --log <path>` hides failures: the parent spawns a
child and exits 0, the child's error only lands in the log file, and
the caller (interactive user or bot) never sees anything went wrong.
Real incident: `jj git push` failed in the `.claude` repo with
"Non-tracking remote bookmark main@origin exists" — parent returned 0,
session ended, nobody noticed until much later.

### Requirements

1. **Pre-flight validation in the parent**, before `detach()`:
   squash revsets resolve, push bookmark exists, push bookmark is
   tracking its remote ref. Synchronous non-zero exit with visible
   stderr for this class of failures.
2. **Subprocess output must be visible**: `common::run()` currently
   demotes captured stdout/stderr to `debug!`, so without `-v` the
   user never sees `jj`'s messages. Change to `info!` on success
   output and `error!` on failure stderr.
3. **Detached child output must reach the user's terminal when one
   exists**: parent opens `/dev/tty` and passes it to the child as
   stdout/stderr. Falls back to null when there is no controlling
   terminal (pipe-invoked / bot / cron), relying on the log file +
   status marker in that case.
4. **Status marker file for post-detach failures**: child writes an
   exit-code + last-error file on completion (alongside the log, or
   in `~/.cache/vc-x1/`). Any subsequent `vc-x1` invocation reads
   pending markers and surfaces failures to stderr, so the bot or user
   sees the previous failure on their next command.

Every `CliLogger`-enabled record must reach *both* the terminal stream
AND the log file (if `--log`). The existing routing already does this
for enabled levels — audit + document.

### Plan (multi-step)

- `0.33.0-dev1` — pre-flight validation in `finalize::preflight()`,
  called before `detach()`. Covers bookmark existence + tracking,
  squash revset resolution.
- `0.33.0-dev2` — `common::run()` logging: `info!` for subprocess
  output, `error!` for failure stderr, so failures surface without
  `-v`.
- `0.33.0-dev3` — `/dev/tty` reconnect in `detach()`: parent opens
  the controlling terminal and passes it to the child, so a detached
  child can still write to the user's shell. Null fallback when no
  tty.
- `0.33.0-dev4` — status marker file: child writes result on
  completion, next `vc-x1` invocation consumes pending markers and
  surfaces any failures prominently.
- `0.33.0` — roll-up, notes finalize.

### 0.33.0-dev1 — pre-flight validation

`finalize::preflight()` runs before `detach()` and validates:

- **Squash revsets resolve** (if `--squash`): `jj log -r <rev> --no-graph`
  on both source and target. Errors surface as
  `squash source '<rev>' does not resolve: …`.
- **Bookmark exists** (if `--push`): `jj bookmark list <name>` must
  return non-empty. Errors as `bookmark '<name>' does not exist`.
- **Bookmark is tracking its remote** (if `--push`): scan
  `jj bookmark list -a <name>` output. Tracked remotes appear indented
  (`  @origin: …`); a non-tracked remote appears at column 0 as
  `<bookmark>@<remote>: …`. If found, error with a remediation hint:
  `run `jj bookmark track <bookmark>@<remote> -R <repo>` to fix`.

`find_non_tracking_remote()` is a pure helper tested in unit tests
against a few representative `jj bookmark list -a` output snippets.

Bookmark-existence is still re-checked inside `finalize_exec()` as
defense-in-depth — pre-flight catches it synchronously, the child
catches any unlikely race between pre-flight and execution.

### 0.33.0-dev1.1 — repo-state checks + plan logging + post-commit workflow

Two findings after dev1 landed:

1. After `jj commit -R .`, the app repo's `main` bookmark did not
   advance to `@-` (the `.claude` repo's config does advance it, the
   app repo's does not). Easy to miss — the `Parent commit (@-)`
   line in `jj commit` output shows `<bookmark>* |` when the bookmark
   sits on that commit; when absent, the bookmark is stuck behind.
2. `finalize` pre-flight should also sanity-check the target repo
   state before detaching, and should *log the plan* so the user
   knows what's about to happen.

**CLAUDE.md additions**:

- New "Post-commit: advance the bookmark" subsection under the
  commit-style section — rationale + commands.
- New numbered "Post-commit checklist" alongside the existing
  pre-commit checklist — step 1 `jj bookmark list`, step 2 `jj
  bookmark set … -r @-` if stuck. Parallel structure to pre-commit
  so the bot runs through both as a unit.

**Finalize pre-flight additions** (`finalize::preflight()`):

- **No conflicts** — `jj log -r 'conflicts()' …` must be empty.
  Refuses to finalize a repo with unresolved conflicts.
- **Forward-only bookmark move** — `jj log -r '<bookmark>::(<target>)'`
  must be non-empty. If the current bookmark position is not an
  ancestor of the post-finalize target, we'd diverge; error out.
  `(…)` around the target protects `@-` and similar from being
  parsed as a suffix of the bookmark name.
- **Push target has a description** (if `--push`) — `jj log -r <target>
  -T description` must be non-empty. Otherwise `jj git push` would
  fail with `Won't push commit … since it has no description`.

**Plan logging** (`finalize::log_plan()`, called at end of preflight):

```
finalize: squash @ → @- in <repo>
finalize: set bookmark 'main' <current-change> <current-commit> → <target-change> <target-commit> (<target-rev>)
finalize: push 'main' to remote
```

`info!` level so it's always visible. Uses jj templating
(`change_id.shortest(8) ++ " " ++ commit_id.shortest(8)`) for short
IDs. Minor helpers `jj_rev_exists()` and `jj_rev_short()` keep the
preflight body readable.

### 0.33.0-dev2 — subprocess visibility + `test-fixture` subcommand

Two things land together:

1. **`common::run()` subprocess output visibility.** Previously both
   captured stdout and stderr were demoted to `debug!`, so without `-v`
   the user saw nothing from invoked subprocesses on success. jj prints
   human-readable messages (`Moved 1 bookmarks to …`, `Rebased N
   commits`, `Nothing changed.`, push summaries) to **stderr**, while
   data output (bookmark lists, commit IDs) goes to **stdout**. Split
   accordingly:
   - **stderr at `info!` on success** — user sees what jj actually did.
   - **stdout at `debug!`** — callers consume it as data; `info!` would
     flood the user with bookmark lists and revset results.
   - Failure path unchanged — `run()` returns `Err` carrying stderr;
     `main::run_command` already logs it at `error!`.

2. **`vc-x1 test-fixture` subcommand** — scaffolds a throwaway dual-repo
   workspace under `$TMPDIR/vc-x1-test-<timestamp>/` (or `--path PATH`).
   Mirrors the real `vc-x1 init` layout minus GitHub and minus the
   `~/.claude/projects/` symlink. Both repos get a described initial
   commit with matching `ochid:` trailers, a tracked `main` bookmark,
   and a pushed local bare-git remote — so `finalize --push` works
   end-to-end on either side.
   ```
   <base>/
     remote-code.git/     bare remote for code repo
     remote-claude.git/   bare remote for .claude repo
     work/                code repo (jj colocated, main tracks origin)
       .vc-config.toml    path="/",        other-repo=".claude"
       .gitignore         /.claude /.git /.jj /target
       .claude/           session repo (jj colocated, main tracks origin)
         .vc-config.toml  path="/.claude", other-repo=".."
         .gitignore       .git .jj
   ```
   Retests for finalize (dev3, dev4, and beyond) point at
   `--repo <base>/work` or `--repo <base>/work/.claude` instead of the
   live workspace. Two prior retests during dev1 and dev2 accidentally
   ran finalize against the live `.claude` with no `--detach`,
   squashing + pushing mid-stream — that's the motivation.

   Step order matters: create outer work repo → write its `.gitignore`
   excluding `.claude` → only then init `.claude/` as a separate repo,
   so the outer jj doesn't snapshot the nested `.jj/.git` before the
   ignore rule is in place.

No unit tests for the subprocess-logging change: it's one `info!`
call; mocking `std::process::Command` is more noise than signal. The
logger already has unit tests for routing. `test-fixture` gets arg
parsing tests; its filesystem side belongs to future integration tests
(see the todo entry for `tests/` with `tempfile`).
