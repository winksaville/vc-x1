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

### 0.33.0-dev3 — `/dev/tty` reconnect + per-dev push+finalize

**`detach()` now explicitly wires the child's stdout/stderr** rather
than inheriting them. Previously only `stdin` was nulled; stdout/stderr
inherited from the parent, which meant:

- In a real terminal, the child's writes happened to still reach the
  shell — but that was accidental, not guaranteed.
- Via a pipe-invoked caller (Claude Code's Bash tool, cron, CI), the
  bash tool closed its read end of the inherited pipe the moment the
  parent exited, so the child's subsequent writes either hit SIGPIPE
  or vanished.

New behavior, in `detach()` before `cmd.spawn()`:

- Open `/dev/tty` read+write. On Unix with a controlling terminal,
  this gives a fd that stays valid regardless of what the parent's
  stdout/stderr pointed at (and regardless of pipe closures on parent
  exit).
- If `/dev/tty` opens, clone the fd and attach as the child's stdout
  and stderr. User sees child output in the shell they invoked from.
- If it doesn't open (pipe-invoked, no controlling terminal, Windows),
  fall back to `Stdio::null()`. The log file is authoritative in that
  case. This is the Claude-Code-Bash-tool path.

No new tests: the behavior branches on runtime environment
(`/dev/tty` presence), which unit tests can't meaningfully exercise
without a tty fixture. Verified end-to-end via `test-fixture` — the
detached child's output reaches the log file in the pipe-invoked
case, and reaches the terminal in an interactive shell.

**CLAUDE.md: per-dev push+finalize workflow.** New "Per-dev step
workflow" subsection explains that each `-devN` step is treated like
a mini session-end: push both repos and finalize `.claude` with
`--detach --delay 10`, then wait for the user to say "continue".
Matches the discipline the user called out — a `-devN` commit is
important, same as any single commit being shipped to remote.

**Refinements from hands-on testing** (same commit):

- **`test-fixture` "Try:" hint** — drop the `work/` finalize suggestion
  (the code repo always uses plain `jj git push`, never finalize) and
  add `jj git push -R <work>` plus a `test-fixture-rm` cleanup hint.
  The earlier `work/ --push main --detach` suggestion was a footgun:
  preflight rejects it because `@` is empty and has no description.
- **`test-fixture --with-pending`** — new flag; writes an uncommitted
  `TODO.md` to `work/` and `session-notes.md` to `work/.claude/` so
  `finalize --squash` has actual changes to squash. Default off
  (clean fixture); opt in for realistic demos.
- **`vc-x1 test-fixture-rm PATH`** — new subcommand for cleanup.
  Safety guard refuses anything whose last path component does not
  start with `vc-x1-test-`, so it can only ever remove intended
  fixtures.
- **Step markers in `finalize_exec`** — `info!` before each subprocess
  call (`finalize: squashing …`, `finalize: setting bookmark … to @-`,
  `finalize: pushing … to origin`, `finalize: done`). Makes the
  interleaved `jj` output easy to follow — the earlier run had the
  user wondering whether "Nothing changed." (from `bookmark set`)
  meant push failed, since the output of each subprocess was adjacent
  with no separator.
- **README local-remotes callout** — new "Local remotes, not GitHub"
  paragraph in `### test-fixture`; `test-fixture` runtime output
  now says "Fixture ready (local bare-git remotes, see README.md §
  test-fixture)"; long doc on `TestFixtureArgs` also points at the
  README section. Users initially saw "origin" and assumed GitHub;
  the docs + runtime pointer make the self-contained nature obvious.

### 0.33.0-dev4 — status marker for post-detach failures

The detached finalize child's exit code isn't observable by the
caller (the parent already returned `0` before the child did
anything meaningful). dev1's pre-flight catches most issues
synchronously, and dev3's `/dev/tty` reconnect gets the child's
output onto the user's terminal when one exists — but if the
child fails during its squash or push (network loss, remote
deleted, race), and the caller is pipe-invoked (bash tool, cron,
CI), the failure would still be silent except in the `--log`
file.

dev4 closes that gap with a **status marker file** that the child
writes on failure, and which any subsequent `vc-x1` invocation
surfaces to the user.

