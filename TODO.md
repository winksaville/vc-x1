# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries. 2 or 3 spaces also work.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### docs: the family agent-files proposal

#### Problem

The family does not have one consistent set of agent-files. Each member carries a copy that has
drifted from the payload and from the others, the rules are spread between summaries in
AGENTS.md and detail in the satellites, and an agent acting from the summary misses details the
detail file holds. This cycle makes the initial proposal for one set.

#### Solution

One set of agent-files, `AGENTS.md`, `custom.md`, and `agent-data/*`, that anyone can adopt as-is
and customize through `custom.md`. The base is zc-ring-x1's set as landed 2026-08-26, which
already carries the cycle-record (one record, no backfill, `## Closed`), the cycle shape fixed at
the first push, and named rules. This cycle syncs vc-x1 to it, then fixes what that set still says
more than once or points at wrongly, then carries the rules this cycle was opened for. Once the
other family members (iiac-perf, zc-ring-x1) agree, the set is copied into `vc-x1-template/work`,
the payload, which becomes the initial home for every future member, and every member re-syncs.
Every rung is convention work, so [Changing the agent-files](AGENTS.md#changing-the-agent-files)'s
own-cycle rule is met by the cycle as a whole.

#### Acceptance check

The three-way comparison between the two members and the payload goes empty, which is the check
the last work rung already names. Within this repo: every agent-file link resolves, no agent-file
names a rule by number, and the per-rung flow is stated in one file.

#### Ladder

The sync first, then the rungs that make the synced set consistent, then the rule-text rungs, the
payload last.

- [docs: the family agent-files proposal opening][1] (done)
- [docs: sync the agent-files to zc-ring-x1's set][2] (done)
- [docs: state the cycle protocol once][3] (done)
- [docs: one TODO.md, Closed as the history][4]
- [docs: widen Typeable punctuation to Prose style][5]
- [docs: the restart and the interlude, between cycles][6]
- [docs: Todo entries as headings, and sweep the retired names][7]
- [docs: move the satellites' inline whys][8]
- [chore(template): propose the set to the payload][9]
- [docs: the family agent-files proposal closing][10]

#### Deliberation

Provenance, the re-plan, two decisions, one new rule, and two scope corrections.

- Provenance: this cycle was opened from conversation, not from one `## Todo` entry. It
  consolidates eleven convention entries that were spread across `## Todo` and
  [todo-backlog.md](notes/todo-backlog.md), plus two rules that surfaced on 2026-08-24
  (hyphenation, restart after close-out), so no single entry is its origin and none is cited as
  one.
- Re-planned on zc-ring-x1's set (wink, 2026-08-27), one rung in, nothing pushed past the opening.
  - wink did not like where the ladder was headed and ran a cycle in zc-ring-x1 instead, whose
    result is a set with three rules this ladder did not have: the cycle-record (the block is the
    only record, no chores move, no `## Done` entry, no `[[N]]` backfill), the shape fixed at the
    first push with squash retired, and rules named rather than numbered. Those subsume the
    duplication this cycle was opened against, at the records rather than the rule text.
  - the assessment: take it as the base. Its costs are named as rungs: it states the per-rung flow
    three times (AGENTS.md, cycle-checklists.md, cycle-protocol.md, the two files vc-x1 had
    already retired), still cites "hard rule N" in three files after the rules were named, has two
    dead links (custom.md to the renamed `## custom.md`, rationale.md to vc-x1's chores-17), and
    leaves messaging.md's citation form and the close-out's "what outlives this block" question
    unasked.
  - the abandoned rung, **docs: cut AGENTS.md's steps to a line and a link**, had reached 370 to
    320 lines with every rule kept. Its finding stands: the remaining bulk is the rules themselves
    (hard rules 33 lines, per-rung flow 25, close-out 19, working practices 21), so a further cut
    would have to cut rules. zc-ring-x1's 350-line AGENTS.md is the same file with named-rule
    headings, and the draft was dropped rather than merged.
  - the specimens rung dissolves: zc-ring-x1's cycle-model.md is the filled specimen, and the
    commit-body specimen goes with the prose rung.
  - the ladder was then cut from fifteen rungs to ten (wink, 2026-08-27): retire-the-copies with
    the citation fixes, the Closed section with what outlives a cycle and the one-TODO.md
    rule, restart with interlude (both between cycles), Todo headings with the name sweep
    (the sweep runs on the headings), and the custom.md restatement into the payload rung, which
    acts under that model.
  - this block follows the new rule from this rung on: no `[[N]]` placeholders, and the cycle
    closes to `## Closed`, not to `notes/chores/`.
- Hyphenation, decided (wink, 2026-08-24): **one spelling per term, always**. Either always
  hyphenated or never, and no rule that varies the spelling by how the term is used, since
  mixing costs a reader a judgment call and buys nothing.
  - measured 2026-08-24: `work-repo` 12 against `work repo` 6, `agent-repo` 14 against
    `agent repo` 7. `agent-files`, `version-of-record`, `dual-repo`, `fast-forward` and
    `as-built` are already single-spelled, so the fix is 13 sites, 10 in jj.md and 3 in
    AGENTS.md
  - the nine `closeout` hits are the trapezoid recipe's `<closeout>` placeholder, a code token,
    and the sweep leaves them alone
  - the rule goes in prose.md and the Prose style rule names it. Terminology keeps the
    spellings, but a glossary is read for meaning rather than for spelling, so a rule stated only
    there is invisible to whoever is writing prose
- Body shape not promoted into hard rule 9, folded 2026-08-25 from its own rung. Extending
  title identity to the body was deferred at the 2026-08-07 convergence review ("a pushed body
  is coordinate-first to fix" would promote every prose rule that touches a body), and the form
  itself was pinned in prose.md at the "docs: pin the commit-body form" rung (2026-08-12). This
  cycle answers the promotion question a different way: a specimen beside the rule and a
  `validate` check on the body (`## Todo` **`validate`: enforce the record shapes the
  agent-files ask for**), which is stronger than a hard rule and keeps a bad body a plain fix.
  Hard rule 9 stays as it is.
- Rung titles are commit titles, and nine rungs were laddered without a type (2026-08-25).
  The rule sat in prose.md's Conventional-commit shape while notes.md's ladder bullet said only
  `<title>`, and Opening step 3's "shape it" did not name the retitle of a moved `## Todo`
  entry. Five rereads did not see it, since nine consistent misses look like a convention. The
  two one-line fixes move to the prose rung and the check is in the `validate` entry.
- Restart the session after a close-out (wink, 2026-08-24): family-universal, so it is a
  pinned-file rule and not custom.md. Detail in its cycle below.
- The series is **not** the old Todo 1-7. Only the first two of those were agent-files work.
  The rest were the config program, validate-repo-data and trapezoid-push, which are the tool
  rather than the proposal. The config entries are now one `## Todo` entry.
- **docs: widen Typeable punctuation to Prose style** absorbs three prose rules that arrived
  separately: the punctuation rule's widening, hyphenation, and the semicolon whole-file question.
  Splitting them means three cycles reopening the same rule and the same prose.md
  neighborhood. **Proposed, easy to drop**: the semicolon half was its own `## Todo` entry
  until this consolidation.
- Ordering: the consistency rungs run before any rule-text rung, so rule text is written once
  into a set that says each thing once. The prose rung runs before **docs: move the satellites'
  inline whys**, since that move rewrites the satellites and would otherwise re-touch what the
  prose rung just changed. The headings half of **docs: Todo entries as headings, and sweep the
  retired names** decides before the sweep in the same rung, since headings make a citation a
  link.
- Scopes dropped from the docs rungs (wink, 2026-08-27): `(notes)`, `(prose)`, `(rationale)`
  named the file the diff landed in, which is a one-word file list in the title. A scope names a
  component as its user would name it, and a docs rung here has none. `chore(template)` keeps
  its scope, since the template is another repo, a real boundary. The rule is one sentence in
  prose.md's Commit description details, a correction made in this rung.

#### Ladder details

One subsection per rung, headed by the rung's exact title: the problem in a sentence or two,
then one `-` per edit, with any decision already taken kept as a line.

##### docs: the family agent-files proposal opening

The cycle's setup commit: backfill, bookmark, In Progress block, sweep, bump, rename.

- Backfill: `chores-18.md`'s seven rungs filled from `main`. The note that `chores-16.md` held
  sixteen more was stale.
- Bookmark: created at `main`'s tip `2dc8d969`.
- Sweep: kept both `## Done` entries as nearby context.
- Bump: a patch, 0.80.5-0, and the package is `vc-x1-dev`.
- Bookmark hygiene, noted for later:
  - `messages-specimen` is merged and deletable.
  - `support-trapezoid-commits` and `trapezoid-push-vc-x1` are parked by a Todo entry and stay.

##### docs: sync the agent-files to zc-ring-x1's set

vc-x1's agent-files and zc-ring-x1's have diverged, and zc-ring-x1's is the one with the
cycle-record, the fixed shape, and named rules. Copy it in whole, so every later rung edits one
known state, and re-plan this block on it.

- `AGENTS.md` and `agent-data/*` are copied from zc-ring-x1 at its `main` `5a08731d`.
  - The copy brings back `cycle-checklists.md` and `cycle-protocol.md` for the next rung to
    retire.
  - `custom.md` was already identical.
  - The draft cut of AGENTS.md from the abandoned rung was restored first, so the diff is
    zc-ring-x1's set against the opening's, nothing else.
  - Two corrections, so the set is byte-for-byte except these: prose.md's scope sentence (the
    deliberation records why), and a stray terminal paste in cycle-checklists.md that had
    reached our copy only.
- This block is re-planned: solution, acceptance check, ladder, deliberation, and these details.
  - The `[[N]]` placeholders are gone from the ladder.
  - The ladder was cut from fifteen rungs to ten by merging same-subject pairs.
- `notes/agent-files-size.md` is opened (wink, 2026-08-27).
  - It holds the agent-files' line count, one row per landing, smaller the quasi-goal.
  - Two rows: 2205 lines at the opening, 2822 after the sync, the growth being the two files the
    next rung retires.
  - Close-out gains a Size step, and `notes/README.md` points at the file.

##### docs: state the cycle protocol once

The synced set states the per-rung flow, the opening, and the close-out three times: AGENTS.md,
cycle-checklists.md, and cycle-protocol.md, and the three disagree on where "flip to done" sits
and on the names of the phases. It also still cites rules by number after naming them, and has
two dead links.

- The two files are deleted.
  - No link was repointed. Nothing in the agent-files, `README.md`, `ARCHITECTURE.md`, or
    `notes/README.md` linked into them, only frozen history and live Todo entries, which are
    the sweep rung's.
  - Almost everything they held jj.md already had: push behaviors, recovery, the trapezoid
    recipe, local ladders, viewing for a review.
  - The three facts jj.md lacked went into its push section:
    - The agent-repo is a linear journal with one commit per push.
    - The agent-repo never has a bookmark mirroring a work-repo one.
    - A squash-push is re-run when `@` is non-empty after a pass.
- Every "hard rule N" in jj.md and rationale.md is a link to the named rule.
- The rationale.md heading mirrors AGENTS.md's `## custom.md`, and custom.md's and AGENTS.md's
  links follow. Its two links into vc-x1's `chores-17.md` are permalinks at `main`, since a
  payload file cannot cite one member's tree by relative path.
- The link check that found them (every `](path#anchor)` and `[N]: path#anchor` in the
  agent-files resolved against the headings) is a bullet in the `validate` Todo entry.

##### docs: one TODO.md, Closed as the history

Under the cycle-record rule a finished block lives in `## Closed` for one cycle and in the
landmark commit's tree after that. vc-x1's TODO.md still has the old shape (`## Done` inline,
`notes/chores/` open), nothing says a member must have a TODO.md of that shape, a relative link
into `notes/chores/` no longer names a finished cycle, and the close-out has no step that asks
whether anything in the block should be kept.

- TODO.md is not an agent-file (wink, 2026-08-27).
  - Its content is the project's record and can never match another member's.
  - The agent-files require that there is one, of the pinned shape: `## In Progress`,
    `## Closed`, `## Todo`, `## Ideas`, `## Bugs`, `# References`, the block per cycle-model.md.
  - The payload ships a skeleton, `## In Progress` reading `_No cycle currently in progress._`,
    the rest empty.
  - The shape check is a `validate-todo` extension, recorded in the `validate` Todo entry.
- `## Closed` is added between `## In Progress` and `## Todo`, empty until this cycle closes.
- The history freezes.
  - `## Done` moves whole to `notes/done.md`, frozen with `notes/chores/`, headers saying so,
    and `notes/README.md` describes the history as frozen.
  - Frozen means never deleted (wink, 2026-08-27): the growth is stopped by freezing, and
    deleting would break every relative link from `## Todo`, the backlog, rationale.md, and the
    other members' messages.
- A closed block is never amended. A late finding about a closed cycle is recorded where it is
  found, citing the landmark.
- messaging.md's "the reply cites the entry" is restated.
  - The citation is the landmark permalink, `blob/<landmark sha>/TODO.md#<slug>`, the form the
    messages repo's `outcome-remote:` already uses.
  - The `outcome-local:` relative form is dropped.
- Close-out step 2 gains the question: what in this block must outlive the cycle, and which
  `notes/` file gets it. Rationale carries why: the block is replaced, and the rung-time rule
  relies on noticing in the moment.
- A `vc-x1 closed "<title>"` verb that prints a landed cycle's block from its landmark is a new
  `## Todo` entry, not this cycle.

##### docs: widen Typeable punctuation to Prose style

The Typeable punctuation rule is the narrowest of a family of prose rules that arrived
separately.

- The rule widens to "Prose style", carrying typeable punctuation, no wall-of-prose, one
  spelling per term, the semicolon rule, and bullet form.
- Hyphenation, decided: one spelling per term, always.
  - The sites are re-measured at the rung, since the sync changed the count (13 at the
    2026-08-24 measure, `work-repo` and `agent-repo`).
  - The rule goes in prose.md, and Terminology keeps the spellings.
- Semicolons: the whole-file conversion is never run on files every cycle touches (TODO.md,
  README.md, chores). Either name the exemption or schedule the sweep. Until decided, a line
  you write carries none and the rest of the file waits.
- Bullet form, decided (wink, 2026-08-27): a bullet is a full sentence, capitalized and ending
  in a period or question mark, unless the list is a plain list of things (files, names,
  options), which takes neither. A list is all one or all the other.
  - A sentence never opens with a lower-case name (`squash-push`, `rationale.md`), it is recast
    so the name is not the first word.
  - One paragraph in Prose form. This block's Ladder details were swept to it on 2026-08-27,
    and the deliberation is swept in this rung.
- From the dissolved specimens rung:
  - `agent-data/commit-model.md`, the cycle-model cycle's work-rung body and bookend body,
    named from per-rung flow step 7 and from prose.md's Commit-body form.
  - notes.md's ladder bullet says `<title>` is the rung's commit title and links the type set.

##### docs: the restart and the interlude, between cycles

Two things happen between cycles that the agent-files do not describe. After a close-out wink
exits and restarts the agent, so the next cycle opens on a fresh context, and a restarted agent
knows only what is written down. And an interlude, a docs or planning commit on the trunk line
with a patch bump, has its facts spread over four places, no decision rule, and the synced
set's "development is never done on `main`" reads as forbidding it.

- Close-out gains a step, **Restart**, after Land.
  - The user restarts the agent, and before the exit anything the next agent needs is recorded
    where reacquaint will find it.
  - One line plus a link. The why (context degrades, nothing in flight, an agent cannot restart
    itself) goes in rationale.md under the mirrored heading.
  - Pinned, not custom.md: family-universal.
- The interlude is defined in jj.md beside the trapezoid recipe, or retired.
  - The recipe already holds the load-bearing fact: a trapezoid's `<base>` is the parent of the
    ladder's first rung, because an interlude sits on the trunk line.
  - If interludes are retired instead, the "never on `main`" sentence stands as written.
- The decision rule is added: when unplanned work is an interlude rather than a rung appended
  to the running cycle.

##### docs: Todo entries as headings, and sweep the retired names

Numbered Todo entries make numbers unstable and titles unlinkable: ranking two entries
renumbered 15 and invalidated references written minutes earlier. And live entries in `TODO.md`
and the backlog still ask in vocabulary the repo retired. The heading question is decided first,
since headings make a citation a link, and the sweep runs on the result.

- What carries an entry, to decide:
  - `###` headings per entry: anchors, links, bold title enforced structurally. The cost is the
    `^\d+\. ` numbering machinery and its validators.
  - GitHub issues: a real tracker, and it would also subsume the mailbox.
  - A database: issues' costs plus a tool.
  - The crux is doctrine. Durable context lives in committed files, so headings are a change
    inside the doctrine and the other two are a change to it. A narrower issues route,
    cross-member coordination only, may survive that objection and is worth separating.
- Cite by title, not number, decided.
  - The rule goes in notes.md beside Reference numbering, and a number may ride along as a hint.
  - The precondition is met in the backlog: every entry bold-titled, none duplicated.
  - Remaining: the rule, the citation sweep, and the uniqueness check once validate-anchors
    lands.
- The sweep, each species a grep:
  - `bot-session`, `--scope=bot`, `repos.bot`, `validate-bot` (now `agent`)
  - `.vc-config.toml` and `[workspace]` (now `.vc-config.md` and `[repos]`)
  - `custom-family.md` (gone)
  - `cycle-protocol.md` / `cycle-checklists.md` (AGENTS.md's Cycle protocol)
  - Preparation / Work / Close-out, Work-N, Ladder (sub-cycle) (opening / commits / closing,
    local ladder)
  - bot repo, the bot (agent-repo, the agent)
  - member names (the `[family]` table)
  - backfill, chores move, `## Done` entry (the cycle-record)
- An entry whose whole ask is met by a pinned rule dissolves, with this block's deliberation
  saying so. History lines keep the names they had.

##### docs: move the satellites' inline whys

rationale.md holds AGENTS.md's whys, and prose.md and notes.md still carry `**Why:**` blocks
inline, with jj.md's whys in prose.

- They are swept into rationale.md per-file sections, one heading per rule, mirrored, leaving
  `[why](rationale.md#<slug>)` links behind.
- The rung runs after every rung that changes rule text, so each satellite is swept once.

##### chore(template): propose the set to the payload

The payload is the family's agreed state, and this cycle's result reaches it only once the
other members agree, which this cycle cannot run by itself. And AGENTS.md's "custom.md" section
still reads as if custom.md were where a member puts its changes, when the family dogfoods the
set: a member edits the agent-files directly and the diff is the proposal, and overrides are for
users of the set, not its authors.

- The custom.md section and its rationale.md entry are restated to that model, keeping
  custom.md and its precedence for downstream users, since it is the model this rung acts under.
- The set is proposed to iiac-perf and zc-ring-x1 and their response recorded here.
  - The copy into `vc-x1-template/work` and the re-sync happen once they agree.
  - The three-way comparison going empty is the acceptance check of the proposal.
- The template's own fossils are fixed while there:
  - `jj-tips.md` glosses `@..` wrongly and sits outside `work/`.
  - `.vc-config.toml` is the pre-0.75.0 schema, to be regenerated as `.vc-config.md`.
- Governance, decided: the template stays agent-less, wink writes and pushes at convergence
  moments, recorded in the acting member's records.

##### docs: the family agent-files proposal closing

Closing out the cycle.

## Todo

 Entries are in **strict priority rank**, #1 highest,
 descending. Reprioritize by moving an entry, then
 `vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
 The numbers are positional rank, not stable IDs, so to refer
 to a Todo, name it by its **title** (a greppable mention,
 since a numbered list item has no anchor to link to), not its
 number. Long-tail entries
 live in [todo-backlog.md](notes/todo-backlog.md). Use the
 [Prose form](/agent-data/prose.md#prose-form). Deeper
 detail goes in `notes/chores/chores-NN.md` design
 subsections (link via `[N]` ref).

1. **The vc-config program: finish the surface, then shrink it.** The markdown carrier
   landed and the rest of the config subcommand's work was spread across six entries. They
   share one surface, one schema and one set of tests, so they run as one program rather
   than six rankings. Sub-entries keep their bold titles, so a citation by title still
   resolves.
   - **Finish the vc-config surface (the five rungs deferred at the 0.78.8 early close).** The
     markdown carrier landed and the cycle closed early for the 0816-proposal agent-files work,
     leaving the surface's completion as its own cycle. The deferred acceptance items ride with
     it: agent vocabulary with old spellings rejected (a test shows the fix-it),
     `config --refresh` with `--check` clean on both sides, `validate-anchors` clean over the
     records, and the `.agent-session` repoint end to end.
     - **feat: agent naming in config and CLI** (moved 2026-08-21 into the "docs: empty
       custom-family into the pinned set and config" ladder, ahead of its schema-tables rung,
       so the new tables are born under the new names, and the notes below ride with it):
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

   - **Fix `vc-x1 config`'s rendering: print once, and write with `--output`.** (wink, 2026-08-21)
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

   - **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot,
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

   - **Drop the global config and the account notion.** vc-x1 loads a user-level
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

   - **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
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
       the start, and value checks land later as ordinary tier-1 findings
     - decide there: whether `--refresh --check`'s difference exit joins this scheme (a
       difference is a finding, not a fatal) or keeps its own

   - **config: extract flag-backed key descriptions from Clap.**
     `config`'s key descriptions live in `config_schema.rs`
     (`doc`/`used_by`). For the handful of keys that map 1:1 to a
     CLI flag (`bot-session.col-width` <-> `--col-width`,
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

2. **validate-repo-data.** Golden ids for a fixture repo, so a
   jj-lib bump that moves the on-disk data fails loudly instead
   of building green. The gate at `0.78.0-4` refuses on a version
   mismatch precisely because we cannot tell whether the data
   moved. This is the check that could eventually tell us, and
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
     is a sample, so say so where it is documented rather than
     letting it read as proof.
   - **Watch operation ids and view ids first.** Those are jj's
     own content-addressed op-store hashes, so they move if
     hashing, serialization, or a stored field's meaning moves.
     Commit SHAs are gix's, computed from commit content, so they
     mostly pin git rather than jj and are the weaker signal.
   - **Change ids are goldenable, and are the best canary in
     the set.** Three cases:
     - a commit authored in jj gets a random chid (`JJRng::new_change_id`)
     - a git commit carrying a `change-id` extra header keeps the original
     - a git commit without one gets a *deterministic* chid, the commit id's bytes `4..20`
       reversed and bit-reversed (`git_backend.rs`, `synthetic_change_id_from_git_commit_id`)
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
     fast, but this repo can still be a manual proving ground.
   - Read-only commands get the complementary assertion: hash
     every file under `.jj/` before and after, and record which
     ones are genuinely inert. That is the measurement the policy
     names as the way to narrow the gate from "every subcommand"
     to something smaller, backed by evidence.

3. **refactor: trapezoid-push + body-intro validation.**
   `vc-x1 trapezoid-push`, a **subcommand** rather than a flag
   on `push` (decided 2026-07-28), publishes a close-out as a
   non-fast-forward merge, and body-intro validation rides as
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

4. **The validate family: bare `validate` as the umbrella, `validate-artifact` the runner,
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

5. **`squash-push --title` / `--body`.** `squash-push` amends
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
     the message and stamps it, but here the user authors it and
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

6. **Restructure templates: single template repo + fixed bot
   seed manifest.** Replace the separate
   `vc-x1-work-repo-template` + `vc-x1-bot-repo-template`
   repos with the one work-repo template, whose live
   `.claude/` doubles as the bot-side seed source, and retire
   `vc-x1-bot-repo-template`. `vc-x1 init` / `clone` updates
   for the new layout. First up after the refactor program.
   - `--use-template` rule: explicit `CODE,BOT` copies all
     non-hidden files from BOT (unchanged, the escape
     hatch for rich bot seeds), and `CODE` alone seeds the bot
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
     optional, since init already generates the true minimum
     itself).
   - `memory/MEMORY.md` moves from copied to generated:
     it is intentionally empty (seeded only because Claude
     tends to create it otherwise), so init emits it like
     `.vc-config.toml` instead of copying, leaving no "is it
     still empty?" invariant in the template.

7. **ochid: bot-repo location qualifier.** An ochid is
   workspace-relative (`/.claude/<chid>`), so nothing in a
   published commit says *where* the companion bot repo
   lives (vc-x1's is `github.com/winksaville/vc-x1.claude`,
   discoverable only by convention). Anyone cloning just the
   work repo can't resolve bot-side ochids. Design already
   sketched in forks-multi-user.md
   [Per-user bot repos via URL-shaped ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid):
   URL-shaped trailers, plus the complementary
   `.vc-config.toml` repo-index form, and resolver dispatch is
   one rule (URL -> fetch, else workspace-relative), existing
   path-form trailers stay the backward-compatible case.
   - Cheap first rung: declare the companion's URL once in
     the committed `.vc-config.toml` (no trailer-format
     change, so any work-repo clone then knows where the bot
     repo lives). Rides naturally with the refactor
     program's facade-owns-topology stage
     (bot-repo-location config).
   - Link rot + mirroring mitigations are in the same doc
     section.

8. **Version-number protocol is fragile: versions are
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
   workflow. Promoted from Ideas at 0.65.2-0. Slated for
   the cycle after 0.65.2.
   - Live in-repo example (2026-07-24): 0.72.0 was
     pre-assigned to the trapezoid close-out cycle, which
     paused on `support-trapezoid-commits` after `-1`, and the
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

9. **sync follow-up: extract `move-bookmark` command.** The
   "put the bookmark / `@` where it belongs" step at the end
   of sync (reposition logic) is useful standalone (e.g. the
   t1B scenario where `main` is right but `@` isn't on it)
   and deserves an honestly-named command instead of a mode.
   - `vc-x1 move-bookmark` (name open): no fetch, and move `@`
     (and optionally the bookmark) onto a target under the
     same safety rules as sync's reposition step.
   - Sync's final step becomes a call to the same logic.
   - Follow-up to the 0.67.0 single-mode sync cycle.

10. **sync follow-up: retire the hidden `--check` alias, and
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

11. **validate-numbering: rename the pair, check all
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
      Drops the Todo/Bugs special-casing, and auto-covers
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

12. **pre-commit: single rule (no docs skip) + doc validators.**
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
      `notes/`, which is repo content, and the repo-declared-checks idea
      was rejected 2026-07-15 in favor of "run checks
      yourself").
    - This dissolves the docs exception: with doc validators in
      the pre-commit there's always something to validate, so
      the carve-out stops making sense.
    - Its own near-term cycle (chosen over a 0.61.1 insert to
      avoid rewriting published 0.62.0-x history). No version
      pre-assigned. See the Todo "Version-number protocol is
      fragile" on fragile version targets.

13. **vc-x1 push: record uncovered code commits (N:1 code<->bot).**
    Today push assumes 1:1 symmetric WC commits with shared
    title/body. The interop / adoption scenario breaks that:
    the code side is worked single-repo style (commit +
    `jj git push` / `git push`, no `vc-x1 push` in the loop),
    so no bot pairings exist. One bot commit then records
    every code commit not yet covered by a prior `ochid:`,
    via a multi-line `ochid:` per the design in [[11]].
    - Out of scope: the trapezoid close-out, handled
      natively by the in-progress "feat: push merge
      close-out (trapezoid)" cycle, whose N-ochid stamping
      also covers a cycle held local and published all at
      once. This Todo is only the no-bot-pairings interop
      case, and the stamping step's multi-line `ochid:` emit is
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

14. **Run validate-bot at every vc-x1 invocation
    (config-gated).** The check is one jj spawn
    (`jj bookmark list main --all-remotes`), cheap enough
    to run at every execution, noted 2026-07-15 as a
    "could, not should". Design points:
    - locate the bot repo (`<cwd>/.claude` or config,
      which shares the lookup with the refactor program's
      [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology))
      and silently skip when absent
    - severity knob in `.vc-config.toml`
      (`warn|error|off`): unrelated commands (fix-todo)
      warn at most, while push / squash-push / validate-bot
      already have their own handling from 0.69.0-3

15. **CLI reference lives in `--help`, and README owns concepts.**
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
      tables. Delete the tables. That is the drift source.
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
    - Optional enforcement, cheapest first:
      - assert README has no flag-table rows
      - snapshot-test `--help` output so unintended changes surface in review
      - generate the reference from clap and assert the committed file matches. The third
        rhymes with "config: extract flag-backed key descriptions from Clap", the same
        single-sourcing shape.
    - Sweep each section against `vc-x1 <cmd> -h`.
    - Consider regenerating transcripts via support
      scripts (the gen-exmpl pattern) so examples stay
      reproducible.

16. **Stale `/.vc-x1` gitignore line: report it, and a safer revert, if ever.** The 0.78.3
    residue. Existing workspaces keep their `/.vc-x1` `.gitignore` line: never edit the
    user's file automatically. Report that the line is no longer needed and leave the
    removal to them (which surface runs the check is TBD, and `config --validate` and the
    proposed `validate-repo` are the candidates). Separately, any `revert` reintroduction first
    needs the op-log-derived design: identifiable sync operations, target the parent of the
    run's earliest op, preview and confirm, refuse on intervening non-sync operations.
    Background in
    [chores-16](notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert).

17. **`vc-x1 validate --full`: accept the default by name.** (wink, 2026-08-21) `full` is the
    `[validate]` table `vc-x1 validate` runs and `--fast` names the other, so `--full` should be
    accepted too, unnecessary but allowed, so a reader of a command sees which table ran.
18. **`validate`: enforce the record shapes the agent-files ask for.** (2026-08-25) The
    shapes the agent-files state in prose are missed at the point of action and invisible on
    reread, so each checkable one becomes a validate element and `vc-x1 push` refuses what
    fails. First set:
    - every ladder rung title is `<type>(<scope>)?: <desc>` with a type from the
      conventional-commit set (nine untyped rungs went unnoticed through five rereads on
      2026-08-25)
    - a `--body` is an intro paragraph then `*` facets each with at least one `-` under it, or
      the intro alone for a bookend
    - an unfilled `[[N]]` whose commit is on `main` (the backlog entry
      **`validate`: fail on an unfilled `[[N]]` rung whose commit is on `main`** folds in here)
    - every Todo and backlog entry carries a bold title, unique within its file
    - every `](path#anchor)` and `[N]: path#anchor` in the agent-files resolves against the
      headings (the check that found two dead links on 2026-08-27, run by hand)

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
   `ochid` design is already the substrate, and the cross-links
   make code<->conversation navigable, so the viewer is UI
   over an already-solved data link.
   - Build direction: keep resolution/assembly in `vc`, an
     editor-agnostic Rust engine/lib extending the
     `show` / `chid` / `desc` family ("given a commit,
     resolve its ochid and assemble the paired diff +
     conversation slice"), and the editor add-on is a thin
     presentation layer over it.
   - Front-end leans a Zed add-on (Rust, preferred), maybe
     VSCode / other. Verify Zed's extension API can host a
     rich side-by-side panel before committing, and an
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
   matrix, gap list). The "audit" name undersells it and
   the halves have different lifecycles. And
   `por-dual-parity.md` (the stub) overlaps on parity but
   uniquely holds the `por <-> dual` conversion design.
   - Split the audit doc into a frozen audit snapshot + a
     living design doc (names TBD, and could reclaim
     `por-dual-parity.md` for the design).
   - Refocus the stub to conversion-only and rename (e.g.
     `por-dual-conversion.md`), and drop its redundant parity
     half.
   - Repoint refs (`todo.md` `[1]` + the `por -> dual` Todo,
     `copying.md`, the audit's internal anchors + Reading
     guide) and validate. `chores-10/11/12` mentions are
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
     *commit*, and the index adds within-session granularity:
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
     files are not seeded, and a new project's history is its
     own commits + bot session from day one.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

_Migrated to [done.md](notes/done.md) on 2026-07-24 (the DRY jj facade
cycle and its two docs interludes: template repo names, notes rework)._

- 0.80.4 **docs: reshape at land** [[12]]
  - Land is a five-step permanence sequence and the close-out only chooses and records the
    shape, so a published bookmark is never moved sideways to carry a reshape
  - a landing's pushes are pushes under hard rules 2 and 3, with closing words before the
    final one
  - the version scheme advances patch by default, minor by the user's call at an opening
  - AGENTS.md's Terminology names Land, Trapezoid and Artifact, and rationale.md's Terminology
    keeps only what argues rather than defines
  - jj.md says how to read a change in gitk at full context, linked from README.md
  - custom.md is back to its shell and the dogfood log is retired
- 0.80.3 **docs: fix dev artifacts** [[13]]
  - the dev-name rename is the Opening's own step and the trapezoid recipe restores the plain
    name before the reshape, so a bookmark builds as `vc-x1-dev` and `main` as `vc-x1` by rule
  - the Cycle term says when a cycle is single-step or multi-step, and development is not done
    on `main`
  - the commit-body form pairs every `-` under a `*`, scopes the intro to this commit, and
    makes a bookend body a pointer, with the form cued at the describe step
  - Close-out shapes says how to preview a squash before choosing

_Migrated to [done.md](notes/done.md) on 2026-08-23 (the 0.80.1 and 0.80.2 pair: fold the
cycle agent-files into AGENTS.md, halve AGENTS.md into rationale.md)._

_Migrated to [done.md](notes/done.md) on 2026-08-22 (the 0.80.0 entry: empty
custom-family into the pinned set and config)._

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

[1]: #docs-the-family-agent-files-proposal-opening
[2]: #docs-sync-the-agent-files-to-zc-ring-x1s-set
[3]: #docs-state-the-cycle-protocol-once
[4]: #docs-one-todomd-closed-as-the-history
[5]: #docs-widen-typeable-punctuation-to-prose-style
[6]: #docs-the-restart-and-the-interlude-between-cycles
[7]: #docs-todo-entries-as-headings-and-sweep-the-retired-names
[8]: #docs-move-the-satellites-inline-whys
[9]: #choretemplate-propose-the-set-to-the-payload
[10]: #docs-the-family-agent-files-proposal-closing
[11]: /notes/forks-multi-user.md
[12]: /notes/chores/chores-18.md#docs-reshape-at-land
[13]: /notes/chores/chores-17.md#docs-fix-dev-artifacts
