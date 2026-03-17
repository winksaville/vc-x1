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
