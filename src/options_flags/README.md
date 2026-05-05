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
  named role (e.g. `ProvisionOptionFlagBundle`). One
  `#[command(flatten)]` line at the consumer picks up the whole
  bundle.
- **Pattern A escape hatch** — when a consumer needs unique help
  text (or different defaults), it skips the leaf's flatten and
  inlines its own `#[arg(value_parser = …)]` field, reusing the
  leaf's typed value and parser.

## Flag vs Option — classify by domain, not wire syntax

Every leaf falls into one of two categories, picked by the value
domain it carries:

- **Flag** — boolean domain. Wire form may be presence/absence
  (`--dry-run`), explicit negation (`--no-dry-run`), or value
  (`--dry-run=true|false`). All three express a `bool`. Type
  name suffix: `*Flag`.
- **Option** — non-boolean domain (string, integer, enum, path,
  …). Wire form is always `--name=<value>` (or `--name <value>`).
  Type name suffix: `*Option` (or `*Options` for a multi-field
  leaf like `PushRetryOptions`).

Wire form is presentation; the category is the underlying domain.

## Adding a new leaf

1. Create `options_flags/<name>.rs` with a
   `#[derive(Args, Debug, Clone, Default)]` struct.
2. Pick the suffix per domain: `*Flag` (boolean) or `*Option(s)`
   (non-boolean).
3. Add `pub mod <name>;` to `mod.rs`.
4. If the leaf has explicit parsing logic, declare a unit-struct
   implementor of `FlagParser` (boolean domain) or `OptionParser`
   (non-boolean domain). Bare `Option<String>` leaves and
   presence/absence boolean leaves need no parser impl — clap
   handles them.
5. Add tests for any non-trivial parser/resolver logic.
6. Leaves do **not** implement a Bundle marker — bundles do.

## Consuming an OF

Default — flatten the leaf into your subcommand's `Args`:

```rust
#[derive(Args)]
pub struct MyArgs {
    #[command(flatten)]
    pub config: ConfigOption,
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

## Consumer function shape

Helper functions called by a subcommand body should accept the
relevant leaf type by reference rather than unpacking individual
fields at the call site:

```rust
// Multi-field leaf — pass the whole leaf
fn run_retry(cmd: &str, args: &[&str], cwd: &Path,
             retry: &PushRetryOptions) -> Result<…> { … }

run_retry("git", &["push", …], cwd, &args.provision.push_retry)?;
```

This wins on:
- Readability — `&args.provision.push_retry` reads as a single
  unit; the function body works with `retry.push_retries` (no
  leaf-name doubling because the parameter name is the consumer's
  choice).
- Argument count — every multi-field leaf collapses N args into
  one ref, so chained helpers don't accumulate
  `clippy::too_many_arguments` warnings.
- Future-proofing — extending a leaf with another field is a
  zero-touch change at every call site.

For **single-field leaves** (e.g. `DryRunFlag`, `PrivateFlag`),
direct read at the consumer site (`args.provision.dry_run.dry_run`)
is fine — wrapping a `bool` in `&LeafType` parameter doesn't earn
the indirection.

## Marker traits

Three Bundle markers — one per content classification. Bundles
implement exactly one. Leaves do **not** implement these markers
(their category lives in the type-name suffix).

- `FlagBundle: clap::Args` — pure-boolean bundle (every
  constituent leaf is a Flag). Rare in practice.
- `OptionBundle: clap::Args` — pure-non-boolean bundle (every
  constituent leaf is an Option).
- `OptionFlagBundle: clap::Args` — mixed bundle (constituents
  include both Flag and Option leaves). Most common in practice.

Two Parser traits — conditional contracts implemented only when
a leaf has explicit parsing logic (custom `value_parser`):

- `FlagParser` — boolean-domain parser. Implemented only for
  boolean leaves that take a value form (`--flag=true|false`).
  Presence/absence flags need no impl; clap parses directly.
- `OptionParser` — non-boolean-domain parser. Implemented only
  for leaves with custom parsing logic (e.g. `ScopeParser`,
  `RepoParser`); bare `Option<String>` leaves need no impl.

`FlagParser` and `OptionParser` share an identical method shape;
the difference is documentation (which value-domain the parser
targets).

All five traits are documentation-level — clap derive doesn't
see them; the discipline is that every leaf and bundle declares
the appropriate ones explicitly.

## Bundle marker discipline

Bundle markers aren't compiler-checked against contents. If a
bundle's leaf set shifts category — e.g. a pure-Flag bundle gains
an Option leaf — you must update the marker by hand
(`FlagBundle` → `OptionFlagBundle`). No lint, no failure; silent
drift if forgotten. Worth a glance at the marker line whenever a
bundle's `#[command(flatten)]` set changes.

## Pattern A worked example

`init`'s `--config` field originally carried init-specific help
("Only valid with `--scope=por`..."). It currently flattens
`ConfigOption` for the generic help; the `--scope=por` constraint
surfaces in `init`'s preflight error. If the help-text generality
becomes a usability problem, init will switch to Pattern A as the
worked example.

## Layout note

OFs currently sit as flat `<name>.rs` files alongside this
README. Bundle modules carry the `_bundle` suffix
(e.g. `provision_bundle.rs`) so the leaf/bundle split is visible
in the directory listing without opening files. If an individual
OF accumulates enough rationale, edge-case detail, or examples
to outgrow doc-comments, it will graduate to a `<name>/mod.rs` +
`<name>/README.md` subdirectory layout. Mechanical when needed;
not done preemptively.
