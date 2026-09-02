# Todo and cycle record

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

### agent-files(proposal): 0.1.0

#### Problem

The `agent-files` title grammar names the members and the date, so the two landed titles run 59
and 61 characters against Line widths' 50-character cap, and a third member breaks any cap for
good. The member list also repeats the message record's `from:` and `to:` fields. The set itself
carries no version, so which set an adopter holds is answerable only by a diff.

#### Solution

Version the set: an empty file with the version in its name,
`agent-data/agent-files-vX.Y.Z{-suffix}`, bumped by proposal cycles through the suffix scheme and
copied bare by adoptions, the version spelled `vX.Y.Z` wherever it is written. The `agent-files`
title becomes `agent-files(<scope>): vX.Y.Z`, the version its uniqueness token and the member list
left to the message record, and the rule barring versions from titles names this title as its one
exception. vc-x1 reports the workspace's set version in its banner and its version report.

#### Acceptance check

`vc-x1 validate` passes, every title in the ladder is under the cap and the bookends match the
declared grammar, `agent-data/agent-files-v0.1.0` exists at the close and no other
`agent-files-v*` file does, `agent-data/prose.md` no longer holds `<to|from> <member-list>`, and
`vc-x1 -V` run in this repo prints `agent-files v0.1.0` beside its own version while `vc-x1 version`
lists it.

#### Ladder

- [agent-files(proposal): 0.1.0 opening][1] (done)
- [docs: version the set and title agent-files by it][2] (done)
- [feat: report the set version with -V and version][3]
- [agent-files(proposal): 0.1.0 closing][4]

#### Deliberation

- Multi-step (wink, 2026-09-01): the cycle was drafted single-step and converted before its first
  push, while the shape was still free, so the mechanics it proposes run once in full, the set
  version walking `-0`, `-1`, `-2`, bare beside the artifact's, with the dev rename exercised.
  - The draft's docs edits were split into a stash commit beside the line and the opening pushed
    from the working copy, the docs rung restoring them from the stash.
- The reporting rung is inside the subject (wink, 2026-09-01): the feature exists only because of
  the set version, and a reader on the command line has no other way to see the number without
  finding the file, so it rides here rather than as its own `feat` cycle.
- 0.1.0 as the first set version (2026-09-01): the set has had no number, and the first proposal
  under the scheme takes the first minor.
- The `v` spelling, `v0.1.0` everywhere (wink, 2026-09-01), adopted after the opening pushed: a
  bare number after the title's colon could be anything, cargo's own output pairs a name with a
  `v` version while a tool's banner stays bare, and the file became `agent-files-v0.1.0` so its
  name says what it versions. This cycle's own title keeps the bare `0.1.0` its published opening
  carries, the one title under the scheme spelled that way.
- The version in the title, against Versions live in the version-of-record only: the rule's
  reason, a build stamp a renumber can invalidate, does not apply, since the set's version names an
  agreed text and a renumber is the failure it exists to prevent. The exception is written into the
  rule with its rationale rather than left as a recorded bend.
- An empty file over a one-line file (wink, 2026-09-01): the listing shows the number without a
  read, a bump is a rename git tracks as one, and `git log --diff-filter=A` on the pattern is the
  version log. The H1 line and a `## Version` section were rejected for anchor churn at every
  bump.
- The trailer in the file name only (wink, 2026-09-01): a rung title is its ladder line and carries
  no version, so a multi-step proposal's rungs are told apart by the bookends while the file walks
  `-0`, `-1`, bare, as the artifact version does.
- Proposals bump, adoptions copy: a proposal bumps at its opening and its title names the new
  number, an adoption copies the source's bare file and its title names what it took, so the same
  title in two repos means the copy and its origin. Two proposers claiming one number are the
  maintainer's to settle at convergence.
- The version reported is the workspace's (wink, 2026-09-01): the set in the repo vc-x1 runs in,
  read at run time, `none` outside one, never the set vc-x1's own repo carried at build.
- Bookmark by the strict anchor slug, `agent-filesproposal-010`, per jj.md's Create rule, where the
  last cycle used a readable form.
- 0.82.0 (2026-09-01): minor, the last convention cycle's precedent for a convention change.
- The proposal record follows the Land: a reply on the messages repo's agent-files topic, with
  iiac-perf's title-width report as the record it answers. A `status` command and an
  `agent-files {diff|copy}` group, and nesting the `validate-*` commands, were weighed and go to
  `## Todo` as the next cycles.

#### Ladder details

##### agent-files(proposal): 0.1.0 opening

The cycle's setup commit: the bookmark, `## Closed` emptied, this block, the artifact bumped to
its `-0` under the dev name, and the first set version file, `version-0.1.0-0`.

##### docs: version the set and title agent-files by it

The two landed `agent-files` titles run past the title cap and repeat the message record, and the
set has no version.

* The set had no version-of-record.
  - versioning.md's The set's version defines one: the empty `agent-data/agent-files-vX.Y.Z` file,
    proposals bumping through the suffix scheme and adoptions copying, the `v` spelling everywhere
    it is written, and the payload's number the agreed one. AGENTS.md's Terminology names it and
    the Opening and per-rung Bump steps bump it beside the artifact's in a proposal cycle. This
    rung renamed the opening's `version-0.1.0-0` to `agent-files-v0.1.0-1`.
