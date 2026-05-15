# por/dual parity + bidirectional conversion

Forward-looking design capture (pre-design). Concrete work
lands when a cycle picks it up — likely on top of the
`--scope` rollout (the `0.50.0` work in `notes/todo.md`).

`vc-x1` today treats the **dual** workspace (a code repo
plus a `.claude` bot-session repo, cross-linked by `ochid:`
trailers, configured by `.vc-config.toml`) as the primary
shape; **por** (Plain Old Repo — a single repo, no
`.claude/`, no `.vc-config.toml`) is supported but bolted
on. Two goals:

## 1. Parity

por and dual should be first-class equals across every
subcommand — same code paths, same quality of support, no
"dual plus an afterthought" asymmetry. The `--scope`
rollout (`code,bot` vs `por` — see `notes/todo.md` and
`src/options_flags/scope.rs`'s `ScopeKind`) is the
substrate; parity is the goal that rollout serves.

## 2. Bidirectional conversion

A new capability: convert a workspace between the two
shapes in place.

- **`por → dual`** — attach a `.claude` companion repo and
  a `.vc-config.toml` to an existing single repo; emit
  cross-links going forward.
- **`dual → por`** — detach the `.claude` companion; drop
  `.vc-config.toml`; the code repo continues standalone.

## Open questions (resolve at design time)

- Surface: a new subcommand (`vc-x1 convert`?
  `promote` / `demote`?) vs. flags on existing commands.
- `dual → por` — what happens to the `.claude` history:
  archived, left in place untouched, deleted? Leading
  guess is "left alone, just unlinked," but TBD.
- `.vc-config.toml` — creation on `por → dual` (which
  fields, defaults from user config?) and removal on
  `dual → por`.
- Whether `init` / `clone` change, or conversion is purely
  a separate operation on an already-existing workspace.
- Idempotency / re-running; recovery from a conversion
  interrupted mid-way.

## See also

- `notes/todo.md` — the por/dual item, and the `--scope`
  cluster it builds on.
- `src/options_flags/scope.rs` — `ScopeKind` (`CodeBot` /
  `Por`) and `parse_scope_kind`.
- `notes/chores/chores-07.md > ## init + clone redesign (0.41.1)`
  — the `init_one` / `clone_one` primitives that aim to
  give a single chokepoint for scope-shaped changes.