**How this relates to dev3.** The two layers are complementary, not
alternatives:

| Caller                                   | Live child output             | Status marker                            |
| ---------------------------------------- | ----------------------------- | ---------------------------------------- |
| Interactive shell                        | `/dev/tty` — visible in the user's terminal | Also written (safety net if the user missed the live output) |
| Pipe-invoked (bash tool, cron, CI) | No tty → null → invisible     | Primary visibility; surfaces on next run |

So interactive users get immediate notification via `/dev/tty`, and
a durable marker they pick up on the next `vc-x1 <anything>`.
Pipe-invoked callers rely entirely on the marker, which is exactly
the case (bash-tool-invoked finalize) that motivated this series.

**Writer** (`finalize::write_failure_marker`): when `finalize()`
returns `Err` and `opts.exec` is true (the detached child path),
write one file per failure to `$HOME/.cache/vc-x1/finalize-status/`.
Filename is `<ns-since-epoch>-<pid>.status`, sortable and unique.
Content is simple `key=value` lines:
```
timestamp_ns=…
pid=…
repo=…
bookmark=…
error=<full error string>
```
Only the detached-child path writes; the synchronous path's `Err`
already surfaces via `error!` through `main::run_command`.

**Reader** (`finalize::surface_previous_failures`): on every
non-detached-child `vc-x1` invocation, called from `main()` right
after `Cli::parse()`. Scans the marker directory, prints each
failure to stderr with a `warn:` prefix, then deletes the file so
it surfaces exactly once.

Suppressed in the detached child (`--exec`) so a child doesn't
consume markers meant for the next interactive run. Detected via
`matches!(cli.command, Commands::Finalize(ref f) if f.exec)` before
dispatch.

**Note: `--version` and `--help` short-circuit** inside `Cli::parse()`
— clap exits before returning, so the surface step doesn't run for
those invocations. Acceptable: marker surfacing is already a
best-effort side channel, and the next actual subcommand
(`vc-x1 chid`, `finalize`, etc.) picks them up.

Verified end-to-end: created a fixture with `--with-pending`,
`rm -rf`'d `remote-claude.git/objects` to break push, ran
`vc-x1 finalize … --detach --delay 1`. Parent returned 0; child
failed push a second later; marker appeared under
`~/.cache/vc-x1/finalize-status/`; the next `vc-x1 chid -R .` ran
the command normally after printing the failure context (including
the full `jj git push` error) to stderr; marker was gone
afterwards.

### 0.33.0-dev4.1 — complete `test-fixture` "Try:" flow

The dev2 "Try:" hint was too terse — it skipped the `jj bookmark
set` step needed to advance `main` after `jj describe @`, so a user
following the hint verbatim got `Warning: No bookmarks found in the
default push revset` and no push happened.

Expanded into a three-step end-to-end recipe that actually works:

```
# 1. code repo: described commit → advance main → push
echo hello > <work>/hello.txt
jj describe @ -R <work> -m 'feat: add hello.txt'
jj bookmark set main -r @ -R <work>
jj git push -R <work>

# 2. session repo: trailing writes → finalize (squash into @-, push)
echo notes > <work>/.claude/notes.md
vc-x1 finalize --repo <work>/.claude --squash --push main --detach

# 3. cleanup
vc-x1 test-fixture-rm <base>
```

Runtime output is a short **pointer** to the README plus a three-line
quick reference with the fixture's absolute paths:
```
Next steps — see README.md § Testing push + finalize for the full flow.
Quick reference with this fixture's paths:
  jj git push -R <work>
  vc-x1 finalize --repo <session> --squash --push main --detach
  vc-x1 test-fixture-rm <base>
```
The README has the full four-step flow (edit, describe, advance,
push; edit, finalize; cleanup). Avoids duplicating a long recipe in
two places — the README is authoritative, the runtime output is a
breadcrumb.

**README.md § "Testing finalize" → "Testing push + finalize".**
Renamed and rewritten around the same three-step flow. Old section
showed `vc-x1 finalize --repo <work> --push main` (no `--squash`),
which the dev1.1 preflight rightly rejects because `@` has no
description — a footgun directly in the docs. New section runs the
real push-for-code, finalize-for-session pattern end-to-end and
explains why the two repos get different tooling. TOC updated.

