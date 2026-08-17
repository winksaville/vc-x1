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

### docs: freshen vc-config and config subcmd

#### Problem

Both repos' `.vc-config.toml` carry a fossil `[push]` comment block from an older binary's
schema, the retired `.vc-x1` state dirs linger (empty dirs on disk, a `/.vc-x1` line in the
work `.gitignore`), generated config comments have no refresh tooling so they rot silently, and
the bot dir's `.claude` name is agent-specific where this workspace wants the neutral
`.agent-session`. Underneath those, the toml format itself caps the doc story: a `#` comment
block linkifies nothing, so a reader cannot click from a key to its documentation.

#### Solution

Adopt a markdown carrier for the config surface (wink, 2026-08-11), then rebuild the pipeline
on it. The format: a config file is a markdown document whose `toml` fences, concatenated in
document order, form the TOML the loader parses, so prose, per-key doc, and real reference
links live beside the keys they document. `vc-config-test.md` is the model rendering
(compact: one fence per table, doc-link bullets above it).

- a md -> toml filter (non-fence lines blanked, so any diagnostics keep the source's line
  numbers) feeds the existing parser
  - the loader resolves `.vc-config.md`, still accepts `.vc-config.toml` for the family's
    migration window, and errors when a side has both
- the prototype `vc-config.toml` becomes `vc-config.md` in the same format: per-key `##`
  sections (the anchors the generated web urls land on) absorb `vc-config-design.md`, which
  retires, so the schema source and the browsable doc are one file
- the config surface adopts the agent vocabulary: `repos.agent`, `[agent-session]`, an
  `agent-session` subcommand and `--scope=agent`, old names accepted as aliases; the pinned
  prose sweep is a later convention cycle
- regenerate both sides' instance configs as `.vc-config.md`, retiring the fossil `[push]`
  comment blocks and the `.vc-x1` state-dir leftovers
- `config --refresh` regenerates the prose between fences while preserving fence interiors
  and `[repos]` byte-for-byte
  - a `--check` mode renders and compares without writing, exiting nonzero on any
    difference, so a prototype edit that skipped the refresh fails loudly
- add a `validate-anchors` subcommand, the validate-repo design's first standalone slice:
  same-file heading anchors checked via the documented slug algorithm, plus `[N]` reference
  resolution, across the repo's markdown records
- per-key worked examples in `vc-config.md`, so a reference link rewards the click
- repoint `repos.agent` at `.agent-session`
  - the directory rename itself is wink's move between sessions (a live session writes
    through the symlink), with the following session committing the record

#### Acceptance check

1. `md_to_toml` turns a config markdown file into the TOML the loader parses by
   concatenating its `toml` fences (line count preserved), with tests covering the model
   file's compact shape, the separated per-key shape, and an unclosed fence erroring.
2. The loader reads `.vc-config.md` on both sides, still loads a `.vc-config.toml`, and
   errors when a side holds both; workspace detection finds a root by either name.
3. build.rs generates the schema table and default constants from `vc-config.md`
   (rerun-if-changed wired), the hand-kept `COL_WIDTH` / `RESULT_LINE_CAP` constants are gone
   from `src/`, `vc-config-design.md` is gone, and each key's generated reference link lands
   on that key's `##` section in `vc-config.md`.
4. `vc-x1 config --validate` is clean on both sides, both instance files are `.vc-config.md`,
   and neither mentions `[push]`, `state-dir`, or `state-file`.
5. New surface names work with old ones as aliases: `repos.agent` / `[agent-session]` in new
   files while `repos.bot` / `[bot-session]` still load, and the `agent-session` subcommand
   answers with `bot-session` aliased to it.
6. `config --refresh` on a fixture with stale prose and user-edited fences preserves the
   fence interiors and `[repos]` while regenerating the prose (a test demonstrates it), and
   `--refresh --check` exits clean on both sides.
7. `validate-anchors` runs clean over `TODO.md`, `notes/`, `README.md`, and `vc-config.md`
   (same-file heading anchors and `[N]` refs), and a test shows it catching a broken anchor.
8. After the rename: `repos.agent = ".agent-session"`, a cycle rung pushes from a session end
   to end, new commits still stamp `/.claude/`-labeled ochid trailers, no `.vc-x1` dir in
   either repo, and no `/.vc-x1` line in `.gitignore`, which ignores the bot dir under its
   new name.

#### Ladder

