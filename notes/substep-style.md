# Sub-step Workflow Conventions

Conventions adopted on `init-clone-refactor` during the
`0.41.1-6.7` OF refactor cycle. Captured here separately from
`CLAUDE.md` (which is on `main`); folds into `CLAUDE.md` at
cycle close-out and merge-back.

## Terminology

The project's version-suffix scheme nests as follows:

- **Single step** — one change, one commit + push + finalize.
  Examples: `0.1.0 → 0.2.0`, `0.2.1 → 0.2.2`.
- **Multi-step** — a planned series of steps within a target
  bump:
  - `0.5.0   → 0.6.0-0` (first step)
  - `0.6.0-0 → 0.6.0-1` (second step)
  - `0.6.0-1 → 0.6.0-2` (third step)
  - …
  - `0.6.0-N → 0.6.0` (close-out, drops suffix)
- **Sub-step** — finer-grained step within a multi-step's
  step. Versions add `.M` to the step's `-N`:
  - `0.6.0-3.1 → 0.6.0-3.2` (sub-step within step `-3`)
  - `0.6.0-3.M → 0.6.0-3` (close-out collapses sub-steps
    into their parent step)
- **Sub-sub-step** — finer still, within a sub-step. Versions
  add another `-K` suffix:
  - `0.6.0-3.4-0 → 0.6.0-3.4-1` (sub-sub-step within sub-step
    `-3.4`)
  - `0.6.0-3.4-K → 0.6.0-3.4` (close-out collapses sub-sub-steps)

The conventions in the rest of this file apply to **the leaf
level of the hierarchy** — the finest granularity the current
cycle's plan went down to. In the in-flight `0.41.1-6.7` cycle,
the leaf level is sub-sub-steps (e.g. `0.41.1-6.7-5` is
sub-sub-step 5 within sub-step `-6.7` within step `-6` of the
0.41.1 multi-step). The `(1)/(2)/.../(N)` markers under `-6.7`
in `notes/todo.md` are therefore *sub-sub-steps*, not
*sub-steps*. The file is named `substep-style.md` for brevity,
but read "sub-step" inside as shorthand for "leaf-step at
whatever depth the current cycle plans to".

## When to push

**Mid-cycle sub-step commits stay local.** Push only at
cycle close-out, after squashing the sub-step stack into
one `X.Y.Z-N` commit.

Reasoning:

- Pushing mid-cycle locks in the per-sub-step granularity
  on the remote. The planned close-out squash would then
  require a force-push to rewrite history — losing the
  linear-history property and burning the early ochid
  pairings.
- Per-sub-step commits are *review/navigation scaffolding*
  for the bot↔user iteration loop, not the published shape
  of the cycle. The published shape is one
  `X.Y.Z-N` commit per cycle, matching the existing project
  convention.
- If a particular sub-step really does need to land on the
  remote independently (e.g. it unblocks parallel work),
  promote it to its own `X.Y.Z-N` step rather than pushing
  it as a sub-step.

Concrete: do not run `vc-x1 push` (or `jj git push`) until
the cycle close-out sub-step has executed
`jj squash --from <range> --into <target>` (or equivalent)
to collapse the sub-step stack and re-established the
single coordinated ochid trailer between the squashed
app and `.claude` commits.

## Version suffix in titles and Cargo.toml

Sub-step commits use the version `X.Y.Z-N-M` in commit titles
**and** in `Cargo.toml`. So `vc-x1 -V` shows the active
sub-step at build time.

```
0.41.1-6.7-1   sub-step (1) of step -6.7
0.41.1-6.7-2   sub-step (2)
…
0.41.1-6.7     squashed cycle commit at close-out
```

Cargo accepts `0.41.1-6.7-5` as a single semver pre-release
identifier; lexical comparison gives the expected ordering
within the sub-step ladder.

Bump Cargo.toml at the **start** of each sub-step. The
existing `X.Y.Z-N` Cargo bump rule (single bump at step
start) extends down a level for sub-steps.

## todo.md status flips

The status markers in `notes/todo.md > ## In Progress` flip
on a defined cadence:

