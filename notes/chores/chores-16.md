# Chores-16

Continuation of `chores-15.md` (closed after `0.78.0`, the jj-lib migration close-out). This
file covers `0.78.1` onward, back on `main` with the refactor program's long-lived bookmark
retired and the 20260803 baseline pin set in force. (`0.78.1` fell in the seam between the two
files and was originally recorded only in its TODO.md ladder rung and commit body; its section
below was added 2026-08-06, when the rung's narrative moved here.)

Reference numbering is file-local; see
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering); chores-16 starts at
`[1]`.

## Table of Contents

- [docs: adopt the 20260803 baseline pin set](#docs-adopt-the-20260803-baseline-pin-set)
- [style: typeable punctuation + line-width source sweep](#style-typeable-punctuation--line-width-source-sweep)
- [refactor: drop sync state and remove revert](#refactor-drop-sync-state-and-remove-revert)

## docs: adopt the 20260803 baseline pin set

- [[2]] 0.78.1 docs: adopt the 20260803 baseline pin set

The instruction-set dedup, dogfooded ahead of iiac-perf's review of the 20260803 baseline; an
interlude between the refactor program's `0.78.0` and the `0.78.2` sweep. Its narrative
originally lived only in its TODO.md ladder rung, a seam decision this section reverses
(2026-08-06, at wink's ask): a ladder rung is an index entry, and when the index entry is the
only record, the narrative is invisible to anyone who looks where narratives live.

- the pin set: cycle-protocol.md + versioning.md pinned into `agent-data/` (moved from
  `notes/`, pinned-file headers added); jj-tips.md + draft-reviews.md deleted with their live
  content salvaged first (jj.md gains the Revsets primer and the long-lived-bookmark
  discipline, cycle-protocol.md the no-preflight-while-a-review-iterates rule); cycle.md
  renamed cycle-checklists.md (scope-collision fix at wink's review); every live link
  re-pointed
- README's "jj Tips for Git Users" became a signpost (jj.md quick reference, upstream
  tutorials, template-hosted jj-tips.md) instead of a drifting copy; it still said `obslog`,
  renamed `evolog` upstream, which is what a drifting copy does
- the dogfood log rehomed from custom.md to notes/dogfood.md: it is a record, and
  records-only placement puts it under `notes/`
- custom.md reset to the bare template skeleton in the same commit, no history lost (the log
  moved to notes/dogfood.md, the bookmark discipline pinned into jj.md, the rest was committed
  at `0.78.0`). The generic-custom.md test starts here: a project should not assume the
  template's location, name, or contents, so the project layer stays skeleton-identical and
  what breaks becomes dogfood findings in notes/dogfood.md

## style: typeable punctuation + line-width source sweep

- [[1]] 0.78.2 style: typeable punctuation + line-width source sweep

`src/` + `tests/` predate the typeable-punctuation rule and code.md's <=100-col comment wrap:
863 banned-character sites across 63 files and 60 overlong lines at pickup. A single-commit
cycle (wink's call at the opening review: the sweep is one homogeneous transformation, so a
ladder would checkpoint nothing a review gate and jj's op log don't already cover).

- the four named characters went to zero: `->` and `...` by mechanical replace, the 708 em
  dashes each given their structural decision (colon for term-elaboration, comma for
  continuation, parentheses for asides, sentence splits for contrasts), converted by four
  parallel lesser-model agents on disjoint file batches and audited after
- the sweep surfaced four *more* untypeable species the ban list does not name: `=>` was
  written `⇒` (13 sites), the config/show output drew rules with `─` (56), plus one `≥` and
  one U+2212 minus; all converted, output tests updated in the same pass (see the dogfood
  finding on the enumeration)
- the ellipsis in `truncate_chars` was load-bearing: appending one-char `…` kept truncated
  gists at cap+1, and the mechanical `...` conversion broke the length test; the function now
  reserves room for the suffix and stays within `max` (contract tightened, test to `<= CAP`)
- `config_cmd` section headers (`# ── x ──`) and `show`'s separator rule are user-visible
  output; they render as `-` runs now, and the README's config samples were regenerated from
  the installed binary so the docs match what the code emits
- line width: string literals split with backslash continuation (content byte-identical),
  comments re-wrapped, trailing `// OK:` comments moved above their statement where wrapping
  was awkward; exempt as literal rows: the JSONL fixture strings in the transcript /
  bot-session tests, and comment URLs

Riding in the same commit: the cycle records, the Done-retirement sweep of four
refactor-program entries into done.md, the `0.78.1` ladder-ref backfill, and the first
generic-custom.md-test findings logged in [notes/dogfood.md](../dogfood.md).

### Delegation notes

Four Sonnet agents converted the em-dash batches and a fifth did the line-width pass; two
incidents worth remembering when delegating sweeps:

- one batch agent "corrected" scope creep it thought it saw, reverting 29 already-converted
  arrows back to `→` because it compared against a pre-sweep baseline; caught by the
  post-batch character audit. A delegated sweep needs the invariant stated as "the end state
  is X", not "convert only Y": agents infer baselines
- the line-width agent first placed the continuation break-space after the backslash, which
  Rust strips with the leading whitespace, silently corrupting a fixture string; its own
  `cargo test` run caught it. String splits verify by test, not by eye

## refactor: drop sync state and remove revert

- [[N]] 0.78.3 refactor: drop sync state and remove revert

`sync-state.toml` was vc-x1's last cross-invocation state file, and this cycle removes the
species, not just the instance. The trigger was bugs.md #8 (2026-08-06): iiac-perf's `vc-x1`
0.71.0 push adopted a stale `push-state.toml`, skipped every stage but the last, and squashed a
new session into the previous cycle's published bot commit. Push's state machinery was already
deleted at 0.77.0-3; the same triage discussion concluded sync's snapshot file carries the same
disease and the backlog already knew it: the `## Todo` entry "Remove `revert`, and `.vc-x1/`
with it" had reached the identical verdict ("we are not in control enough to do this
reliably"). Decided by wink at the triage discussion, branched off `main` as a patch cycle
while `0.79.0` runs on its own bookmark.

- sync still snapshots each repo's pre-sync op id, in memory only: the snapshot exists to be
  printed by its own invocation's failure report, whose per-repo undo line is now the explicit
  `jj op restore <op> -R <repo>`
- `revert` is removed outright (first built as a disabled, explaining stub; wink asked for the
  keep-it argument at review and the argument lost: the teaching already lives at the only
  moment anyone reaches for revert, sync's failure report, and a CLI name is not a compat
  surface, so reintroduction is free if a safer design ever earns it)
- `init` stops writing `/.vc-x1` to new workspaces' `.gitignore` (both templates). Existing
  workspaces keep theirs: a report-not-rewrite check is deferred to the `## Todo` entry
  "Stale `/.vc-x1` gitignore line: report it, and a safer revert, if ever"
- the deleted `sync/state.rs` module comment was also the last place claiming push keeps a
  `push-state.toml`, a line bugs.md #8 had flagged as stale
- promoted to plain `vc-x1` at this cycle (wink, 2026-08-06), the first promotion since the
  dev-artifact-name convention: first as a binary copy, then through an interim dual-`[[bin]]`
  target, finally as the single-name scheme's ordinary `cargo install --path . --locked` from
  this close-out tree. The `--force` along the way transferred cargo's install-ownership
  record from the old stable, `vc-x1 v0.71.0` built from a separate plain-named checkout
  (`../vc-x1-main`), which is how promotion worked before this cycle. The installed 0.71.0
  was the actively dangerous default, carrying bugs #1, #4, and #8's stale-state machinery
- the commit-description convention flips with this close-out (wink, 2026-08-06): problem
  and solution statements, no version, no file list, adopted from iiac-perf via custom.md
  override, this cycle's own description the first dogfood. The trigger was watching the
  file-by-file body absorb five redrafts across the cycle's riders while restating a diff
  already reviewed at the ready-to-commit gate; iiac-perf's two rationales both carry vc-x1
  exhibits (the 0.78.0 body over-claim survived review as prose; the 0.72.0 version gap was
  accepted precisely because renumbering falsifies immutable text)

### The single-name convention

Settled across one review conversation (wink, 2026-08-06), through three designs, each
killed by its own sharp edge:

- **manifest-name-per-branch-location** ("on main say vc-x1, on a branch say dev") fails
  structurally: a commit is on a branch when authored and on main after landing,
  byte-identical, so no committed name can track location, and the rename-at-merge variant
  forces every landing to be a merge while still letting pre-push validation clobber stable
- **dual `[[bin]]` targets** (`vc-x1` + `vc-x1-dev` over one `src/main.rs`, install by
  `--bin`) shipped briefly inside this cycle and was discarded at review: the second name is
  an unguarded spare key to stable, since a bare `cargo install --path .` installs both, and
  the whole scheme rests on never forgetting a flag
- **single name** (adopted): the package name IS the installed binary's name. `vc-x1` on
  main; a per-line dev name on a branch, renamed at cycle open and back at close-out, next to
  the version bump those steps already make. No `[[bin]]` stanzas at all; the CLI tests'
  `CARGO_BIN_EXE_<package-name>` lookup tracks renames automatically; promotion is an
  ordinary install from a main-positioned tree; concurrent dev lines coexist because each
  line's manifest names its own binary

The guard is mechanical because the procedural record is bad: the version-first commit
bullet slipped twice past the checklist, the ladder's trunk-order invariant is "stated but
nothing enforces it", and 0.78.0's over-claim survived two reviewers. `build.rs` refuses
from compile-time facts to build a suffixed version, which marks a dev rung, under the
stable package name, reported through the `cargo::error` directive: one clean error line,
not a build-script panic dump (wink's readability catch, same day). It began as a `#[test]`
and moved to the build script at wink's catch:
`cargo install` never runs tests, so a test guards only those who follow the flow, while a
build script fails every cargo verb, install included, before any binary exists to clobber
stable. The mild direction (a dev name reaching main under a bare version) stays procedural
on purpose: enforcing it at build time would fail branch close-outs that rename at their
merge commit, and its harm is landing-time, not install-time (wink verified the full
quadrant matrix 2026-08-06; the dev-name + bare-version tree endangers nothing installed).
Each direction belongs to the site where its harm occurs, so the mild direction's
mechanical home, if it ever wants one, is push: unlike a build, push knows the target
bookmark, and `push main` under a non-stable manifest name could refuse or confirm.

**The banner is the invoked name, at runtime**: `argv[0]`'s basename + version, replacing the
compile-time `CARGO_PKG_NAME` constant (with `CARGO_BIN_NAME` as the empty-argv fallback), so
renames and copies self-report whatever they run as, verified with `vc-x1-dev-dropsync
0.78.3`. The smoke test's banner-leak assertions anchor on the version string, the banner's
one invocation-independent marker. Two warts surfaced dogfooding it, both fixed: the bare
invocation stacked the ambient banner over help's about line (ambient now skips when no
subcommand was given), and the manual `print_help` path fell back to the package name in its
usage line (`bin_name` now set from the invoked name).

This overrides versioning.md "Dev artifact name" in mechanism, not intent; dogfooded here
first, the template proposal follows.

### Why the file is unsound and the op log is not

The state file answers "where was the repo before the sync" and is blind to everything after
its write. `revert` ran `jj op restore` to it with no guard, so one commit after a failed
sync, "undo the sync" and "rewind to the snapshot" silently diverge and revert discards the
difference. No file-side fix closes that: presence-as-semaphore says nothing about intervening
operations.

jj's op log holds the same answer without the blindness: identify sync's operations, derive the
revert target as the parent of the most recent sync run's earliest op, and, decisively, detect
and refuse intervening non-sync operations. One honest prerequisite: only sync's fetch runs
through the in-process session today; its reposition and rebase steps still spawn the installed
jj (`jj new` / `jj rebase`, as do `op log` / `op restore` themselves), whose ops carry jj's
stock descriptions, so "identifiable sync operations" first means migrating those spawns, a
remainder 0.78.0's "ending jj and git subprocess spawning" claim reads as having finished but
did not (found 2026-08-06 at this cycle's review; tracked in the `## Todo` entry "Retire the
remaining jj spawns; make the build enforce it"). That derived-not-persisted design is the
bar any `revert` reintroduction must clear; until someone wants it, `jj op log` +
`jj op restore` is the documented recovery, shown by sync's failure report and README.

# References

[1]: https://github.com/winksaville/vc-x1/commit/a8b43a18999e "a8b43a18999ece30e7b807650ba45eb9b236ebdc"
[2]: https://github.com/winksaville/vc-x1/commit/b2a5171292c5 "b2a5171292c553d000d6ead88fc5f5e537bebb7c"
