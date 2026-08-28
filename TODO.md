# Todo

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, and reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: finish the vc-config surface

#### Problem

The config subcommand's surface was left unfinished when the 0.78.8 cycle closed early for the
agent-files work. `vc-config-test.md` is maintained by hand beside a schema that build.rs already
knows in full, `init` still writes `.vc-config.toml` so every new member starts on the legacy
carrier, nothing checks a config file beyond its key spellings, no check resolves the records' own
anchors, and the agent side is still `.claude` in a vocabulary that renamed everything else to
agent.

#### Solution

Run the rungs the early close deferred, one per remaining gap: generate the model and emit the md
carrier from it at init, add `validate-anchors` so the records' links are checked rather than read,
point it at a config file so `config --validate` catches a broken link, a missing required key, and
a missing table, and repoint the agent side at `.agent-session`. The config work spread across the
two lists is groomed first, so the ladder is written against a list that agrees with itself.

#### Acceptance check

The items the early close deferred, made runnable:

- `vc-config-model.md` is generated, and a test finds every schema key in it, so completeness holds
  without a hand count.
- `vc-x1 init` emits `.vc-config.md` for a new workspace, and no `.vc-config.toml`.
- `vc-x1 validate-anchors` reports clean over the records.
- `vc-x1 config --validate` reports both sides clean, and it reports findings on the `.vc-config.md`
  this cycle inherited, whose `[[3]]` through `[[7]]` have no definitions and whose `[2]` points at
  an anchor the agent rename killed.
- `repos.agent` resolves to `.agent-session` end to end.
- Agent vocabulary rejects the old spellings with a fix-it, already met and re-checked here.

#### Ladder