### 0.33.0 — release rollup

Layered visibility for `finalize` failures. Each dev step closed a
distinct gap; together they mean a failed finalize is never silent
regardless of how it was invoked:

| dev   | What it adds                                            | Failure caught / surfaced                                           |
| ----- | ------------------------------------------------------- | ------------------------------------------------------------------- |
| dev1  | Synchronous pre-flight in the parent (`preflight()`)    | Bad bookmark, non-tracking remote, unresolved squash revset         |
| dev1.1| Conflict / forward-only / push-target-description checks + plan logging; CLAUDE.md post-commit bookmark-advance workflow | Conflicts, divergent bookmark move, undescribed push target         |
| dev2  | `common::run()` stderr at `info!`; `test-fixture` dual-repo scaffold | jj's own messages (e.g. `Nothing changed.`, push summaries) visible without `-v` |
| dev3  | `/dev/tty` reconnect for detached child; step markers in `finalize_exec`; CLAUDE.md per-dev push+finalize workflow; `test-fixture` refinements (`--with-pending`, `test-fixture-rm`, hint fixes, README local-remotes callout) | Interactive terminal sees child output live after detach; per-dev `-devN` is pushed + finalized, so intermediate work isn't lost |
| dev4  | Status marker in `~/.cache/vc-x1/finalize-status/`; `surface_previous_failures` at every `vc-x1` startup | Pipe-invoked callers (bash tool, cron, CI) get their previous child's failure on the next run |
| dev4.1| README "Testing push + finalize" rewrite with complete flow; runtime breadcrumb in test-fixture Try | Users following the docs actually succeed at push+finalize end-to-end |

End state: for a detached finalize,
- synchronous issues never reach detach (exit non-zero, error on stderr),
- child output lands on the user's terminal when `/dev/tty` exists,
- child output is captured in the `--log` file regardless,
- any child-side failure leaves a marker that the next invocation surfaces,
- per-dev push+finalize hygiene means nothing sits unpushed at a
  `-devN` commit.

## Fix deprecated `jj bookmark track` syntax (0.33.1)

jj 0.40.0 deprecated `<bookmark>@<remote>` as an argument form in
favour of `<bookmark> --remote=<remote>`. Two call sites in
`src/init.rs` and one hint string in `src/finalize.rs` still used
the old form. Surfaced by `vc-x1 init actor-x1` on jj 0.40.0, which
printed `Warning: <bookmark>@<remote> syntax is deprecated, use
`<bookmark> --remote=<remote>` instead.`

- `src/init.rs` — fix `jj bookmark track main@origin`
- `src/finalize.rs` — Update the preflight error hint

Surveyed the other jj commands vc-x1 invokes
(`git init --colocate`, `bookmark set`, `bookmark list [-a]`,
`describe`, `git push --bookmark`, `commit`) against jj 0.40.0 —
none emit deprecation warnings.

## Silence untracked-remote hint in init step 9 (0.33.2)

On jj 0.40.0, `vc-x1 init` step 9 ("Re-initializing jj on both
repos") shows a hint from `jj git init --colocate`:

```
Hint: The following remote bookmarks aren't associated with the existing local bookmarks:
  main@origin
Hint: Run the following command to keep local bookmarks updated on future pulls:
  jj bookmark track main --remote=origin
```

The hint is advisory — two commands later we run `jj bookmark
track main --remote=origin` anyway. The noise suggests the user
has to act when they don't.

- `src/init.rs` — both step-9 colocate calls get the global
  `--quiet` flag (`jj --quiet git init --colocate`). Also silences
  the "Initialized repo in \".\"" primary output and the generic
  `git clean -xdf` hint; our own `info!("Step 9: …")` already
  announces the step and the subsequent `Started tracking 1
  remote bookmarks.` line confirms the track worked.

Left the redundant `jj bookmark set main -r @-` call alone — it
prints `Nothing changed.` after colocate (which imports git's
`main` ref as a local bookmark at the right commit) but it's
cheap defense against any edge case where colocate doesn't land
`main` exactly where we need it.

No config knob to target just the untracked-remote hint — only
`hints.resolving-conflicts` is wired.
