# Versioning

How this project versions its commits and the running
artifact. The scheme — the cycle step-numbering and the
unique-per-commit aim — is generic and shared across projects:
this file is copied **verbatim**, with
[Recording the version-of-record](#recording-the-version-of-record)
covering each medium by conditional rather than per-project
edits.

Universal file, pinned to the template repository; do not edit
here. Project overrides go in [custom.md](../custom.md).

## Terms

Three names, used as defined here across
[AGENTS.md](../AGENTS.md),
[cycle-protocol.md](cycle-protocol.md), and the notes files:

- **version** — the per-commit version (e.g. `0.3.0-5.3.0`),
  written with `-` in prose, ladders, chores, todo, and commit
  talk; its suffix encodes the cycle phase (see
  [Step numbering](#step-numbering)).
- **version-of-record** — the authoritative stored copy of the
  version, in the project's manifest (see
  [Recording the version-of-record](#recording-the-version-of-record));
  a built or running artifact derives from it.
- **versioning** — the topic: this scheme as a whole.

## Grammar and storage

One prose spelling everywhere (ladders, chores, commit
bullets, conversation):

```
<public>[-<suffix>]
```

- `<public>`: `X.Y.Z`, integers.
- `<suffix>`: dot-separated identifiers, each ASCII letters
  or digits; usually integers (`3`, `3.1`, `3.1.0`), an
  alphanumeric id (`3.hotfix`) allowed, sparingly.
- **Exactly one `-` in the whole version**, the one that
  opens the suffix: never a dash inside the suffix, never a
  `+` in the prose form. This is the portability invariant
  that makes the version storable in every medium below.
- **`v` is a display prefix, not part of the version**:
  commit bullets and prose may write `v0.78.0-3.1` for
  scannability; manifests store the bare form. (PEP 440
  ignores a leading `v`; Cargo rejects one.)

Storage is a per-medium remap of that one spelling:

- **SemVer mediums** (Rust/Cargo): store verbatim; the
  suffix rides in the prerelease slot, valid at any dot
  depth.
- **PEP 440 mediums** (Python): remap the single `-` to `+`:
  `0.78.0-3.1` -> `0.78.0+3.1`. Mechanical and bijective
  because there is exactly one dash to find.
- **Other mediums**: verbatim if the format allows the one
  `-`; else the `+` remap; a new medium adds its case to
  [Recording the version-of-record](#recording-the-version-of-record).

Two reservations keep the remap sound:

- **The stored version identifies; it does not order.**
  SemVer sorts a suffixed version *before* its bare release
  (matching cycle semantics: rungs precede close-out);
  PEP 440 sorts the remapped form *after* it, and
  reinterprets a lone `-N` as a post-release. Opposite
  directions, so no cross-medium logic may compare stored
  versions; ordering truth lives in the ladder and git
  history. Comparing the public triple alone (e.g. a
  version gate) is unaffected.
- **`+` is reserved** for the PEP 440 remap: no SemVer
  build-metadata use in Rust repos even though Cargo allows
  it, since spending `+` there breaks the bijection with
  the Python spelling. A repo that truly needs it declares
  the deviation in its `custom.md`.

**Why one dash, dots only:** a Python linter/formatter in a
sibling repo rejected every multi-dash version outright, and
the PEP 440 reference parser (`packaging` 26.2, tested
2026-07-30) confirms the boundary: `1.2.3-3.1`,
`1.2.3-x+ab+cd`, `1.2.3-34-abc`, and any second `+` are all
invalid; `1.2.3+3.1` / `1.2.3+3.1.hotfix` parse at any
depth; `1.2.3-1` parses but silently becomes the
post-release `1.2.3.post1`. The one-dash dotted grammar is
the largest form every medium accepts.

## Recording the version-of-record

Where the version-of-record lives, how it's stored and
surfaced, and how often it changes — pick the case that fits
your medium:

- **Manifest** — where the version-of-record is stored:
  - if Rust, `Cargo.toml` `[package].version`
  - if Python, `pyproject.toml` `[project].version` (or the
    committed config it's sourced from)
  - otherwise wherever the medium records it (a generic
    `version.toml`, a book's frontmatter, …) — add the case as
    needed
- **Notation**: how the `-` form is stored; see
  [Grammar and storage](#grammar-and-storage):
  - verbatim where the format allows the one `-` (TOML
    `version.toml`, `Cargo.toml`)
  - the `+` remap where it bars it (PEP 440, e.g. a Python
    project): `0.3.0-5.3.0` -> `0.3.0+5.3.0`, same version,
    just the stored spelling
- **Reporter** — how a built artifact surfaces the
  version-of-record:
  - if a CLI app, `<cli-app> -V`
  - if a TUI/GUI, add to Help/About or display on the title
- **Cadence** — how often to bump: see
  [Unique per commit](#unique-per-commit-preference-not-requirement);
  this project follows the per-commit preference.

## Dev artifact name

When other projects consume the built artifact (e.g. the
installed CLI) while this repo is under active development,
the dev build installs under a separate name so a mid-cycle
install never clobbers the binary consumers are running:

- **Name** — the manifest's package name carries a `-dev`
  suffix (`vc-x1` → `vc-x1-dev`); if Rust, `[package].name`,
  so the per-commit flow's `cargo install` produces
  `<name>-dev` and leaves plain `<name>` untouched.
- **Constant, not per-step** — the step already lives in the
  version-of-record (`<name>-dev -V` reports the exact rung);
  a per-step name would churn the manifest every commit and
  litter the install dir with stale binaries.
- **Promotion** — plain `<name>` updates only by an explicit
  act: a separate clone built at the chosen commit with the
  plain name (or a copy of the dev binary), never by the
  per-commit flow's install.

This project adopted the convention 2026-07-23 (during
0.75.0); projects without external consumers can skip it.

## Unique per commit (preference, not requirement)

Our general notion is that the version-of-record should change
on **every commit**, so a built or running artifact identifies
the exact commit it came from.

- This is a preference, **not** a hard requirement. A project
  following the cycle protocol may bump less often — once per
  cycle, only at release, and so on. Record the choice in
  [Recording the version-of-record](#recording-the-version-of-record)
  if it differs.
- It is achievable because the cycle's versions (below) are
  **pre-assignable**, unlike the git SHA, which a commit cannot
  contain (see the cycle protocol's
  [Commits backfill](cycle-protocol.md#commits-backfill)).

## Step numbering

The cycle (Preparation -> Work -> Close-out; see
[cycle-protocol.md](cycle-protocol.md)) encodes each commit's
phase in the version suffix — the **final identifier `0` marks
a Preparation**:

- `X.Y.Z-0` — Preparation
- `X.Y.Z-1`, `X.Y.Z-2`, … — Work commits
- `X.Y.Z` — Close-out (bare version, no suffix)

**Preparation is optional.** A lightweight cycle — no ladder,
no setup commit — skips `-0` and starts at `-1` (its first
Work commit). The same holds at every level: a sub-cycle
needing no Preparation omits its `.0` (see Nesting). One that
grows a Preparation later adds the `0` step without
renumbering siblings.

Disambiguation:

- `-10` — Work commit #10 (final identifier `10`), not a
  Preparation.
- `-1.0` — Preparation of the `-1` sub-cycle (final
  identifier `0`).

**Nesting.** Sub-cycles append another level, recursively:

- `X.Y.Z-3.0` — Preparation of the `-3` sub-cycle
- `X.Y.Z-3.1`, `X.Y.Z-3.2` — its Work
- `X.Y.Z-3` — its Close-out
- `X.Y.Z-3.1.0` — Preparation of the `-3.1` sub-sub-cycle

Bump the version-of-record at the start of each phase, so the
active phase is recorded — and, per the preference above,
every commit carries a distinct version.
