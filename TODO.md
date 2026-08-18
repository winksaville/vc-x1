# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. At Preparation
the picked-up `## Todo` item **moves** here (never copied, one home per text) and becomes six
provisional items, all required, all revised as steps land. At close-out the whole block moves
into `notes/chores/chores-NN.md` and becomes that cycle's `##` section. It is never written in
two places. Shape:

```
### <type>: <title>

#### Problem
<what is wrong, a sentence or two>

#### Solution
<what will be done about it, broad; provisional until the close-out>

#### Acceptance check
<the measure of "are you finished?">

#### Ladder
- [[N]] [<cycle title> opening][M] (done)
- [[N]] [<title>][M] (current)
- [[N]] [<title>][M]
- [[N]] <cycle title> closing

#### Deliberation
<how the five above were decided; `_None._` if there was nothing to deliberate>

#### Ladder details
<one `#####` subsection per rung, headed by its exact title, opened at laddering with the
rung's intent and completed at landing with the conceptual delta; the closing rung's only at
close-out, gotchas in problem/solution form>
```

A multi-cycle program adds one level: the program is the `###`, its current cycle the `####`,
and the six items sit one level below that (headings give the current work durable anchors,
which numbered Todo entries can't). Full rules in
[cycle-protocol.md](agent-data/cycle-protocol.md#preparation); the move's four transforms are
in [Chores sections](agent-data/cycle-protocol.md#chores-sections).

_No cycle currently in progress._

_The program block below predates the six-item convention and is grandfathered. Its versioned
rungs convert when touched._

### Refactor: typed jj facade -> jj-lib in-process; end subprocess spawning

Version-control operations were ~30 hand-rolled `run("jj", ...)`
spawns plus every mutation, with per-module private wrappers
and raw-git vestiges in init: stderr parsing instead of typed
errors, and jj's single-attempt index-lock acquisition (the
push `bookmark-set` lock race in [bugs.md](notes/bugs.md))
can't be retried where it fails. A multi-ladder program; the
staged plan, design detail, and the eight absorbed former
Todos live in
[refactor-20260716.md](notes/refactor-20260716.md).

Program ladder in **straight trunk order**, one bullet per
commit that lands on the branch, so reading it top to bottom
is reading `git log --first-parent`. `refactor:` entries are
the program's rungs, one per cycle (adjacent stages
consolidated 2026-07-24; titles are the anticipated close-out
titles; unshipped versions are provisional, since jj-lib may
split into two cycles). `docs:` entries are interludes that
sit between cycles and belong to no rung. Every shipped ref
points at a close-out commit published via the long-lived
`refactor-vc-x1` bookmark (merge-only while it ran; fully
merged into main and deleted 2026-08-03 under jj.md's
long-lived bookmark discipline).

The order is load-bearing: a
trapezoid's `<base>` is the parent of its own first rung, not
the previous close-out, so 0.78.0 bases on 0.77.4. Taking the
close-out instead swallows the interludes into the merge's
ladder side, which already bit at 0.76.0, whose base was the
0.75.1 interlude. See
[the recipe](agent-data/cycle-protocol.md#trapezoid-close-out-recipe).

- [[1]] 0.73.0 refactor: DRY jj facade (done)
- [[2]] 0.74.0 refactor: hygiene riders (done)
- [[3]] 0.75.0 refactor: facade owns topology (done)
- [[12]] 0.75.1 docs: refactor program ladder + conventions (done)
- [[4]] 0.76.0 refactor: repo registry (done)
  - first trapezoid published by the four-step recipe
- [[13]] 0.76.1 docs: trapezoid close-out recipe (done)
- [[10]] 0.77.0 refactor: stateless push (done)
- [[11]] 0.77.1 docs: jj-lib design notes + trapezoid recipe (done)
- [[16]] 0.77.2 docs: typeable punctuation (done)
- [[18]] 0.77.3 docs: re-describe rule + defer punctuation
  sweep (done)
  - retires the two `0.77.x` docs rungs into a `## Todo`
    (source sweep) and the backlog (interlude shape)
- [[20]] 0.77.4 build: bump jj-lib to 0.43 (done)
  - not migration work; keeps the read-side compiling
    against the installed jj 0.43.0, and correct whichever
    way the mutation decision goes
- [[22]] 0.78.0 refactor: jj-lib migration (done)
- [[23]] 0.78.1 docs: adopt the 20260803 baseline pin set (done)
  - narrative moved to its chores-16 section 2026-08-06; it had lived here as a seam
    exception between chores files
- [[26]] 0.78.2 style: typeable punctuation + line-width source sweep (done)
  - a `## Todo` cycle rather than a program rung, on the ladder for the same reason the
    `docs:` interludes are: it landed on trunk, the first cycle to land directly on `main`
    after the `refactor-vc-x1` bookmark's deletion
- [[27]] 0.78.3 refactor: drop sync state and remove revert (done)
  - not a program rung despite the prefix: a `## Todo` cycle from the bugs.md #8 triage,
    branched off `0.78.2` on its own bookmark `drop-sync-state-vc-x1` while `0.79.0` runs
    on `trapezoid-push-vc-x1`; trunk order holds, since it lands on `main` ahead of the
    `0.79.0` merge
- [[30]] 0.78.4 test: Claude Code can complete a cycle (done)
  - not a program rung either: a one-commit cycle that opened as a throwaway experiment on
    `cc-bm-and-push-test` and was promoted mid-run, branched off `0.78.3`; lands on `main`
    ahead of the `0.79.0` merge, so trunk order holds
- [[N]] 0.79.0 refactor: trapezoid-push + body-intro
  validation
  - the `## Todo` entry "refactor: trapezoid-push +
    body-intro validation"
  - at its merge: reconcile with the 0.78.3 single-name convention (chores-16). The branch
    manifest still says package `vc-x1-dev`, which under the convention is a legitimate dev
    name for its rungs; the merge commit's manifest says `vc-x1`. custom.md's resolution
    keeps the branch's filled copy, with the version-bump line's `cargo update -p` phrased
    against the manifest's current name, and gains the open/close rename step beside the
    version bump (custom.md on `main` is the bare skeleton, so neither has a home until that
    merge)

## Todo

 Entries are in **strict priority rank**, #1 highest,
 descending. Reprioritize by moving an entry, then
 `vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
 The numbers are positional rank, not stable IDs, so to refer
 to a Todo, name it by its **title** (a greppable mention;
 a numbered list item has no anchor to link to), not its
 number. Long-tail entries
 live in [todo-backlog.md](notes/todo-backlog.md). Use the
 [Prose form](/agent-data/prose.md#prose-form); deeper
 detail goes in `notes/chores/chores-NN.md` design
 subsections (link via `[N]` ref).

1. **Review iiac-perf's three convergence proposals.** Their formal review (2026-08-15 via
   `../vc-x1-messages`, [their chores-07 section][54]): our set is the base, their whole diff
   three proposals: validate every commit, the flat semicolon rule with its sweep, and the
   always-linked closing rung. **All three accepted** (wink, 2026-08-16), their text already in
   our set via the [trial rung][55]. The heads-up record went out 2026-08-18. Remains: the
   reply closing their record with `outcome-*`, riding the 0816-proposal.
   - also riding the reply: their pinned-set gap claim (messaging rule vs chores timing),
     their notes-entry answer, four tooling findings (twin-title desync, orphan bot commits
     on empty-`@` push, no publish-an-amendment verb, rebase order-skew), and the old
     template mailbox's open threads

2. **Empty the custom* files into the pinned set and config (the 0816-proposal)** (wink + bot,
   2026-08-16). Both members' custom files hold family infrastructure that only lives there
   because the pinned set and the config schema had no home for it. Goal: nearly 100%
   byte-identical agent-files, implemented here first, then proposed to iiac-perf as a
   working result.
   - messaging behavior pins into `agent-data`, thin, the messages repo's README staying the
     protocol authority
   - environment facts move to config: workspace `.vc-config.md` for member identity, user
     `~/.config/vc-x1/settings.{md|toml}` for machine paths (facts only, never rules).
     Reconcile at pickup with "Drop the global config and the account notion"
   - validation commands become a `[validate]` table run by `vc-x1 validate`, so pinned
     checklists name one universal command
   - `custom-family.md` retires, `custom.md` converges to the payload's `_None._` shape
   - the payload takes the result (backlog "Update the template payload, and empty the
     three-way diff"), and the closing reply cites it

3. **Finish the vc-config surface (the five rungs deferred at the 0.78.8 early close).** The
   markdown carrier landed and the cycle closed early for the 0816-proposal agent-files work,
   leaving the surface's completion as its own cycle. The deferred acceptance items ride with
   it: agent vocabulary with old spellings rejected (a test shows the fix-it),
   `config --refresh` with `--check` clean on both sides, `validate-anchors` clean over the
   records, and the `.agent-session` repoint end to end.
   - **feat: agent naming in config and CLI**: `repos.agent` / `[agent-session]` /
     `agent-session` / `--scope=agent`, old spellings rejected rather than aliased, the
     rejection printing its fix-it for both sides (`legacy_vc_config::reject` is the model)
     - rejection, not aliases (wink, 2026-08-12, iiac-perf concurring): an alias is a live
       dual-name path that stays temporary only if someone later deletes it, while a
       rejection is permanent and harmless. `repos.bot` is a topology key, so the fix-it
       turns the flag day into a five-second edit at a moment the member picks
     - values untouched: `repos.agent = ".claude"` until the repoint rung, and the ochid
       label stays `/.claude/` (test-pinned). The pinned-prose sweep stays excluded
       (convention work runs as its own cycle), and `homes` -> `files` waits for the
       Drop-the-global-config entry below, this rung respelling `workspace-bot` alone
   - **chore: regenerate configs in md format**: `vc-config-test.md` becomes the generated
     model `vc-config-model.md` (generated, not maintained: build.rs already knows every
     key, so "contains every key" holds by construction), the work-side `.vc-config.md`
     byte-identical to it (a test renders and compares), both sides regenerated, and the
     `.vc-x1` leftovers retired (`.claude/.vc-x1`, the work `.gitignore` line)
     - ownership model (2026-08-10): fence interiors are the workspace's own, the prose
       between fences is machine-owned rendering, and the durable link edit is
       `reference-base`, an active key surviving refresh
     - the `homes` correction rides here: the three `bot-session` keys drop the agent side,
       which nothing reads
     - the model carries derived `reference-base` https urls, and the info-string rule's
       negative half (only fences tagged exactly `toml` are live) lands in `vc-config.md`
   - **feat: add config --refresh**: regenerate a file's prose while preserving fence
     interiors and `[repos]` byte-for-byte, `--check` exiting nonzero on drift
   - **feat: add validate-anchors**: same-file heading anchors via the documented slug
     algorithm plus `[N]` definition/use matching, the validate-repo design's first
     standalone slice (backlog #24 absorbs it at pickup). Stretch: cross-file `[N]:` targets
     (backlog #53)
   - **chore: point config at .agent-session**: wink's between-session move (mv, config
     edit, `.gitignore` edit, `vc-x1 symlink`), with the following session committing the
     record
   - per-key worked examples in `vc-config.md` remain from the original plan, unscheduled

4. **Drop the global config and the account notion.** vc-x1 loads a user-level
   `~/.config/vc-x1/config.toml` whose whole remaining job, once the unread keys go, is
   expanding an `init` shorthand that the `owner/name` and path target forms already cover
   without it (wink, 2026-08-11: he passes the full url in practice and a local name only when
   testing). A config tier nothing needs is the same rot as the fossil `[push]` block, so it
   goes, and the schema drops from eleven keys to five.
   - out: `src/config.rs` entire (loader, `UserConfig`, the account map, the
     `--account` -> `[default].account` resolution chain), the `--account` flag,
     `Context.user_config` and its disk read at every subcommand entry, `Home::User`
   - out of the schema: `default.account`, `default.debug` (parsed, logged, never consumed),
     `repo.default`, `repo.category.<cat>`, and both `account.<name>.*` families
   - what remains is five keys in two files: `[repos]` on both sides, `[bot-session]` on the
     work side
   - `homes` becomes `files` with values naming the two sides only, so "user" leaves the
     vocabulary and stops colliding with `account` (wink: a human reading "user" and "account"
     connects them, and here they were unrelated axes)
   - removing `--account` breaks an invocation, so it errors by name and points at this
     entry's record rather than reporting an unknown flag
   - the account model is worth resurrecting if a second repo host ever matters: a backlog
     entry names the cycle that removed it and lets the diff carry the design, rather than
     restating it in prose that can rot
   - runs after the vc-config cycle on purpose: `--refresh --check` makes a schema shrink
     mechanical, so this is the first real customer of the machinery that cycle builds

5. **Retire the remaining jj spawns; make the build enforce it.** The refactor program's
   banner goal ("end subprocess spawning") outlived its ladder: 0.78.0 migrated the facade
   and every mutation routed through it, and its commit body claimed "ending jj and git
   subprocess spawning", but spawns the facade never carried remain. Found 2026-08-06 at the
   0.78.3 review; how the gap survived seven disciplined cycles is in
   [chores-16](notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert).
   - the inventory, non-test code: sync's reposition and rebase steps (`jj new` twice,
     `jj rebase` twice) and `current_op_id` / `op_restore` (`jj op log` / `jj op restore`);
     push's three `jj diff --stat` reads; init/clone's `jj git init --colocate`
     (`repo_utils.rs`)
   - deliberately excluded, subprocess by design: the version gate's `jj --version` / `jj -V`
     (clone, init, version), which exist to ask the *installed* binary; push's `$EDITOR`
   - the teeth, so the goal cannot silently regress once met: remove `run()` from non-test
     code, or ban `std::process::Command` via clippy.toml `disallowed-methods` with the
     version-gate module explicitly allowlisted as the documented exception
   - a prerequisite for the safer revert's "identifiable sync operations" (see "Stale
     `/.vc-x1` gitignore line: report it, and a safer revert, if ever")
   - the process lesson (a program's header states its acceptance check at open; close-out
     runs it) is a template-proposal candidate for cycle-protocol.md's Close-out

6. **validate-repo-data.** Golden ids for a fixture repo, so a
   jj-lib bump that moves the on-disk data fails loudly instead
   of building green. The gate at `0.78.0-4` refuses on a version
   mismatch precisely because we cannot tell whether the data
   moved; this is the check that could eventually tell us, and
   the route to relaxing the gate's coarseness. See
   [the policy](notes/jj-version-policy.md#how-this-could-be-relaxed).
   Two modes over one fixture and one id extractor:

   - **Ratchet**, in `cargo test`. Record ids under the current
     jj-lib, commit them, and let the *next* bump re-run them.
     Zero standing cost, catches drift the moment we take a new
     version.
   - **Live pair**, a `support/` script, not a `#[test]`, so
     `cargo test` never pays for it. Build a probe binary twice,
     against N-1 and N, run both over the same fixture, diff the
     reported ids. Generate a throwaway manifest in a temp dir
     for each version rather than adding a crate to our lock.
   - **Trigger the live pair on the jj-lib bump, not on our
     release cycle.** Our cycles run faster than jj's releases,
     so per-cycle mostly re-compares the same pair. The bump is
     when the answer can change, and it is also when the answer
     is most useful: "should we take 0.44?" is a question the
     probe can answer *before* we commit to the bump.
   - The probe needs only the storage-facing API: load a
     workspace, read operation / view / commit / change ids,
     create a commit. That is jj-lib's stable surface. The 0.43
     break that motivated this whole cycle
     (`use_glob_by_default` leaving `RevsetParseContext`) was in
     revset *parsing*, which the probe never touches, so keeping
     it compiling against N-1 should stay cheap.
   - **What it does not cover.** It compares two versions *on
     our fixture*. A change touching a path the fixture does not
     exercise reports "same" and is wrong. A sample, like `jj -V`
     is a sample; say so where it is documented rather than
     letting it read as proof.
   - **Watch operation ids and view ids first.** Those are jj's
     own content-addressed op-store hashes, so they move if
     hashing, serialization, or a stored field's meaning moves.
     Commit SHAs are gix's, computed from commit content, so they
     mostly pin git rather than jj and are the weaker signal.
   - **Change ids are goldenable, and are the best canary in
     the set.** Three cases: a commit authored in jj gets a
     random chid (`JJRng::new_change_id`); a git commit carrying
     a `change-id` extra header keeps the original; and a git
     commit without one gets a *deterministic* chid, the commit
     id's bytes `4..20` reversed and bit-reversed
     (`git_backend.rs`, `synthetic_change_id_from_git_commit_id`).
     Build the fixture by importing git commits and every chid
     is reproducible with no seeding at all.
   - That function's doc says "the exact algorithm for the
     computation should not be relied upon", so jj reserves the
     right to change it. That is a documented instance of the
     schema-invisible drift the gate exists for, and this test
     is what would catch it: the algorithm moving changes every
     synthetic chid at once.
   - **Determinism for the rest.** Operation ids embed
     timestamps and commit ids embed author and committer time,
     so those still need a pinned clock. Random chids, if the
     fixture needs any, are pinned by the `debug.randomness-seed`
     config key (`settings.rs`), which arrives through
     `StackedConfig` and so is reachable from jj-lib without
     going near the CLI.
   - **A committed fixture, not this repo.** Using vc-x1's own
     repo as the guinea pig was the original sketch, but its
     history grows every commit, so the goldens would churn and
     stop meaning anything. A small fixture stays stable and
     fast; this repo can still be a manual proving ground.
   - Read-only commands get the complementary assertion: hash
     every file under `.jj/` before and after, and record which
     ones are genuinely inert. That is the measurement the policy
     names as the way to narrow the gate from "every subcommand"
     to something smaller, backed by evidence.
7. **refactor: trapezoid-push + body-intro validation.**
   `vc-x1 trapezoid-push`, a **subcommand** rather than a flag
   on `push` (decided 2026-07-28), publishes a close-out as a
   non-fast-forward merge; body-intro validation rides as
   the first rung. See
   [trapezoid close-out](notes/refactor-20260716.md#stage-trapezoid-close-out)
   and
   [push body-intro validation](notes/refactor-20260716.md#stage-push-body-intro-validation).
   After jj-lib, so the reshape is built in-process.
   - `push` keeps a stateable invariant: it never produces
     a merge. A mode flag that rewires the stage sequence
     would cost that.
   - Shared implementation, not a second copy: the common
     pipeline (preflight, both gates, message, commit-work,
     commit-bot, bookmark-set, push-work, bot squash) moves
     into its own module that both subcommands call, with
     the reshape as the one inserted step. The
     stateless-push cycle shrinks that pipeline first,
     which is what makes the extraction cheap.
   - A backend `trait` (jj today, git or another VCS later)
     is the natural next abstraction if a second backend
     ever appears. Worth converting these concepts to
     traits then, not now: we are committed to jj, and a
     one-implementation trait buys nothing but indirection.
8. **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
   `ExitCode::FAILURE`: a misspelled key and a config the tool could not read exit alike, so a
   caller can branch on "clean or not" and nothing finer. Proposed: **0** all tables and keys
   known and their values reasonable, **1** unknown or otherwise non-fatal findings, **2** a
   fatal situation. The convention is grep's and diff's, so it needs no teaching.
   - the fatal cases already exist and are the subject of bugs.md's **`config --validate`
     reports "I gave up" as a finding** (#9): malformed TOML, an unclosed fence, a side holding
     both carriers, a legacy `[workspace]` schema. Every one of them means the check could not
     be performed rather than that it failed
   - **sequenced after that bug**, which draws the "found something" / "could not check"
     distinction as a local fix. Once drawn, the exit status is a rendering of it, and doing
     the tiering first would mean inventing the classification twice
   - the cost is not in `config`: `main` maps every subcommand error to `ExitCode::FAILURE`
     (`main.rs:477`, `:507`, `:514`), so a distinct code needs the error path every subcommand
     shares. Cheapest to take while that path is open for another reason
   - tier 0's "values reasonable" describes a capability that does not exist: `key_known`
     compares key paths only and no value is ever inspected. Read tier 0 as "keys known" at
     the start; value checks land later as ordinary tier-1 findings
   - decide there: whether `--refresh --check`'s difference exit joins this scheme (a
     difference is a finding, not a fatal) or keeps its own
9. **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot,
   2026-08-12). The md carrier costs a config file the toml-aware editors and formatters a
   `.toml` gets, and nothing answers "what do these fences actually concatenate to?", which is
   also the question a parse diagnostic raises. Outside the "docs: freshen vc-config and
   config subcmd" ladder, whose acceptance items do not need it, but ranked here because a
   format's debugger is worth most while the format is new.
   - run the `md_fence` filter over the target file and print the result verbatim, blanks
     included, so the printed line numbers are the source's and a diagnostic's line lands
   - **not `--resolved`**, iiac-perf's word: this subcommand already spends "resolved" on
     effective-after-layering (the `[repos]` resolved-agreement invariant, `resolved_hint`'s
     which-carrier-exists answer), and this is the far end of that, one file's raw extraction
     before any parse or layering
   - it has no existing surface to join: `config` with no flag prints the *schema*, not a
     workspace's values, so nothing today shows a config file's own contents at all
   - decide there: the name (`--toml`, `--as-toml`, `--fences`), and whether it composes with
     `--validate` or excludes it
10. **A committed cycle-check runner.** The per-commit flow's
    validation (fmt -> clippy -> test -> install) exists only as
    prose in cycle-protocol.md, so it is recomposed by hand
    every commit, and a hand-composed shell one-liner can
    silently stop checking. Found at the 0.77.0 close-out: in
    `clippy ... 2>&1 | tail -2 && cargo test ...`, the pipeline's
    status is *tail's*, which is always 0, so the `&&` gate
    was decorative and `cargo test` ran even on a run where
    clippy had failed. The failures were caught by reading the
    output, not by any check.
    - The defect class is the one this cycle spent its time
      deleting: a mechanism that looks like a guarantee and
      isn't.
    - Write the sequence down once. Options, cheapest first:
      `support/cycle-check.sh` with `set -euo pipefail` (there
      is precedent in `support/gen-exmpl-1-3.sh`); a `justfile`
      target; or `cargo xtask`, where the steps are `Command`
      calls whose statuses are handled like any other
      `Result`, most aligned with the no-unwrap discipline and
      heaviest for four commands.
    - **Not a vc-x1 subcommand.** That line was drawn at
      0.69.0-3 when the hardcoded cargo preflight was removed:
      vc-x1 assumes nothing about a repo's contents beyond
      `.jj` and `.vc-config.toml`.
    - Until it exists: run validating commands as separate
      invocations, never piping one into `tail`/`grep`, and
      never `&&` after a piped stage. `${PIPESTATUS[0]}` is
      the escape hatch when a pipe is genuinely wanted.
    - Split by ownership: the *runner* is project-local (the
      cargo cycle is Rust-specific), but the *rule* (a
      validation step's exit status is checked, not read)
      belongs in cycle-protocol.md's per-commit flow, which
      fans out to the template family.
11. **`squash-push --title` / `--body`.** `squash-push` amends
    content only: it folds the working copy into the last
    commit and force-updates the remote, but the commit keeps
    its existing message. Fixing a published commit's *message*
    is therefore two steps (`jj describe -r @-`, then
    `squash-push`). Accepting `--title` / `--body` makes it
    one.
    - No new risk: squash-push already rewrites a published
      commit and force-updates the remote. This only changes
      which part of the commit it edits.
    - **ochid handling: tell, don't force.** A user-supplied
      body drops the `ochid:` trailer unless it repeats it,
      which silently breaks the cross-repo link. vc-x1 should
      *not* inject the trailer (unlike `push`, which authors
      the message and stamps it; here the user authors it and
      the tool shouldn't rewrite their text). It should error
      when the new message loses a trailer the commit had,
      naming what would be lost, with an explicit override
      flag for the case where dropping it is intended.
    - The content-side guard is the precedent: squash-push
      already refuses a squash that would drop source-only
      trailers (the 0.65.1 ochid-loss incident). Same check,
      new input.
    - **The guard has a hole the flags would close.** Today the
      two-step workaround routes around the very check that
      protects the trailer: `squash-push` guards the squash
      path, `jj describe` guards nothing, so the workaround is
      strictly less safe than the feature. Hit at the 0.77.2
      amend (2026-07-29), where fixing that commit's own
      close-out bookkeeping meant editing content *and*
      message, and the trailer survived only by hand-copying
      it. `vc-x1 fix-desc` can repair a dropped ochid by title
      match, so the failure is recoverable, not silent-forever.
    - Amending a just-pushed commit is a real workflow, not a
      rare one: backfill lands one push later by design, so
      every commit has a one-push window where its SHA is
      cited nowhere and a rewrite costs nothing. Message fixes
      naturally cluster there, which is exactly where the
      two-step shape bites.
12. **Restructure templates: single template repo + fixed bot
    seed manifest.** Replace the separate
    `vc-x1-work-repo-template` + `vc-x1-bot-repo-template`
    repos with the one work-repo template, whose live
    `.claude/` doubles as the bot-side seed source; retire
    `vc-x1-bot-repo-template`. `vc-x1 init` / `clone` updates
    for the new layout. First up after the refactor program.
    - `--use-template` rule: explicit `CODE,BOT` copies all
      non-hidden files from BOT (unchanged, the escape
      hatch for rich bot seeds); `CODE` alone seeds the bot
      side from a fixed manifest (`LICENSE-*`, `README.md`)
      taken from `<CODE>/.claude/`. The `<CODE>.claude`
      sibling default is dropped.
    - The manifest is the safety property: a live `.claude`
      has non-hidden session artifacts at top level, and
      the known subset is what lets it double as the seed
      source without leaking session history into new
      projects.
    - Manifest members missing in the source are skipped, so
      a code template with no `.claude/` content yields a
      bare-but-valid bot repo (the bot template is
      optional; init already generates the true minimum
      itself).
    - `memory/MEMORY.md` moves from copied to generated:
      it is intentionally empty (seeded only because Claude
      tends to create it otherwise), so init emits it like
      `.vc-config.toml` instead of copying, leaving no "is it
      still empty?" invariant in the template.
13. **ochid: bot-repo location qualifier.** An ochid is
    workspace-relative (`/.claude/<chid>`), so nothing in a
    published commit says *where* the companion bot repo
    lives (vc-x1's is `github.com/winksaville/vc-x1.claude`,
    discoverable only by convention). Anyone cloning just the
    work repo can't resolve bot-side ochids. Design already
    sketched in forks-multi-user.md
    [Per-user bot repos via URL-shaped ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid):
    URL-shaped trailers, plus the complementary
    `.vc-config.toml` repo-index form; resolver dispatch is
    one rule (URL -> fetch, else workspace-relative), existing
    path-form trailers stay the backward-compatible case.
    - Cheap first rung: declare the companion's URL once in
      the committed `.vc-config.toml` (no trailer-format
      change; any work-repo clone then knows where the bot
      repo lives). Rides naturally with the refactor
      program's facade-owns-topology stage
      (bot-repo-location config).
    - Link rot + mirroring mitigations are in the same doc
      section.
14. **Version-number protocol is fragile: versions are
    baked into titles/bodies/todo/done/chores before the
    change lands.** The cycle protocol embeds an `X.Y.Z-N`
    version in commit titles and bodies, `## Todo` /
    `## Done` entries, and chores headers, all written
    while the work is in progress, i.e. before it lands.
    But version numbers are subject to change: in a public,
    merge-based flow (e.g. Linux), the version a change
    ships under is only fixed when it merges into `main`,
    so the landing version can't be anticipated while the
    work is underway. Pervasive version-in-text is
    therefore fragile for any non-linear / multi-contributor
    workflow. Promoted from Ideas at 0.65.2-0; slated for
    the cycle after 0.65.2.
    - Live in-repo example (2026-07-24): 0.72.0 was
      pre-assigned to the trapezoid close-out cycle, which
      paused on `support-trapezoid-commits` after `-1`; the
      refactor program then ran 0.73.0+ directly off the
      0.71.0 main tip, leaving 0.72.0 a permanent gap, since
      renumbering either branch would rewrite cross-linked
      history. Disposition recorded in the
      [split push.rs stage](notes/refactor-20260716.md#stage-split-pushrs).
    - Related numbering thought (2026-07-24): program-shaped
      work could claim one minor and number its cycles
      `X.Y.1..n` (the jj refactor's seven cycles would have
      been 0.73.1..0.73.7), with program membership encoded in
      the version. Trade-off: a per-prep "is this a program?"
      call vs today's decision-free minor-per-cycle.
    - Open question: what identifies a cycle's commits if
      not a pre-assigned version?
      - Needs to be unique within some agreed upon domain.
        A contributors email address would do it, but also
        a UUID (short-version) for a contribution. I could
        imagine a UUID generated from the initial email/issue
        that and then "version number" schema appended to that.
    - Surfaces to update once the identifier is chosen:
      cycle-protocol.md (title shape, Numbering), AGENTS.md
      (commit-recording headers), and the `vc-x1` validators
      that parse `(X.Y.Z)` strings.
15. **sync follow-up: extract `move-bookmark` command.** The
    "put the bookmark / `@` where it belongs" step at the end
    of sync (reposition logic) is useful standalone (e.g. the
    t1B scenario where `main` is right but `@` isn't on it)
    and deserves an honestly-named command instead of a mode.
    - `vc-x1 move-bookmark` (name open): no fetch; move `@`
      (and optionally the bookmark) onto a target under the
      same safety rules as sync's reposition step.
    - Sync's final step becomes a call to the same logic.
    - Follow-up to the 0.67.0 single-mode sync cycle.
16. **sync follow-up: retire the hidden `--check` alias;
    revisit push's auto-rollback.** The first half of this
    entry (push shelling out to `vc-x1 sync --check`, which
    was racy and not actually read-only) is done: 0.77.0-3
    deleted preflight outright, taking the shell-out and its
    PATH dependency with it. What survives:
    - Remove sync's deprecated hidden `--check` alias. Nothing
      invokes it now except `tests/cli_sync.rs`'s alias test,
      so this became actionable the moment preflight went.
    - Push's commit-stage rollback auto-runs `jj op restore`,
      which hides the evidence of what failed. This cycle
      deliberately kept it, since an in-process snapshot taken
      moments earlier is knowledge, not a guess, and both
      index-lock failures during 0.77.0 cost nothing because
      of it. Revisit only with a concrete case where the
      hidden evidence mattered.
17. **validate-numbering: rename the pair, check all
    sequence-managed notes files generically.** `validate-todo`
    / `fix-todo` only operate on the single file passed, so a
    renumber slip in `bugs.md`, `todo-backlog.md`, or
    `TODO.md`'s `## Ideas` section passes unnoticed, too weak
    for a pre-commit gate. Prereq for the pre-commit doc
    validators (Todo "pre-commit: single rule ...").
    - Rename the pair: `validate-todo` -> `validate-numbering`,
      `fix-todo` -> `fix-numbering`, since they validate
      numbered-sequence integrity, not todos specifically.
    - Generic detection: for every `#...#` section, validate the
      column-0 `^\d+\.␠` entries form a contiguous 1..N run.
      Drops the Todo/Bugs special-casing; auto-covers
      `## Ideas` and any new numbered section. Keep the
      column-0 anchor so indented sub-lists aren't counted.
    - Default scope: a fixed list of sequence-managed notes
      files (`TODO.md`, `todo-backlog.md`, `bugs.md`) so the
      no-arg pre-commit run covers them all. Fixed rather than
      a `notes/**.md` walk because prose docs
      (`cycle-protocol.md`, design notes) carry ordinary
      numbered lists that aren't managed sequences, and a walk
      would false-positive (markdown renders `1. 1. 1.` as
      1-2-3, a legitimate prose pattern).
    - Override args follow the `--init-from` convention:
      positional files/dirs (a dir -> its `*.md`) plus an
      `@<file>` manifest, additive, for ad-hoc validation of
      a specific file.
    - Add wrapper-level tests while restructuring: the analyze
      cores are covered (`todo_helpers` 15 tests,
      `desc_helpers` 22) but the `validate-todo` / `fix-todo` /
      `validate-desc` / `fix-desc` wrappers have none: file
      I/O, output formatting, exit codes, and the no-arg
      default path (changed to `TODO.md` at 0.69.2-2) are
      unexercised.
    - Open: revisit fixed-vs-glob at implementation if the
      fixed list proves annoying to maintain.
18. **pre-commit: single rule (no docs skip) + doc validators.**
    The pre-commit (cargo cycle: fmt/clippy/test/install) only
    checks code, so it's "skip-able for purely-docs commits",
    but that exception is exactly where checks slip (skipped on
    0.62.0-7/-8 until caught). (Since 0.69.0-3 push's
    `preflight` no longer re-runs the cargo cycle, because
    vc-x1 assumes nothing about repo contents, the pre-commit is
    the *only* gate, strengthening the no-skip case.)
    - Adopt one rule, no exception: the pre-commit runs before
      Work review on every commit. (docs: AGENTS.md Cycle
      Protocol summary + cycle-protocol.md per-commit-flow.)
    - Enrich the pre-commit so it's meaningful on docs commits:
      add the doc validators, `validate-numbering` (its own
      Todo, a prereq) plus `validate-repo` when it exists.
      Whether push's `preflight` may run them needs a decision
      against the content-agnostic principle (they read
      `notes/`, which is repo content; the repo-declared-checks idea
      was rejected 2026-07-15 in favor of "run checks
      yourself").
    - This dissolves the docs exception: with doc validators in
      the pre-commit there's always something to validate, so
      the carve-out stops making sense.
    - Its own near-term cycle (chosen over a 0.61.1 insert to
      avoid rewriting published 0.62.0-x history); no version
      pre-assigned; see the Todo "Version-number protocol is
      fragile" on fragile version targets.
19. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
    Today push assumes 1:1 symmetric WC commits with shared
    title/body. The interop / adoption scenario breaks that:
    the code side is worked single-repo style (commit +
    `jj git push` / `git push`, no `vc-x1 push` in the loop),
    so no bot pairings exist. One bot commit then records
    every code commit not yet covered by a prior `ochid:`,
    via a multi-line `ochid:` per the design in [[5]].
    - Out of scope: the trapezoid close-out, handled
      natively by the in-progress "feat: push merge
      close-out (trapezoid)" cycle, whose N-ochid stamping
      also covers a cycle held local and published all at
      once. This Todo is only the no-bot-pairings interop
      case; the stamping step's multi-line `ochid:` emit is
      shared groundwork.
    - Teach push to:
      - detect the shape (code WC empty, uncovered commits at
        the bookmark)
      - skip `commit-app`
      - compose a `.claude`-specific message
      - emit one `ochid:` line per uncovered commit
    - Open: computing "uncovered", likely a revset from the
      code bookmark back to the newest commit referenced by
      the bot journal's ochids.
20. **Run validate-bot at every vc-x1 invocation
    (config-gated).** The check is one jj spawn
    (`jj bookmark list main --all-remotes`), cheap enough
    to run at every execution, noted 2026-07-15 as a
    "could, not should". Design points:
    - locate the bot repo (`<cwd>/.claude` or config;
      shares the lookup with the refactor program's
      [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology))
      and silently skip when absent
    - severity knob in `.vc-config.toml`
      (`warn|error|off`): unrelated commands (fix-todo)
      warn at most; push / squash-push / validate-bot
      already have their own handling from 0.69.0-3
21. **CLI reference lives in `--help`; README owns concepts.**
    Each command is described in three places (clap's
    `long_about`, a README section with a flag table, and
    sometimes AGENTS.md) and only the flag *descriptions*
    self-sync, because those come from the field doc
    comments. Every hand-written block drifts silently:
    0.69.0-4 found the init section documenting retired
    `--owner` / `--dir` / `--repo-local`, and 0.77.0-3 found
    push's `long_about` still advertising a state machine
    that had just been deleted. The fix is removing the
    duplication, not auditing it on a schedule.
    - `--help` becomes the reference: what a command does,
      its stages, its flags, its invariants. It ships with
      the binary, so it always matches the binary being run.
    - README keeps workflows and concepts (the dual-repo
      model, the cycle, testing recipes, worked examples)
      and points at `--help` instead of restating flag
      tables. Delete the tables; that is the drift source.
      The `## Usage` block is the same species: its trailing
      `#` comments have drifted into three columns (40, 43,
      44) as commands were added, because the alignment is
      hand-maintained and invisible. Left unaligned at 0.77.2
      deliberately, since this entry deletes the block.
    - Clap reflows prose and collapses bullets unless a
      field carries `verbatim_doc_comment`, so help owns the
      reference, not the explanations. `long_about` does
      preserve explicit newlines (0.77.0-3's push stage
      list renders as an aligned two-column list).
    - Optional enforcement, cheapest first: assert README
      has no flag-table rows; snapshot-test `--help` output
      so unintended changes surface in review; or generate
      the reference from clap and assert the committed file
      matches. The third rhymes with "config: extract
      flag-backed key descriptions from Clap", the same
      single-sourcing shape.
    - Sweep each section against `vc-x1 <cmd> -h`.
    - Consider regenerating transcripts via support
      scripts (the gen-exmpl pattern) so examples stay
      reproducible.
22. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs.** Adopted in chores-13 (0.69.2 ladder,
    backfilled during 0.70.0-0): each rung is prepended
    with its commit reference so the rung↔commit
    correlation is direct; `Commits:` stays as the
    section-level list. The convention's home,
    cycle-protocol.md Close-out ("Add an `### As-built
    ladder`..."), is in the shared doc set
    (family: vc-x1, vc-x1-work-repo-template, iiac-perf, zc-msg-x1,
    tprobe), so landing it everywhere needs a coordinated
    family-wide sync. Not included in the 2026-07-20
    vc-x1-work-repo-template sync (straight copy); still pending for the
    whole family, vc-x1 included.
    - **Byte-identical is the goal, not the current state.**
      The set is diverged today and will stay that way while
      vc-x1 and iiac-perf churn; convergence is reachable only
      by a deliberate coordinated pass once both are stable.
      So a local edit to a shared doc is not a violation and
      does not need family sign-off, it just adds to what that
      pass will have to reconcile.
23. **Shared-doc sync: per-commit chores convention.**
    0.71.0 changed how chores are recorded: each work commit
    appends its As-built rung + narrative as it lands, rather
    than the narrative waiting for close-out. That wording edit
    was made locally in vc-x1's `cycle-protocol.md` / `AGENTS.md` (the
    shared doc set; see the byte-identical note on the
    As-built-rungs Todo above). vc-x1-work-repo-template synced
    2026-07-20 (AGENTS.md + cycle-protocol.md matching again, plus
    the TODO.md move); iiac-perf, zc-msg-x1, and tprobe still
    diverge, so the plan is to fan out from
    vc-x1-work-repo-template (same family as
    the Todo "Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs").
24. **config: extract flag-backed key descriptions from Clap.**
    `config`'s key descriptions live in `config_schema.rs`
    (`doc`/`used_by`). For the handful of keys that map 1:1 to a
    CLI flag (`bot-session.col-width` ↔ `--col-width`,
    `--result-lines`), the description could instead be pulled
    from the Clap arg's help via `Cli::command()` introspection,
    so `vc-x1 config` and `--help` share one source and can't
    disagree.
    - Only ~2 keys map cleanly (most are config-only, flag-sets,
      or value-providers), so it's a partial source and the
      schema stays authoritative for the rest.
    - Defaults still come from the schema/consts (the args
      dropped `default_value_t`, so Clap no longer holds them).
    - Output format is unchanged, only the text source, so no
      rework of the 0.71.0-9 rendering.
25. **Stale `/.vc-x1` gitignore line: report it, and a safer revert, if ever.** The 0.78.3
    residue. Existing workspaces keep their `/.vc-x1` `.gitignore` line: never edit the
    user's file automatically; report that the line is no longer needed and leave the
    removal to them (which surface runs the check is TBD; `config --validate` and the
    proposed `validate-repo` are the candidates). Separately, any `revert` reintroduction first
    needs the op-log-derived design: identifiable sync operations, target the parent of the
    run's earliest op, preview and confirm, refuse on intervening non-sync operations.
    Background in
    [chores-16](notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert).
## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **`vc` as a code+conversation provenance tool (grander
   ambition).** Today `vc-x1` manages a dual repo (code +
   `.claude`) cross-linked by `ochid:`. The larger aim is
   to *surface* that link: view history with the
   conversation and the code side by side, giving provenance, the
   *why* of a change, not just the *what*. The dual-repo +
   `ochid` design is already the substrate; the cross-links
   make code↔conversation navigable, so the viewer is UI
   over an already-solved data link.
   - Build direction: keep resolution/assembly in `vc`, an
     editor-agnostic Rust engine/lib extending the
     `show` / `chid` / `desc` family ("given a commit,
     resolve its ochid and assemble the paired diff +
     conversation slice"); the editor add-on is a thin
     presentation layer over it.
   - Front-end leans a Zed add-on (Rust, preferred), maybe
     VSCode / other. Verify Zed's extension API can host a
     rich side-by-side panel before committing; an
     editor-agnostic core hedges the bet.
   - `vc-x2`? A rewrite is unwarranted: the audit's
     Commonality pass found the architecture sound (por is
     bolted on where an existing good pattern wasn't
     applied), so equalize incrementally. "vc-x2" only makes
     sense if the viewer changes the *core* architecture
     (an index / daemon / data model). Separate
     engine-rewrite (no) from product-reposition (open).
   - Possible artifact: a top-level
     `notes/design-cli/vision.md` framing the direction,
     with the parity and conversion docs as sub-designs.
2. **Restructure the design-cli parity docs (target
   0.63.0).** `por-dual-parity-audit.md` (~1200 lines)
   fuses a *frozen* audit (the `## 1`-`## 8` snapshot
   evidence) with a *living* design (axes, decisions,
   matrix, gap list); the "audit" name undersells it and
   the halves have different lifecycles. And
   `por-dual-parity.md` (the stub) overlaps on parity but
   uniquely holds the `por ↔ dual` conversion design.
   - Split the audit doc into a frozen audit snapshot + a
     living design doc (names TBD; could reclaim
     `por-dual-parity.md` for the design).
   - Refocus the stub to conversion-only and rename (e.g.
     `por-dual-conversion.md`); drop its redundant parity
     half.
   - Repoint refs (`todo.md` `[1]` + the `por -> dual` Todo,
     `copying.md`, the audit's internal anchors + Reading
     guide) and validate; `chores-10/11/12` mentions are
     historical and stay.
   - Promote the Gap-list items to anchored
     `#### Gap N: <title>` sub-headings so cross-cycle
     citations can deep-link a specific gap (markdown
     anchors headings, not list items). Trade-off: stable
     anchors, but the ordinal lives in the heading text
     (manual renumber on reorder), fine for a consumed
     backlog. The 3 `Gap #N` links in the `0.62.0`
     close-out chores narrative resolve only to the section
     until this lands.
   - Deferred from the 0.62.0 close-out: close-out is
     bookkeeping-only, and the split is substantive,
     anchor-heavy work warranting its own cycle.
3. **Chores retire into a session index (post-viewer).**
   Once the provenance viewer ("`vc` as a code+conversation
   provenance tool" above) can present a commit's session
   and code side by side, the hand-written chores narrative
   is a distillation of a conversation the bot repo already
   records verbatim, so the DRY argument that removed edit
   lists from chores (git owns the mechanics) then applies
   to the narrative too (the session owns it). Chores
   collapses to an index into the session.
   - The `ochid:` trailer links a work commit to a session
     *commit*; the index adds within-session granularity:
     which conversation span produced the commit, where the
     design argument happened. We think it can be generated
     (the transcript records when pushes happen), making it
     drift-proof where hand-written chores never were.
   - What survives: the curated design layer (the
     refactor-20260716.md pattern). Sessions are an
     immutable journal, good as record and poor to cite
     into, so live design references keep pointing at
     curated docs, not per-cycle narrative sections.
   - The template side already points this way: chores
     files are not seeded; a new project's history is its
     own commits + bot session from day one.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

_Migrated to [done.md](notes/done.md) on 2026-07-24 (the DRY jj facade
cycle and its two docs interludes: template repo names, notes rework)._

- 0.78.9 **docs: drop the orphaned depth-note paragraph** [[34]]
  - iiac-perf's 2026-08-18 proposal accepted: the depth-note paragraph after
    cycle-protocol.md's closing-rung passage restated what Chores sections owns and is deleted
    family-wide, the template branch landed, and the pinned set now diffs empty across the
    three repos modulo custom*

- 0.78.8 **docs: freshen vc-config and config subcmd** [[33]]
  - the markdown config carrier landed: `toml` fences are the config, prose the doc, and
    `.vc-config.md` runs on both sides, a lingering `.toml` still loading and both-present
    erroring
  - the iiac-perf convergence trio (validate every commit, the flat semicolon rule, the
    always-linked closing rung) trialed, accepted, and landed, with the template baseline
    and the jj-lib 0.44 bump riding the cycle
  - closed early (wink, 2026-08-17) so the 0816-proposal agent-files work starts from a
    landed base, the five config rungs deferred to the "Finish the vc-config surface" Todo
    entry. Nothing migrated to done.md at this close

- 0.78.7 **docs: consolidate line widths** [[32]]
  - the width numbers live only in prose.md's new Line widths subsection, every former
    restatement now a pointer
  - commit bodies wrap at <=75 (the Linux kernel patch standard) instead of git's older 72

- 0.78.6 **docs: fix three semicolons** [[31]]
  - the three prose semicolons in AGENTS.md reword to comma and period joins with no
    information change, leaving only the shell-syntax ones in code spans
  - prose.md's Semicolons rule is unchanged: the proposed prose-wide ban was examined and
    dropped, with the argument in the chores section

- 0.78.5 **docs: adopt the merged agent-file set** [[29]]
  - iiac-perf's `agent-files-model` proposal merged onto this repo's file layout with the
    review's corrections: cycles on their own bookmark, the six-item cycle record with one
    home, problem-then-solution bodies at <=50-col titles, steps named not numbered, and
    versions living only in the version-of-record
  - two rules written during the review and applied set-wide: a semicolon joins equals, and
    a pinned file names no project
  - `custom.md` shrinks to the generic stub reaching the new `custom-family.md`; `CLAUDE.md`
    collapses to `@AGENTS.md`
  - dissolves the Todos "commit-description follow-through" (its convention is now pinned;
    the hard-rule question moved to the backlog) and "One home for a cycle's narrative"
    (implemented)

- test: Claude Code can complete a cycle [[28]]: a controlled
  experiment settling why `vc-x1 push` failed from sandboxed
  sessions, the cycle itself being the demonstration. Both repos were
  cloned over ssh, and the sandbox denies both the key material and
  a port-22 route, so the `git` child that jj-lib spawns had
  neither; wink repointed both remotes at https and the push went
  through. The competing hypotheses (bot-repo writability, the
  sandbox-masked config paths inside `.claude`, the interactive
  editor) were each killed by test rather than by argument.
- refactor: drop sync state and remove revert [[25]]: sync keeps its
  pre-sync op snapshots in memory only, retiring `sync-state.toml`,
  vc-x1's last cross-invocation state file; `revert` is removed, its
  role taken by sync's failure report printing the manual
  `jj op log` / `jj op restore` recovery, until an op-log-derived
  design earns a reintroduction; init stops writing `/.vc-x1` to new
  workspaces' `.gitignore`. Triggered by bugs.md #8, the push
  stale-state incident at iiac-perf. Riders: the single-name
  convention (the package name is the binary's name, `vc-x1` on main
  and per-line dev names on branches, guarded by build.rs on every
  cargo verb), the argv0 runtime banner, and the first `vc-x1`
  promotion under it.

- style: typeable punctuation + line-width source sweep [[24]]: src/ +
  tests/ are ASCII-clean and <=100 cols (JSONL fixture literals and
  comment URLs exempt as literal rows); 863 counted sites plus four
  uncounted species the enumeration missed, the load-bearing ellipsis
  in truncate_chars, and the config/show output separators; README
  config samples regenerated from the installed binary.

_Migrated to [done.md](notes/done.md) on 2026-08-09 (the
jj-lib migration and 0.43-bump cycles, and the three docs
interludes: jj-lib design notes, typeable punctuation,
re-describe rule)._

_Migrated to [done.md](notes/done.md) on 2026-08-03 (the
program-ladder, repo-registry, trapezoid-recipe, and
stateless-push entries), and on 2026-07-28 (the
hygiene-riders and facade-owns-topology cycles)._

# References

[1]: https://github.com/winksaville/vc-x1/commit/b5e40e7458b8 "b5e40e7458b8506574b2ae01f52f7ccae9023418"
[2]: https://github.com/winksaville/vc-x1/commit/946dc964b75c "946dc964b75ca29e2cc4b6c59f03aec2c364feee"
[3]: https://github.com/winksaville/vc-x1/commit/dc14a421d850 "dc14a421d8509e58fa05741fd1a868329540731e"
[4]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[5]: /notes/forks-multi-user.md
[10]: https://github.com/winksaville/vc-x1/commit/9d6f7c0b0f05 "9d6f7c0b0f05ae74dd7100d457b92b72d913404f"
[11]: https://github.com/winksaville/vc-x1/commit/3be698fcde83 "3be698fcde831b09949077e1ce934839ee01f4ea"
[12]: https://github.com/winksaville/vc-x1/commit/eb4a12eb3b56 "eb4a12eb3b561234d176953d3773960fb9f4cdaa"
[13]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
[16]: https://github.com/winksaville/vc-x1/commit/62d71818d78b "62d71818d78bc06ae8f5cc17ca060d30a08b6ea1"
[18]: https://github.com/winksaville/vc-x1/commit/03df811a72fe "03df811a72fe61bdd013e34961e72aecd671c126"
[20]: https://github.com/winksaville/vc-x1/commit/0cf200b9b3eb "0cf200b9b3eb2ad652b99e518edcdfe69b657075"
[22]: https://github.com/winksaville/vc-x1/commit/99f45fcb87d9 "99f45fcb87d901c00b0c650e520cb98b30e74208"
[23]: https://github.com/winksaville/vc-x1/commit/b2a5171292c5 "b2a5171292c553d000d6ead88fc5f5e537bebb7c"
[24]: /notes/chores/chores-16.md#style-typeable-punctuation--line-width-source-sweep
[25]: /notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert
[26]: https://github.com/winksaville/vc-x1/commit/a8b43a18999e "a8b43a18999ece30e7b807650ba45eb9b236ebdc"
[27]: https://github.com/winksaville/vc-x1/commit/b90f948defc6 "b90f948defc6be6dc7231ca1fde2eb293dc558ac"
[28]: /notes/chores/chores-16.md#test-claude-code-can-complete-a-cycle
[29]: /notes/chores/chores-16.md#docs-adopt-the-merged-agent-file-set
[30]: https://github.com/winksaville/vc-x1/commit/a478e124791c "a478e124791c3eda688c37747d103151acc5c70f"
[31]: /notes/chores/chores-16.md#docs-fix-three-semicolons
[32]: /notes/chores/chores-16.md#docs-consolidate-line-widths
[33]: /notes/chores/chores-16.md#docs-freshen-vc-config-and-config-subcmd
[34]: /notes/chores/chores-16.md#docs-drop-the-orphaned-depth-note-paragraph
[54]: https://github.com/winksaville/iiac-perf/blob/0520c17ca352/notes/chores/chores-07.md#docs-converge-the-agent-files-with-vc-x1
[55]: /notes/chores/chores-16.md#docs-trial-the-iiac-perf-convergence-proposals
