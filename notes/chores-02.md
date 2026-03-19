# Chores-02

## Graphical tools for jj repos

### GG (gg)

Cross-platform GUI for jj (Tauri-based). Interactive graph view.

- Install: https://github.com/gulbanana/gg
- `gg gui` does NOT work; use `gg web` instead (opens in browser)

### jj-fzf

fzf-based TUI centered on `jj log` graph. Previews diffs, evolog,
op log. Many keybindings for common jj operations.

- Install: https://github.com/tim-janik/jj-fzf
- Requires `fzf` installed separately (`sudo pacman -S fzf` on Arch)

### Others (not tested)

- **jjui** — TUI for jj (https://github.com/idursun/jjui)
- **jjgui** — commercial desktop app (https://jj-gui.com/)
- **Jujutsu Kaizen** — VS Code plugin
- **VisualJJ** — VS Code plugin (native jj, no git colocation)
- **Selvejj** — JetBrains plugin

### Built-in

- `jj log` — shows graph by default (no `--graph` flag needed)
- `gitk --all` — works on the colocated `.git` but sees raw git refs,
  not jj's view of history

## jj commit organization and traversal mechanisms

### Motivation

Understanding how jj commits are organized and what jj-lib APIs exist
for traversing them — a prerequisite for building richer views of
codebase evolution over time. The todo item "determine the organization
of commits in jj and how we can iterate over them" prompted this
investigation.

### Three axes of traversal

jj provides three distinct ways to navigate commit history:

1. **Commit DAG** — parent/child relationships between commits
2. **Refs/bookmarks** — named pointers (bookmarks, tags, git refs) into the DAG
3. **Evolution log (evolog)** — rewrite history of a single ChangeId across operations

Each answers a different question:
- DAG: "what is the commit history?"
- Refs: "where are the named branches pointing?"
- Evolog: "how was this particular change built up?"

### Commit DAG traversal via revsets

jj-lib's revset engine supports both string-based and programmatic queries.

**String-based** (what vc-x1 uses today via `common::resolve_revset`):
```rust
resolve_revset(&workspace, &repo, "::@")?  // ancestors of @
resolve_revset(&workspace, &repo, "all()")? // all visible commits
```

**Programmatic** (builder API, no parsing needed):
```rust
use jj_lib::revset::RevsetExpression;

RevsetExpression::all()                    // all visible + referenced
RevsetExpression::visible_heads()          // tips of visible branches
RevsetExpression::symbol("@".into())       // working copy
RevsetExpression::commits(vec![id])        // explicit set
RevsetExpression::bookmarks(pattern)       // bookmark refs
RevsetExpression::git_refs()               // all git refs

// Chainable traversal
expr.ancestors()           // ::x
expr.descendants()         // x::
expr.parents()             // direct parents
expr.children()            // direct children
expr.heads()               // maximal elements
expr.roots()               // minimal elements
expr.fork_point()          // merge base

// Ranges and set operations
expr1.range(&expr2)        // x..y
expr1.dag_range_to(&expr2) // x::y
expr1.union(&expr2)
expr1.intersection(&expr2)
expr1.minus(&expr2)
```

### Iteration modes

Evaluated revsets support three iteration modes:

```rust
let revset = expr.evaluate(repo)?;

revset.iter()               // yields CommitId in topo order (newest first)
revset.commit_change_ids()  // yields (CommitId, ChangeId) pairs
revset.iter_graph()         // yields GraphNode<CommitId> with edge info
```

`iter_graph()` provides `GraphEdge` data with edge types (`Direct`,
`Indirect`, `Missing`) — needed for rendering graph views like
`jj log --graph`.

### Ref/bookmark access via View

```rust
let view = repo.view();
view.heads()              // HashSet<CommitId> — visible branch tips
view.bookmarks()          // all bookmarks with local+remote state
view.local_bookmarks()    // local bookmarks only
view.git_refs()           // BTreeMap of git refs
view.git_head()           // git HEAD
```

### Index (direct DAG queries)

```rust
let index = repo.index();
index.is_ancestor(&id1, &id2)?           // ancestry check
index.common_ancestors(&[id1], &[id2])?  // merge base
index.heads(&mut candidates)?            // compute heads
index.all_heads_for_gc()?                // every head including hidden
```

### Low-level DAG walks (dag_walk module)

Generic traversal functions on arbitrary DAGs:

```rust
jj_lib::dag_walk::dfs(start, id_fn, neighbors_fn)
jj_lib::dag_walk::topo_order_forward(...)
jj_lib::dag_walk::topo_order_reverse(...)
jj_lib::dag_walk::heads(...)
jj_lib::dag_walk::closest_common_node(...)
```

### Evolution log (evolog)

The evolog shows every version a ChangeId went through — snapshots,
squashes, rebases, describes. It's tracked at the **operation level**,
not in the commit DAG.

Example output from `jj evolog -r @-`:
```
◆    rpyrwwnw wink@saville.com 2026-03-17 09:53:12 main f7f567f6
├─╮  Finalize: replace --foreground with --detach, ...
│ │  -- operation 655ff50a3386 squash commits into ...
│ ○  tluyvppo/0 wink@saville.com 2026-03-17 09:53:12 89b1c104 (hidden)
│ │  (no description set)
│ │  -- operation e71fa874e37a snapshot working copy
│ ○  tluyvppo/1 wink@saville.com 2026-03-17 09:49:54 317cefcf (hidden)
│    (empty) (no description set)
│    -- operation e1fe385617e5 commit ...
○  rpyrwwnw/1 wink@saville.com 2026-03-17 09:49:54 50c30496 (hidden)
│  Finalize: replace --foreground with --detach, ...
...
```

**Core API** (`jj_lib::evolution`):

```rust
use jj_lib::evolution::{walk_predecessors, CommitEvolutionEntry};

// Walk the rewrite history of a commit
let iter = walk_predecessors(repo, &[commit_id]);
for entry in iter {
    let entry: CommitEvolutionEntry = entry?;
    // entry.commit        — the commit at this point in history
    // entry.operation      — the operation that created this version
    // entry.predecessor_ids() — IDs of the previous version(s)
    // entry.predecessors()    — iterator over predecessor Commit objects
}

// Batch resolve predecessors across operation ranges
let map: BTreeMap<CommitId, Vec<CommitId>> =
    accumulate_predecessors(new_ops, old_ops)?;
```

### Evolog data is local-only

**Critical finding**: operation/evolution data does NOT survive a git
push + clone cycle. This is by design.

From `op_store.rs`:
> "Operations and views are not meant to be exchanged between repos or
> users; they represent local state and history."

| Data                  | Pushed? | Recoverable on clone? |
|-----------------------|---------|----------------------|
| Git commits + trees   | Yes     | Yes                  |
| Change IDs            | No      | Yes (from git header)|
| Commit predecessors   | No      | No                   |
| Operation history     | No      | No                   |
| Evolution log         | No      | No                   |

Predecessor tracking is stored per-operation in
`op_store::Operation::commit_predecessors` (since jj 0.30). The old
`backend::Commit::predecessors` field is deprecated and slated for
removal around jj 0.42.

### Implications for codebase evolution story

The evolog provides a rich local narrative of how changes were built
up — intermediate snapshots, false starts, squashes — that commit
messages alone don't capture. However, this data is ephemeral across
git transport.

For the dual-repo workflow (app + `.claude`), the `.claude` repo's
evolog captures how session commits were constructed. If `.claude`
eventually becomes a git submodule or is backed by a database, the
evolog data could be extracted and persisted via `walk_predecessors()`
before it's lost to a clone. This would preserve the full provenance
chain: not just what changed, but how the change was assembled.

## Interactive DAG navigation: the missing feature (2026-03-18)

### The problem

When working with jj-git repos, especially complex ones, you need to
navigate the commit DAG interactively — click a parent or child and
jump to it instantly. This is table-stakes in gitk but absent from
jj-native tooling.

### gitk: still the best for DAG traversal

gitk's killer feature: the parent/child list lets you **click-to-jump**
to any commit, even if it's far off-screen. True graph traversal, not
scrolling. Works in colocated jj repos via `jj git export`.

**Downsides:** Shows Git's view only — no change IDs, no revsets, no
jj bookmarks. Detached HEAD display is confusing. Cannot perform jj
operations. Does not work with non-colocated jj repos.

### gg-cli: close but not there

Renders a true graphical DAG. Can show parents in the detail pane and
click to select a parent — but **only if that parent is already visible
in the left panel**. If the parent is off-screen, there is no jump-to
behavior; you must manually scroll using the slider. This makes it
**useless on large/complex repos** for DAG traversal.

Drag-and-drop rebase is nice, but the lack of click-to-jump is a
dealbreaker for navigation.

### Other tools: list navigators, not graph navigators

- **jj-fzf** — displays `jj log` graph in fzf with live revset editing.
  Navigation is list-based (up/down), not graph-based (follow edges).
- **jjui** — revision tree view, growing fast, but still fundamentally
  a list navigator with a graph drawn alongside.
- **lazyjj** — operations-focused TUI, not DAG-focused.
- **jjdag** — pre-alpha, explicitly DAG-focused (Magit-style keys,
  foldable tree). Worth watching.
- **jj log** — revset language is powerful for querying (`children(x)`,
  `ancestors(main..@)`) but output is static, not interactive.

### Comparison

| Tool     | DAG display | Click-to-jump parent/child | Native jj |
|----------|-------------|---------------------------|-----------|
| gitk     | graphical   | **yes** (both directions)  | no        |
| gg-cli   | graphical   | parent only, no jump if off-screen | yes |
| jj-fzf   | ASCII       | no                         | yes       |
| jjui     | ASCII       | no                         | yes       |
| jj log   | ASCII       | n/a (not interactive)      | yes       |

### Conclusion

**gitk remains the best tool for interactive DAG traversal** despite
its jj blindspots. The jj ecosystem is missing a tool that combines
jj-native concepts (change IDs, revsets, bookmarks) with gitk-style
click-to-jump on parent/child links. This would be a strong feature
request for gg-cli, which already has the infrastructure but lacks the
jump-to behavior.

### Cautionary tale: multi-tool dangers

While testing jj-fzf and gg-cli simultaneously (with Zed editor also
open), it's easy to accidentally move `@` to an unexpected commit via
one tool without noticing. When Zed then sees files disappear from the
working copy, closing a "changed" file and declining to save effectively
deletes it. Recovery: `jj new main` to repoint `@`, or
`jj file show <path> -r <rev> --at-op <op>` to extract from history.

## jj-lib Commit API and tree storage model (2026-03-18)

### What's on the Commit type

Given a `Commit` (from `repo.store().get_commit(&commit_id)?`), the
public API includes:

| Method | Returns | Cost |
|--------|---------|------|
| `id()` | `&CommitId` | zero — stored in struct |
| `change_id()` | `&ChangeId` | zero |
| `description()` | `&str` | zero |
| `author()` / `committer()` | `&Signature` | zero |
| `parent_ids()` | `&[CommitId]` | zero — stored in commit data |
| `parents()` | `Vec<Commit>` | async, loads parent commits |
| `tree()` | `MergedTree` | near-zero — wraps tree IDs, no I/O |
| `tree_ids()` | `&Merge<TreeId>` | zero |
| `parent_tree(repo)` | `MergedTree` | async, merges parent trees |
| `is_empty(repo)` | `bool` | compares tree to parent tree |
| `has_conflict()` | `bool` | checks if tree IDs are conflicted |

### What's NOT on Commit

- **Children** — parent→child is a reverse lookup. Requires revsets:
  ```rust
  resolve_revset(&workspace, &repo, &format!("children({})", change_id))?
  ```
  Parents are stored in the commit; children require scanning the DAG.

- **Diff/changes** — no method returns "what changed in this commit."
  Must be computed by comparing two tree snapshots.

### Getting diffs between a commit and its parent

```rust
use jj_lib::matchers::EverythingMatcher;

let parent_tree = commit.parent_tree(repo).await?;
let commit_tree = commit.tree();

let mut diff_stream = parent_tree.diff_stream(&commit_tree, &EverythingMatcher);
while let Some(entry) = diff_stream.next().await {
    // entry.path  — RepoPathBuf (the file that changed)
    // entry.values — Diff { before, after } of MergedTreeValue
}
```

`parent_tree()` handles merge commits by merging multiple parent trees.

### What's stored on disk

jj uses git as its backend. On disk: git's content-addressable objects.

- **Blobs** — file contents, deduplicated by SHA
- **Trees** — directory listings (pointers to blobs and sub-trees)
- **Commits** — pointer to root tree + parent IDs + metadata

Each commit points to a **full tree snapshot**, but the tree is composed
of shared objects. If a file didn't change, both commits' trees point to
the same blob. Git also delta-compresses objects in packfiles.

The "full snapshot" is **logical, not physical**. `commit.tree()` just
wraps the root tree hash — no I/O happens until you traverse.

### Cost model for tree operations

| Operation | Cost | Why |
|-----------|------|-----|
| `commit.tree()` | near zero | wraps tree IDs, no I/O |
| `tree.path_value(&path)` | O(path depth) | walks ~4-5 tree nodes |
| `tree.diff_stream(&other)` | O(changed subtrees) | skips identical subtrees by hash |
| `tree.entries()` | O(entire repo) | walks everything — avoid on large repos |

The diff algorithm compares tree entries level by level. If a subtree
hash matches, it's skipped entirely — never descended into. On a Linux
monorepo commit that touches 5 files, it loads ~20-30 tree objects out
of millions.

The expensive part of a gitk-style patch view is reading **blob
contents** for the line-by-line diff, but only for changed files.

### jj-lib vs gitk efficiency

jj-lib is a separate Rust implementation reading the same `.git/objects/`
and packfiles — it does NOT shell out to git or link libgit2.

The tree diff uses the **same hash-based short-circuiting** as git:
`TreeDiffIterator` (in `merged_tree.rs`) checks `is_changed()` on
`MergedTreeValue` (which compares tree hashes) and only recurses into
directories that differ.

jj-lib also has a feature gitk lacks: **concurrent tree loading**. When
`store.concurrency() > 1`, it uses `TreeDiffStreamImpl` with async
parallel I/O for loading tree objects, potentially faster than gitk on
large repos.

Source: `jj-lib-0.39.0/src/merged_tree.rs`, lines 260-690.

### Key jj-lib source files (0.39.0)

- `src/revset.rs` — revset expression types, builders, parsing
- `src/evolution.rs` — evolog/predecessor traversal
- `src/graph.rs` — graph structures, TopoGroupedGraphIterator
- `src/dag_walk.rs` — generic DAG traversal (DFS, topo sort)
- `src/repo.rs` — Repo trait, ReadonlyRepo
- `src/view.rs` — View (heads, bookmarks, git refs)
- `src/index.rs` — Index trait (ancestry, common ancestors)
- `src/commit.rs` — Commit wrapper (parent_ids, change_id, description)
- `src/backend.rs` — backend data types (CommitId, ChangeId, Signature)
- `src/op_store.rs` — operation storage (commit_predecessors)
- `src/operation.rs` — Operation type (predecessors_for_commit)

## Show subcommand (0.18.0)

### 0.18.0 — Initial show subcommand

Implement `vc-x1 show` matching `jj show` output: commit/change IDs,
bookmarks (local + remote), author/committer with timestamps,
indented description, and file-level diff summary (Added/Modified/Removed).
Uses `TreeDiffIterator` for sync tree diffing via jj-lib.

### 0.18.1 — Flesh out show header

Show header now has gitk-style fields: Ids, Author, Committer, Parent,
Child, Branches (ancestry-based), Follows/Precedes (nearest tags),
Description (body only). Supports `..` notation for multi-commit display
with separators. File list: `-f` flag (default 50, `0`=none, `all`=unlimited)
with first/last split when truncated. Remove `--indent` from desc,
hardcode 4-space indent in both desc and show.

### 0.19.0 — Unify `..` notation and CLI across all subcommands

- Extract `collect_ids` in `common.rs` — returns `(Vec<CommitId>, usize)`,
  ordered commit IDs and anchor index. Handles all `..` directions.
- Extract `resolve_spec` — converts positional/flag args into `DotSpec`
  (desc_count/anc_count) ready for `collect_ids`. Single code path for
  all subcommands.
- Replace `DotDirection` enum with `desc_count`/`anc_count` on `DotSpec`.
- Remove `resolve_dot_args`, `ResolvedArgs`, `collect_both`/`collect_commit_lines`,
  `show_both` — all replaced by `resolve_spec` + `collect_ids`.
- All subcommands (chid, desc, list, show) now share one loop pattern.
- `-r` flag supports `..` notation (e.g. `-r ..mpl -n 3`).
- Rename `-l`/`--limit` to `-n`/`--commits` (`-l`/`--limit` kept as
  hidden aliases). Positional `COMMITS` replaces `POS_COUNT`.
- Count means total commits including anchor (e.g. `x.. 3` = x + 2
  ancestors). Fixes off-by-one in descendants and bare-rev-with-count.
- File list truncation: show first N files only. Skip truncation when
  only 1 file would be omitted.
- Raw jj revset syntax (`::`, `|`, `&`) no longer supported via `-r`;
  use `..` notation instead.
