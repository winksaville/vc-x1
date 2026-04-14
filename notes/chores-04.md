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
