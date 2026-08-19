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
- [test: Claude Code can complete a cycle](#test-claude-code-can-complete-a-cycle)
- [docs: adopt the merged agent-file set](#docs-adopt-the-merged-agent-file-set)
- [docs: fix three semicolons](#docs-fix-three-semicolons)
- [docs: consolidate line widths](#docs-consolidate-line-widths)
- [docs: freshen vc-config and config subcmd](#docs-freshen-vc-config-and-config-subcmd)
- [docs: drop the orphaned depth-note paragraph](#docs-drop-the-orphaned-depth-note-paragraph)
- [docs: retire the refactor program block](#docs-retire-the-refactor-program-block)

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

- [[3]] 0.78.3 refactor: drop sync state and remove revert

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

## test: Claude Code can complete a cycle

- [[5]] 0.78.4 test: Claude Code can complete a cycle

`vc-x1 push` had been failing from sandboxed sessions, and the cause was never pinned down: the
failure always arrived at the end of a session, where the cheapest response was to hand the push
to wink and move on. The 2026-08-03 dogfood entry recorded the symptom and guessed at long-SSH
transfers. This cycle is the controlled experiment that settles it, run one step at a time so
each layer could be cleared separately, and the cycle is its own test: completing one end to end
from inside the sandbox is the result being demonstrated. It began as a throwaway on the
`cc-bm-and-push-test` bookmark and was promoted to a numbered cycle mid-run (wink, 2026-08-07)
once the finding proved worth keeping.

The cause, confirmed: both repos were originally cloned over ssh, and the sandbox denies ssh
twice over. Reads of `~/.ssh` are blocked except the commit-signing key and `known_hosts`, so no
auth key is available, and we think a host allowlist cannot admit a port-22 connection at all,
since ssh carries no SNI or Host header for a filter to match on. wink repointed both remotes at
https before the experiment, which is what made the push work.

- the network leg is a spawned `git`: `git_push_bookmark` hands
  `git_settings.to_subprocess_options()` to `jj_lib::git::push_refs`, so jj-lib runs the real
  `git` binary instead of doing the transfer in-process. That child inherits the sandbox, which
  is why a sandboxed run and wink's own terminal diverge on identical config
- credentials follow from that: git's own `store --file ~/.gitcreds` helper serves both repos,
  the sandbox permits reading it, and vc-x1 needs no credential callback of its own
- three competing hypotheses were killed by test rather than by argument. The bot repo is a real
  directory inside the project root, so it sits inside the write allowlist. The config paths the
  sandbox masks inside `.claude` are untracked, so no snapshot can record them as deleted. And
  jj's snapshot skips the character devices those masked paths become, without erroring
- the interactive-editor hypothesis died separately: `vc-x1 push` opens `$EDITOR` unless
  `--title`, `--body`, and `--yes` are all supplied, which would hang a non-interactive session,
  but wink confirmed the three-flag form works
- bugs.md #8's "sandboxed shell drops multi-MB pushes mid-transfer" is refined here rather than
  left standing: the size correlation was coincidence. Whether ssh fully explains that incident
  depends on iiac-perf's remotes having been ssh at the time, which this cycle did not check

One finding for the dogfood log: AGENTS.md places the bot repo at a symlink from
`~/.claude/projects/<path-to-project-root>/.claude`, which has one path component too many and
the direction reversed. `<project>/.claude` is the real directory, and the `projects` entry is
the symlink pointing in.

## docs: adopt the merged agent-file set

- [[4]] 0.78.5 docs: adopt the merged agent-file set

A single-commit cycle, the first run under the rules it adopts, so its six items were written
here directly at close-out (no `## In Progress` block existed to move).

### Problem

The family's two members carried diverging agent-files. iiac-perf's `agent-files-model`
bookmark proposed one owner per rule and one home per record; this repo held the 20260803
baseline, with corrections and content the proposal lacked; and the proposal itself carried
two behavioral regressions (its protocol still taught `jj commit` with a hand-written `ochid:`
trailer while declaring that file authoritative, and the per-commit version bump had fallen
out of every checklist). Neither set could be adopted byte-identical as it stood.

### Solution

The sets merge here as this repo's counter-proposal, per hard rule 12: the proposal's
substance (cycles on their own bookmark, the six-item record with one home, problem-then-
solution bodies, <=50-col titles, steps named not numbered, versions only in the
version-of-record, the custom.md stub + custom-family.md split, one-line CLAUDE.md) on this
repo's file layout (cycle-checklists.md keeps its name; cycle-protocol.md and versioning.md
stay pinned in `agent-data/`), with both regressions fixed, the baseline-only content
preserved (revset primer rewritten to jj semantics, Grammar-and-storage, Dev-artifact-name,
the authored/transcribed punctuation nuance), and two rules the review produced written into
prose.md and applied set-wide: a semicolon joins equals, and a pinned file names no project.

### Acceptance check

A mechanical audit of the final set, runnable by anyone:

1. every internal link and anchor resolves (slugger-validated walk of all ten files)
2. no authored banned punctuation (`grep '—\|–\|…\|→\|≤'`, specimens exempt)
3. every line <=100 cols, unwrappable links exempt
4. no member project, member history, or member version in pinned text
5. checklist and protocol agree step-for-step on the per-commit flow
6. `custom.md` matches the proposed stub with the single pointer line as its one entry

Ran at close-out, all pass: 1 clean (two hits are notes.md's literal `[text](url)` examples);
2 clean (prose.md's own specimens); 3 leaves three 101-103-col lines, each the unwrappable
version-of-record link; 4 clean after six references were degeneralized during the review;
5 verified, nine steps each, bump at 4, validate at 5, push at 9; 6 verified against
iiac-perf's stub modulo this review's semicolon sweep. The stronger check, a fresh session
reading only the new files and performing a cycle correctly, needs a next session and is the
dogfood log's watch item, not this cycle's claim.

### Deliberation

The review verdicts, argued in-session (the ochid-linked session holds the full exchange):

- **file set and names**: protocol and versioning are family-universal, so they stay pinned;
  `cycle-checklists.md` keeps its name, a reversal of an earlier lean, because beside
  `cycle-protocol.md` the short name `cycle.md` would read as the boss file while the
  protocol is what wins on disagreement
- **one-line CLAUDE.md**: initial pushback (custom.md, the conflict-winning layer, loses
  auto-load) withdrawn on wink's single-source argument: AGENTS.md-aware tools never read
  CLAUDE.md, so AGENTS.md must carry the truth anyway, and a duplicate import masks breakage
  for exactly one tool
- **50 vs 72**: titles <=50, settled by practice; both repos already wrote 50-col titles
- **semicolons**: wink's "item; detail" objection generalized into prose.md's `Semicolons`
  section; ~140 joins converted set-wide, ~50 survivors in three sanctioned classes
- **ladder placement**: notes.md wants the as-built ladder first under a chores header; the
  six-item shape lists it fourth. This section put the ladder first; logged as dogfood
  friction rather than resolved here

## docs: fix three semicolons

- [[6]] 0.78.6 docs: fix three semicolons

A single-commit cycle. Like
[docs: adopt the merged agent-file set](#docs-adopt-the-merged-agent-file-set) above, its six
items were written here directly at close-out, the opening and close-out collapsing into the
one commit.

### Problem

wink proposed forbidding semicolons in all prose, agent-files and code documentation included,
to shrink and simplify the rules. Review found the footprint already small (one prose.md
section plus three passing mentions) and every prose semicolon in AGENTS.md to be the
antithesis-between-equals form the rule protects. What stood after the argument was a narrower
preference: the family's top-level file should read without any.

### Solution

The three joins reword with no information change, and nothing else moves:

- the hard-rule contrast ("costs seconds / costs much more") takes ", while"
- the diff/history pair splits into two sentences, since its second half carries internal
  commas that a ", and" join would blur into
- the convergence pair ("the diff empties / the history keeps the record") takes ", and"
- code-span semicolons are shell syntax and stay
- prose.md's Semicolons rule is untouched, so the member's diff against the payload proposes
  only AGENTS.md wording

### Acceptance check

`grep -n ';' AGENTS.md` returns only lines whose semicolons sit inside backtick code spans,
and the reworded sentences carry the same claims as before. Ran at close-out: pass, the three
remaining hits being the shell specimens under "One command per shell invocation" and "Never
mask a command's exit status".

### Deliberation

- the outright ban was argued down on four points: the current rule already names and bans the
  lazy joins, the enforcement rationale that makes the typeable-punctuation rule absolute
  (untypeable, ungreppable) does not transfer to an ASCII character, a ban still needs the
  syntax-versus-prose and authored-versus-transcribed judgments so it cannot become a byte
  scan, and the whole footprint at stake is about fifteen lines of one file
- each rewording was checked for information change before adoption, which is how the
  diff/history pair became two sentences instead of taking the ", and" the other pair took
- prose.md's Semicolons section keeps quoting the old convergence pair as its specimen, now a
  quotation of this file's older wording rather than a live sentence, acceptable for a
  specimen and left to converge at the template
- the session the ochid links holds the full exchange

## docs: consolidate line widths

- [[7]] 0.78.7 docs: consolidate line widths

A single-commit cycle, its six items written here directly at close-out like the two sections
above.

### Problem

The line-width numbers were restated at eight sites across four pinned files, so changing one
meant hunting the rest. And the commit-body width (72, git's older log-indent convention) was
not the Linux kernel patch standard (75) wink understood the project to have adopted.

### Solution

The numbers move to one home, a new Line widths subsection in prose.md, and every other site
becomes a pointer to it:

- the subsection holds all four numbers (prose <=100, source <=100, titles <=50, bodies <=75)
  plus the wrap discipline (re-wrap when touched, write to the full width, long-line
  exceptions), which travels with the numbers
- commit bodies move from 72 to 75, dated in the subsection, with published bodies keeping the
  wrap they shipped with
- prose.md's own intro, its surfaces list, and its Conventional-commit-shape bullet defer to
  the subsection, as do cycle-checklists step 7, cycle-protocol's Title and Body sections, and
  code.md's Line width section, which keeps only its enforcement notes (`cargo fmt`, comment
  reflow)

### Acceptance check

`grep -rn '<=50\|<=72\|<=75\|<=100\|50-col\|72-col\|75-col' AGENTS.md custom.md
custom-family.md agent-data/` hits only prose.md's Line widths subsection, and each pointer's
anchor resolves. Ran at close-out: pass, four hits, all inside the subsection.

### Deliberation

- 72 vs 75: both are real standards, git's convention against the kernel's
  submitting-patches number. wink chose the kernel standard, and the switch is cheap because a
  published body is never re-wrapped anyway
- consolidation was weighed against checklist self-containedness: the checklists keep their
  instructions and lose only the number, a pointer being the price of copies that cannot
  drift. This cycle exists because the copies were already one drift into disagreement with
  what wink believed the standard was
- the session the ochid links holds the full exchange

## docs: freshen vc-config and config subcmd

### Problem

Both repos' `.vc-config.toml` carry a fossil `[push]` comment block from an older binary's
schema, the retired `.vc-x1` state dirs linger (empty dirs on disk, a `/.vc-x1` line in the
work `.gitignore`), generated config comments have no refresh tooling so they rot silently, and
the bot dir's `.claude` name is agent-specific where this workspace wants the neutral
`.agent-session`. Underneath those, the toml format itself caps the doc story: a `#` comment
block linkifies nothing, so a reader cannot click from a key to its documentation.

### Solution

Adopted the markdown carrier for the config surface (wink, 2026-08-11) and built its core.
The format: a config file is a markdown document whose `toml` fences, concatenated in
document order, form the TOML the loader parses, so prose, per-key doc, and real reference
links live beside the keys they document.

- landed: the md -> toml filter feeding the existing parser (non-fence lines blanked, so
  diagnostics keep the source's line numbers), the loader resolving `.vc-config.md` on either
  side with `.vc-config.toml` still accepted and both-present an error, `vc-config.md` as the
  prototype-and-doc absorbing `vc-config-design.md` with build.rs generating the schema from
  it, both instance configs on the md carrier, bot-session reading it, and the validate-desc
  bot-side fix
  - the carrier rule is per repo, not per workspace (wink, 2026-08-17): each repo carries
    exactly one of the two extensions, and a dual repo's sides may differ (one `.toml`, the
    other `.md`), so a member transitions one repo at a time. `.md` is the recommendation
- also landed, riding the cycle: the commit-body form pinned, the iiac-perf convergence
  proposals trialed then accepted (2026-08-16), the template baseline recorded, and the
  jj-lib 0.44 bump
- cut at the early close (wink, 2026-08-17): agent naming in config and CLI, regenerating
  the instance configs from the binary, `config --refresh`, `validate-anchors`, and the
  `.agent-session` repoint moved to `## Todo` as the vc-config follow-up cycle, so the
  0816-proposal agent-files work could start from a landed base

### Acceptance check

1. `md_to_toml` turns a config markdown file into the TOML the loader parses by
   concatenating its `toml` fences (line count preserved), with tests covering the model
   file's compact shape, the separated per-key shape, and an unclosed fence erroring.
2. The loader reads `.vc-config.md` on both sides, still loads a `.vc-config.toml`, and
   errors when a side holds both. Workspace detection finds a root by either name, and a
   test shows a workspace mixing carriers across sides (one repo `.toml`, the other `.md`).
3. build.rs generates the schema table and default constants from `vc-config.md`
   (rerun-if-changed wired), the hand-kept `COL_WIDTH` / `RESULT_LINE_CAP` constants are gone
   from `src/`, `vc-config-design.md` is gone, and each key's generated reference link lands
   on that key's `##` section in `vc-config.md`.
4. `vc-x1 config --validate` is clean on both sides, both instance files are `.vc-config.md`,
   and neither mentions `[push]`, `state-dir`, or `state-file`.
5. The surface names are `repos.agent` / `[agent-session]` / `agent-session` / `--scope=agent`
   with no old spelling accepted anywhere, and a config still carrying `repos.bot` or
   `[bot-session]` is rejected with a fix-it naming the replacement, the way a legacy
   `[workspace]` schema already is (a test shows the message).
6. `config --refresh` on a fixture with stale prose and user-edited fences preserves the
   fence interiors and `[repos]` while regenerating the prose (a test demonstrates it), and
   `--refresh --check` exits clean on both sides.
7. `validate-anchors` runs clean over `TODO.md`, `notes/`, `README.md`, and `vc-config.md`
   (same-file heading anchors and `[N]` refs), and a test shows it catching a broken anchor.
8. After the rename: `repos.agent = ".agent-session"`, a cycle rung pushes from a session end
   to end, new commits still stamp `/.claude/`-labeled ochid trailers, no `.vc-x1` dir in
   either repo, and no `/.vc-x1` line in `.gitignore`, which ignores the bot dir under its
   new name.

Close-out record (2026-08-17, run at the early close): items 1 and 2 hold, measured by the
carrier tests and `config --validate` passing on both sides. The mixed-carrier clause item 2
gained on 2026-08-17 holds too: `mixed_carriers_across_sides` landed with the closing commit
at wink's direction, one side `.toml` and the other `.md`. Item 3 holds: the schema
and defaults are OUT_DIR codegen from `vc-config.md` and `vc-config-design.md` is gone. Item 4
holds: validate clean on both sides, both instance files `.vc-config.md`, no `[push]` or
state-dir mentions in either. Items 5, 6, 7, and 8 describe the five deferred rungs and move
with them to the follow-up Todo entry, item 8's leftovers (`.claude/.vc-x1`, the work
`.gitignore` line) included. Full validation green at the close.

### Ladder

- [[28]] 0.78.8-0 [docs: freshen vc-config and config subcmd opening][8]
- [[29]] 0.78.8-1 [docs: separate work review stop][9]
- [[30]] 0.78.8-2 [feat: vc-config.toml prototype + build.rs codegen][10]
- [[31]] 0.78.8-3 [docs: ladder ToC + per-rung sections][11]
- [[32]] 0.78.8-4 [docs: amend cycle conventions][12]
- [[33]] 0.78.8-5 [feat: markdown config handler][13]
- [[34]] 0.78.8-6 [fix: prompt double echo][14]
- [[35]] 0.78.8-7 [feat: vc-config.md absorbs prototype and doc][15]
- [[36]] 0.78.8-8 [docs: pin the commit-body form][16]
- [[37]] 0.78.8-9 [fix: bot-session reads the md carrier][17]
- [[38]] 0.78.8-10 [docs: config-surface records, bold backlog titles][18]
- [[39]] 0.78.8-11 [fix: validate-desc from the bot side][19]
- [[40]] 0.78.8-12 [docs: trial the iiac-perf convergence proposals][20]
- [[41]] 0.78.8-13 [fix: bump jj-lib to 0.44][21]
- [[42]] 0.78.8-14 [chore: update vc-x1-template][22]
- [[43]] 0.78.8 [docs: freshen vc-config and config subcmd closing][23]

### Deliberation

- the md pivot (wink, 2026-08-11): a toml instance's `#` comment blocks cannot carry a
  clickable link, and every patch on that (a `localfile://` scheme, taught handlers) treated
  the symptom
  - a markdown carrier dissolves it: fences hold the TOML, prose holds the doc, reference
    links are real markdown, and the whole spec is one sentence: the `toml` fences,
    concatenated in document order, must form a valid config
  - pivoted mid-cycle at the cheapest point: the landed prototype + codegen rungs survive
    unchanged, and every not-yet-landed rung was about to render the format this replaces
    (the `--refresh` comment-block heuristic disappears outright: prose is the generator's,
    fence interiors are the user's)
  - one format is the end state; `.toml` stays loadable through the family's migration
    because the internal pipeline is md -> toml, making dual support nearly free
  - session experiments pinned the format rules: a `[table]` header captures every key after
    it (TOML has no terminator), so the model is compact fences per table
    (vc-config-test.md), the separated per-key form falls out of the spec unadvertised, and
    markdown tables stay presentation-only, since parsing them re-invents what TOML does free
- **an outside verdict on the carrier** (iiac-perf, 2026-08-12): a good fit, with the schema
  side the stronger half. The full reasons became citable when the template's preservation
  snapshot landed: `messages/vc-x1.md` at a225793bd179 in `../vc-x1-template`
- name allocation: `vc-config.md` is the prototype-and-doc (vc-config-design.md merges in and
  retires), `.vc-config.md` the instance, so the derived web url's filename never changes and
  backlog #52 distributes the file that is both doc and schema source
- agent vocabulary (wink, 2026-08-11): the machine surface was to flip within this cycle, riding the one
  config migration; the pinned-prose sweep ("bot repo" and kin) is its own later cycle, per
  "convention work runs as its own cycle"
- versioning (wink, 2026-08-11): no version is spoken for until it lands on main, correcting
  the earlier note here that reserved 0.79.0; this cycle stays a patch, 0.78.8 at close-out
- the `.agent-session` rename was leaning toward its own cycle; it folds in here because it
  needs an inter-session quiesce, which a multi-step cycle's `/exit` between rungs naturally
  provides
- the ochid prefix is a canonical side label decoupled from the bot dir's path (test-pinned: a
  custom bot dir still stamps `/.claude/`), so history and future trailers stay coherent across
  the rename
- pinned files name `.claude` as the bot repo's path; the rename step updates that text to
  path-neutral wording as a family proposal, this member's diff carrying it until convergence
- the close-out title dropped ".toml" from wink's phrasing so the opening bookend stays inside
  the title cap
- the single-name guard refuses a suffixed version under the stable name, so the opening
  renames the package to `vc-x1-dev` (versioning.md's Dev artifact name) and the close-out's
  bump to the bare version renames it back
- Done sweep at this opening: nothing migrated, the 0.78.2+ entries staying as nearby context
  after the 0.78.6 sweep
- the "docs: separate work review stop" rung was inserted at this opening's own review: the
  story is in [its subsection][9]
- the "chore: regenerate stale config files" rung reached its work review with the files
  regenerated, then was reverted uncommitted: the review judged regenerating before fixing the
  generator backwards, and the discussion that followed re-scoped the cycle around wink's
  prototype idea (the fossil `[push]` block rotted because the schema is hand-kept in code
  with no file-level source, so the fix moves the source into a file)
- `vc-config.toml` (unhidden, repo root) becomes the schema's single source: its structure
  mirrors the config one level richer (each settable key a table of metadata: doc, used-by,
  default, reference url), build.rs parses it and generates the schema table and default
  constants, and the behavioral defaults consume the generated constants so the code cannot
  disagree with the file
  - codegen lands in OUT_DIR rather than a tracked src file, avoiding dirty-tree-on-build
  - the per-key reference url answers the same review's finding that a schema entry
    (col-width 68) could not be traced to its docs
- the two pushed rungs' TODO snapshots carry the pre-pivot ladder. The reorder is recorded
  here rather than amended into them: the drafts rule's self-consistency yields because the
  pivot itself is a record worth keeping, and the snapshots show the plan the review changed
- `--refresh --check` came from wink's "run the generation and verify nothing changes"
  framing of the acceptance check: the prototype-to-binary leg cannot drift (every build
  re-derives), so the check guards the one leg that can, prototype to committed configs
- per-rung sections were adopted mid-cycle by the "docs: ladder ToC + per-rung sections"
  rung: [its subsection][11] holds the design
- ladder-to-section links were first declined over hand-computed anchors, which pulled
  validate-anchors into the cycle as a scope stretch. The links were later adopted ahead of
  the checker, table-routed ([12]), and the checker itself rides the follow-up Todo entry
- the "feat: per-key doc references" rung was inserted ahead of the regenerate: the why and
  the ordering are in [its subsection][13]
- the [carrier fix][17] was inserted at the absorb rung's review, found by pulling on wink's
  observation that `[repos]` belongs first because both sides use it while `bot-session` does
  not. The same pull exposed the `homes` correction, which rides the follow-up's regenerate
  rung
- **ladder freeze** (wink, 2026-08-11): 6 of the first 8 rungs were insertions rather than
  laddered work, so the cycle was expanding faster than it was landing. The remedy, at wink's
  call: the model rung folds into the regenerate rung and per-key examples leaves for the
  backlog, taking the remainder to six commits plus the closing, and every finding from here
  goes to the backlog or bugs.md by default
  - a rung is now added only when the acceptance check needs it or the cycle caused the
    defect. The carrier fix is the second case; validate-anchors stays because item 7 names
    it, though wink kept it on its merits rather than on the rule
  - the convention rule this rehearses ("convention work runs as its own cycle") covers
    convention itches and says nothing about *findings*, which is where four of the six
    insertions came from. Generalizing it is itself convention work, so it waits for its own
    cycle rather than being written here
- **commit-body form adopted and pinned** (wink, 2026-08-12), from iiac-perf's mailbox
  proposal ([their chores-06 section][24]). The form is [prose.md's][25] now, not restated
  here. The [pin rung][16] goes first so the rungs after it are written under a rule in force,
  which is the dogfood the family's review wants
  - **freeze lifted for this one rung** (wink, 2026-08-12), recorded per the hard rules'
    preamble. Backlog was tried first and failed its own test: a rule binding every remaining
    body cannot live in the file of things we might do
  - **prose.md holds the form, not cycle-protocol.md**: that file's [Body][26] already defers
    body content to prose.md, and the marker typing is prose mechanics
  - the mandatory intro retires bugs.md #7 as a body-shape concern; the bug stays, since a
    caller can still hand `--body` a hyphen-first string
- **acceptance item 5 was rewritten from aliases to rejection** (wink, 2026-08-12), before the
  rung it measures started, so this is a scope decision and not a check bent toward what got
  built. The old item asked that `repos.bot` / `[bot-session]` still load; the new one asks
  that they be refused with a fix-it. iiac-perf had argued for no aliases in the same mailbox
  message, so the family agrees; what changed wink's remaining doubt was the observation that
  alias baggage is carried forever unless someone is accountable for removing it. The
  reasoning, and the rejection mechanism that makes a hard rename affordable, are with the
  deferred rung in the follow-up Todo entry
- **the config-surface records rung was inserted at this ladder's second freeze lift** (wink,
  2026-08-12): iiac-perf's capability review needed verdicts, and a verdict with no durable
  home is a claim the mailbox deletes when the message is handled. Backlog was the default and
  failed the same test the commit-body rung's did, so the records land as their own commit and
  the reply cites them. The story is in [its subsection][18]
- the global config and `--account` leave vc-x1 entirely, as `## Todo` #1 rather than a rung:
  wink passes full urls in practice, so the user config's last job is a shorthand that the
  `owner/name` and path target forms already cover. Sequenced after this cycle on purpose,
  since `--refresh --check` makes the schema shrink mechanical
- the first three rungs advanced `main` as they pushed, against rule 13: work `main` moved
  back to 0.78.7 (73319b8c) so the cycle drafts on its bookmark until the trapezoid
  close-out, the bot `main` stayed at its tip, and the premature backfill reverted to
  `[[N]]` (2026-08-10). The rationale is in [the conventions rung's subsection][12]
- the "docs: amend cycle conventions" rung absorbs the cycle's convention work: intent
  subsections and the linked ladder, the cycle definition and bookmark discipline (after
  the main move-back), and the delegation doctrine (after the exceptions discussion),
  rolled into one commit at wink's call: [its subsection][12]
- **the template-baseline rung was inserted at wink's direction** (2026-08-16), a further
  freeze exception: the template's uncommitted state blocked the convergence work the
  accepted proposals unblock, and the parked-edits salvage was already open. Plan and scope
  in [the rung's subsection][22]
- **closed early** (wink, 2026-08-17): with the template baseline landed and the three
  proposals accepted, the family's next step is the 0816-proposal agent-file set, which wants
  a landed base rather than a second draft stacked on this one. The five config rungs moved
  to their own follow-up cycle in `## Todo`, which the 0816-proposal's machinery half (member
  facts to config, the `[validate]` table) also depends on

### Ladder details

#### docs: freshen vc-config and config subcmd opening

- Devise a mechanism for managing vc-config
-  
#### docs: separate work review stop

- the work-review stop ("please review", replacing "ready to commit") now carries no
  description, drafted or final: the description is written only once the work review
  completes, and the user's go is provisional since the review may restart
- sharpened in the per-commit checklist, the protocol's per-commit flow, and the
  bot-communication guidance, so the two reviews cannot collapse into one message
- inserted at the opening's own review, where the bot collapsed the two reviews into one
  message; an agent-file change is its own commit, which is why it is a rung rather than a
  rider on the opening

#### feat: vc-config.toml prototype + build.rs codegen

- the prototype is one TOML table per settable key (homes, kind, doc, used-by, default or
  example, required, optional reference override), key order being rendering order; a loud
  header separates it from `.vc-config.toml`
- long-form per-key docs live in `vc-config.md`, one `##` section per key path; references
  are derived (the `[vc-config] reference-base` repo url + `/blob/HEAD/` + the key's heading
  anchor, so links follow the default branch and no branch is baked in) rather than written
  per key, and a fork customizes base + file together
- build.rs parses it line-based (house style: build scripts stay dependency-free) and
  generates the schema table plus typed `<PATH>_DEFAULT` constants into OUT_DIR; a malformed
  prototype fails the build, and rerun-if-changed makes edits take effect on the next build
- `config_schema.rs` keeps the types and renderers, includes the generated table, and renders
  a new `reference:` line in every key block, so generated configs link to their docs
- bot-session's hand-kept `COL_WIDTH` / `RESULT_LINE_CAP` retired for the generated
  constants; the 68 rationale moved onto the prototype's col-width entry, whose doc now names
  the consuming views (--fields / --unknown / --per-line)
- drift guards: the clap help "[default: N; ...]" notes are tested against the generated
  defaults, parse_item_list(items default) must equal `ItemSet::BUILTIN`, every key needs an
  https reference, and every derived reference must anchor at a real vc-config.md heading (a
  key added to the prototype without docs fails the suite)
- deferred: the renderer still wraps comment blocks at 72; adopting the 100-col width belongs
  to the regenerate rung, where the rendered text is reviewed anyway

#### docs: ladder ToC + per-rung sections

- the ladder-as-ToC + `Ladder details` convention pinned across the pinned set: the
  protocol's Preparation (definition, timing, the program-depth note), per-commit flow step 3
  and the checklist's step 3 (the subsection is written at the flip), the close-out finalize
  bullet, prose.md's ladder-step surface and title identity, and notes.md's chores
  conventions (rung subsections are commit-recording, unlike free-named design subsections)
- wink's restructure named the area: a `Ladder details` container with rung subsections one
  level deeper, replacing the first draft's flat sections
- hard rule 9's "three places" stands: the subsection heading is conditional (no placeholder
  subsections), so prose.md names it a conditional fourth surface rather than raising the
  rule's count

#### docs: amend cycle conventions

- one commit for the cycle-convention amendments this cycle accumulated: wink rolled three
  docs rungs into this one, and the conventions-own-cycle rule below makes it the last of
  its kind
- rung subsections gain a second beat
  - opened at laddering with an abstract-sized intent statement (the rung's problem and
    solution, provisional like the rest of the block)
  - completed at landing with the conceptual delta, as today
  - the closing rung opens no intent stub (its problem and solution are the block's own
    Problem and Solution items); its subsection is created at close-out only when gotchas
    occurred, written in problem/solution form
- the working ladder adopts the as-built rung shape with links: `[[N]]` placeholder, linked
  title, marker
  - the `[[N]]` fills with slot and version once its commit lands on a permanent branch, so
    the close-out move only drops markers
  - each rung's title links to its subsection reference-style, `[<title>][M]` with
    `[M]: #<slug>` in the file's `# References`, the title string verbatim inside the
    brackets; the closing rung's link arrives with its gotchas subsection
  - table-routed rather than inline: the slug lives in the references table, keeping rung
    lines quiet, and a numbered tag survives title edits where a shortcut label would break
    silently
  - anchors are hand-computed until validate-anchors lands and guards them
- **cycle** gets an AGENTS.md Terminology entry: three stages, an opening, one or more
  work-repo changes, and a closing. A single-step cycle folds all three into one commit; a
  multi-step commits them individually, minimum two (a Work commit plus the close-out, the
  opening commit being optional), typically three or more
- multi-step bookend commits are the cycle title plus " opening" / " closing" (wink, at this
  rung's review), so the bare cycle title is the cycle's name: the chores header and Done
  entry carry it, no multi-step commit does, and the closing subsection's anchor no longer
  collides with the section header's. A single-step cycle's one commit keeps the bare title
- agreed text for rule 13 (wink's final simplification: the bot-repo exemption is carried by
  "in the work repo" and detailed in the linked checklist section): "A cycle runs on one
  topic bookmark in the work repo, named by the cycle title's slug, created at the opening,
  carrying every step. `main` advances only when the finished cycle lands on it, never by
  pushing commits straight to `main`. Once the bookmark lands on `main` the bookmark is
  deleted, locally and remotely."
- the hard-rules preamble gains the exceptions sentence: "The rules bind the bot, and none
  is absolute: any rule bends when wink says so explicitly at the moment, or in advance as
  an explicit scoped delegation (rule 10's stop-and-ask is the path), and a taken exception
  is recorded in the cycle's records. No rule bends silently, and no exception is
  self-granted."
- delegation doctrine, for cycle-protocol's Pushing policy: delegation waives stops (the
  synchronous review gates), never flow (records, validation, the bookmark discipline),
  since the records are what deferred review reads. Destructive ops pause in every tier,
  and landing is its own tier, delegated separately
  - the tiers: interactive (every stop), delegated cycle (rungs push without per-push asks,
    `main` untouched by construction, review at landing), delegated project (landing too,
    review after, corrections as new cycles)
- convention work runs as its own cycle: a mid-feature convention itch becomes a backlog
  entry or a small dedicated cycle, never another inserted rung. This cycle, five
  convention rungs deep with a deliberation that outgrew reading, is the grandfathered
  exhibit
- origin and folds: the intent-and-links half was inserted at the doc-references laddering
  from wink's empty placeholder sections, first as two rungs, folded; the cycle/bookmark
  and delegation halves were laddered after the main move-back and the exceptions
  discussion; wink then rolled all three into this one commit
  - the inline `(#<slug>)` link form lasted one review before wink's noise call routed the
    links through the `# References` table
- riders: rename this cycle's bookmark `config-refresh` to
  `docs-freshen-vc-config-and-config-subcmd`, and sweep the bookmarks `main` contains
- targets: AGENTS.md (rule 13, Terminology, the hard-rules preamble, Changing the
  agent-files), cycle-checklists.md (at-a-glance, bookmark section, shape wording,
  close-out), cycle-protocol.md (Preparation, Pushing policy), prose.md (ladder-step
  surface, fourth-surface note, cycle bookend titles), notes.md (as-built rung form,
  fragment defs, the Done-entry title), jj.md (cycle bookmarks)

#### feat: markdown config handler

Problem: a `.vc-config.toml` is edited by the user, and thus the user needs to be able to
thoroughly understand every aspect of it. But a .toml file is limited in its expressivity
as its documentation lives in `#` comment blocks and typical toml renderers do not
allow you to link to local sources of documentation.

Solution: change the config from a .toml file to a markdown file, which is much more
expressive. The prose and reference links live beside the keys, and the tables and key/value
pairs are defined in `toml` code blocks (see [vc-config-test.md](../../vc-config-test.md), the
model). This rung adds the markdown config handler and routes every config reader through
it:

- `md_to_toml` keeps `toml`-tagged fence interiors, blanks every other line (line count
  preserved, so diagnostics keep the source's line numbers), errors on an unclosed fence,
  and ignores untagged and other-tagged fences as the illustration idiom
- the loader resolves a side's config as `.vc-config.md` else `.vc-config.toml` for the
  family's migration window, erroring when both exist; workspace detection probes both names
- fixtures follow the model file's compact shape and the separated per-key shape, plus the
  mixing hazards the session experiments pinned (header-then-dotted nests silently,
  dotted-then-header errors loudly)
- landed: `config_md::load` is the one resolver/loader every topology reader goes through
  (the seven `common.rs` sites, `config --validate`, the schema-print hints)
  - `toml_simple` split into read and parse halves; the md dispatch keys on the `.md`
    extension in `config_md::load_file`, so a path-target `config --validate` accepts a
    markdown config too
  - a present-but-unloadable config (both carriers, a bad fence) now marks the workspace
    root instead of being walked past, so it surfaces as the resolvers' error rather than a
    silent degrade to POR
  - the schema drift test anchors into `vc-config-design.md` until the absorb rung restores
    the `vc-config.md` name its urls already carry
  - the both-present guard fired immediately: a draft `.vc-config.md` sat beside the live
    root config, parked in `tmp/draft-dot-vc-config.md`
  - `legacy_vc_config` untouched: its schemas are toml-only by definition
  - this workspace switched carriers at this rung (wink, 2026-08-11): both sides now hold a
    hand-written minimal `.vc-config.md` ([repos] plus doc links), the `.toml` instances are
    deleted, and the rest of the cycle dogfoods the handler
    - consequence: the stable `vc-x1` (no md support until promotion) can no longer resolve
      this workspace, so cycle operations run as `vc-x1-dev` from here to close-out
    - the regenerate rung's job narrows to rewriting these hand-written files from the
      generator

#### fix: prompt double echo

- intent: every interactive `[y/N]` line prints twice (wink's push transcript, 2026-08-11),
  because `common::prompt` writes the live prompt to stderr and then replays prompt+answer
  at info level, which also reaches the terminal via stdout
  - the replay exists for captured stdout (a transcript's only record of the answer) and
    the log file, so it cannot simply be dropped
  - fix: route by `stdout.is_terminal()`: a terminal gets the replay at debug (the log file
    still captures all levels), a captured stdout keeps the info replay
  - one helper, four call sites (push review x2, symlink replace, sync), so the fix lands
    everywhere at once
- landed as designed; the suite runs with stdout captured, so the terminal branch has no
  in-suite test and the check is wink's next interactive push showing the line once

#### feat: vc-config.md absorbs prototype and doc

- intent: three files describe the same keys today (the prototype, vc-config-design.md, the
  instance rendering); after this rung the prototype is `vc-config.md`, one file that is
  both the schema source and the doc every generated link lands on
  - per-key `##` sections carry the design doc's prose above each key's schema fence, and
    the section slugs are the anchors the derived web url already names, so the url template
    never changes
  - build.rs parses the prototype via the shared filter, rerun-if-changed re-pointed
  - `vc-config-design.md` retires at the merge; the <=100 wrap move rides here so the
    regenerate that follows writes final text
- landed as designed: `vc-config.md` is one file per key, prose then that key's `toml` fence,
  and `vc-config.toml` + `notes/vc-config-design.md` are both gone
  - the filter moved to `src/md_fence.rs`, std-only and naming no crate item, so build.rs
    declares it `#[path = "src/md_fence.rs"] mod md_fence` and the prototype and a
    `.vc-config.md` are read by one implementation rather than two that can drift
    - `include!` was the first attempt and fails: the file's `//!` header is not at a crate
      root when spliced mid-build.rs, so `#[path]` is what carries a module doc across
- the schema drift test now reads the prototype itself, which changes what it proves: no
  longer that two files agree, but that each key's fence sits under the heading its derived
  url names
- the file's three non-key `##` sections (how it is read, its own `[vc-config]` metadata, how
  the keys resolve) absorb what the prototype's header comment block and the design doc's
  intro each said separately
- the deferred 72 -> 100 comment wrap rode along, so the regenerate rung writes final text;
  no test pinned the old width, and the README's two schema samples were re-rendered from the
  binary rather than hand-rewrapped
- riders: the interim hand-written `.vc-config.md` files on both sides and `vc-config-test.md`
  had doc links into the retired design doc, repointed here so the deletion leaves nothing
  dangling; ARCHITECTURE.md's support list gains `config_md` (omitted when it landed) beside
  the new `md_fence`

#### docs: pin the commit-body form

- intent: the pinned files said a body was a problem statement then a solution statement and
  nothing about how a body with several sub-problems arranges them, so both repos improvised
  the same shape and neither could point at it
- landed: [Commit-body form][25] is the one home, with [Body][26] and cycle-checklists.md's
  step 7 linking it. A body's *structure* is now a rule where before only its ingredients were
  - **left unpinned**: whether a rung's `## In Progress` edits are a facet. Taken as cycle
    mechanics, on the logic that keeps the file list out; one instance is too few to pin
  - the family's copies stay unedited pending this repo's verdict, so the payload diff this
    rung creates is the reply

#### fix: bot-session reads the md carrier

- intent: `bot_session::workspace_bot_session` reads `root.join(".vc-config.toml")` through
  `toml_simple::toml_load`, so with both sides on the md carrier the file does not exist, the
  function returns all-`None`, and the workspace layer of `[bot-session]` settings is silently
  gone. A live regression from the handler rung's carrier switch
  - that rung's claim that every reader goes through `config_md::load` was scoped to the
    topology readers; this scalar reader was never converted and no test covered it
  - fix: route it through `config_md::load`, so a both-carriers or bad-fence config errors
    here as it does everywhere else rather than degrading to the user config in silence
  - a test setting a `[bot-session]` key in a `.vc-config.md` fixture and reading it back,
    since the silence is what made this invisible
- landed: the read splits into `bot_session_at(root)` and a cwd-anchored wrapper, matching
  `find_workspace_root_from`'s shape, and the core goes through `config_md::load`. Three
  tests: a `[bot-session]` block in a `.vc-config.md` arrives, a config without one is a plain
  miss, and both carriers on one side errors
  - **correction to the intent above**: the regression was latent, not live. `main`'s
    `.vc-config.toml` shipped the `[bot-session]` block with every key commented out, so no
    workspace value was actually being dropped. The dropping capability was real and a set key
    would have vanished, which is what the fix removes
  - so the ordering held for a better reason than the one given: the regenerate rung (now
    deferred to the follow-up Todo entry) re-emits these blocks, and a reader who uncommented
    one after that would have hit silence

#### docs: config-surface records, bold backlog titles

- intent: triaging iiac-perf's `.vc-config.md` review produced verdicts with nowhere to live.
  A mailbox reply's "Done when" cannot itself be the record, since handling the message deletes
  it, so every verdict needed a home before the reply could honestly claim one
- landed: the config-surface half is one bug and three entries. bugs.md gains
  **`config --validate` reports "I gave up" as a finding** (#9), where reading the code found
  `validate` breaking its own documented contract. `## Todo` gains
  **Tiered exit status for `config --validate`** and
  **`config --toml`: print the TOML a markdown carrier yields**, both ranked on wink's
  call; todo-backlog keeps **Config provenance names the schema, not just the binary**.
  The `toml`-tag escape is documented with the deferred regenerate rung in the follow-up
  Todo entry rather than spot-fixed
  - the notes-file half was not planned and came from the work itself: ranking two entries
    renumbered 17 and invalidated citations written minutes earlier, one already drafted into
    another member's mailbox. That produced
    **Cite a Todo or backlog entry by its bold title, not its number** (#56), its precondition
    swept here (32 backlog entries gained bold titles, none duplicated), and
    **What carries a Todo entry: numbered list, heading, or a tracker outside the repo?** (#57)
    for the structural question underneath, wink's, where the crux is that issues and a
    database both move records out of the repo and so change doctrine rather than format
  - **freeze lifted for this one rung** (wink, 2026-08-12), the same exception and the same
    reasoning as [the commit-body rung][16]: records for a reply that goes out now cannot wait
    on a cycle that has four rungs left
  - the bolding wrapped existing lead phrases rather than rewriting them, so four titles carry
    pre-existing em dashes and arrows that hard rule 8 forbids. Left in place: a punctuation
    sweep hiding inside a bolding pass is how an unrelated change gets missed at review

#### fix: validate-desc from the bot side

- intent: `validate-desc` from inside `.claude` dies "workspace incoherent: repos.work resolves
  to <work repo>, not to the workspace root itself" (wink, 2026-08-14, fix ASAP). Diagnosis:
  `validate-desc` and `fix-desc` hand their `-R` path (default `.`) to `bot_repo_path`, whose
  argument is by contract the workspace root, so from the bot repo the coherence preflight is
  fed the bot dir posing as the root and correctly refuses. Neither command was ever taught
  sides; the bug predates this cycle (reproduced on `main` 0.78.7 with a toml fixture)
- landed: `other_repo_path(repo)`, used by both commands, finds the workspace root *from* the
  given repo path, runs the same preflight against that true root, and returns the far side
  (bot repo from the work side, work repo from the bot side; the bot test runs first, via
  `starts_with`, because the bot dir nests inside the work repo). POR no-op and legacy
  rejection unchanged. Three unit tests, the from-bot-side case being the regression
  - the rung took a detour: it was first built as its own single-step cycle off `main`
    (bookmark `fix-validate-desc`, version 0.78.8, the branch to renumber forward), but the
    push died on a carrier skew this workspace cannot escape: a main-derived checkout restores
    the work side's `.vc-config.toml` while `.claude` already carries only `.vc-config.md`, a
    state no binary pushes (stable reads toml only, dev refuses mixed sides). So the workspace
    is push-locked for off-main cycles until this cycle lands, and the fix came home as a rung;
    the renumber cancels and the cycle keeps 0.78.8
  - the off-main commit briefly stayed as a local, never-published anchor for the interim
    stable `vc-x1 0.78.8` built from it (installed 2026-08-14, verified by wink on iiac-perf),
    then was judged noise and abandoned the same day, the rung's records carrying the
    provenance; the close-out build replaces the interim binary
  - the `0.78.7` backfill of the "docs: consolidate line widths" chores rung rides this rung's
    chores-16 edit, this being the workspace's first push since that close-out landed

#### docs: trial the iiac-perf convergence proposals

- intent: iiac-perf's convergence reply (2026-08-15 via `../vc-x1-messages`, Todo #1) verdicts
  our set as the base with their whole diff being three proposals: validate every commit, the
  flat semicolon rule with its agent-file sweep, and the always-linked closing rung. Rather
  than judge them on paper, adopt them for trial (wink, 2026-08-14): take their eight differing
  files verbatim, live under the rules for this cycle's remaining rungs, and let the review
  cycle's verdicts cite the experience. Rule 12 sanctions the edit (a local pinned copy may
  hold an unagreed experiment, the diff against the payload carrying it), and the trial rides
  the draft branch, unlanded meaning unadopted for the family even though this member runs it
- landed: the eight files taken verbatim after a hunk-by-hunk read attributing every change to
  one of the three proposals, with one exception kept ours: their closing-rung rewrite dropped
  two sentences no proposal claims (the area-moves-with-the-block sentence and the
  program-heading depth note), reinstated here semicolon-free and flagged as the review's
  first finding
  - an inserted convention rung in a feature cycle, which the pinned rules forbid, taken as an
    explicit exception on wink's instruction: the trial must precede the review cycle to
    inform it, and the remaining rungs are the test bed (validate-every-commit bites each
    rung, the semicolon rule all new prose, the always-linked closing rung the close-out)
  - the repo-wide semicolon state is untouched by design: the rule itself sweeps agent-files
    only and makes other files ask-on-alter, so no broader sweep rides the trial
  - the subsection link uses slot 55 because the parked pending edits already claim 54 for
    the Todo #1 entry, and the restore merge should not collide
  - the convergence goal, stated by wink at this rung's review (2026-08-14): the entire
    family carries identical agent-files including `custom*.md`, ideally with
    `custom-family.md` absent and `custom.md` the payload default. Member facts (name,
    template path, messaging parameters) move to `.vc-config.md` once the schema can carry
    them, medium and validation follow versioning.md's per-medium-conditional pattern, and
    the messaging practice pins against `vc-x1-messages`'s protocol. Goes to iiac-perf with
    the reply as the review cycle's frame

#### fix: bump jj-lib to 0.44

- intent: the installed jj moved to 0.44.0 (wink, 2026-08-17) and the version gate refuses to
  run against a mismatched jj-lib, so every rung's validation fails until the crate tracks it.
  Inserted ahead of "chore: update vc-x1-template" so that rung's pre-push validation can pass
- landed: jj-lib 0.43 -> 0.44 in Cargo.toml (gix stays 0.85, still the version jj-lib
  resolves, so the lock-contention downcast keeps type identity), plus the API renames the
  compiler surfaced: `default_working_copy_factories` and the populated `StoreFactories` moved
  to the new `jj_lib::default_backend_factories` module (`StoreFactories::default()` is now
  `default_backend_factories()`), `GitFetch::fetch` dropped its fifth argument, and
  `changed_remote_bookmarks` yields `GitImportRefUpdate` structs instead of tuples. All 549
  tests pass, the gate test against the installed jj 0.44.0 included
  - the subsection link uses slot 57 because the parked pending edits claim 54 and 56

#### chore: update vc-x1-template

- intent: give the payload's official home real history before convergence work builds on it,
  and record the 2026-08-16 decisions in this repo
- landed: the four-commit baseline in `../vc-x1-template` (plain `jj`, outside push/ochid
  machinery): a preservation snapshot, the retired messaging files removed, then both members'
  agent-files placed in `work/`, ours the agreed tip ("chore: place vc-x1 agent-files in
  work", d6aaaaf13092)
- this side records it: messaging moves to `../vc-x1-messages` (custom-family.md), the parked
  plan edits restore (agent naming flips from aliases to rejection), the heads-up to iiac-perf
  is drafted, and backlog #58 replans around the baseline
- baseline only: the 0816-proposal and the record close land their own payload commit later

#### docs: freshen vc-config and config subcmd closing

- gotcha: the acceptance check was written for the full ladder, and an early close leaves
  most items describing unrun work. Problem: recording them as passed would bend the check,
  and dropping them would lose the plan. Solution: the close-out record marks each item held,
  new-scope, or deferred, and the deferred items move with their rungs to the follow-up
  Todo entry
- gotcha: the system jj moved to 0.44 mid-cycle, so the stable `vc-x1` (jj-lib 0.43) refuses
  its own version gate on this machine. Problem: between the jj upgrade and this close-out,
  no stable binary runs. Solution: the close-out's validation installs `vc-x1` 0.78.8
  (jj-lib 0.44), the explicit promotion that replaces the interim 0.78.8 build. The gate is
  also what makes the 0.43/0.44 skew safe: jj-lib exposes no on-disk format version, so
  binaries on different jj-libs cannot be allowed to co-write, and the gate bricks the stale
  one instead
- gotcha: the promotion ran early, at a validation before the cycle's content was final.
  Problem: a stable install from a still-changing tree means one version string can name two
  behaviors, which is the exact thing the versioning scheme exists to prevent. Solution:
  the mixed-carrier test folded into this closing commit (wink, an explicit exception to
  the closing-is-bookkeeping-only rule), so exactly one commit carries bare 0.78.8 and the
  final validation's install is the one true promotion. The timing rule this teaches is a
  dogfood entry: the stable-name install is the cycle's last act, never earlier

## docs: drop the orphaned depth-note paragraph

### Problem

Our `agent-data/cycle-protocol.md` carried a paragraph after the closing-rung passage, a
restatement of the close-out move plus a program-depth note, and the convergence baseline
recorded it as the pinned set's one divergence: iiac-perf's copy lacks it, trimmed at the review
their [docs: always link the closing rung][27] Deliberation records. Their 2026-08-18 message
proposed deleting it family-wide, staged as the template branch
`iiac-perf-drop-depth-note-paragraph`.

### Solution

Accepted (wink, 2026-08-18). The template branch landed on template `main`, the same paragraph
came out of our pinned copy, and the reply record closes the exchange.

- verified before accepting: the paragraph's first sentence restates the move that
  [Chores sections](../../agent-data/cycle-protocol.md#chores-sections) owns, the depth shift is
  already that section's first transform ("whatever the title's depth demands under a program
  heading"), and the one unique detail, the heading-floor observation, retires as history in
  iiac-perf's Deliberation
- the neighbor paragraph already ends by pointing at Chores sections, so no reader loses the
  route to the owning section

### Acceptance check

`agent-data/cycle-protocol.md` is byte-identical across the member, the payload, and iiac-perf,
and the whole pinned set (`AGENTS.md`, `agent-data/*`) diffs empty across the three repos.

**Result: passed**, 2026-08-18, by `diff` of the file against both copies and `diff -r -q` of
`agent-data/` plus `AGENTS.md` against both, all clean.

### Ladder

- [[44]] 0.78.9 docs: drop the orphaned depth-note paragraph

### Deliberation

**Run as a single-commit cycle**, iiac-perf's own 0.25.1 model, since the change is one
paragraph's deletion with its records. The incoming record in `../vc-x1-messages/vc-x1.md` gains
its `outcome-*` fields pointing here, and the reply record in `iiac-perf.md` cites this section
(the messaging policy in [custom-family.md](../../custom-family.md#messaging)).

**The freshen cycle's backfill stays outstanding** (wink, 2026-08-18): its sixteen `[[N]]` rungs
take their versions and SHAs in a later chore rather than riding this push, keeping this commit
scoped to the deletion.

## docs: retire the refactor program block

### Problem

`TODO.md > ## In Progress` still held the jj facade refactor program block while no cycle of
it was running: the no-cycle marker sat above the block, reading as "section empty", the
ladder claimed to mirror `git log --first-parent` but stopped at `0.78.4` with five trunk
landings since unrecorded, and every rung but one was `(done)`. A finished program's record
kept in the working file drifts exactly the way the one-home rule ends, and two backfills
(the freshen cycle's sixteen rungs, the depth-note cycle's one) were still open.

### Solution

The program block retired into its own document, an as-built trunk ladder section in
[refactor-20260716.md](../refactor-20260716.md#as-built-trunk-ladder-program-retired-2026-08-18),
bounded at `0.78.4` rather than extended, and `## In Progress` now holds only the no-cycle
marker.

- the freshen and depth-note rungs backfilled with their versions and SHAs, retiring the
  open backfill chore
- the trapezoid-push `## Todo` entry absorbs the retired rung's merge-reconciliation note
  and the parked-branch state
- the Done sweep migrates the three pre-convention entries (the `0.78.2`..`0.78.4` cycles)
  to done.md

### Acceptance check

`## In Progress` holds nothing but the no-cycle marker, chores-16 has no unbackfilled
`[[N]]` except this cycle's own rung, every ref the moved as-built ladder cites resolves,
and the program ladder matches `git log --first-parent` rung for rung from 0.74.0 through
0.78.4 (the check as written at laddering said 0.73.0, revised below).

**Result: passed**, 2026-08-18. The marker check by reading `## In Progress`, the backfill
check by `grep '\[\[N\]\]'` (only prose mentions and this rung remain), the ref check by a
definition/citation scan over the touched files, and the ladder check by
`git log --first-parent` from 0.78.4: fifteen trunk commits match the fifteen rungs from
0.74.0 up. Below 0.74.0 the check found the seam the as-built section now records, two
pre-convention docs interludes on the trunk that were never rungs.

### Ladder

- [[45]] 0.78.10 docs: retire the refactor program block

### Deliberation

**Retire rather than extend** (wink, 2026-08-18): the ladder's trunk-mirror claim was
quietly false from `0.78.5` on, and extending it forever restores the dual maintenance the
one-home rule abolished. The completed work moves to where finished narratives live, and the
remaining stage stays a ranked `## Todo` entry that grows a fresh ladder at pickup.

**The record's home is the program's own document** (wink, 2026-08-18, at review): the
ladder spans `0.73.0` onward while chores-16 covers `0.78.1` on, so a chores-16 section put
the file's oldest work after its newest and read as a chronology break. refactor-20260716.md
already owns the program's plan and design, its intro promises each shipped stage a status
link, and the as-built ladder completes the document.

**The moved text's semicolons converted at the move** (prose.md's Semicolons rule makes the
alteration the moment to decide): the four in rung bullets became colon or period joins, no
information change. The quoted Todo title "Retire the remaining jj spawns; make the build
enforce it" keeps its semicolon, since a title is an identifier and the record only cites
it.

**Done sweep scope**: only the three unversioned pre-convention entries migrated, keeping
the `0.78.5`..`0.78.9` run in `TODO.md > ## Done` as nearby context for the convergence and
config work still ranked at the top of `## Todo`.

**The acceptance check's lower bound moved from 0.73.0 to 0.74.0** when running it found the
two pre-convention interludes on the trunk below `0.74.0`. The original bound restated the
retired block's own trunk-mirror claim, which was already approximate at that seam, so the
revision records what the trunk actually holds rather than relaxing what this cycle
promised. The finding is kept in both the check's result and the section's seam note.

# References

[1]: https://github.com/winksaville/vc-x1/commit/a8b43a18999e "a8b43a18999ece30e7b807650ba45eb9b236ebdc"
[2]: https://github.com/winksaville/vc-x1/commit/b2a5171292c5 "b2a5171292c553d000d6ead88fc5f5e537bebb7c"
[3]: https://github.com/winksaville/vc-x1/commit/b90f948defc6 "b90f948defc6be6dc7231ca1fde2eb293dc558ac"
[4]: https://github.com/winksaville/vc-x1/commit/198cc4b3150e "198cc4b3150ea4c7e2ae2ac9911ad5398ae40cce"
[5]: https://github.com/winksaville/vc-x1/commit/a478e124791c "a478e124791c3eda688c37747d103151acc5c70f"
[6]: https://github.com/winksaville/vc-x1/commit/d22c787658a1 "d22c787658a1e87a8da5e43edb23913a1215f5df"
[7]: https://github.com/winksaville/vc-x1/commit/73319b8c887c "73319b8c887c05f2ed6e4440d0817e217971dfda"
[8]: #docs-freshen-vc-config-and-config-subcmd-opening
[9]: #docs-separate-work-review-stop
[10]: #feat-vc-configtoml-prototype--buildrs-codegen
[11]: #docs-ladder-toc--per-rung-sections
[12]: #docs-amend-cycle-conventions
[13]: #feat-markdown-config-handler
[14]: #fix-prompt-double-echo
[15]: #feat-vc-configmd-absorbs-prototype-and-doc
[16]: #docs-pin-the-commit-body-form
[17]: #fix-bot-session-reads-the-md-carrier
[18]: #docs-config-surface-records-bold-backlog-titles
[19]: #fix-validate-desc-from-the-bot-side
[20]: #docs-trial-the-iiac-perf-convergence-proposals
[21]: #fix-bump-jj-lib-to-044
[22]: #chore-update-vc-x1-template
[23]: #docs-freshen-vc-config-and-config-subcmd-closing
[24]: https://github.com/winksaville/iiac-perf/blob/agent-files-model/notes/chores/chores-06.md#commit-body-form-proposal-2026-08-12
[25]: /agent-data/prose.md#commit-body-form
[26]: /agent-data/cycle-protocol.md#body
[27]: https://github.com/winksaville/iiac-perf/blob/c38f8a6087e5/notes/chores/chores-07.md#docs-always-link-the-closing-rung
[28]: https://github.com/winksaville/vc-x1/commit/22c3fb55675a "22c3fb55675a19f6258baf72103df1737ed8d90d"
[29]: https://github.com/winksaville/vc-x1/commit/2adbbcf8e775 "2adbbcf8e775e86b6d2c9bf3883e2627265cf239"
[30]: https://github.com/winksaville/vc-x1/commit/0fd9f5eba01d "0fd9f5eba01dfcfc703f429d078579d854a8a90b"
[31]: https://github.com/winksaville/vc-x1/commit/a2849bc1da6c "a2849bc1da6ce622eaa7d90329239c0437958233"
[32]: https://github.com/winksaville/vc-x1/commit/ebc8f1fedc39 "ebc8f1fedc39d62016c7dedc0c9b760123968658"
[33]: https://github.com/winksaville/vc-x1/commit/7d643aec3bb0 "7d643aec3bb062cadfa7dfb49f9ccc883f374cb3"
[34]: https://github.com/winksaville/vc-x1/commit/906dcf161bb9 "906dcf161bb9fcad42965be0d55a1a8aa09d9ec1"
[35]: https://github.com/winksaville/vc-x1/commit/6b955a7bdeee "6b955a7bdeee43adfc36006708b4d9b59cebd7d7"
[36]: https://github.com/winksaville/vc-x1/commit/f668e07dabda "f668e07dabdaf7b1c6734cbb8328a2fa49acedbd"
[37]: https://github.com/winksaville/vc-x1/commit/5d15c71b6a60 "5d15c71b6a600084b02f3eea75874dd3a65a6010"
[38]: https://github.com/winksaville/vc-x1/commit/14e729e60e97 "14e729e60e970107992e6c17046152c4a3f6824a"
[39]: https://github.com/winksaville/vc-x1/commit/9e8d85f7218f "9e8d85f7218f678190a80490635d1e34be25245d"
[40]: https://github.com/winksaville/vc-x1/commit/181e760d4e3d "181e760d4e3d996e3feeb00ff6e4e752c9c53229"
[41]: https://github.com/winksaville/vc-x1/commit/de7afef14b5c "de7afef14b5c8e2c5d5d1bb311df254e661a9706"
[42]: https://github.com/winksaville/vc-x1/commit/a84a34eefd21 "a84a34eefd2128ca4eaabca48fcb057ee3b4b3a7"
[43]: https://github.com/winksaville/vc-x1/commit/dc0b64e6b253 "dc0b64e6b253c472cef2b68ea46b7e1675dbb256"
[44]: https://github.com/winksaville/vc-x1/commit/1aba2133a240 "1aba2133a2404f287e68873f11d79762c5d666cb"
[45]: https://github.com/winksaville/vc-x1/commit/4f07f8af55e3 "4f07f8af55e30696a92a694a36962c10ed152d1e"