- [[N]] [docs: freshen vc-config and config subcmd opening][33] (done)
- [[N]] [docs: separate work review stop][34] (done)
- [[N]] [feat: vc-config.toml prototype + build.rs codegen][35] (done)
- [[N]] [docs: ladder ToC + per-rung sections][36] (done)
- [[N]] [docs: amend cycle conventions][37] (done)
- [[N]] [feat: markdown config handler][38] (done)
- [[N]] [fix: prompt double echo][46] (done)
- [[N]] [feat: vc-config.md absorbs prototype and doc][39] (done)
- [[N]] [docs: pin the commit-body form][51] (done)
- [[N]] [fix: bot-session reads the md carrier][47] (done)
- [[N]] [docs: config-surface records, bold backlog titles][52] (done)
- [[N]] [fix: validate-desc from the bot side][53] (done)
- [[N]] [docs: trial the iiac-perf convergence proposals][55] (done)
- [[N]] [fix: bump jj-lib to 0.44][57] (done)
- [[N]] [feat: agent naming in config and CLI][40]
- [[N]] [chore: regenerate configs in md format][41]
- [[N]] [feat: add config --refresh][42]
- [[N]] [feat: add validate-anchors][43]
- [[N]] [chore: point config at .agent-session][45]
- [[N]] docs: freshen vc-config and config subcmd closing

#### Deliberation

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
- name allocation: `vc-config.md` is the prototype-and-doc (vc-config-design.md merges in and
  retires), `.vc-config.md` the instance, so the derived web url's filename never changes and
  backlog #52 distributes the file that is both doc and schema source
- agent vocabulary (wink, 2026-08-11): the machine surface flips this cycle, riding the one
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
  story is in [its subsection][34]
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
  rung: [its subsection][36] holds the design
- ladder-to-section links were first declined over hand-computed anchors, which pulled
  validate-anchors into the cycle as a scope stretch ([its subsection][41]); the links were
  later adopted ahead of the checker, table-routed ([37])
- the "feat: per-key doc references" rung was inserted ahead of the regenerate: the why and
  the ordering are in [its subsection][38]
- the [carrier fix][47] was inserted at the absorb rung's review, found by pulling on wink's
  observation that `[repos]` belongs first because both sides use it while `bot-session` does
  not. The same pull exposed the `homes` correction, which rides the [regenerate rung][41]
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
  proposal ([their chores-06 section][48]). The form is [prose.md's][50] now, not restated
  here. The [pin rung][51] goes first so the rungs after it are written under a rule in force,
  which is the dogfood the family's review wants
  - **freeze lifted for this one rung** (wink, 2026-08-12), recorded per the hard rules'
    preamble. Backlog was tried first and failed its own test: a rule binding every remaining
    body cannot live in the file of things we might do
  - **prose.md holds the form, not cycle-protocol.md**: that file's [Body][49] already defers
    body content to prose.md, and the marker typing is prose mechanics
  - the mandatory intro retires bugs.md #7 as a body-shape concern; the bug stays, since a
    caller can still hand `--body` a hyphen-first string
- **the config-surface records rung was inserted at this ladder's second freeze lift** (wink,
  2026-08-12): iiac-perf's capability review needed verdicts, and a verdict with no durable
  home is a claim the mailbox deletes when the message is handled. Backlog was the default and
  failed the same test the commit-body rung's did, so the records land as their own commit and
  the reply cites them. The story is in [its subsection][52]
- the global config and `--account` leave vc-x1 entirely, as `## Todo` #1 rather than a rung:
  wink passes full urls in practice, so the user config's last job is a shorthand that the
  `owner/name` and path target forms already cover. Sequenced after this cycle on purpose,
  since `--refresh --check` makes the schema shrink mechanical
