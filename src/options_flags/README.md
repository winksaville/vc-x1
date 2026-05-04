# options_flags — Reusable CLI Options and Flags

Per-flag CLI surfaces shared across multiple subcommands. Each
shared option or flag (OF) lives in its own submodule so its
type, parser, and tests stay together.

## Architecture

Three composition patterns:

- **Leaf** — a `#[derive(Args)]` struct holding the flag(s),
  `value_parser`, default, and help text (via the field
  doc-comment). One `#[derive(Args)]` per logical OF — single
  flag or a small fixed pair (e.g. `push_retry`'s two fields).
  Help text aims to be generic enough for any reasonable
  consumer.
- **Bundle** — a `#[derive(Args)]` that flattens N leaves into a
  named role (e.g. `ProvisionCommon`). One `#[command(flatten)]`
  line at the consumer picks up the whole bundle.
- **Pattern A escape hatch** — when a consumer needs unique help
  text (or different defaults), it skips the leaf's flatten and
  inlines its own `#[arg(value_parser = …)]` field, reusing the
  leaf's typed value and parser.

## Adding a new leaf

1. Create `options_flags/<name>.rs` with a
   `#[derive(Args, Debug, Clone, Default)]` struct.
2. Add `pub mod <name>;` to `mod.rs`.
3. Add `impl super::FlagBundle for <YourFlag> {}`.
4. If the flag uses a custom value-parser, declare a unit-struct
   implementor of `FlagParser`.
5. Add tests for any non-trivial parser/resolver logic.

## Consuming an OF

Default — flatten the leaf into your subcommand's `Args`:

```rust
#[derive(Args)]
pub struct MyArgs {
    #[command(flatten)]
    pub config: ConfigFlag,
    // ...
}
```

Pattern A — when generic help doesn't fit:

```rust
#[derive(Args)]
pub struct MyArgs {
    /// My subcommand-specific help for --config.
    #[arg(long = "config", value_name = "none|PATH",
          verbatim_doc_comment)]
    pub config: Option<String>,
    // ...
}
```

A Pattern A consumer reuses the leaf's types (e.g. `ConfigKind`)
and parsers (e.g. `parse_config_kind`) but owns its own clap
attributes.

## Marker traits

- `FlagBundle: clap::Args` — every leaf and bundle implements
  it. Documentation-level marker; future generic helpers can
  constrain on it.
- `FlagParser` — leaves whose value-parser produces a typed
  value declare a unit-struct implementor with
  `parse(&str) -> Result<Self::Value, String>`. Matches clap's
  `value_parser` signature; consumers wire as
  `#[arg(value_parser = MyParser::parse)]`.

Both are documentation-level — clap derive doesn't see them;
the discipline is that every leaf declares them explicitly.

## Pattern A worked example

`init`'s `--config` field originally carried init-specific help
("Only valid with `--scope=por`..."). It currently flattens
`ConfigFlag` for the generic help; the `--scope=por` constraint
surfaces in `init`'s preflight error. If the help-text
generality becomes a usability problem, init will switch to
Pattern A as the worked example.

## Layout note

OFs currently sit as flat `<name>.rs` files alongside this
README. If an individual OF accumulates enough rationale,
edge-case detail, or examples to outgrow doc-comments, it will
graduate to a `<name>/mod.rs` + `<name>/README.md` subdirectory
layout. Mechanical when needed; not done preemptively.