- **Start of sub-step (M):** mark (M) `(current)` as the
  first edit. Reflects what's actually in flight.
- **End of sub-step (M):** flip (M) from `(current)` to
  `(done)` **before** running the cargo cycle and committing.
  The commit then captures the completed state.
- **Start of (M+1):** mark (M+1) `(current)`. Goes in
  (M+1)'s own commit.

Each sub-step's commit carries the "this sub-step is done"
record; the next sub-step's commit carries "next sub-step
starts".

## Pre-commit cargo cycle

Run before every sub-step commit (not just at cycle
close-out):

1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo install --path . --locked`
5. (re-test if anything substantive)

This keeps every intermediate commit buildable so
bisection works across the cycle's stack. Broken
intermediates can't be bisected.

## Commit-first review model

Workflow per sub-step:

1. Make sub-step changes.
2. Run the cargo cycle (above).
3. **Commit immediately** (both repos with ochid trailers,
   no separate approval gate).
4. Summarize the commit briefly in chat.
5. User reviews the commit in their editor (full file
   context).
6. User iterates if needed; bot squashes follow-up
   changes into the existing sub-step commit via
   `jj squash --into @-` (and `jj describe @-` if the
   title needs to change).
7. User signals approval to move to the next sub-step
   (e.g. "go to (M+1)").

This replaces the previous "summarize → review → approve →
commit" gate at the sub-step level. Reasoning: local jj
commits are mutable until close-out squash, so committing
freely is safe; reviewing in a real editor with full file
context beats chat-pasted diffs.

The two-gate ceremony (review + message approval) is
preserved for the **cycle-level push** at close-out — that
crosses the local→remote boundary and warrants explicit
approval.

## Ochid trailers on sub-step commits

Sub-step commits include ochid trailers paired across the
two repos:

- App repo body trailer: `ochid: /.claude/<.claude-chid>`
- `.claude` repo body trailer: `ochid: /<app-chid>`

Use `vc-x1 chid -R .,.claude -L` to capture both pre-commit
change IDs (first line app, second line `.claude`).

The trailers survive squash at close-out (the squashed
commit's chid is one of the sub-step chids, and the
trailers point at the corresponding `.claude` chid).
Specifically the cycle close-out should re-establish a
single coordinated ochid trailer between the squashed app
commit and squashed `.claude` commit.

## `.claude` cadence

The `.claude` repo commits **per sub-step alongside the
app repo** (option (ii) per the discussion that landed
this convention). Each sub-step's `.claude` commit
captures the session state at that moment; squashed at
close-out alongside the app stack.

Alternative considered: `.claude` accumulates session WC
across the cycle and commits once at close-out (option
(i)). Rejected — keeping the per-sub-step pairing
preserves flexibility (any sub-step could be promoted to
its own push without restructuring).

## Multi-field leaf → `&LeafType` parameter

Helper functions called by a subcommand body should accept
a multi-field leaf type by reference rather than unpacking
fields at the call site:

```rust
// Multi-field leaf — pass the whole leaf
fn run_retry(cmd: &str, args: &[&str], cwd: &Path,
             retry: &PushRetryFlags) -> Result<…> { … }

run_retry("git", &["push", …], cwd, &args.push_retry)?;
```

Wins on readability, future-proofing (extending a leaf
adds zero call-site touches), and `clippy::too_many_arguments`
avoidance.

For **single-field leaves** (e.g. `DryRunFlag`,
`PrivateFlag`), direct read at the consumer site
(`args.dry_run.dry_run`) is fine — wrapping a `bool` in a
`&LeafType` parameter doesn't earn the indirection.

This convention is also captured in
`src/options_flags/README.md` under "Consumer function
shape".

## OF layout graduation

OFs currently sit as flat `<name>.rs` files under
`src/options_flags/`. Once an OF accumulates enough
rationale, edge-case detail, or examples to outgrow doc-
comments, it graduates to a `<name>/mod.rs` +
`<name>/README.md` subdirectory layout (per the discussion
in `src/options_flags/README.md > Layout note`).

Mechanical when needed; not done preemptively.

## Open follow-ups (defer to later cycles)

- Field-rename inside leaves (e.g. `push_retries` →
  `retries` with `#[arg(long = "push-retries")]` override)
  to drop redundant prefixes when accessed via the leaf.
