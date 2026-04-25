# `cargo install` and `Cargo.lock` — the `--locked` gotcha

Background reference for the rule in `CLAUDE.md` pre-commit
checklist step 4: `cargo install --path . --locked` is required;
plain `cargo install --path .` is unsafe.

## What we hit

On a clean worktree (after `rustup update` + `cargo clean`),
`cargo install --path .` failed inside `gix-object 0.58.0`:

```
error[E0308]: mismatched types
  --> gix-object-0.58.0/src/parse.rs:72:5
   |
71 | ) -> ModalResult<gix_actor::SignatureRef<'a>, E> {
72 |     gix_actor::signature::decode(i)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<…, ErrMode<E>>`,
   |                                       found    `Result<…, ErrMode<_>>`
note: there are multiple different versions of crate `winnow` in the dependency graph
   --> winnow-0.7.15/src/error.rs
   --> winnow-1.0.2/src/error.rs
```

`cargo build --release` and `cargo test --release` both passed
on the same tree.

## Root cause

`cargo install` ignores `Cargo.lock` by default and re-resolves
from scratch, picking the newest semver-compatible version of
every dep independently.

- The lockfile happens to pin one `winnow` that satisfies all
  the gix-* crates simultaneously.
- A fresh resolve picks two: `0.7.x` for one part of the gix
  graph, `1.0.x` for another. `gix-object` and `gix-actor` get
  compiled against different `winnow`s, so their `ErrMode<E>`
  types don't match even though they look identical.

The smoking gun in `cargo install` output is the
`Locking N packages to latest …` line — that's cargo
announcing it just did a fresh resolve.

## The asymmetry

| command | reads `Cargo.lock`? | re-resolves? |
| --- | --- | --- |
| `cargo build` / `cargo test` / `cargo run` | yes | only if `Cargo.toml` changed |
| `cargo build --locked` (etc.) | yes | never; errors if it would need to |
| `cargo install` (default) | **no** | always, from scratch |
| `cargo install --locked` | yes | never; errors if it would need to |

So build/test pass against the lockfile's dep graph, then
`cargo install` silently produces a binary built from a
different (and possibly broken) graph.

## Workaround

`cargo install --path . --locked`. Cargo has **no stable config
key or env var** to make `--locked` the default for
`cargo install`. Verified against the [cargo config
reference](https://doc.rust-lang.org/cargo/reference/config.html):
`[install]` only controls the install root directory; nothing
toggles lockfile behavior.

The discipline therefore lives in the project's pre-commit
checklist (CLAUDE.md step 4) and in shell aliases
(e.g. `alias ci='cargo install --locked'`).

## Upstream tracking — cargo issues

Canonical:

- [#7169 — "`cargo install` apparently ignores `Cargo.lock` as opposed to `cargo build`"](https://github.com/rust-lang/cargo/issues/7169) —
  the original report (2019). Exact phrasing of the surprise.
- [#9436 — "`cargo install` on local repo ignores `Cargo.lock`"](https://github.com/rust-lang/cargo/issues/9436) —
  narrower variant for the `--path .` case.

Adjacent edges:

- [#14308 — "`cargo install --locked --path .` isn't locking unlike `cargo build --locked`"](https://github.com/rust-lang/cargo/issues/14308) —
  recent; even with `--locked` there are edge cases.
- [#9289 — "`cargo install --locked` is not really locked"](https://github.com/rust-lang/cargo/issues/9289) —
  staleness gotcha; `--locked` doesn't enforce that the lockfile
  is up-to-date with `Cargo.toml` for install.
- [#16649 — contradictory documentation for `cargo install --locked`](https://github.com/rust-lang/cargo/issues/16649) —
  the docs themselves disagree on the behavior.
- [#9106 — warn when `--locked` is used but no `Cargo.lock` exists](https://github.com/rust-lang/cargo/issues/9106).
- [PR #14556 — "lockfile path implies `--locked` on cargo install"](https://github.com/rust-lang/cargo/pull/14556) —
  incremental fix in this area.

## Community discussion

- [internals.rust-lang.org — "`cargo install --locked` or not locked middle ground?"](https://internals.rust-lang.org/t/cargo-install-locked-or-not-locked-middle-ground/23893)
  (Jan 2026). The active design debate. Two camps:
  predictable-as-tested vs. up-to-date-with-security-fixes.
- [users.rust-lang.org — "Cargo install and lock file"](https://users.rust-lang.org/t/cargo-install-and-lock-file/4203) —
  older but the long-running version of the same complaint.
- [users.rust-lang.org — "`Cargo.{toml,lock}` equivalent for cargo install"](https://users.rust-lang.org/t/cargo-toml-lock-equivalent-for-cargo-install/108703) —
  others asking for exactly the config knob we wanted.

The bot thinks the cargo team is aware and conflicted (not
unaware), based on the internals thread being open and active
in 2026 while #7169 has been open since 2019 — flipping the
default has been weighed and rejected (or at least deferred)
multiple times.
