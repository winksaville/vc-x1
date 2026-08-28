# Todo Backlog

Lower-priority `## Todo` entries, the long tail. When an entry becomes a priority, move it (and
any refs it cites) into `TODO.md > ## Todo` at its place in the order.

Same shape as `TODO.md > ## Todo`, [Todo format](../agent-data/notes.md#todo-format).

## Todo

### `validate`: fail on an unfilled `[[N]]` on `main`

(wink, 2026-08-21) Backfill was missed at two consecutive openings because nothing checked it. The
Opening's step 1 now names the `rg` check, and the durable fix is a validate element that fails when
a chores as-built rung still reads `[[N]]` and `git log --grep` of its title finds the commit on
`main`. Then no one has to remember.

### vc-x1 push: per-repo bookmark names

Allow code-side and `.claude`-side bookmarks to differ. Currently `.claude` is locked to `main`
regardless of the app-side `<bookmark>`. Sibling generalization to the N:1 / `--squash` / `--merge`
work, and together they make push handle all close-out shapes with arbitrary bookmark layouts.

### Investigate `linkme` for subcommand registration

Distributed-slice registry: each subcommand registers itself at link time, and `main.rs` discovers
them via the slice rather than matching a `Commands` enum. Reduces per-subcommand touchpoints from 3
(mod decl + enum variant + match arm) to 1 (registration). Costs:
- loses compile-time exhaustiveness, since a missing registration becomes a runtime gap
- help-output ordering depends on link order unless sorted
- a macro-magic dependency

Revisit once the `0.50.0` trait sweep's per-arm cost has been felt under real "add a subcommand"
load. <https://github.com/dtolnay/linkme>

### Investigate `inventory` as `linkme` alternative

Same shape as `linkme`: a runtime-iterable registry populated by `inventory::submit!` per
subcommand.
Trade-offs mirror linkme's. Pick one if/when the trait sweep's match becomes the felt bottleneck.
<https://github.com/dtolnay/inventory>

### forks-multi-user + bot-data-formats follow-through

Design captured across two notes, with concrete work to land when a cycle picks it up. Major
pieces:
- a multi-line `ochid:` parser and emitter
- agent-side immutability enforcement
- URL-shaped ochid (per-user, cross-repo)
- the vendor-subdir layout (`.bot/<vendor>/<version>/<id>.<ext>`) plus the flat-to-vendor migration

(The `.claude/` -> `.bot/` rename piece moved into the refactor program's facade-owns-topology
stage, config knob included.) Each piece is its own future TODO when the design hardens.
[[2]],[[3]]

### `test_helpers::Fixture` migration + downstream callers

Plus rename `Fixture` -> `TestFixtureDual` and `FixturePor` -> `TestFixturePor` so call sites carry
the test-only signal that `#[cfg(test)] mod test_helpers` doesn't communicate. Was `0.41.1-7`. [[5]]

### `vc-x1 finalize --scope` flag

Replace `--repo` with the role vocabulary used elsewhere (`work|agent|work,agent`). Carry-over from
the 0.42.0 `--scope` sweep (was 0.42.0-5, deferred at -4.7 close-out). The paired `Single(_)`
dogfood item (0.42.0-7) is moot after `0.53.0`, where `Single(_)` was deleted. Design lives in
chores-07.
[[6]]

### Cross-file `chores-NN.md` ordering sanity pass

`chores-08.md` (the 0.41.1 cycle) landed on `main` via the `0.42.0-4.7` rebase. Check that section
ordering across `chores-06`/`-07`/`-08`/`-09` is chronologically coherent and normalize if not. Low
priority.

### vc-x1 push: rework the two bookmark parameters

`PushArgs` has `bookmark_pos` (positional `BOOKMARK`) + `bookmark` (`--bookmark` flag) for one
logical value, forcing an `or_else` in `From<&PushArgs>`. Collapse to a single positional with
`--bookmark` as a true clap alias, or drop one spelling. [[7]]

### vc-x1 push: `--recheck`, implement or remove

Parsed by `PushArgs`, never read, and mirrored into `PushParams` with `#[allow(dead_code)]`. Either
wire
the skip-preflight-on-resume behavior or drop the flag. [[8]]

### vc-x1 push: `--scope=work|bot|work,bot` flag

Was 0.42.0-4 (deferred when the cycle pivoted to icr rebase work, and the cycle closed at -4.7).
State machine becomes scope-aware: a single-side scope skips
`commit-claude`/bookmark-claude/`finalize-claude`.
[[9]],[[10]],[[11]],[[12]]

### vc-x1 clone: `--scope=work|bot|work,bot` flag

Parallel to `init --scope` for role selection. Topology (`--por` vs dual) is the separate `--por`
boolean. Was 0.42.0-6 (deferred at -4.7 close-out). [[10]],[[11]],[[12]]

### validate-desc / fix-desc: `--scope` flag

Same role vocabulary as elsewhere: `work` validates the work-repo's commits against agent, `agent`
reverses, `work,agent` does both (new default). [[10]],[[11]],[[12]]

### Unify `.vc-config.md` accessors onto Pattern B

(typed struct + `load_from(path)`, like new `config::UserConfig` and `push::resolve_state_layout`).
Replaces the map-typed helpers in `desc_helpers.rs` / `fix_desc.rs` / `validate_desc.rs` with a
typed `WorkspaceConfig` struct. ~50 LOC, mechanical. Candidate for 0.41.2. [[13]]

### Help layout: force over-under everywhere

Apply `next_line_help(true)` at the root (or via the existing `cli_with_banner` walker) so every
subcommand's `-h` / `--help` uses the same layout. Today clap auto-picks per-command based on the
widest flag spec, so `sync -h` is two-column but `init -h` is over-under, a visual inconsistency.

### Single-word `label: body` log prefixes, not "Step N"

(`bookmark`, `provision`, `colocate`, `cross-ref`, `symlink`, ...), and indent labels under
per-side `work:` / `agent:` headers in dual. Originally planned as 0.41.1-6.7, then deferred.

### "Stop saying workspace in user-facing surfaces" sweep

The `[workspace]` -> `[repos]` rename itself shipped in the 0.76.0 repo-registry cycle (legacy
schemas hard-reject with a fix-it, and `src/legacy_vc_config.rs` holds the compat surface until the
repo migration sweep). What remains of the original entry is the broader wording sweep: prose, help
text, and identifiers still say "workspace" for the dual-repo project root.

### Add `status` (alias `st`): `jj st` across both repos

Uses `--scope` from day one. This is natural home for the working-copy signal called out and it
needs to include remotes, like remotes/origin/main. [[14]].

### `init --dry-run`: bypass the `--repo-remote` preflight

(currently fires before the dry-run early-return, observed dogfooding 2026-04-24).

### vc-x1 push: `--squash` flag

Squashes WC into `@-` via `--ignore-immutable` and force-pushes. Needs a `--force-with-lease`
equivalent and a state-sanity preflight in place first. [[9]]

### vc-x1 push: `--message-file PATH` flag

Git-style commit message file (first line = title, blank, rest = body). Alternative to `--title` +
`--body`. [[15]]

### Mirror `--check` / `--no-check` onto `vc-x1 push`

(forwards through to the preflight `vc-x1 sync` invocation). 0.37.1 hard-codes `--check`, and the
default stays `--check`.

### sync: surface working-copy state in the up-to-date summary

(per-repo pending-files count or compact stat). Wording-only fix shipped in 0.37.1, and this is the
design plus implementation. [[14]]

### bm-track silent-when-clean refinement

Print on entry/exit only when state isn't fully tracked or when exit state differs from entry.
[[17]]

### "Oh shit" revert: post-success undo via `.vc-x1-ops/`

Idea-stage. Every repo-mutating command drops a pre-op snapshot, and `vc-x1 undo` restores both
repos.
[[9]]

### Source-code design ref sweep + AGENTS.md codification

Adopt the section-name plus `blob/main/...` URL pattern for source code refs to designs, and codify
it in AGENTS.md alongside the existing markdown ref conventions. Sweep targets: src/push.rs lines 4,
121, 645, 1219. [[18]]

### Richer bookmark enumeration

per-bookmark remote presence + tracking status [[19]]

### Per-line/per-thread runtime log points

(future, maybe) [[20]]

### Add Windows symlink support

via `std::os::windows::fs::symlink_dir` [[21]]

### Add "::" revision syntax for jj compatibility

### Add -p, --parents, -c, --children

so parent and child counts can be asymmetric

### Add integration tests in tests/ for subcommands

using temp jj repos (tempfile crate)

### Fix .claude repo history

dev0 through dev2 sessions squashed into wrong commit [[22]],[[23]]

### Add `vc-x1 setup` subcommand

completions install, .claude repo init, symlink setup [[24]]

### Add dynamic revision completion via `ArgValueCompleter`

(jj doesn't complete revsets either) [[25]],[[26]]

### Test-tempdir override resolution chain

Both `src/test_helpers::unique_base` and `tests/common/unique_base` currently use
`std::env::temp_dir()` (= `$TMPDIR`). Generalize to resolve in priority order: an explicit env var
(e.g. `VC_X1_TEST_TMPDIR`), then the local `.vc-config.md`, then the `std::env::temp_dir()`
fallback. The user-config tier the original chain named goes with **Drop the global config and the
account notion** (2026-08-28). Useful when a developer wants tests on a tmpfs / SSD /
project-local path without exporting `TMPDIR` globally. Open question: do we also expose a CLI
parameter (e.g. `vc-x1 --workspace-tmp ...`)? Test binaries can't easily accept arbitrary flags via
`cargo test --`, so env is the realistic surface for tests. For the `vc-x1` binary itself a flag is
feasible, but it is unclear that it adds value over the resolution chain.

### `vc-x1` version-string ref resolution

Today version strings (`0.58.0`, `0.58.0-3`) live in commit titles and `Cargo.toml` but aren't git
refs, so `git diff 0.58.0^1 0.58.0` fails with "ambiguous argument". Auto-tagging at close-out
clutters the tag namespace fast (one tag per cycle), so a resolver is cleaner:
- `vc-x1 sha <ref>` primitive: accepts version strings, standard git refs (pass-through), jj refs
  (chids, `@`) and outputs a SHA on stdout. Composable into `git diff $(vc-x1 sha X)^1 $(vc-x1 sha
  X)`.
- `vc-x1 diff` / `vc-x1 log` thin wrappers that accept the version-string vocabulary and dispatch to
  git.
- Cycle-relative aliases: `prev`, `cur`, `prev~1c` for "N cycles back along `--first-parent`".
- `--first-parent` awareness: version refs resolve to the merge commit (bare `X.Y.Z`), so `0.58.0^1`
  walks first-parent to `0.57.0` naturally.

Builds on existing `vc-x1 chid` (which resolves jj revsets to chids). Separate gap on the jj side:
no clean first-parent revset operator in jj 0.40, and the equivalent today is
`jj diff --from <fp-chid> --to <merge-chid>`.

### `vc-x1 push --squash`: symmetric squash on both repos

Automate Option F (manually exercised in the 0.59.0 close-out [[27]]): app-side squash + bot-side
description rewrite + force-push, atomically. Demoted from `## Todo` after 0.67.0: merge non-ff is
the routine close-out shape and a pre-publication squash needs no tooling, so this entry's domain
(squashing already-published cycle commits) is off the routine path.
- App side: squash cycle commits into one new commit, and capture the squashed chid.
- Bot side: rewrite the prior push commits' descriptions, replacing their per-commit `ochid:`
  trailers with one pointing at the squashed chid, and add a rewrite-note acknowledging the change
  (preserves historical truth for future readers).
- Force-push bot `main` (rewrites the published commit, and the chid is preserved via
  `jj describe`).
- Push app `main`. The new bot commit paired with this push receives `ochid: /<squashed-chid>` as
  normal, and K prior bot records plus the new one gives (K+1):1 bot->code (2:1 in the 0.59.0
  case).

### `clone`: single-repo fallback without a companion remote

Default dual clone errors mid-way when `<source>.claude` doesn't exist. `--por` is the workaround
(works, but you must know to pass it).
- After the code clone, probe the session URL with `git ls-remote <url> HEAD`. On failure report "no
  companion session repo, cloning code repo only" (GitHub reports missing and no-access
  identically), skip the agent clone and symlink, and say single repo in the done message.
- A clone failure after a successful probe stays a real error.
- Dry-run text notes steps 2/3 are skipped when no companion exists.
- Integration-testable offline via path-form sources (code bare repo present, no `.claude` bare
  repo).

### Sketch cross-repo ochid migration in the Cycle protocol

Remnant of the retired Ideas entry "Codify ochid invariant + bot-repo rules + squash gating +
cross-repo migration" (rest folded into the 0.72.0 merge close-out cycle's docs step): in a
multi-contributor flow, ochids change at every merge until the change reaches the canonical repo's
`main`, so document the migration story.

### Refactor stage: por -> dual conversion

Attach an agent companion plus `.vc-config.md` to an existing por workspace as a routine
subcommand, described in [the stage](refactor-20260716.md#stage-por--dual-conversion). Last program
stage on purpose, since it leans on facade-owns-topology and the in-process init pieces.

### `init` template ergonomics

Take-or-leave ideas from the template restructure (mailbox, 2026-07-31), triaged here 2026-08-02:
- `--use-template <repo-root>` auto-detecting `work/` / `work.claude/`, so the caller passes the
  template repo rather than its payload subdirectory
- a no-remote init mode for local testing
- an empty-dir agent template only needs a `.gitkeep`, since init writes hidden files itself

### Warn on the legacy `[workspace]` schema from any command

A repo on the old schema reads, logs, and diffs fine for weeks and discovers the problem only when
`push` hard-errors, the least convenient moment. Cheap fix: any command warns once on a legacy
config rather than only `config --validate` and `push`. iiac-perf finding (mailbox, 2026-08-07).

### Revset pass-through: stop translating the house dialect

The CLI still converts house-convention revsets before issuing jj commands. It should pass revsets
to jj verbatim, so one dialect exists and `jj help -k revsets` is the single authority (decided
2026-08-03). The docs side landed with the agent-files adoption, and this is the CLI side.

### OSC 8 hyperlinks in `config` TTY output

(wink, 2026-08-09). When stdout is a TTY, `vc-x1 config` renders each key's name as an OSC 8
hyperlink to its reference url, so the printed schema is clickable in supporting terminals, and
suppressed when piped, with an `ls`-style `--hyperlink=auto|always|never` override.
- the stored files keep plain urls: TOML forbids control characters in comments, and editors plus
  most modern terminals linkify plain urls already
- builds on the per-key reference field of the vc-config prototype, which landed with the "docs:
  freshen vc-config and config subcmd" cycle, so this is unblocked (2026-08-28)

### Reference defs: go file-relative, with anchors

(wink + agent, 2026-08-10). The house `[N]:` form `/notes/<file>.md` resolves by *convention*, not
by any markdown standard: no spec says what a leading `/` means, so each viewer picks. Wink's
measurements (2026-08-10, todo-backlog's `[2]` / `[3]`): GitHub resolves it against the repo root
(works), VS Code's preview against the workspace root (works, though preview "back" navigation is
weak, a VS Code limitation no path form fixes), and Zed treats it as a filesystem-absolute path
(dead). The agent's initial claim that GitHub 404s was wrong, eliminated by the same measurements.
- only file-relative paths (`<file>.md` from a sibling, `../notes/<file>.md` from elsewhere) resolve
  identically everywhere, since relative-to-the-containing-file is the one interpretation needing no
  convention
- so the direction is relative paths plus `#anchor` fragments where a def targets a section, at
  three costs: a notes.md convention change, a sweep of existing defs, and defs becoming
  location-dependent (the
  close-out move's transform list already rewrites relative links, and validate-anchors' cross-file
  check keeps the sweep honest)
- sweep inventory (2026-08-10): ~167 root-absolute defs across 9 files (done.md 122, TODO.md 7,
  refactor-20260716.md 5, design-cli/por-dual-parity-audit.md 3, chores-07 2, chores-06 / chores-09
  / design-cli/copying.md 1 each, todo-backlog.md 25), plus ~5 inline `](/...)` links
- wink trial-converted todo-backlog.md's 25 defs to the `./` form (2026-08-10): all target files
  verified present, links confirmed working on GitHub and in VS Code (we think Zed resolves the
  relative form too, untested). Reverted the same day so the records stay uniform with the pinned
  convention until this entry's sweep flips both together

### vc-config.md per-key worked examples

(2026-08-11). The per-key sections mostly restate each key's one-liner, and a reference link has to
reward the click. Cut from the "docs: freshen vc-config and config subcmd" ladder at its freeze: no
acceptance item needs it, since item 3 asks only that each generated link land on a real section,
which it does.
- a worked TOML example per section, as it would appear in an actual config file (for
  `bot-session.items`, a couple of named recipes)
- effects, not just meaning: what visibly changes when the key is set
- decide there: extend the anchor drift test to require a toml fence per key section, making "has an
  example" suite-enforced (quality itself stays a review judgment)
- the entry shrank when it moved: its `repo.category.<cat>` and `account.*` bullets described keys
  that **The vc-config program: finish the surface, then shrink it** deletes
- a conventions change, so its own small cycle per AGENTS.md's Changing the agent-files, sequenced
  after validate-anchors lands and grows the cross-file check, so the ~145-edit sweep is
  machine-verified rather than hand-checked

# References

[2]: /notes/forks-multi-user.md
[3]: /notes/bot-data-formats.md
[5]: /notes/chores/chores-08.md#cycle-structure--multi-step
[6]: /notes/chores/chores-07.md#--scope-enum-refactor-0420
[7]: /notes/chores/chores-09.md#push-dual-bookmark-parameters
[8]: /notes/chores/chores-09.md#push-unimplemented-recheck-flag
[9]: /notes/chores/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[10]: /notes/chores/chores-06.md#generalize---scope-to-all-commands-design
[11]: /notes/chores/chores-06.md#--scope-continuation-0410
[12]: /notes/chores/chores-08.md#init--clone-redesign-0411
[13]: /notes/chores/chores-08.md#user-config-0411-3
[14]: /notes/chores/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[15]: /notes/chores/chores-05.md#capture---message-file-design-for-push-0375
[17]: /notes/chores/chores-06.md#bm-track-silent-when-clean-design
[18]: /notes/chores/chores-06.md#source-code-design-ref-convention-design
[19]: /notes/chores/chores-05.md#open-questions--tbd
[20]: /notes/chores/chores-03.md#per-lineper-thread-runtime-log-points-future
[21]: /notes/chores/chores-03.md#windows-symlink-support
[22]: /notes/chores/chores-01.md#refactor-and-add-desc-subcommand
[23]: /notes/chores/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[24]: /notes/chores/chores-02.md#0260--shell-completion-via-clap_complete-env
[25]: /notes/chores/chores-02.md#testing-results
[26]: /notes/chores/chores-02.md#shell-completion-discovery
[27]: /notes/chores/chores-12.md#docs-extract-cycle-protocol-0590
