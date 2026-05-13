# Rust Idioms

Idioms used in this codebase that aren't obvious from a quick
read of standard-library docs. Add a new section when an idiom
shows up in code and you reach for an external explanation.

## `Option<Owned>` → `Option<&Borrowed>`: `as_deref()` vs `as_ref()`

**Why it looks ugly.** When a function takes the *unsized
borrowed form* of a type (`&Path`, `&str`, `&[T]`) and you have
an `Option<T>` of the *owned form* (`PathBuf`, `String`,
`Vec<T>`), converting requires two different methods depending
on the type, and at a call site that touches several such
fields the asymmetry shows:

```rust
let repos = common::resolve_repos(
    c.repo.as_deref(),  // Option<PathBuf>  → Option<&Path>
    c.scope.as_ref(),   // Option<Scope>    → Option<&Scope>
)?;
```

**The two methods.**

- `Option::as_ref()` — `Option<T>` → `Option<&T>`. Always
  available; hands back a reference to whatever's in the `Some`.
- `Option::as_deref()` — `Option<T>` → `Option<&<T as
  Deref>::Target>`. Available only when `T: Deref`. So
  `Option<PathBuf>::as_deref() → Option<&Path>`,
  `Option<String>::as_deref() → Option<&str>`,
  `Option<Vec<U>>::as_deref() → Option<&[U]>`.

**Why the asymmetry.** Rust APIs take the *unsized* borrowed
form (`&Path`, `&str`, `&[T]`) rather than the owned-borrowed
form (`&PathBuf`, `&String`, `&Vec<T>`) — the former is strictly
more general. A caller with a `PathBuf` can pass `&pb`; a caller
with a `&Path` can pass it directly; the `&PathBuf` form
wouldn't accept the latter. So functions tend to want `&Path`,
which puts the burden on the caller to deref through
`PathBuf`'s `Deref<Target = Path>` impl. Types without a useful
`Deref` (plain enums like `Scope`) skip that step — `as_ref()`
suffices.

**The fix when verbosity bites.** When `.as_deref()` /
`.as_ref()` show up at a call site, consider wrapping the
ceremony in a domain method on the owner type:

```rust
impl CommonArgs {
    pub fn resolve_repos(&self) -> Result<Vec<PathBuf>, …> {
        common::resolve_repos(self.repo.as_deref(),
                              self.scope.as_ref())
    }
}
```

…so callers read `c.resolve_repos()` and the ceremony lives in
one well-named place. The verbosity is often a hint that an
encapsulating method belongs there.

**Clippy.** The `option_as_ref_deref` lint flags
`opt.as_ref().map(|x| x.as_path())` and suggests the equivalent
`opt.as_deref()` — useful if you mentally reach for `as_ref`
first and miss the shortcut.