- Migrate `clone.rs` and `push.rs` `pub dry_run: bool` to
  flatten `DryRunFlag` (cycle scope was init only).
- `FlagBundle` first generic-bound use (currently
  `#[allow(dead_code)]`); `FlagParser` first impl (in (6)
  with ScopeFlag / RepoFlag).

## Reviewing committed sub-steps

The commit-first review model assumes the reviewer can read
the diff of an already-committed revision. Don't `jj edit -r
@-` back into a past commit to view it — that marks the
commit mutable, shifts the WC pointer, and forces a
`jj new -r <head>` dance to recover. Use one of the
non-destructive paths below.

### Terminal (always works)

```
jj diff -r @-                  # diff of the previous commit
jj diff --from <X> --to <Y>    # diff between two arbitrary revs
jj show -r <X>                 # description + diff for a single rev
jj log -r @-..@                # what's between two points
```

Pipe through a pretty differ for color and side-by-side:

```
jj diff -r @- | delta
jj diff -r @- | diff-so-fancy | less -R
```

### External diff tool (jj-launched)

Configure jj to launch an editor for diff review:

```
# ~/.config/jj/config.toml (or `jj config edit --user`)
[ui]
diff-editor = ["zed", "--diff", "$left", "$right"]
# or your editor's diff CLI; falls back to $EDITOR/`vimdiff` etc.
```

Then `jj diff -r @- --tool builtin:meld-3` (or your tool name)
opens a side-by-side viewer with the two trees pre-staged.
Works for arbitrary `--from`/`--to` ranges too. Concrete CLI
flags vary by editor; check your editor's "open as diff" docs.

### VS Code (confirmed working)

VS Code can diff arbitrary commits. Concrete paths:

- **Built-in Source Control + Commit Graph**
  (newer VS Code versions): open the Source Control view
  (Ctrl/Cmd+Shift+G) → "Graph" or "Commits" panel →
  right-click commit A → "Copy Commit ID" → right-click
  commit B → "Compare with…" → paste / pick A. Two-commit
  diff opens in the editor with the changed-files list in
  the side bar.
- **GitLens extension** (richer UX): adds a "Commit Graph"
  view with quick filtering; right-click any commit →
  "Open Comparison" → pick the other commit (HEAD,
  branch, tag, or arbitrary). Per-file actions in the
  diff list let you open individual file comparisons.
- **Command Palette fallback**: Ctrl/Cmd+Shift+P →
  `Git: Compare with…` (also `Git: Compare Branches…`)
  prompts for two refs and opens the comparison.
- **CLI fallback**: `code --diff <fileA> <fileB>` opens
  the editor's diff viewer for two specific files (e.g.
  files extracted with `jj file show -r <rev> <path>`).

All work transparently with jj-created commits since they
land as standard git objects in `.git/`.

If working primarily in another editor, this is a fine
fallback — keep VS Code installed for review even if not
for edits.

### Zed (less certain at time of writing)

Zed's git integration has been evolving; arbitrary
commit-to-commit diff in a panel may or may not be available
depending on version. If absent, the realistic workflow is:

1. Run `jj diff -r @-` in the terminal alongside Zed.
2. Use Zed for full-file context on files of interest (Zed
   shows the post-commit state since the WC sits on top).
3. For a side-by-side view of the just-landed change,
   configure `jj`'s diff tool to invoke Zed (see "External
   diff tool" above) — Zed has a `--diff` CLI flag that
   opens two paths in a diff view.

Confirm support in your installed Zed version; fall back to
`jj diff | delta` in the terminal if not.

## Folds into CLAUDE.md when…

…the `init-clone-refactor` branch merges back to `main`.
At that point:

1. Lift this file's content into `CLAUDE.md` (probably
   under the `## Versioning` and `## Pre-commit
   Requirements` sections, with a new `## Sub-step
   Workflow` section or similar).
2. Delete the `notes/substep-style.md` pointer line at
   the top of `CLAUDE.md`.
3. Delete this file.