* The title grammar named members and a date, and the Versions rule barred any version from a
  title.
  - prose.md's declaration becomes `agent-files(<scope>): vX.Y.Z`, the member list left to the
    message record, and the Versions rule names the `agent-files` title as its one exception,
    rationale.md carrying why the rule's reason does not reach it.
* The file name and the `v` were settled during the rung, not at the opening.
  - Recorded in the deliberation. The cycle's own title stays bare, its opening being published.

##### feat: report the set version with -V and version

A reader on the command line cannot see the set version without finding a file in a
subdirectory. vc-x1 reads the workspace's `agent-data/agent-files-v*` at run time and prints it in
the
banner beside its own version, and as a line of `vc-x1 version`, `none` when there is no workspace
or no file.

##### agent-files(proposal): 0.1.0 closing

Closing out the cycle.

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's copy
of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores) and
[notes/done.md](notes/done.md).

## Waiting

Important work that cannot start yet. Each entry names what it waits on, in a form that can be
checked, and the rank it takes in `## Todo` once unblocked. Every opening checks each condition
and promotes what is met ([Opening](AGENTS.md#opening)).

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

### The status and agent-files commands

(wink, 2026-09-01) Both repos' state takes two invocations, `jj st -R .` and `jj st -R .claude`,
and the agent-files diff against a peer takes three `diff -s` lines nobody types. `vc-x1 status`,
alias `st`, prints both repos' status under their labels, and is the home of At rest's "clean",
both `@` empty. `vc-x1 agent-files {diff|copy}` joins the `version` subcommand the proposal cycle
adds: `diff` against the template payload by default (`family.template`) or `--against <path>`,
covering AGENTS.md and `agent-data/` and reporting custom.md on one line since the project layer
always differs, and `copy`, inbound only, from the template or `--from <path>`, mirroring
AGENTS.md and `agent-data/` deletions included, never custom.md or TODO.md, the working copy left
uncommitted for the adoption rung's review. A `bump` for the set version's per-rung rename waits
until the scheme has run by hand once.

### Nest the validate and fix commands

(wink, 2026-09-01) Six flat `validate-*` commands, their `-old` variants, and two `fix-*` are a
namespace asking for `validate {bot|desc|config|anchors|todo}` and `fix {desc|todo}`, bare
`validate` staying the full run. The flat names stay as hidden aliases for a while, the validate
table and typing habits using them, and retire in a later cycle.

### Size is recorded only when an agent-file changed

(wink, 2026-08-29) [Close-out step 4](AGENTS.md#close-out) has every cycle record the agent-files
line count in `notes/agent-files-size.md`, so a cycle that touched no agent-file adds a row saying
so, which is a row that records nothing. Two such rows were added, by **feat: finish the vc-config
surface** and **docs: propose the messages rules**, and the second cycle deleted both. Change the
step to record the count only when the cycle changed an agent-file, and say the same in the
file's own preamble. An agent-file change, so its own commit.

- zc-ring-x1's acceptance remark rides here (2026-08-31): the Size close-out step reads as
  vc-x1's habit and has no rationale entry, so this entry's cycle writes the missing rationale,
  or retires the step, when it runs.

### Support POR workspaces in `push`

(2026-08-31) `vc-x1 push` refuses a POR workspace at its first stage (`require_bot_dir`: "this
operation requires a dual workspace"), confirmed by probe at 0.80.7, and the July audit records it
as dual-only ([audit](notes/design-cli/por-dual-parity-audit.md)). Support is auto-detected, not a
flag: `init` needs `--por` because it creates the topology, while `push` reads an existing one, so
`bot_repo_path()` returning `None` is the POR signal. On a POR the bot stages skip and the
`ochid:` trailer is omitted, there being no other side to name. The opening rung is a test pinning
today's refusal, the error names a dual workspace and nothing is mutated, which inverts into the
POR-success test when support lands.

### Get defaults from .vc-config in cli processing

When processing the default parameters for a vc-x1 subcommand look in .vc-config.
For instance `vc-x1 validate-desc` should have a default that is both repos
and their locations are in vc-config::repos.*. This is also a way to determine
if a repo is a dual repo or not.

- Overlaps [Support POR workspaces in `push`](#support-por-workspaces-in-push): `bot_repo_path()`
  reading `repos.agent` is the same dual-or-POR signal that entry names.

### Add support subcommand status of the repos

Implement `jj st` for either or both repos, I do the following command frequently
to be sure the repos are "clean" or not i.e. clean = (empty) (no description set) 

### Enhance squash-push

Display status of both repos and conditionally push if not clean if the status
changed display the final status. A --yes would mean do a push without prompting.

- A near neighbor of [Add support subcommand status of the
  repos](#add-support-subcommand-status-of-the-repos): the status display here could be that
  subcommand's output.

### Write up who owns a config file's prose

(2026-08-28) The cycle **feat: finish the vc-config surface** reversed half the 2026-08-10
ownership model and left the reversal in its closed record, where it is found only by knowing which
cycle produced it. The rule now is that a fence interior and the prose around it are both the
workspace's own, and the tool checks a config file rather than regenerating it, because a renderer
owning every adopter's prose would cost each adopter the ability to explain its own config. That is
a rule about what the tool does rather than a record of what was done, so it wants its own topic
file ([notes/README.md](notes/README.md)), say `notes/config-ownership.md`.

- Ranked first (wink, 2026-08-28). History holds the reversal either way, and the risk it guards
  against is a regenerating config surface being proposed again before the rule is findable.
- The retired **feat: add config --refresh** and the `--output` question still open in **Fix
  `vc-x1 config`'s rendering** are the two places the reversal changed a plan, so the file should
  name them.

### `init` still seeds `.claude` as the agent directory

(2026-08-28) The agent rename covered the vocabulary and this repo's own layout and left `init`'s
default alone, so every workspace it creates lands in the mount collision this repo just left. With
the agent repo at `<project>/.claude`, the harness's ten `/dev/null` bind mounts land inside the
agent repo, and the seeded agent-side `.gitignore` carries only `.git` and `.jj`. We think an agent
working there meets ten untracked device nodes in `jj st`, since the work side's `/.claude` line
cannot reach inside a second repo.

- The default has five sites: `DEFAULT_BOT_DIR` (`src/init.rs:161`), the seeded work config's
  `agent = ".claude"` (`src/init.rs:347`), `GITIGNORE_CODE` (`src/init.rs:470`), the GitHub repo
  name `<name>.claude` (`src/init.rs:986`, `:1032`), and the `example` for `repos.agent` in
  `vc-config.md`, which build.rs regenerates into `vc-config-model.md`.
- Three tests pin it: `src/init/tests.rs:82`, `:125`, `:156`.
- Open question, and the reason this is an entry rather than a one-line default swap: does the
  GitHub repo suffix follow the directory? Repos named `<name>.claude` already exist under the old
  name, so changing the suffix is a decision about published names rather than about a default.
- `vc-x1 push` labels the agent side `.claude` in its pending-changes report while resolving the
  configured path beside it, so the line reads `.claude (<root>/.agent-session):`. The label is the
  string literal at `src/push.rs:377`, and eight doc comments in that file name `.claude` for the
  bot repo generally. Cosmetic, and the same rename not having reached everywhere.
- Found while running **chore: point config at .agent-session**, whose record holds what the
  collision costs a workspace that stays on `.claude`.

### `validate`: enforce the record shapes the agent-files ask for

(2026-08-25) The shapes the agent-files state in prose are missed at the point of action and
invisible on reread, so each checkable one becomes a validate element and `vc-x1 push` refuses what
fails. First set:
- every ladder rung title is `<type>(<scope>)?: <desc>` with a type from the conventional-commit set
  or a project-declared type per [Project-declared
  types](agent-data/prose.md#project-declared-types) (nine untyped rungs went unnoticed through
  five rereads on 2026-08-25)
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
- with the rename, tighten jj.md's Land step 4: it names the act ("promote the artifact from
  `main`") but no command, so a raw `cargo install` was improvised there (2026-08-31). The step
  should name the runner, whose full table ends in the install, so the config stays the single
  source of the commands and the landed tree is re-proved at the commit being promoted. A
  set-level edit, so it rides the next agent-files proposal
- add `validate-work`: the work side at rest, the cycle bookmark tracked and at origin,
  `validate-config` clean, mostly the push preflight exposed read-only
- landed early, 2026-08-28: `validate-config` is out of `config --validate` and into this family,
  with the old flag rejected by name. What is left here is the runner rename and the umbrella
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
- A real case of our own (2026-08-31): the ARM session made six commits in the sibling
  `vc-x1-messages` clone (the read and done marks plus the marked-done revision record, `main`
  at `5cf8aad7`), while its transcript rode vc-x1's agent-repo squash-push with no work-repo
  commit to pair with, so nothing links the two histories. A URL-shaped ochid on the transcript
  commit could have named the messages-repo commit, the same shape as a pull request whose real
  history lives in another repo.

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

### Tool-results land in the agent-repo, tmp/ for the non-durable

(wink, 2026-08-31) The harness persists oversized command output to
`<session>/tool-results/*.txt`, which resolves through the projects symlink into
`./.agent-session`, so those blobs are committed and pushed with the session record at the next
squash-push (session `07191fe5`'s is already tracked). The agent can use, or be directed to use,
repo-local `tmp/` (gitignored) by redirecting chatty commands (`cmd > tmp/<name>.log 2>&1`) when
the output does not need to be durable: both parties inspect the same file and the agent-repo's
history stays lean. Open choices: pin the redirect practice as a `custom.md` line, and whether
the agent-repo should gitignore `*/tool-results/` at the cost of transcript references dangling.

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

[1]: #agent-filesproposal-010-opening
[2]: #docs-version-the-set-and-title-agent-files-by-it
[3]: #feat-report-the-set-version-with--v-and-version
[4]: #agent-filesproposal-010-closing
[12]: /notes/forks-multi-user.md