- the first three rungs advanced `main` as they pushed, against rule 13: work `main` moved
  back to 0.78.7 (73319b8c) so the cycle drafts on its bookmark until the trapezoid
  close-out, the bot `main` stayed at its tip, and the premature backfill reverted to
  `[[N]]` (2026-08-10). The rationale is in [the conventions rung's subsection][37]
- the "docs: amend cycle conventions" rung absorbs the cycle's convention work: intent
  subsections and the linked ladder, the cycle definition and bookmark discipline (after
  the main move-back), and the delegation doctrine (after the exceptions discussion),
  rolled into one commit at wink's call: [its subsection][37]

#### Ladder details

##### docs: freshen vc-config and config subcmd opening

- Devise a mechanism for managing vc-config
-  
##### docs: separate work review stop

- the work-review stop ("please review", replacing "ready to commit") now carries no
  description, drafted or final: the description is written only once the work review
  completes, and the user's go is provisional since the review may restart
- sharpened in the per-commit checklist, the protocol's per-commit flow, and the
  bot-communication guidance, so the two reviews cannot collapse into one message
- inserted at the opening's own review, where the bot collapsed the two reviews into one
  message; an agent-file change is its own commit, which is why it is a rung rather than a
  rider on the opening

##### feat: vc-config.toml prototype + build.rs codegen

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

##### docs: ladder ToC + per-rung sections

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

##### docs: amend cycle conventions

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

##### feat: markdown config handler

Problem: a `.vc-config.toml` is edited by the user, and thus the user needs to be able to
thoroughly understand every aspect of it. But a .toml file is limited in its expressivity
as its documentation lives in `#` comment blocks and typical toml renderers do not
allow you to link to local sources of documentation.

Solution: change the config from a .toml file to a markdown file, which is much more
expressive. The prose and reference links live beside the keys, and the tables and key/value
pairs are defined in `toml` code blocks (see [vc-config-test.md](vc-config-test.md), the
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

##### fix: prompt double echo

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

##### feat: vc-config.md absorbs prototype and doc

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

##### docs: pin the commit-body form

- intent: the pinned files said a body was a problem statement then a solution statement and
  nothing about how a body with several sub-problems arranges them, so both repos improvised
  the same shape and neither could point at it
- landed: [Commit-body form][50] is the one home, with [Body][49] and cycle-checklists.md's
  step 7 linking it. A body's *structure* is now a rule where before only its ingredients were
  - **left unpinned**: whether a rung's `## In Progress` edits are a facet. Taken as cycle
    mechanics, on the logic that keeps the file list out; one instance is too few to pin
  - the family's copies stay unedited pending this repo's verdict, so the payload diff this
    rung creates is the reply

##### fix: bot-session reads the md carrier

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
  - so the ordering held for a better reason than the one given: [the regenerate rung][41]
    re-emits these blocks, and a reader who uncommented one after that would have hit silence

##### docs: config-surface records, bold backlog titles

- intent: triaging iiac-perf's `.vc-config.md` review produced verdicts with nowhere to live.
  A mailbox reply's "Done when" cannot itself be the record, since handling the message deletes
  it, so every verdict needed a home before the reply could honestly claim one
- landed: the config-surface half is one bug and three entries. bugs.md gains
  **`config --validate` reports "I gave up" as a finding** (#9), where reading the code found
  `validate` breaking its own documented contract. `## Todo` gains
  **Tiered exit status for `config --validate`** (#5) and
  **`config --toml`: print the TOML a markdown carrier yields** (#6), both ranked on wink's
  call; todo-backlog keeps **Config provenance names the schema, not just the binary** (#55).
  The `toml`-tag escape is documented at [the regenerate rung][41] rather than spot-fixed
  - the notes-file half was not planned and came from the work itself: ranking two entries
    renumbered 17 and invalidated citations written minutes earlier, one already drafted into
    another member's mailbox. That produced
    **Cite a Todo or backlog entry by its bold title, not its number** (#56), its precondition
    swept here (32 backlog entries gained bold titles, none duplicated), and
    **What carries a Todo entry: numbered list, heading, or a tracker outside the repo?** (#57)
    for the structural question underneath, wink's, where the crux is that issues and a
    database both move records out of the repo and so change doctrine rather than format
  - **freeze lifted for this one rung** (wink, 2026-08-12), the same exception and the same
    reasoning as [the commit-body rung][51]: records for a reply that goes out now cannot wait
    on a cycle that has four rungs left
  - the bolding wrapped existing lead phrases rather than rewriting them, so four titles carry
    pre-existing em dashes and arrows that hard rule 8 forbids. Left in place: a punctuation
    sweep hiding inside a bolding pass is how an unrelated change gets missed at review

##### fix: validate-desc from the bot side

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
  - the off-main commit stays as a local, never-published anchor for the interim stable
    `vc-x1 0.78.8` built from it (installed 2026-08-14, verified by wink on iiac-perf); the
    close-out build replaces it and the anchor is then deleted
  - the `0.78.7` backfill of the "docs: consolidate line widths" chores rung rides this rung's
    chores-16 edit, this being the workspace's first push since that close-out landed

##### docs: trial the iiac-perf convergence proposals

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

##### fix: bump jj-lib to 0.44

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

##### feat: agent naming in config and CLI

- intent: the family is retiring "bot" for "agent" on the machine surface, and this cycle's
  config migration is the one moment the flip costs a single transition
  - keys: `repos.agent` and `[agent-session]`, the loader accepting the old names alongside
  - CLI: `agent-session` subcommand with `bot-session` as alias, `--scope=agent` with `bot`
    accepted
  - values are untouched: `repos.agent = ".claude"` until the dir-rename rung, and the ochid
    side label stays `/.claude/` (test-pinned, decoupled from the path)
  - the pinned-prose sweep ("bot repo" -> "agent repo" and kin) is deliberately excluded:
    convention work runs as its own cycle
  - `homes` -> `files` and the `workspace-code` fossil are deliberately *not* here (wink,
    2026-08-11): both collapse when `## Todo` #1 deletes the user config, and respelling a
    value one cycle before deleting it is work done twice. This rung respells
    `workspace-bot` alone, as its own job

##### chore: regenerate configs in md format

The rung's mechanical work: regenerate both sides' instance configs as `.vc-config.md` in the
model rendering (the carriers switched by hand at the handler rung, so this replaces interim
hand-written files with generated ones) and retire the `.vc-x1` state-dir leftovers.
The ownership model settled in the pre-rung discussion (2026-08-10) carries over to the md
form, prose taking the role comment blocks had:

The model file folds in here rather than leading as its own rung (wink, 2026-08-11, at the
ladder freeze): the straw man is drafted at `tmp/vc-config-model.md`, so the spec-first
benefit is already banked and a separate commit would only buy a boundary `--refresh --check`
gives anyway.

- `vc-config-test.md` renames to `vc-config-model.md`, the word the records already use, and
  grows from a format sketch into the full instance
- `.vc-config.md` (work side) is byte-identical to it, and a test renders the work role and
  compares, since an unenforced byte-identity is an intention rather than a property
- the model carries the derived `reference-base` https urls, not relative links: relative ones
  resolve only because this workspace is vc-x1's own repo, and no other workspace has a local
  `vc-config.md` to point at
- only the work side can match it: the agent side's `[repos]` values invert, its doc links
  need `../`, and its opening sentence names the other role
- `vc-config.md`'s "How this file is read" gains the negative half of the format rule: the
  info string must be exactly `toml`, so a fence tagged anything else (` ```toml-example `,
  ` ```text `, untagged) is illustration and never reaches the parser. The filter has always
  worked this way; only the positive half was written down, which left the format reading as
  though every toml block in a config file is live (iiac-perf, 2026-08-12). Rides this rung
  because the model file is where the format's presentation is settled
- the `homes` correction rides here: the three `bot-session` keys drop the agent side, which
  `config bot` advertises and `init` writes into but nothing ever reads
  (`workspace_bot_session` consults the work root alone). Two workspace homes for a behavioral
  key would need a precedence rule between them, and none exists; `[repos]` lives on both
  sides only because its resolved-agreement invariant forces them to match

- ownership model: the instance config is a workspace's only behavior-changing file
  - fence interiors (`[repos]`, active keys) are the workspace's own, preserved by every
    regenerate
  - the prose between fences is machine-owned rendering of the binary's baked schema:
    disposable by construction, never a durable edit surface
- hand edits to generated prose are permitted but ephemeral
  - refresh runs only when invoked, and `--refresh --check` reports divergence rather than
    rewriting, so the file's owner always decides
  - the durable link edit is `reference-base`, an active key that moves every doc-reference
    web url together and survives refresh
- this rung's by-hand regenerate degenerates into a full overwrite
  - safe only because this workspace has no active keys beyond `[repos]` values matching the
    role defaults
  - the general preserve-actives operation is the `config --refresh` rung
- the doc-proposal surface is `vc-config.md`, the prototype-and-doc every generated link
  lands on
  - agreed direction: distribute it as a pinned family file, init stamping `reference-base`
    to the member's own repo, so the link lands on a copy the member owns and may edit as a
    proposal
  - beyond this cycle: backlog #52 (init distributes vc-config.md)
- schema changes stay vc-x1 changes
  - the prototype `vc-config.md` remains vc-x1-only build source, proposals traveling by
    family channel or fork
  - workspace-local keys wait on the `[private]` table proposal

##### feat: add config --refresh

- intent: the generated prose rots silently when the binary's schema moves, and the only fix
  today is a hand regenerate
  - `--refresh` regenerates a file's prose while preserving fence interiors and `[repos]`
    byte-for-byte, a mechanical boundary the md format gives for free
  - `--check` renders and compares without writing, exiting nonzero on drift, so a prototype
    edit that skips the refresh fails loudly

##### feat: add validate-anchors

- the validate-repo design's first slice, standalone: no jj machinery, a markdown scanner
  plus the slug algorithm notes.md documents. Joins the `validate-desc` / `validate-todo` /
  `validate-bot` family rather than starting the full `validate-repo` shell
- checks: every same-file `#anchor` in the covered files resolves to a real heading's slug,
  and every `[N]` use has a matching `[N]:` definition (and vice versa, unused definitions
  reported)
- backlog #24 (the full validate-repo) stays open and absorbs this as its first
  implemented check when it is picked up; the heading-anchor check was missing from that
  design and is recorded here
- the ladder-to-section links this was to enable were adopted ahead of it (the rung intent +
  ladder refs and links rung), so landing this closes their unchecked window
- stretch, from backlog #53: cross-file `[N]:` path definitions, whatever the path form
  (#53 leans file-relative after wink's viewer measurements; the check verifies the target
  file holds the heading)

##### chore: point config at .agent-session

- wink's between-session move sits just before this rung: after the previous rung lands
  and the bot tail is flushed, /exit, then `mv .claude .agent-session`, edit the work-side
  config's `repos.agent` and the work `.gitignore` entry to `.agent-session`, run
  `vc-x1 symlink`, and start the session that commits this rung
- the `.gitignore` edit belongs in the move, not the commit: un-ignored, the renamed bot
  dir would be swept into the work repo's next snapshot

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

1. **Drop the global config and the account notion.** vc-x1 loads a user-level
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

2. **Retire the remaining jj spawns; make the build enforce it.** The refactor program's
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

3. **validate-repo-data.** Golden ids for a fixture repo, so a
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
4. **refactor: trapezoid-push + body-intro validation.**
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
5. **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
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
6. **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot,
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
7. **A committed cycle-check runner.** The per-commit flow's
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
8. **`squash-push --title` / `--body`.** `squash-push` amends
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
9. **Restructure templates: single template repo + fixed bot
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
10. **ochid: bot-repo location qualifier.** An ochid is
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
11. **Version-number protocol is fragile: versions are
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
12. **sync follow-up: extract `move-bookmark` command.** The
    "put the bookmark / `@` where it belongs" step at the end
    of sync (reposition logic) is useful standalone (e.g. the
    t1B scenario where `main` is right but `@` isn't on it)
    and deserves an honestly-named command instead of a mode.
    - `vc-x1 move-bookmark` (name open): no fetch; move `@`
      (and optionally the bookmark) onto a target under the
      same safety rules as sync's reposition step.
    - Sync's final step becomes a call to the same logic.
    - Follow-up to the 0.67.0 single-mode sync cycle.
13. **sync follow-up: retire the hidden `--check` alias;
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
14. **validate-numbering: rename the pair, check all
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
15. **pre-commit: single rule (no docs skip) + doc validators.**
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
16. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
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
17. **Run validate-bot at every vc-x1 invocation
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
18. **CLI reference lives in `--help`; README owns concepts.**
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
19. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
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
20. **Shared-doc sync: per-commit chores convention.**
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
21. **config: extract flag-backed key descriptions from Clap.**
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
22. **Stale `/.vc-x1` gitignore line: report it, and a safer revert, if ever.** The 0.78.3
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
[33]: #docs-freshen-vc-config-and-config-subcmd-opening
[34]: #docs-separate-work-review-stop
[35]: #feat-vc-configtoml-prototype--buildrs-codegen
[36]: #docs-ladder-toc--per-rung-sections
[37]: #docs-amend-cycle-conventions
[38]: #feat-markdown-config-handler
[39]: #feat-vc-configmd-absorbs-prototype-and-doc
[40]: #feat-agent-naming-in-config-and-cli
[41]: #chore-regenerate-configs-in-md-format
[42]: #feat-add-config---refresh
[43]: #feat-add-validate-anchors
[45]: #chore-point-config-at-agent-session
[46]: #fix-prompt-double-echo
[47]: #fix-bot-session-reads-the-md-carrier
[48]: https://github.com/winksaville/iiac-perf/blob/agent-files-model/notes/chores/chores-06.md#commit-body-form-proposal-2026-08-12
[49]: /agent-data/cycle-protocol.md#body
[50]: /agent-data/prose.md#commit-body-form
[51]: #docs-pin-the-commit-body-form
[52]: #docs-config-surface-records-bold-backlog-titles
[53]: #fix-validate-desc-from-the-bot-side
[55]: #docs-trial-the-iiac-perf-convergence-proposals
[57]: #fix-bump-jj-lib-to-044