- [feat: finish the vc-config surface opening][1] (done)
- [chore: groom the config backlog][2] (done)
- [chore: generate the config model and seed init][3] (done)
- [fix: init takes a URL or a path][8] (done)
- [feat: add validate-anchors][4]
- [feat: check a config file's links and keys][5]
- [chore: point config at .agent-session][6]
- [feat: finish the vc-config surface closing][7]

#### Deliberation

- Provenance: the first sub-entry of **The vc-config program: finish the surface, then shrink it**,
  moved here whole.
  - The entry's other sub-entries keep their ranks, and **Drop the global config and the account
    notion** runs after this cycle by its own note, since a mechanical schema check is what makes a
    schema shrink cheap.
- The agent-naming rung is already landed, so the five deferred rungs are four.
  - Verified 2026-08-28: the schema carries `repos.agent` and `agent-session.*`,
    `legacy_vc_config` rejects the `bot-` spellings, and a test asserts the printed fix-it.
  - The entry records why, the rung was moved into the 2026-08-21 ladder ahead of its schema-tables
    rung so the new tables were born under the new names.
  - The `bot_*` identifiers still in `src/` are internal names rather than the rung's subject, and
    belong to the backlog's **"Stop saying workspace in user-facing surfaces" sweep**.
- The prose is not machine-owned after all, so a config file is checked rather than regenerated
  (wink, 2026-08-28, re-planned two rungs in).
  - The plan was a renderer owning every config file's prose, with the work-side `.vc-config.md`
    byte-identical to what it emits. That cannot hold: our file carries a `[family]` table and a
    `[validate]` table that are this project's own, while a model shows typical values.
  - Making it hold would mean the renderer reproducing every adopter's file word for word, which
    costs each adopter the ability to explain its own config in its own words. Ours opens with an
    intro paragraph worth keeping.
  - Every defect the inherited `.vc-config.md` actually has is check-shaped: five citations with no
    definition, one anchor the agent rename killed, and a paragraph proposing this very rung. A
    checker finds the first two, and no renderer would have found the third, it would have
    overwritten it without a word.
  - So the 2026-08-10 ownership model is half reversed. Fence interiors stay the workspace's own,
    and now the prose does too.
  - Costs accepted: **feat: add config --refresh** loses its subject and leaves the ladder, since
    regenerating the model and seeding a new file are the model rung and `--check` is `--validate`.
    The same day's "one verb, not two" decision goes with it, and **Fix `vc-x1 config`'s rendering**
    keeps `--output` as its own open question, its other bullets standing on their own.
- The checks extend `config --validate` rather than minting a `validate-config` subcommand.
  - **The validate family: umbrella, runner, and `validate-work`** already routes config checking
    through `config --validate`, which `validate-work` calls, so a new sibling would need absorbing
    by that entry the day it landed.
  - If the name is wanted it is a rename inside that entry, not a second surface opened here.
- The cross-file `[N]:` stretch stays out of validate-anchors (wink, 2026-08-28).
  - Same-file anchors plus `[N]` definition and use matching is the slice the entry states.
  - The cross-file half is what the agent-files anchor check in **`validate`: enforce the record
    shapes the agent-files ask for** wants, and that check waits on the family's answer to the
    agent-files proposal, so building for it now would pin a shape still under review.
  - The backlog's **Reference defs: go file-relative, with anchors** names the same cross-file check
    as what would keep its sweep honest, so it inherits the same wait.
  - The config file's own cross-file links need no stretch: it cites `vc-config.md#<key anchor>` by
    construction, and build.rs already derives those anchors from the key paths, so "does this link
    resolve" is a schema lookup rather than a markdown crawl.
- The grooming is a rung rather than part of the opening (wink, 2026-08-28).
  - The list edits touch `notes/todo-backlog.md`, which carries 39 semicolons and 31 untypeable
    characters, and a commit that edits a file converts that whole file's prose
    ([Semicolons](agent-data/prose.md#semicolons)).
  - A 70-site sweep would swamp the opening's diff at review, and splitting the sweep from the
    grooming would leave the two folded entries in both files for a rung or two.
- Ordering: the list settles first, the generated model before its consumers, the repoint last.
  - The model has to exist before the checks have a statement of what a config file should carry.
  - The repoint changes `repos.agent`'s value, which the model and init's fixtures both carry, so
    it follows the rungs that write them.
- `## Closed` reads `_None._` between cycles (wink, 2026-08-28).
  - No rule states a placeholder for the gap between a cycle's opening and the next closing, and
    every other empty section in the file carries one.

#### Ladder details

##### feat: finish the vc-config surface opening

The cycle's setup commit: bookmark, the Waiting check, the In Progress block, the bump, the rename.

- Bookmark: created and published at `main`'s tip `a4309084`.
- Waiting: neither entry's condition is met. Neither outbound proposal record in `../vc-x1-messages`
  carries a `read:` field yet, and `vc-x1 closed` does not exist.
- Moved here: the vc-config program's first sub-entry, whose four undone rungs are this ladder's
  work rungs.
- Bump: a patch, 0.80.6-0, and the package is `vc-x1-dev`.
- `## Closed` reads `_None._`, the last cycle's block now held only by the landmark commit
  `a4309084`.

##### chore: groom the config backlog

Config work is spread across `## Todo` and the backlog, two entries dead and two already claimed by
other entries, so the lists disagree with themselves about what config work remains. Groom them, and
convert `notes/todo-backlog.md`'s prose as the touch obliges.

- Deleted as dead: **Add a vc-x1 validate-repo?**, a bare heading duplicating the entry below it,
  and **Layered config precedence (user -> workspace -> CLI)**, whose user tier **Drop the global
  config and the account notion** removes and whose worked example named a key the schema no longer
  has.
- Retired: **Add `validate-repo` subcommand** into **The validate family: umbrella, runner, and
  `validate-work`**, which already said it absorbed it, carrying the implementation note and the
  chores-06 design link.
  - Two of its items did not survive. The chores-to-commit consistency check went with the chores
    record form that held the `Commits:` lines it compares, and the exit code as a count of failed
    checks is replaced by the umbrella's "the worst of them". Both drops are recorded in the
    receiving entry, so a later reader is not left to wonder where they went.
  - Its punctuation-baseline finding went instead to **`validate`: enforce the record shapes the
    agent-files ask for**, which is where the checkable shapes live.
- The `## Todo` stub of **vc-config.md per-key worked examples** needed no edit. It left the file
  with the sub-entry the opening moved, so the backlog entry is the only copy already.
- Repointed: the surviving mention of `validate-repo`, in **Stale `/.vc-x1` gitignore line**, now
  names `validate-work`. The mention the plan expected to repoint, in the validate-anchors
  sub-entry, had already left with the moved sub-entry.
- Amended: **Test-tempdir override resolution chain** drops the user-config tier from its
  resolution chain, and **OSC 8 hyperlinks in `config` TTY output** drops its wait on a cycle that
  has landed.
- Folded into **The vc-config program**: **init distributes vc-config.md, reference-base at the
  member** and **Config provenance names the schema, not just the binary**, each carrying a line
  naming the rung whose code it shares.
  - `notes/chores/chores-16.md` says the backlog keeps the second of them, which this stales. Frozen
    history is never amended, so that sentence stays as the record of what was decided then.
- The sweep the touch obliges: 35 semicolons and 24 untypeable characters, the counts after the
  deletions took the rest with them.
  - Two headings moved their anchors, `--recheck` and `por -> dual`. Neither had an inbound link,
    checked before converting.
  - Two typos fell out of the rewrapping, "hard- reject" and "facade-owns- topology".
  - `[16]` lost its last citation with the retired entry, so its definition goes too.
- One fix outside the subject: **`vc-x1 validate --full`: accept the default by name** and the entry
  after it had no blank line between them.

##### chore: generate the config model and seed init

`vc-config-test.md` is maintained by hand beside a schema build.rs already knows in full, and `init`
still writes `.vc-config.toml`, so every new member starts on the legacy carrier.

- Decided at the entry: the model is generated rather than maintained, so "contains every key" holds
  by construction. It is a specimen and init's seed, not a comparison target, per the re-plan.
- The model is the artifact and its generator is test-only. `vc-config-model.md` is committed, and
  `render_model` lives in a `#[cfg(test)] mod model` that writes it under `VC_X1_UPDATE_MODEL=1`,
  with a test failing when the committed copy falls behind the schema.
  - Nothing in the shipped binary reads a model, so shipping the generator would have been dead
    code behind an `allow`, and the golden-file arrangement says the same thing honestly.
  - Two further tests carry the properties the file exists for: every workspace-side key reaches
    it, and no user-home-only key does.
- Init seeds from the same schema rather than from the model file. Its `render_vc_config` now emits
  markdown, one `toml` fence around the active `[repos]` block and the commented key surface it
  already generated.
  - Not the model verbatim: a model shows every table live, and seeding a new workspace with this
    repo's `[family]` and `[validate]` values would hand it configuration it never chose. The
    commented-optional shape init already had is the right one, and only its carrier changed.
  - `--config <path>` copies under the name the source's carrier calls for, `.md` to
    `.vc-config.md` and anything else to `.vc-config.toml`, since naming a TOML file `.md` hands
    the markdown filter a file with no fences and yields an empty config.
- Rides here: the `homes` correction dropping the agent side from the three session keys (nothing
  reads it, `bot_session` resolves them from the work-side root), and the `.vc-x1` leftovers,
  `.claude/.vc-x1` and the work `.gitignore` line. Removing our own line is not the automatic edit
  of a user's file that **Stale `/.vc-x1` gitignore line** bars, which is about other workspaces.
- Two small extractions the rendering wanted: `toml_simple::parse_array` out of `toml_get_list`, so
  a value that loads is a value the model can show, and `wrap_hash_comment` generalized to
  `wrap_prefixed`, since a markdown bullet wraps the way a TOML comment does.
- A defect the hand-check found: a generated config ended with a bare `[family]` and `[validate]`,
  two table headers with nothing under them. `render_optional_keys_block` wrote a section's header
  before deciding whether the key had anything to render, so a section whose keys all lack defaults
  left its header behind. It predates the carrier change and shipped in the toml files too. Fixed
  by skipping the key first, and `config_generated_has_no_empty_table` pins it.
- The inherited `.vc-config.md` keeps its broken links on purpose. Its `[[3]]` through `[[7]]` have
  no definitions and its `[2]` points at an anchor the agent rename killed, and those are the
  fixture the acceptance check names for **feat: check a config file's links and keys**. Only the
  paragraph naming the deleted `vc-config-test.md` changed, which also retires the note proposing
  this rung.
- Finding, not fixed here: `.vc-config.toml` is named in a dozen doc comments, two user-facing
  error messages in `common.rs`, and four README rows, all describing the carrier rather than
  init's output. The drift predates this rung, and the README half is already **CLI reference lives
  in `--help`, and README owns concepts**.

##### fix: init takes a URL or a path

Hand-checking the rung above, wink ran `vc-x1 init tmp/vc-x1-dev.0.80.6-2` and init tried to create
a GitHub repo in an organization named `tmp`. Three defects came out of the two runs that followed,
recorded as `notes/bugs.md` #12, #13, and #14.

- Unplanned work, taken as a rung at wink's call (2026-08-28), since the fix is init's own target
  handling rather than anything the cycle's other rungs touch.
- The `owner/name` shorthand is retired rather than disambiguated, which is the end state **Drop
  the global config and the account notion** decided on 2026-08-21.
  - The first proposal was a heuristic, reading `X/Y` as a path when `X` names an existing
    directory. wink's question, under what conditions `tmp/foo` is ambiguous, dissolved it: always,
    since nothing needs to exist for a path target (init creates missing parents), so both readings
    are well-formed for every such string and the heuristic guesses at intent rather than detecting
    anything.
  - So a slashed target with no path prefix is refused, naming both readings. Refusing rather than
    silently choosing, since silently choosing is the bug. Once nobody reaches for the shorthand a
    slashed target can simply mean the path, and the code says so where the rule lives.
  - This takes a slice of that entry early. What stays there is bare `NAME` and the user-config
    remote chain, which `plan_from_path` also uses, so the config tier is load-bearing for path
    targets and unpicking it is still that entry's work, along with `--account` and `--repo`.
  - `resolve_url` went with the shorthand, since the shorthand was the one form it resolved. That
    also deletes one of the two places a remote is hardcoded to ssh, which was the entry's stated
    reason for retiring the shorthand.
- One name derivation for both target branches. `plan_from_path` ran the directory's `file_name`
  through `derive_name`, the normalization a URL target already got, so a `foo.git` directory
  yields the repo name `foo`.
- A repo name GitHub would rename is refused before it is asked for. GitHub drops a trailing `.git`
  at creation, so `xx1.git` became `xx1` while init wrote a remote pointing at `xx1.git`, and the
  push failed with "the repository exists" as the false half. The guard lives in
  `github_slug_from_url`, the one place on the `gh repo create` path, and refuses rather than
  repairing, since silently renaming what the caller asked for is how the mismatch started.
  - Reachable still by a `--name` override or a URL ending `.git.git`, which is why it is a guard
    and not just a consequence of the derivation fix.
- The ssh remote is recorded as #14 and not fixed here. The remote scheme is the user config's to
  state, so it rides the entry that deletes that chain.
- Two things fell out of touching `notes/bugs.md`: its `# References` heading sat between entries
  10 and 11 rather than at the end, and the file's prose owed the conversion, 27 semicolons and 14
  untypeable characters. The two `…` left are inside a quoted jj error message, transcribed rather
  than authored.
  - The file already records fixed bugs (#1, #4), so the entries fixed here say so in that shape,
    naming the rung rather than a version.
- A wrong diagnosis on the way, recorded because it cost time: the ssh remote looked like the cause
  of the failed push, since "Could not read from remote repository" reads as auth. wink's `gh repo
  list` settled it, showing `xx1` where `xx1.git` had been asked for.

##### feat: add validate-anchors

No check resolves the records' own anchors, so the two dead links the agent-files cycle found on
2026-08-27 were found by hand.

- Decided (wink, 2026-08-28): same-file heading anchors by the documented slug algorithm, plus `[N]`
  definition and use matching. The cross-file stretch is out.

##### feat: check a config file's links and keys

`config --validate` checks key spellings and nothing else, so the `.vc-config.md` this cycle
inherited passes while citing five references it never defines and one anchor the agent rename
killed.

- The anchor checker from the rung above gains a second caller, and the cross-file half is a schema
  lookup rather than a markdown crawl (see the deliberation).
- Two checks ride along: a required key absent for the side, and a schema table the workspace ought
  to carry but does not, which is the completeness the model was going to guarantee by construction.

##### chore: point config at .agent-session

The agent side is still `.claude` while the vocabulary around it renamed to agent.

- wink's between-session move (mv, the config edit, the `.gitignore` edit, `vc-x1 symlink`), with
  the following session committing the record.

##### feat: finish the vc-config surface closing

Closing out the cycle.

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's copy
of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores) and
[notes/done.md](notes/done.md).

_None._

## Waiting

Important work that cannot start yet. Each entry names what it waits on, in a form that can be
checked, and the rank it takes in `## Todo` once unblocked. Every opening checks each condition
and promotes what is met ([Opening](AGENTS.md#opening)).

- **Copy the proposed set into the payload, and re-sync.** (2026-08-28) The cycle **docs: the
  family agent-files proposal** proposed the set to iiac-perf and zc-ring-x1 in
  [agent-files-proposal-0827.md](notes/messages/agent-files-proposal-0827.md#proposal-2026-08-27).
  On agreement: copy `AGENTS.md`, `custom.md`, and `agent-data/*` into `vc-x1-template/work`
  with the two fossil fixes the proposal names (`jj-tips.md`, `.vc-config.toml`), then re-sync
  every adopter. The acceptance check is the three-way comparison going empty.
  - Waits on: a reply from each of iiac-perf and zc-ring-x1 in `../vc-x1-messages`, agreeing or
    amending.
  - Place when unblocked: first.
- **Retire the frozen history: `notes/chores/` and `notes/done.md`.** (wink, 2026-08-27) They
  are frozen and no longer grow, and the agent-repo transcript plus git history hold what they
  hold. Delete both in one cycle, after a sweep turns every link into them (Todo entries, the
  backlog, rationale.md, the other members' messages) into a permalink at the SHA before the
  deletion.
  - Waits on: **`vc-x1 closed "<title>"`** landed, and the session viewer good enough to read
    a cycle's record from the transcript.
  - Place when unblocked: first.

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the [Prose form](agent-data/prose.md#prose-form).
Deeper detail goes in a `notes/` design file (link via `[N]` ref).

### `validate`: enforce the record shapes the agent-files ask for

(2026-08-25) The shapes the agent-files state in prose are missed at the point of action and
invisible on reread, so each checkable one becomes a validate element and `vc-x1 push` refuses what
fails. First set:
- every ladder rung title is `<type>(<scope>)?: <desc>` with a type from the conventional-commit set
  (nine untyped rungs went unnoticed through five rereads on 2026-08-25)
- a `--body` is an intro paragraph then `*` facets each with at least one `-` under it, or the intro
  alone for a bookend
- an unfilled `[[N]]` whose commit is on `main` (the backlog entry **`validate`: fail on an unfilled
  `[[N]]` on `main`** folds in here)
- every Todo and backlog entry is a `###` heading, its title unique within its file
- every `](path#anchor)` and `[N]: path#anchor` in the agent-files resolves against the headings
  (the check that found two dead links on 2026-08-27, run by hand)
- `TODO.md` has the pinned shape: `## Continuation notes`, `## In Progress`, `## Closed`, `##
  Waiting`, `## Todo`, `## Ideas`, `## Bugs`, `# References`, in that order, and the In Progress
  block per cycle-model.md (2026-08-27)
- `## Todo` entries are `###` headings, no numbered entries anywhere in the file, and
  `validate-todo` / `fix-todo` retire, the numbered form they served being gone (2026-08-27)
- a per-file punctuation baseline, since a byte scan cannot be the check: the rule forbids
  *authoring* the four characters rather than their presence, and transcribed tool output and
  published commit titles keep theirs, so record a per-file count and fail when one rises (carried
  2026-08-28 from the retired backlog entry **Add `validate-repo` subcommand**, and it supersedes
  the chores-15 note asking the checker to read its character set from one place, which assumed a
  checkable zero)
- wrapper-level tests for `validate-desc` / `fix-desc` ride along: the analyze cores are covered,
  the wrappers (file I/O, output, exit codes) are not

### The vc-config program: finish the surface, then shrink it

The markdown carrier landed and the rest of the config subcommand's work was spread across six
entries. They share one surface, one schema and one set of tests, so they run as one program rather
than six rankings. Sub-entries keep their bold titles, so a citation by title still resolves.
- **Fix `vc-x1 config`'s rendering: print once, and write with `--output`.** (wink, 2026-08-21) Bare
  `vc-x1 config` prints the schema once per side of the default `work,agent` target, and since every
  remaining key has both workspace homes the two blocks are identical apart from the header, so the
  reader sees the same ~40 lines twice. In a workspace with no agent side the second block is still
  printed, under the `<root>/<agent-dir>/...` fallback hint, for a side that does not exist.
  - print the schema once by default, grouped per side only when the sides' key sets differ (they
    will again once `[family]` and `[validate]` land as `workspace-code`-only)
  - add `--output <scope>:<path>[,<scope>:<path>]` or some such, writing each side's rendering to a
    file instead of stdout, so a side's config can be (re)generated in place. This overlaps `config
    --refresh` in "Finish the vc-config surface" and should be designed with it, one verb or two
  - skip a side the workspace does not have, rather than rendering its fallback hint
  - accept a directory as a path target (wink, 2026-08-21): `config --validate .claude` or
    `../iiac-perf` resolves through the carrier lookup (`config_md::vc_config_path`) to that side's
    config file, the both-carriers error included, and the report labels the side by the directory.
    Today a path target must name the file itself
  - an explicit path must exist (wink, 2026-08-21). Today `config xyz` prints the whole schema for
    "any home" with the path as a label and never opens it, and `config --validate xyz` reports the
    file "not found, skipping" and passes with zero problems, so a typo'd path validates clean. The
    skip is right for a keyword side the workspace lacks, wrong for a path the user typed: error by
    name, and say which file was read
  - the rendered hints still say `.vc-config.toml` (the `VC_CONFIG_FILE` constant), and the md
    carrier rename is the "regenerate configs in md format" rung's

- **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot, 2026-08-12). The
  md carrier costs a config file the toml-aware editors and formatters a `.toml` gets, and nothing
  answers "what do these fences actually concatenate to?", which is also the question a parse
  diagnostic raises. Outside the "docs: freshen vc-config and config subcmd" ladder, whose
  acceptance items do not need it, but ranked here because a format's debugger is worth most while
  the format is new.
  - run the `md_fence` filter over the target file and print the result verbatim, blanks included,
    so the printed line numbers are the source's and a diagnostic's line lands
  - **not `--resolved`**, iiac-perf's word: this subcommand already spends "resolved" on
    effective-after-layering (the `[repos]` resolved-agreement invariant, `resolved_hint`'s
    which-carrier-exists answer), and this is the far end of that, one file's raw extraction before
    any parse or layering
  - it has no existing surface to join: `config` with no flag prints the *schema*, not a workspace's
    values, so nothing today shows a config file's own contents at all
  - decide there: the name (`--toml`, `--as-toml`, `--fences`), and whether it composes with
    `--validate` or excludes it

- **Drop the global config and the account notion.** vc-x1 loads a user-level
  `~/.config/vc-x1/config.toml` whose whole remaining job, once the unread keys go, is expanding an
  `init` shorthand that the `owner/name` and path target forms already cover without it (wink,
  2026-08-11: he passes the full url in practice and a local name only when testing). A config tier
  nothing needs is the same rot as the fossil `[push]` block, so it goes, and the schema drops from
  eleven keys to five.
  - out: `src/config.rs` entire (loader, `UserConfig`, the account map, the `--account` ->
    `[default].account` resolution chain), the `--account` flag, `Context.user_config` and its disk
    read at every subcommand entry, `Home::User`
  - out of the schema: `default.account`, `default.debug` (parsed, logged, never consumed),
    `repo.default`, `repo.category.<cat>`, and both `account.<name>.*` families
  - what remains is five keys in two files: `[repos]` on both sides, `[bot-session]` on the work
    side
  - `homes` becomes `files` with values naming the two sides only, so "user" leaves the vocabulary
    and stops colliding with `account` (wink: a human reading "user" and "account" connects them,
    and here they were unrelated axes)
  - removing `--account` breaks an invocation, so it errors by name and points at this entry's
    record rather than reporting an unknown flag
  - decided 2026-08-21 (wink): `init` takes a URL or a path, nothing else. No bare name, since a
    bare name has no host-neutral meaning and delegating it to `gh` would make the convenience
    GitHub-only. No user tier of any kind: identity is jj's config, credentials are git's helpers (a
    GitLab token in the helper makes `vc-x1 clone` work unchanged), and the remote is the URL. The
    `owner/name` shorthand goes too, since it hardcodes an ssh remote. bugs.md #10 (a pre-created
    GitHub repo rejected at preflight) rides this entry
  - provisioning stays host-keyed from the URL: `gh` for github.com as today, and a `glab repo
    create` arm for gitlab.com is the next one worth adding. Measured 2026-08-21: an authenticated
    push to a nonexistent gitlab.com path is refused ("could not be found or you don't have
    permission"), so push-to-create is not something init can lean on (we think a token with `api`
    scope might allow it, but an instance setting and a token scope are not a foundation). Until an
    arm exists, non-GitHub hosts pre-create both remotes
  - the account model is worth resurrecting if a second repo host ever matters: a backlog entry
    names the cycle that removed it and lets the diff carry the design, rather than restating it in
    prose that can rot
  - runs after the vc-config cycle on purpose: `--refresh --check` makes a schema shrink mechanical,
    so this is the first real customer of the machinery that cycle builds

- **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
  `ExitCode::FAILURE`: a misspelled key and a config the tool could not read exit alike, so a caller
  can branch on "clean or not" and nothing finer. Proposed: **0** all tables and keys known and
  their values reasonable, **1** unknown or otherwise non-fatal findings, **2** a fatal situation.
  The convention is grep's and diff's, so it needs no teaching.
  - the fatal cases already exist and are the subject of bugs.md's **`config --validate` reports "I
    gave up" as a finding** (#9): malformed TOML, an unclosed fence, a side holding both carriers, a
    legacy `[workspace]` schema. Every one of them means the check could not be performed rather
    than that it failed
  - **sequenced after that bug**, which draws the "found something" / "could not check" distinction
    as a local fix. Once drawn, the exit status is a rendering of it, and doing the tiering first
    would mean inventing the classification twice
  - the cost is not in `config`: `main` maps every subcommand error to `ExitCode::FAILURE`
    (`main.rs:477`, `:507`, `:514`), so a distinct code needs the error path every subcommand
    shares. Cheapest to take while that path is open for another reason
  - tier 0's "values reasonable" describes a capability that does not exist: `key_known` compares
    key paths only and no value is ever inspected. Read tier 0 as "keys known" at the start, and
    value checks land later as ordinary tier-1 findings
  - decide there: whether `--refresh --check`'s difference exit joins this scheme (a difference is a
    finding, not a fatal) or keeps its own

- **config: extract flag-backed key descriptions from Clap.** `config`'s key descriptions live in
  `config_schema.rs` (`doc`/`used_by`). For the handful of keys that map 1:1 to a CLI flag
  (`bot-session.col-width` <-> `--col-width`, `--result-lines`), the description could instead be
  pulled from the Clap arg's help via `Cli::command()` introspection, so `vc-x1 config` and `--help`
  share one source and can't disagree.
  - Only ~2 keys map cleanly (most are config-only, flag-sets, or value-providers), so it's a
    partial source and the schema stays authoritative for the rest.
  - Defaults still come from the schema/consts (the args dropped `default_value_t`, so Clap no
    longer holds them).
  - Output format is unchanged, only the text source, so no rework of the 0.71.0-9 rendering.

- **init distributes vc-config.md, reference-base at the member.** (wink + agent, 2026-08-10, folded
  in from the backlog 2026-08-28) `vc-x1 init` seeds a new member with a copy of `vc-config.md` from
  the template payload and stamps `[vc-config] reference-base` with the member's own repo url, so
  every generated doc-reference web link in the member's `.vc-config.md` lands on a copy the member
  owns.
  - the copy is a pinned family file: a member's edits diff against the payload and are its doc
    proposals, folded back at convergence
  - folded here because **chore: regenerate configs in md format** already teaches init to emit
    `.vc-config.md` from the generated model and already owns `reference-base` as the key that
    survives a refresh, so the two are the same code one rung apart

- **Config provenance names the schema, not just the binary.** (iiac-perf + agent, 2026-08-12,
  folded in from the backlog 2026-08-28) The schema is generated at build time from `vc-config.md`,
  so an installed binary validates against its build's prototype rather than the workspace's, and a
  key added after that build is reported unknown with the config blamed. Member repos run a binary
  built from this one, so the exposure is the family's.
  - provenance already prints, keyed to the binary: `--validate` opens with the version banner and
    `print_schema` with its "settable config keys (from ...)" line, so this is one field on two
    lines that already exist rather than a new flag
  - the gap is that a version identifies the *build*, while the question behind an unknown-key
    complaint is whether that build's `vc-config.md` equals the workspace's. A content hash of the
    prototype, baked by build.rs beside the schema and printed next to the version, answers it
    exactly
  - not covered by **Tiered exit status for `config --validate`**, which was asked and is worth
    recording: an unknown key is tier 1 whether it is a typo or a stale binary, so the exit status
    is the same either way. That entry carries severity (could the check run at all) and this one
    attribution (whose fault the unknown key is), and only the second tells a reader whether to fix
    a spelling or to rebuild
  - decide there: a hash or a schema version. A hash is free, exact, and unreadable, while a version
    is readable and someone has to remember to bump it
  - folded here because build.rs is already open at **chore: regenerate configs in md format**,
    which generates the model from the same prototype

### validate-repo-data

Golden ids for a fixture repo, so a jj-lib bump that moves the on-disk data fails loudly instead of
building green. The gate at `0.78.0-4` refuses on a version mismatch precisely because we cannot
tell whether the data moved. This is the check that could eventually tell us, and the route to
relaxing the gate's coarseness. See [the
policy](notes/jj-version-policy.md#how-this-could-be-relaxed). Two modes over one fixture and one id
extractor:

- **Ratchet**, in `cargo test`. Record ids under the current jj-lib, commit them, and let the *next*
  bump re-run them. Zero standing cost, catches drift the moment we take a new version.
- **Live pair**, a `support/` script, not a `#[test]`, so `cargo test` never pays for it. Build a
  probe binary twice, against N-1 and N, run both over the same fixture, diff the reported ids.
  Generate a throwaway manifest in a temp dir for each version rather than adding a crate to our
  lock.
- **Trigger the live pair on the jj-lib bump, not on our release cycle.** Our cycles run faster than
  jj's releases, so per-cycle mostly re-compares the same pair. The bump is when the answer can
  change, and it is also when the answer is most useful: "should we take 0.44?" is a question the
  probe can answer *before* we commit to the bump.
- The probe needs only the storage-facing API: load a workspace, read operation / view / commit /
  change ids, create a commit. That is jj-lib's stable surface. The 0.43 break that motivated this
  whole cycle (`use_glob_by_default` leaving `RevsetParseContext`) was in revset *parsing*, which
  the probe never touches, so keeping it compiling against N-1 should stay cheap.
- **What it does not cover.** It compares two versions *on our fixture*. A change touching a path
  the fixture does not exercise reports "same" and is wrong. A sample, like `jj -V` is a sample, so
  say so where it is documented rather than letting it read as proof.
- **Watch operation ids and view ids first.** Those are jj's own content-addressed op-store hashes,
  so they move if hashing, serialization, or a stored field's meaning moves. Commit SHAs are gix's,
  computed from commit content, so they mostly pin git rather than jj and are the weaker signal.
- **Change ids are goldenable, and are the best canary in the set.** Three cases:
  - a commit authored in jj gets a random chid (`JJRng::new_change_id`)
  - a git commit carrying a `change-id` extra header keeps the original
  - a git commit without one gets a *deterministic* chid, the commit id's bytes `4..20` reversed and
    bit-reversed (`git_backend.rs`, `synthetic_change_id_from_git_commit_id`)
  Build the fixture by importing git commits and every chid is reproducible with no seeding at all.
- That function's doc says "the exact algorithm for the computation should not be relied upon", so
  jj reserves the right to change it. That is a documented instance of the schema-invisible drift
  the gate exists for, and this test is what would catch it: the algorithm moving changes every
  synthetic chid at once.
- **Determinism for the rest.** Operation ids embed timestamps and commit ids embed author and
  committer time, so those still need a pinned clock. Random chids, if the fixture needs any, are
  pinned by the `debug.randomness-seed` config key (`settings.rs`), which arrives through
  `StackedConfig` and so is reachable from jj-lib without going near the CLI.
- **A committed fixture, not this repo.** Using vc-x1's own repo as the guinea pig was the original
  sketch, but its history grows every commit, so the goldens would churn and stop meaning anything.
  A small fixture stays stable and fast, but this repo can still be a manual proving ground.
- Read-only commands get the complementary assertion: hash every file under `.jj/` before and after,
  and record which ones are genuinely inert. That is the measurement the policy names as the way to
  narrow the gate from "every subcommand" to something smaller, backed by evidence.

### refactor: trapezoid-push + body-intro validation

`vc-x1 trapezoid-push`, a **subcommand** rather than a flag on `push` (decided 2026-07-28),
publishes a close-out as a non-fast-forward merge, and body-intro validation rides as the first
rung. See [trapezoid close-out](notes/refactor-20260716.md#stage-trapezoid-close-out) and [push
body-intro validation](notes/refactor-20260716.md#stage-push-body-intro-validation). After jj-lib,
so the reshape is built in-process.
- `push` keeps a stateable invariant: it never produces a merge. A mode flag that rewires the stage
  sequence would cost that.
- Shared implementation, not a second copy: the common pipeline (preflight, both gates, message,
  commit-work, commit-bot, bookmark-set, push-work, bot squash) moves into its own module that both
  subcommands call, with the reshape as the one inserted step. The stateless-push cycle shrinks that
  pipeline first, which is what makes the extraction cheap.
- A backend `trait` (jj today, git or another VCS later) is the natural next abstraction if a second
  backend ever appears. Worth converting these concepts to traits then, not now: we are committed to
  jj, and a one-implementation trait buys nothing but indirection.
- The last stage of the retired jj facade refactor program (its as-built ladder is in
  [refactor-20260716.md](notes/refactor-20260716.md#as-built-trunk-ladder-program-retired-2026-08-18)).
  Parked state at the 2026-08-18 retirement: the published `trapezoid-push-vc-x1` bookmark holds a
  stale opening commit forked off `0.78.2`, with `support-trapezoid-commits` its support line.
  Rebase or restart is decided at pickup.
- At its merge: reconcile with the 0.78.3 single-name convention (chores-16). The branch manifest
  still says package `vc-x1-dev`, which under the convention is a legitimate dev name for its rungs,
  and the merge commit's manifest says `vc-x1`. custom.md's resolution keeps the branch's filled
  copy, with the version-bump line's `cargo update -p` phrased against the manifest's current name,
  and gains the open/close rename step beside the version bump (custom.md on `main` is the bare
  skeleton, so neither has a home until that merge).

### The validate family: umbrella, runner, and `validate-work`

(wink + agent, 2026-08-21) The 0.80.0 cycle shipped bare `vc-x1 validate` running the `[validate]`
table, beside `validate-agent`, `validate-desc`, and `validate-todo`, which are at-rest checks of
repo state. Read as a family the bare name looks like their parent and is not: it runs cargo while
its siblings check bookmarks and records. Supersedes "A committed cycle-check runner" (resolved by
`vc-x1 validate`, whose "not a vc-x1 subcommand" line was decided the other way at that cycle: the
commands live in config, so the tool assumes nothing about the medium) and absorbs the backlog's
"Add `validate-repo` subcommand", retired into this entry 2026-08-28, whose "runs all" is this
umbrella under a name that no longer fits the family.
- rename the runner to `validate-artifact` (`--fast` kept), `validate` rejecting the old meaning the
  way `bot-session` does, and the per-rung flow saying `validate-artifact` per rung and plain
  `validate` at close-out
- add `validate-work`: the work side at rest, the cycle bookmark tracked and at origin, `config
  --validate` clean, mostly the push preflight exposed read-only
- bare `validate` runs everything that applies to the workspace (artifact, work, agent, desc, todo),
  each reported by name, exit status the worst of them, a side the workspace lacks skipped by name
- the `[validate]` config key stays as it is: it is the artifact's validation, and the umbrella
  reads it
- implementation, carried from the retired entry: promote `verify_state_sanity` /
  `verify_completion_sanity` from `push.rs` to `common.rs`, which is the surface `validate-work`
  reads the push preflight through, and the sketch is [the validate-repo
  design](notes/chores/chores-06.md#vc-x1-validate-repo-command-design)
- two of that entry's items did not survive it: the chores-to-commit consistency check, whose
  `Commits:` lines retired with the chores record form, and the exit code as a count of failed
  checks, which the umbrella's "the worst of them" above replaces

### `squash-push --title` / `--body`

`squash-push` amends content only: it folds the working copy into the last commit and force-updates
the remote, but the commit keeps its existing message. Fixing a published commit's *message* is
therefore two steps (`jj describe -r @-`, then `squash-push`). Accepting `--title` / `--body` makes
it one.
- No new risk: squash-push already rewrites a published commit and force-updates the remote. This
  only changes which part of the commit it edits.
- **ochid handling: tell, don't force.** A user-supplied body drops the `ochid:` trailer unless it
  repeats it, which silently breaks the cross-repo link. vc-x1 should *not* inject the trailer
  (unlike `push`, which authors the message and stamps it, but here the user authors it and the tool
  shouldn't rewrite their text). It should error when the new message loses a trailer the commit
  had, naming what would be lost, with an explicit override flag for the case where dropping it is
  intended.
- The content-side guard is the precedent: squash-push already refuses a squash that would drop
  source-only trailers (the 0.65.1 ochid-loss incident). Same check, new input.
- **The guard has a hole the flags would close.** Today the two-step workaround routes around the
  very check that protects the trailer: `squash-push` guards the squash path, `jj describe` guards
  nothing, so the workaround is strictly less safe than the feature. Hit at the 0.77.2 amend
  (2026-07-29), where fixing that commit's own close-out bookkeeping meant editing content *and*
  message, and the trailer survived only by hand-copying it. `vc-x1 fix-desc` can repair a dropped
  ochid by title match, so the failure is recoverable, not silent-forever.
- Amending a just-pushed commit is a real workflow, not a rare one: the cycle-record cites no SHA by
  design, so a rewrite costs nothing. Message fixes naturally cluster there, which is exactly where
  the two-step shape bites.

### Restructure templates: one repo + a fixed agent seed manifest

Replace the separate `vc-x1-work-repo-template` + `vc-x1-bot-repo-template` repos with the one
work-repo template, whose live `.claude/` doubles as the agent-side seed source, and retire
`vc-x1-bot-repo-template`. `vc-x1 init` / `clone` updates for the new layout. First up after the
refactor program.
- `--use-template` rule: explicit `CODE,BOT` copies all non-hidden files from BOT (unchanged, the
  escape hatch for rich agent seeds), and `CODE` alone seeds the agent side from a fixed manifest
  (`LICENSE-*`, `README.md`) taken from `<CODE>/.claude/`. The `<CODE>.claude` sibling default is
  dropped.
- The manifest is the safety property: a live `.claude` has non-hidden session artifacts at top
  level, and the known subset is what lets it double as the seed source without leaking session
  history into new projects.
- Manifest members missing in the source are skipped, so a code template with no `.claude/` content
  yields a bare-but-valid agent-repo (the agent template is optional, since init already generates
  the true minimum itself).
- `memory/MEMORY.md` moves from copied to generated: it is intentionally empty (seeded only because
  Claude tends to create it otherwise), so init emits it like `.vc-config.md` instead of copying,
  leaving no "is it still empty?" invariant in the template.

### ochid: agent-repo location qualifier

An ochid is workspace-relative (`/.claude/<chid>`), so nothing in a published commit says *where*
the companion agent-repo lives (vc-x1's is `github.com/winksaville/vc-x1.claude`, discoverable only
by convention). Anyone cloning just the work-repo can't resolve agent-side ochids. Design already
sketched in forks-multi-user.md [Per-user bot repos via URL-shaped
ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid): URL-shaped trailers, plus
the complementary `.vc-config.md` repo-index form, and resolver dispatch is one rule (URL -> fetch,
else workspace-relative), existing path-form trailers stay the backward-compatible case.
- Cheap first rung: declare the companion's URL once in the committed `.vc-config.md` (no
  trailer-format change, so any work-repo clone then knows where the agent-repo lives). Rides
  naturally with the refactor program's facade-owns-topology stage (agent-repo-location config).
- Link rot + mirroring mitigations are in the same doc section.

### sync follow-up: extract `move-bookmark` command

The "put the bookmark / `@` where it belongs" step at the end of sync (reposition logic) is useful
standalone (e.g. the t1B scenario where `main` is right but `@` isn't on it) and deserves an
honestly-named command instead of a mode.
- `vc-x1 move-bookmark` (name open): no fetch, and move `@` (and optionally the bookmark) onto a
  target under the same safety rules as sync's reposition step.
- Sync's final step becomes a call to the same logic.
- Follow-up to the 0.67.0 single-mode sync cycle.

### sync follow-up: retire `--check`, revisit push's auto-rollback

The first half of this entry (push shelling out to `vc-x1 sync --check`, which was racy and not
actually read-only) is done: 0.77.0-3 deleted preflight outright, taking the shell-out and its PATH
dependency with it. What survives:
- Remove sync's deprecated hidden `--check` alias. Nothing invokes it now except
  `tests/cli_sync.rs`'s alias test, so this became actionable the moment preflight went.
- Push's commit-stage rollback auto-runs `jj op restore`, which hides the evidence of what failed.
  This cycle deliberately kept it, since an in-process snapshot taken moments earlier is knowledge,
  not a guess, and both index-lock failures during 0.77.0 cost nothing because of it. Revisit only
  with a concrete case where the hidden evidence mattered.

### vc-x1 push: record uncovered work commits (N:1 work<->agent)

Today push assumes 1:1 symmetric WC commits with shared title/body. The interop / adoption scenario
breaks that: the work side is worked single-repo style (commit + `jj git push` / `git push`, no
`vc-x1 push` in the loop), so no agent pairings exist. One agent commit then records every work
commit not yet covered by a prior `ochid:`, via a multi-line `ochid:` per the design in [[12]].
- Out of scope: the trapezoid close-out, handled natively by the in-progress "feat: push merge
  close-out (trapezoid)" cycle, whose N-ochid stamping also covers a cycle held local and published
  all at once. This Todo is only the no-agent-pairings interop case, and the stamping step's
  multi-line `ochid:` emit is shared groundwork.
- Teach push to:
  - detect the shape (work WC empty, uncovered commits at the bookmark)
  - skip `commit-work`
  - compose a `.claude`-specific message
  - emit one `ochid:` line per uncovered commit
- Open: computing "uncovered", likely a revset from the work bookmark back to the newest commit
  referenced by the agent journal's ochids.

### Run validate-agent at every vc-x1 invocation (config-gated)

The check is one jj spawn (`jj bookmark list main --all-remotes`), cheap enough to run at every
execution, noted 2026-07-15 as a "could, not should". Design points:
- locate the agent-repo (`<cwd>/.claude` or config, which shares the lookup with the refactor
  program's [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology)) and
  silently skip when absent
- severity knob in `.vc-config.md` (`warn|error|off`): unrelated commands (`desc`) warn at most,
  while push / squash-push / validate-agent already have their own handling from 0.69.0-3

### CLI reference lives in `--help`, and README owns concepts

Each command is described in three places (clap's `long_about`, a README section with a flag table,
and sometimes AGENTS.md) and only the flag *descriptions* self-sync, because those come from the
field doc comments. Every hand-written block drifts silently: 0.69.0-4 found the init section
documenting retired `--owner` / `--dir` / `--repo-local`, and 0.77.0-3 found push's `long_about`
still advertising a state machine that had just been deleted. The fix is removing the duplication,
not auditing it on a schedule.
- `--help` becomes the reference: what a command does, its stages, its flags, its invariants. It
  ships with the binary, so it always matches the binary being run.
- README keeps workflows and concepts (the dual-repo model, the cycle, testing recipes, worked
  examples) and points at `--help` instead of restating flag tables. Delete the tables. That is the
  drift source. The `## Usage` block is the same species: its trailing `#` comments have drifted
  into three columns (40, 43, 44) as commands were added, because the alignment is hand-maintained
  and invisible. Left unaligned at 0.77.2 deliberately, since this entry deletes the block.
- Clap reflows prose and collapses bullets unless a field carries `verbatim_doc_comment`, so help
  owns the reference, not the explanations. `long_about` does preserve explicit newlines (0.77.0-3's
  push stage list renders as an aligned two-column list).
- Optional enforcement, cheapest first:
  - assert README has no flag-table rows
  - snapshot-test `--help` output so unintended changes surface in review
  - generate the reference from clap and assert the committed file matches. The third rhymes with
    "config: extract flag-backed key descriptions from Clap", the same single-sourcing shape.
- Sweep each section against `vc-x1 <cmd> -h`.
- Consider regenerating transcripts via support scripts (the gen-exmpl pattern) so examples stay
  reproducible.

### Stale `/.vc-x1` gitignore line: report it, maybe revert

The 0.78.3 residue. Existing workspaces keep their `/.vc-x1` `.gitignore` line: never edit the
user's file automatically. Report that the line is no longer needed and leave the removal to them
(which surface runs the check is TBD, and `config --validate` and the proposed `validate-work` are
the candidates). Separately, any `revert` reintroduction first needs the op-log-derived design:
identifiable sync operations, target the parent of the run's earliest op, preview and confirm,
refuse on intervening non-sync operations. Background in
[chores-16](notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert).

### `vc-x1 validate --full`: accept the default by name

(wink, 2026-08-21) `full` is the `[validate]` table `vc-x1 validate` runs and `--fast` names the
other, so `--full` should be accepted too, unnecessary but allowed, so a reader of a command sees
which table ran.

### `vc-x1 closed "<title>"`: print a landed cycle's block

(2026-08-27) A landed cycle's record is its `## Closed` block in the landmark commit's `TODO.md`,
and reading it back is two awkward commands (`git log --first-parent main --grep`, then `git show
<sha>:TODO.md`). One verb that finds the landmark by title and prints the block.

### docs: no single-owner assumption in the agent-files

(wink, 2026-08-27) The rules assume the user owns the trunk: Land fast-forwards `main` and deletes
the bookmark, and "`main` advances only when the finished cycle lands on it" states it as a rule.
Everything else, the per-rung flow, approval per push, the cycle-record, the bookmark per cycle,
holds whether the bookmark ends in a local merge or a review request to another owner. Rewrite Land
as "hand the bookmark to the trunk's owner" with two endings: the owner is the user (today's
sequence) or someone else (push the bookmark, open the review request, close the cycle as a `##
Waiting` entry on the merge, delete the bookmark once merged, the long-lived case in jj.md). The
close-out shape then follows the owner's merge policy. Also drop any wording that assumes a single
user, so the agent-files fit anyone's repo as well as wink's. With it, rename the `[family]` config
table and its `family.member` key (2026-08-27: the words left the agent-files' prose for set /
adopter / maintainer, and the key is the last holder), a schema change with the usual fix-it
rejection of the old spelling.

## Ideas

Items not yet solid enough for `## Todo` (or surfaced during close-out / end-of-day before they are
fully formed). Triaged at the next opening: promote to `## Todo` / `notes/todo-backlog.md`, fold
into a picked-up cycle, or drop.

### `vc` as a code+conversation provenance tool (grander ambition)

Today `vc-x1` manages a dual repo (code + `.claude`) cross-linked by `ochid:`. The larger aim is to
*surface* that link: view history with the conversation and the code side by side, giving
provenance, the *why* of a change, not just the *what*. The dual-repo + `ochid` design is already
the substrate, and the cross-links make code<->conversation navigable, so the viewer is UI over an
already-solved data link.
- Build direction: keep resolution/assembly in `vc`, an editor-agnostic Rust engine/lib extending
  the `show` / `chid` / `desc` family ("given a commit, resolve its ochid and assemble the paired
  diff + conversation slice"), and the editor add-on is a thin presentation layer over it.
- Front-end leans a Zed add-on (Rust, preferred), maybe VSCode / other. Verify Zed's extension API
  can host a rich side-by-side panel before committing, and an editor-agnostic core hedges the bet.
- `vc-x2`? A rewrite is unwarranted: the audit's Commonality pass found the architecture sound (por
  is bolted on where an existing good pattern wasn't applied), so equalize incrementally. "vc-x2"
  only makes sense if the viewer changes the *core* architecture (an index / daemon / data model).
  Separate engine-rewrite (no) from product-reposition (open).
- Possible artifact: a top-level `notes/design-cli/vision.md` framing the direction, with the parity
  and conversion docs as sub-designs.

### Restructure the design-cli parity docs (target 0.63.0)

`por-dual-parity-audit.md` (~1200 lines) fuses a *frozen* audit (the `## 1`-`## 8` snapshot
evidence) with a *living* design (axes, decisions, matrix, gap list). The "audit" name undersells it
and the halves have different lifecycles. And `por-dual-parity.md` (the stub) overlaps on parity but
uniquely holds the `por <-> dual` conversion design.
- Split the audit doc into a frozen audit snapshot + a living design doc (names TBD, and could
  reclaim `por-dual-parity.md` for the design).
- Refocus the stub to conversion-only and rename (e.g. `por-dual-conversion.md`), and drop its
  redundant parity half.
- Repoint refs (`todo.md` `[1]` + the `por -> dual` Todo, `copying.md`, the audit's internal anchors
  + Reading guide) and validate. `chores-10/11/12` mentions are historical and stay.
- Promote the Gap-list items to anchored `#### Gap N: <title>` sub-headings so cross-cycle citations
  can deep-link a specific gap (markdown anchors headings, not list items). Trade-off: stable
  anchors, but the ordinal lives in the heading text (manual renumber on reorder), fine for a
  consumed backlog. The 3 `Gap #N` links in the `0.62.0` close-out chores narrative resolve only to
  the section until this lands.
- Deferred from the 0.62.0 close-out: close-out is bookkeeping-only, and the split is substantive,
  anchor-heavy work warranting its own cycle.

### Chores retire into a session index (post-viewer)

Once the provenance viewer ("`vc` as a code+conversation provenance tool" above) can present a
commit's session and code side by side, the hand-written chores narrative is a distillation of a
conversation the agent-repo already records verbatim, so the DRY argument that removed edit lists
from chores (git owns the mechanics) then applies to the narrative too (the session owns it). Chores
collapses to an index into the session.
- The `ochid:` trailer links a work commit to a session *commit*, and the index adds within-session
  granularity: which conversation span produced the commit, where the design argument happened. We
  think it can be generated (the transcript records when pushes happen), making it drift-proof where
  hand-written chores never were.
- What survives: the curated design layer (the refactor-20260716.md pattern). Sessions are an
  immutable journal, good as record and poor to cite into, so live design references keep pointing
  at curated docs, not per-cycle narrative sections.
- The template side already points this way: chores files are not seeded, and a new project's
  history is its own commits + agent session from day one.

## Bugs

_See [bugs.md](notes/bugs.md)._

# References

[1]: #feat-finish-the-vc-config-surface-opening
[2]: #chore-groom-the-config-backlog
[3]: #chore-generate-the-config-model-and-seed-init
[4]: #feat-add-validate-anchors
[5]: #feat-check-a-config-files-links-and-keys
[6]: #chore-point-config-at-agent-session
[7]: #feat-finish-the-vc-config-surface-closing
[8]: #fix-init-takes-a-url-or-a-path
[12]: /notes/forks-multi-user.md
