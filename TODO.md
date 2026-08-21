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
[Opening](AGENTS.md#opening), and the move's four transforms are in
[Chores sections](AGENTS.md#chores-sections).

### docs: halve AGENTS.md into rationale.md

#### Problem

AGENTS.md is the one file every session loads, and about half of it is argument rather than
rule: the "Why" paragraphs, the incident stories behind the acceptance check and the
https-remote rule, the delegation tiers' justification, the Terminology notes' "because"
clauses. A session needs the rule. The argument is for whoever would change the rule, and for
the family at convergence, and it keeps the rule from being simplified away by an editor who
does not know its cost, so it moves rather than dies.

#### Solution

A new pinned file `agent-data/rationale.md` whose headings mirror AGENTS.md's, so a rule
reaches its why by one fixed pattern, `[why](agent-data/rationale.md#<same-slug>)`, and a
missing entry is a grep away. An entry is the why, then the evidence: back references to the
chores section where the rule was paid for, the dogfood entry, the messages-repo record, the
commit. Mostly pointers, not a re-telling, and the "measured YYYY-MM-DD" lines move there with
the story. AGENTS.md keeps the rule and its boundary sentences (a sentence that says what the
rule does not cover is the rule, not rationale), with zero words of why, and gains a
**Rationale** term in Terminology and a File map line. The satellites (jj.md, prose.md) get the
same treatment later, AGENTS.md first since it is the auto-loaded one.

#### Acceptance check

- `wc -l AGENTS.md` as close to 297 (half of the 593 it opened at) as the two moves and the
  pointer rung reach, the closing reporting the count and what remains argument-free but long
- every `##` / `###` heading in `agent-data/rationale.md` has a same-slug heading in AGENTS.md
- `vc-x1 validate` clean, anchors included
- the two move rungs read as words moved: each one's AGENTS.md diff is deletions plus links,
  no rule's sentence changed
- the backfill rung and the label-colon rung are the two rule changes, each its own commit:
  after them the Opening's first step names backfill and its check, and every bold lead in
  AGENTS.md's lists and hard rules is a label ending in a colon

#### Ladder

- [[N]] [docs: halve AGENTS.md into rationale.md opening][59] (done)
- [[N]] [docs: make backfill the opening's first step][64] (done)
- [[N]] [docs: seed rationale.md and the Rationale term][60] (done)
- [[N]] [docs: move the cycle protocol's why into rationale.md][61] (done)
- [[N]] [docs: move the rest of AGENTS.md's why into rationale.md][62] (done)
- [[N]] [docs: point AGENTS.md's restated mechanics at the satellites][66] (done)
- [[N]] [docs: label-colon form for AGENTS.md's lists and rules][65] (done)
- [[N]] [docs: tighten AGENTS.md's prose][67] (done)
- [[N]] [docs: Winks AGENTS.md's tightening][68]
- [[N]] [docs: halve AGENTS.md into rationale.md closing][63]

#### Deliberation

Two move rungs rather than one, split at the `## Cycle protocol` tree, so each diff is
reviewable in a sitting. The judgment line is "boundary sentences stay": a sentence saying what
a rule does not cover is the rule, and the closing will report how many borderline calls were
made and which way. A patch bump rather than minor, as the fold cycle was: the agent-files set
gains a file but the shape of the system is unchanged, words moved. Satellites deferred per the
Todo that filed this.

A rule change added at the opening (wink, 2026-08-21): the 0.80.0 and 0.80.1 as-built rungs
were both found unfilled at this opening, the backfill missed at two consecutive openings.
Close-out step 8 says the edits "ride the next push", which names no owner, and the Opening's
steps never mention it, so the only place backfill is named is the one moment rule 3 forbids
doing it. Convention work does not ride a feature ladder, but this is an AGENTS.md cycle, so
the rule rides as its own rung and its own commit, and the acceptance check's "words moved"
clause narrows to the two move rungs rather than the branch: the check changed, and this is
why. The durable fix is tooling, a validate element failing on a `[[N]]` rung whose commit is
on `main`, filed at the closing.

A second rule change, found at the backfill rung's review (wink, 2026-08-21): the new Opening
step 1 read as "every as-built ladder ...", its bold verb skipped as a label, so the sentence
lost its imperative. The fix is a form rule, label-colon with a sentence complete without the
label, stated in prose.md and applied across AGENTS.md's step lists and hard rules, the rules
gaining short names as their labels. Its own rung, after the two move rungs, so the sweep
runs over the halved file and the family sees the final shape in one diff. The acceptance
check's "one rule change" becomes two, this one named.

The line target revised at the first move rung's review (wink, 2026-08-21): the faithful
why-lift took the protocol tree from 348 to 319 lines and AGENTS.md from 593 to 585, so the
"about half is argument" premise does not hold for the tree, which is mostly rule and
mechanics. This is the revised-toward-what-was-achieved case the Opening warns about, argued
here rather than slipped: the cycle's real deliverable is zero words of why in AGENTS.md, the
line count was a guess, and the number stays in the check as a reported measure rather than
a pass/fail gate. Alongside, a further rung widens the move: mechanics that restate a
satellite (the jj diff commands, the rungs-are-named paragraph, the push behaviors, the
close-out shape definitions) become pointers, which is no longer words moved to rationale.md
and touches the satellites this cycle deferred, so it is its own rung with its own diff.

#### Ladder details

##### docs: halve AGENTS.md into rationale.md opening

The cycle's bookmark, this block, the Done sweep, and the version bump.

##### docs: make backfill the opening's first step

Problem: backfill was specified only at close-out, as "the edits ride the next push", with no
owner and no place in the Opening's steps, so it was missed at two consecutive openings.
Solution: Opening step 1, ahead of the bookmark, backfills every as-built ladder whose commits
have landed, with the check spelled out (`rg '\[\[N\]\]' notes/chores/`, a hit outside a code
span is owed work), and close-out step 8 names the next opening as the owner of its debt.

- first, not folded into the Done sweep: the sweep is already a compound step, and a step with
  two halves is where the second half hides
- a lightweight cycle with no opening commit carries the step in its first commit, said in the
  Opening's own parenthesis rather than as a fourth place the rule lives
- the check is a grep, not a tool: a `validate` element that fails on an unfilled rung whose
  commit is on `main` is the durable fix, filed at the closing

##### docs: seed rationale.md and the Rationale term

Problem: nothing links a rule to its why, and a why has no home outside the rule's own
paragraph. Solution: `agent-data/rationale.md` with the heading skeleton mirroring AGENTS.md
1:1, the **Rationale** term in Terminology stating the link pattern once, and the File map line.

- every AGENTS.md heading is mirrored, each holding `_None recorded._` until a move fills it,
  so an unfilled heading after the moves is a visible finding (a rule with no written why)
  rather than an absent anchor. Whether the empties stay or go is the closing's call
- the file's "How to read this file" states the three rules of the move once: headings
  mirror, an entry is why then evidence, a boundary sentence is not rationale. The move rungs
  apply them and do not restate them
- the term in AGENTS.md carries the boundary-sentence test, since that is what tells an
  editor which side of the line a sentence is on, and nothing else: the argument for having
  the file is the file's own first entry, once the Terminology why moves

##### docs: move the cycle protocol's why into rationale.md

Problem: the `## Cycle protocol` tree carries the bookmark why, the acceptance-check story, the
delegation tiers' justification, the at-rest explanation, and the one-home argument, the bulk
of AGENTS.md's argument. Solution: each moves under its mirrored heading, the rule keeping a
`[why]` link.

- thirteen of the tree's headings took an entry, ten stay `_None recorded._` (the intro-less
  ones: Pushing, Before any push, and the ones outside the tree)
- the lift was faithful to the boundary test and removed about 30 net lines from the
  protocol's 348: the tree is mostly rule and mechanics, and the Todo's "about half is
  argument" premise does not hold for it. Surfaced at this rung's review as a finding
  against the acceptance check's 297-line target, with the call on what to do left to
  the user
- the `[why]` link sits on the sentence it explains, or on the section intro when the entry
  covers several

##### docs: move the rest of AGENTS.md's why into rationale.md

Problem: the why outside the protocol is scattered: the Hard rules intro, the Terminology
"because" clauses, the Working practices stories (https remotes, the 2026-08-05 quoting note),
the measured line in Changing the agent-files, the custom.md section's argument. Solution: the
same move, and the size check against the target.

- five more headings filled (Hard rules, Terminology, Working practices, Changing the
  agent-files, custom.md), five stay `_None recorded._`: The dual-repo model, Pushing, Before
  any push, Close-out, File map, all of them pure rule or pure map
- the https-remote story is the longest single move, and its rule shrank to two sentences:
  the unconditional, and the first-thing-to-check boundary
- the exit-status sub-bullets kept their commands and lost their explanations, which read
  as one paragraph in the entry
- AGENTS.md 585 to 561: the outside-the-tree text was argument-heavier than the tree, as
  the Todo said, but the tree is most of the file

##### docs: point AGENTS.md's restated mechanics at the satellites

Problem: with the why gone, AGENTS.md is still twice the target because it restates
mechanics the satellites own: jj commands, the named-not-numbered rule, push behaviors, the
close-out shapes. Solution: each restatement becomes a one-line pointer to the satellite
section that holds it, the satellite gaining any sentence AGENTS.md alone had.

- AGENTS.md 561 to 456. The protocol tree is now rules plus step lists, each step pointing
  at the satellite that owns its mechanics, and nothing in it is stated a second time
- new satellite homes, each named for what it holds: jj.md's "vc-x1 push: what it does and
  does not do", "Close-out shapes", and the per-commit contract in "Local ladders" and the
  reshape moves in "Cycle bookmarks"; notes.md's "The In Progress block" (the six items, the
  Ladder details area, the rung form) and "The close-out move" (the four transforms, the Done
  entry, the In Progress reset); prose.md's "Commit description details"
- close-out steps 3 and 4 merged (the move and the Done entry are one act in notes.md), so
  backfill is step 7 and the Opening's pointer follows
- the Terminology "Retired" notes went to rationale.md as history, not rule
- no separate details file: every block had a satellite owner, so rationale.md stays why-only
- what is left is rule text: the hard rules, Pushing's three policy paragraphs, the at-rest
  contract, Working practices, the agent-file rules. Reaching 297 from here means shortening
  rules, which is a different cycle's decision

##### docs: label-colon form for AGENTS.md's lists and rules

Problem: a bold lead that is the sentence's own verb reads as a skippable label, and the
hard rules' bold sentences are the rule itself, skipped the same way. Solution: a prose.md
rule, a bold lead in a list is a short label ending in a colon and the sentence after it is
complete without it, applied across AGENTS.md, the hard rules taking short names as labels.

- prose.md's "Leads are labels, unmarked" holds the rule, its example, the inverted case (a rule
  stated in bold with commentary after it), definitions (`**Term:**`, not `**Term.**`), and
  the one-word redundancy as the accepted price; the measured miss stays inline there, since
  prose.md's why has not moved yet
- the label carries no markup (wink, 2026-08-21, at review): bold is what makes the eye
  skip, and an agent needs no emphasis, so the colon alone marks it
- every bold lead in AGENTS.md converted: the 14 hard rules (now named: Read custom.md first,
  Push commits, Approval per push, Hard stop after the final push, No re-describe without
  coordinating, No hand-written trailers, jj not git, Read the step before the action,
  Typeable punctuation, One title per step, Stop and ask, Alert on unwrap, Intent picks the
  file, One bookmark per cycle), the Terminology definitions, the four step lists, the Policy
  paragraphs, Working practices, Changing the agent-files, and the custom.md section
- the labels cost 15 lines (456 to 471), and a refill to the full 100 columns, links and code
  spans unbreakable, gave 24 back (447)
- a tightening rung added after this one (wink, 2026-08-21): what is left is rule text, and
  the next cut is wording

##### docs: tighten AGENTS.md's prose

Problem: with the why and the mechanics gone, AGENTS.md is rule text at 447 lines, and the
rules are worded at the length they were argued, not the length they need. Solution: reword
each rule shorter without dropping a boundary, the diff reviewed sentence by sentence.

- AGENTS.md 447 to 400, every heading kept so the rationale mirror holds, every anchor and ref
  checked
- the cuts: restated subjects after a label ("Backfill: fill every ..." not "Backfill: backfill
  every ..."), parentheticals that named the obvious ("(medium, conventions)"), doubled
  qualifiers ("explicit", "specific", "mandatory" where the sentence already was), the
  Terminology and Cycle notes folded from bullet lists into paragraphs, and the File map's
  records list into one sentence
- boundaries kept, by check: every "never", "only", "not", and "exception" clause of the old
  text has a counterpart in the new
- `[rde]` added as a reference-style link, since Retiring Done entries is cited twice

##### docs: Winks AGENTS.md's tightening

Problem: the agent's tightening is one reader's cut, and the reader the rules are for is the
other one. Solution: wink's own pass over AGENTS.md, landed as its own rung so the two cuts
are separately reviewable.

##### docs: halve AGENTS.md into rationale.md closing

Closing out the cycle.


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

1. **Fix `vc-x1 config`'s rendering: print once, and write with `--output`.** (wink, 2026-08-21)
   Bare `vc-x1 config` prints the schema once per side of the default `work,agent` target, and
   since every remaining key has both workspace homes the two blocks are identical apart from
   the header, so the reader sees the same ~40 lines twice. In a workspace with no agent side
   the second block is still printed, under the `<root>/<agent-dir>/...` fallback hint, for a
   side that does not exist.
   - print the schema once by default, grouped per side only when the sides' key sets differ
     (they will again once `[family]` and `[validate]` land as `workspace-code`-only)
   - add `--output <scope>:<path>[,<scope>:<path>]` or some such, writing each side's rendering
     to a file instead of stdout, so a side's config can be (re)generated in place. This
     overlaps `config --refresh` in "Finish the vc-config surface" and should be designed with
     it, one verb or two
   - skip a side the workspace does not have, rather than rendering its fallback hint
   - accept a directory as a path target (wink, 2026-08-21): `config --validate .claude` or
     `../iiac-perf` resolves through the carrier lookup (`config_md::vc_config_path`) to that
     side's config file, the both-carriers error included, and the report labels the side by
     the directory. Today a path target must name the file itself
   - an explicit path must exist (wink, 2026-08-21). Today `config xyz` prints the whole schema
     for "any home" with the path as a label and never opens it, and `config --validate xyz`
     reports the file "not found, skipping" and passes with zero problems, so a typo'd path
     validates clean. The skip is right for a keyword side the workspace lacks, wrong for a
     path the user typed: error by name, and say which file was read
   - the rendered hints still say `.vc-config.toml` (the `VC_CONFIG_FILE` constant), and the
     md carrier rename is the "regenerate configs in md format" rung's

2. **Finish the vc-config surface (the five rungs deferred at the 0.78.8 early close).** The
   markdown carrier landed and the cycle closed early for the 0816-proposal agent-files work,
   leaving the surface's completion as its own cycle. The deferred acceptance items ride with
   it: agent vocabulary with old spellings rejected (a test shows the fix-it),
   `config --refresh` with `--check` clean on both sides, `validate-anchors` clean over the
   records, and the `.agent-session` repoint end to end.
   - **feat: agent naming in config and CLI** (moved 2026-08-21 into the "docs: empty
     custom-family into the pinned set and config" ladder, ahead of its schema-tables rung,
     so the new tables are born under the new names; the notes below ride with it):
     `repos.agent` / `[agent-session]` /
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
     - init emits `.vc-config.md` from the generated model for new workspaces (found
       2026-08-19: init still writes `.vc-config.toml`, starting every new member on the
       legacy carrier)
   - **feat: add config --refresh**: regenerate a file's prose while preserving fence
     interiors and `[repos]` byte-for-byte, `--check` exiting nonzero on drift
   - **feat: add validate-anchors**: same-file heading anchors via the documented slug
     algorithm plus `[N]` definition/use matching, the validate-repo design's first
     standalone slice (backlog #24 absorbs it at pickup). Stretch: cross-file `[N]:` targets
     (backlog #52)
   - **chore: point config at .agent-session**: wink's between-session move (mv, config
     edit, `.gitignore` edit, `vc-x1 symlink`), with the following session committing the
     record
   - per-key worked examples in `vc-config.md` remain from the original plan, unscheduled

3. **Drop the global config and the account notion.** vc-x1 loads a user-level
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
   - decided 2026-08-21 (wink): `init` takes a URL or a path, nothing else. No bare name,
     since a bare name has no host-neutral meaning and delegating it to `gh` would make the
     convenience GitHub-only. No user tier of any kind: identity is jj's config, credentials
     are git's helpers (a GitLab token in the helper makes `vc-x1 clone` work unchanged), and
     the remote is the URL. The `owner/name` shorthand goes too, since it hardcodes an ssh
     remote. bugs.md #10 (a pre-created GitHub repo rejected at preflight) rides this entry
   - provisioning stays host-keyed from the URL: `gh` for github.com as today, and a
     `glab repo create` arm for gitlab.com is the next one worth adding. Measured 2026-08-21:
     an authenticated push to a nonexistent gitlab.com path is refused ("could not be found
     or you don't have permission"), so push-to-create is not something init can lean on (we
     think a token with `api` scope might allow it, but an instance setting and a token scope
     are not a foundation). Until an arm exists, non-GitHub hosts pre-create both remotes
   - the account model is worth resurrecting if a second repo host ever matters: a backlog
     entry names the cycle that removed it and lets the diff carry the design, rather than
     restating it in prose that can rot
   - runs after the vc-config cycle on purpose: `--refresh --check` makes a schema shrink
     mechanical, so this is the first real customer of the machinery that cycle builds

4. **validate-repo-data.** Golden ids for a fixture repo, so a
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
5. **refactor: trapezoid-push + body-intro validation.**
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
   - The last stage of the retired jj facade refactor program
     (its as-built ladder is in
     [refactor-20260716.md](notes/refactor-20260716.md#as-built-trunk-ladder-program-retired-2026-08-18)).
     Parked state at the 2026-08-18 retirement: the published
     `trapezoid-push-vc-x1` bookmark holds a stale opening
     commit forked off `0.78.2`, with `support-trapezoid-commits`
     its support line. Rebase or restart is decided at pickup.
   - At its merge: reconcile with the 0.78.3 single-name convention (chores-16). The branch
     manifest still says package `vc-x1-dev`, which under the convention is a legitimate dev
     name for its rungs, and the merge commit's manifest says `vc-x1`. custom.md's resolution
     keeps the branch's filled copy, with the version-bump line's `cargo update -p` phrased
     against the manifest's current name, and gains the open/close rename step beside the
     version bump (custom.md on `main` is the bare skeleton, so neither has a home until that
     merge).
6. **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
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
7. **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot,
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
8. **The validate family: bare `validate` as the umbrella, `validate-artifact` the runner,
   `validate-work` the twin of `validate-agent`.** (wink + bot, 2026-08-21) The 0.80.0 cycle
   shipped bare `vc-x1 validate` running the `[validate]` table, beside `validate-agent`,
   `validate-desc`, and `validate-todo`, which are at-rest checks of repo state. Read as a
   family the bare name looks like their parent and is not: it runs cargo while its siblings
   check bookmarks and records. Supersedes "A committed cycle-check runner" (resolved by
   `vc-x1 validate`, whose "not a vc-x1 subcommand" line was decided the other way at that
   cycle: the commands live in config, so the tool assumes nothing about the medium) and
   absorbs backlog "Add `validate-repo` subcommand", whose "runs all" is this umbrella under
   a name that no longer fits the family.
   - rename the runner to `validate-artifact` (`--fast` kept), `validate` rejecting the old
     meaning the way `bot-session` does, and the checklists saying `validate-artifact` per
     rung and plain `validate` at close-out
   - add `validate-work`: the work side at rest, the cycle bookmark tracked and at origin,
     `config --validate` clean, mostly the push preflight exposed read-only
   - bare `validate` runs everything that applies to the workspace (artifact, work, agent,
     desc, todo), each reported by name, exit status the worst of them, a side the workspace
     lacks skipped by name
   - the `[validate]` config key stays as it is: it is the artifact's validation, and the
     umbrella reads it

9. **`squash-push --title` / `--body`.** `squash-push` amends
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
10. **Restructure templates: single template repo + fixed bot
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
11. **ochid: bot-repo location qualifier.** An ochid is
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
12. **Version-number protocol is fragile: versions are
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
      AGENTS.md's Cycle protocol (title shape) and Terminology,
      prose.md (commit-recording headers), and the `vc-x1` validators
      that parse `(X.Y.Z)` strings.
13. **sync follow-up: extract `move-bookmark` command.** The
    "put the bookmark / `@` where it belongs" step at the end
    of sync (reposition logic) is useful standalone (e.g. the
    t1B scenario where `main` is right but `@` isn't on it)
    and deserves an honestly-named command instead of a mode.
    - `vc-x1 move-bookmark` (name open): no fetch; move `@`
      (and optionally the bookmark) onto a target under the
      same safety rules as sync's reposition step.
    - Sync's final step becomes a call to the same logic.
    - Follow-up to the 0.67.0 single-mode sync cycle.
14. **sync follow-up: retire the hidden `--check` alias;
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
15. **validate-numbering: rename the pair, check all
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
      (`AGENTS.md`, design notes) carry ordinary
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
16. **pre-commit: single rule (no docs skip) + doc validators.**
    The pre-commit (cargo cycle: fmt/clippy/test/install) only
    checks code, so it's "skip-able for purely-docs commits",
    but that exception is exactly where checks slip (skipped on
    0.62.0-7/-8 until caught). (Since 0.69.0-3 push's
    `preflight` no longer re-runs the cargo cycle, because
    vc-x1 assumes nothing about repo contents, the pre-commit is
    the *only* gate, strengthening the no-skip case.)
    - Adopt one rule, no exception: the pre-commit runs before
      Work review on every commit. (docs: AGENTS.md's Cycle
      protocol, The per-rung flow.)
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
17. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
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
18. **Run validate-bot at every vc-x1 invocation
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
19. **CLI reference lives in `--help`; README owns concepts.**
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
20. **config: extract flag-backed key descriptions from Clap.**
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
21. **Stale `/.vc-x1` gitignore line: report it, and a safer revert, if ever.** The 0.78.3
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

- 0.80.1 **docs: fold the cycle agent-files into AGENTS.md** [[29]]
  - `AGENTS.md` holds the cycle protocol as `## Cycle protocol`, one account in the order a
    cycle meets it, with the at-rest contract (agent pushes, agent stops, user squash-pushes
    the agent repo) stated once, and `cycle-protocol.md` and `cycle-checklists.md` are gone,
    their jj mechanics (trapezoid recipe, local-ladder moves) in jj.md
  - "Ladder (sub-cycle)" retired for "local ladder", "bot repo" for "agent repo" with the
    actor now "the agent", the "Work" stage name retired, and the Cycle term simplified to
    one change run from opening to closing as one commit or a ladder of them
  - filed the next convention cycle: halve AGENTS.md by moving its rationale into
    `agent-data/rationale.md`
  - dissolves the Todos "Shared-doc sync: As-built ladder rungs carry `[[N]]` commit refs"
    and "Shared-doc sync: per-commit chores convention": both conventions are pinned in
    AGENTS.md and the sync they asked for is the convergence model, and files the backlog
    entry "Sweep the Todo files for pre-0.80 names and retired conventions"

- 0.80.0 **docs: empty custom-family into the pinned set and config** [[58]]
  - the agent side is `agent` in config (`repos.agent`, `[agent-session]`), on the CLI
    (`--scope=agent`, `agent-session`, `validate-agent`), and in the schema's homes, the old
    spellings rejected with fix-its
  - the family facts and the validation commands moved from `custom-family.md` into
    work-side `[family]` and `[validate]` tables, the latter a new `str-list` kind the TOML
    reader learned arrays for
  - `vc-x1 validate [--fast]` runs the configured table, one invocation per element,
    stopping at the first failure, and the pinned checklists name it
  - the messaging behavior is pinned in `agent-data/messaging.md`, `custom.md` holds the
    medium prose alone, and `custom-family.md` is gone
  - a breaking config change for the family: install 0.80.0 first, then respell, since a
    0.79.x binary reads a respelled config as single-repo, silently

_Migrated to [done.md](notes/done.md) on 2026-08-21 (the 0.79.x pair: retire the remaining jj spawns,
pin two rules and close the convergence record)._

_Migrated to [done.md](notes/done.md) on 2026-08-21 (the six 0.78.x entries: the merged
agent-file set, three semicolons, line widths, freshen vc-config, the depth-note paragraph,
and the refactor program block)._

_Migrated to [done.md](notes/done.md) on 2026-08-18 (the three pre-convention entries: the
typeable-punctuation source sweep, drop sync state, and the Claude Code cycle test)._

_Migrated to [done.md](notes/done.md) on 2026-08-09 (the
jj-lib migration and 0.43-bump cycles, and the three docs
interludes: jj-lib design notes, typeable punctuation,
re-describe rule)._

_Migrated to [done.md](notes/done.md) on 2026-08-03 (the
program-ladder, repo-registry, trapezoid-recipe, and
stateless-push entries), and on 2026-07-28 (the
hygiene-riders and facade-owns-topology cycles)._

# References

[5]: /notes/forks-multi-user.md
[29]: /notes/chores/chores-17.md#docs-fold-the-cycle-agent-files-into-agentsmd
[58]: /notes/chores/chores-17.md#docs-empty-custom-family-into-the-pinned-set-and-config
[59]: #docs-halve-agentsmd-into-rationalemd-opening
[60]: #docs-seed-rationalemd-and-the-rationale-term
[61]: #docs-move-the-cycle-protocols-why-into-rationalemd
[62]: #docs-move-the-rest-of-agentsmds-why-into-rationalemd
[63]: #docs-halve-agentsmd-into-rationalemd-closing
[64]: #docs-make-backfill-the-openings-first-step
[65]: #docs-label-colon-form-for-agentsmds-lists-and-rules
[66]: #docs-point-agentsmds-restated-mechanics-at-the-satellites
[67]: #docs-tighten-agentsmds-prose
[68]: #docs-winks-agentsmds-tightening
