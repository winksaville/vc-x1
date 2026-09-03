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

_No cycle currently in progress._

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's copy
of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores) and
[notes/done.md](notes/done.md).

### feat: the status and agent-files commands

#### Problem

Both repos' state takes two invocations, `jj st -R .` and `jj st -R .agent-session`, and the
agent-files diff against a peer takes three `diff -s` lines nobody types. At rest's "clean", both
`@` empty, has no command that answers it.

#### Solution

`vc-x1 status [SCOPE]`, alias `st`, prints the scoped repos' status under their labels in `jj
st`'s shape and a verdict line, `SCOPE` a positional or `-s` completing to `work|agent|both`,
`work` the default and `both` the home of At rest's "clean", `both` a new scope keyword
everywhere, a plain jj repo answering for `work`, and a plain repo nested in a workspace's tree
answering as itself. `vc-x1 agent-files {diff|copy} [A|SRC] [B|DST]` joins `version`: the first
operand is the other copy of the set, else `agent-files.<cmd>.dir`, else `family.template`, the
second this workspace unless given, so two operands work from anywhere. `diff` names each set
file's state and exits non-zero when anything differs, `copy` makes DST's set a byte copy of
SRC's, deletions included, never TODO.md, refuses a DST whose jj working copy already changes the
set, and leaves the result uncommitted. custom.md is the project layer, reported and never
copied, until `-c|--custom` brings it in, `--no-custom` overriding the config. The
`[agent-files.diff]` and `[agent-files.copy]` tables hold `dir` and `custom`, `custom` the
schema's first `bool`. `family.template` names the payload directory. A `bump` for the set
version's per-rung rename still waits until the scheme has run by hand once.

#### Acceptance check

`vc-x1 validate` passes. `vc-x1-dev status both` in this repo prints both repos under their
labels with the `@` and `@-` lines `jj st` prints and a verdict, bare `vc-x1-dev st` prints the
work side alone, and `vc-x1-dev st` in a plain jj repo prints that repo.
`vc-x1-dev agent-files diff` in this repo names AGENTS.md, the four changed agent-data files, the
version file as ours only, and custom.md as the project layer, and exits non-zero, and with `-c`
compares custom.md like the rest. `vc-x1-dev agent-files copy` into a scratch copy of this repo
leaves its working copy equal to the payload in AGENTS.md and `agent-data/`, custom.md and TODO.md
untouched, with nothing committed. With `[agent-files.diff]` setting `dir` and `custom = true` in
the scratch copy's config, a bare `diff` uses both, `diff --no-custom` overrides the one, and an
operand overrides the other, and `vc-x1-dev validate-config` accepts the tables. Ran at the
close with `vc-x1-dev 0.83.0-5`: full validation passed; `st both` printed both repos under
`work` and `.agent-session` with the `@` and `@-` lines and the dirty verdict, bare `st` the
work side alone, and `st` in a plain jj repo that repo, clean; bare `agent-files diff` named
AGENTS.md, notes, prose, rationale, versioning, and the version file as only here, exit 1, and
`-c` against iiac-perf reported 0 of 11 differ; in a scratch clone, `copy` from the payload
applied six steps and the diff after reported 0 of 9, custom.md and TODO.md untouched, nothing
committed; with `[agent-files.diff]` setting `dir` and `custom = true` in the clone's config,
`validate-config` accepted it, bare `diff` took both, `--no-custom` overrode the one, and an
operand overrode the other. Pass.

#### Ladder

- [feat: the status and agent-files commands opening][1] (done)
- [feat: status, both repos' state in one call][2] (done)
- [feat: status completes its scope keywords][7] (done)
- [feat: the agent-files config table][3] (done)
- [feat: agent-files diff against a set directory][4] (done)
- [feat: agent-files copy from a set directory][5] (done)
- [feat: the status and agent-files commands closing][6] (done)

#### Deliberation

- The transcript join entry is promoted first and passed over (wink, 2026-09-02): its condition is
  met and it asked to be first, but nothing blocks on it and the landed trapezoid keeps, so this
  cycle runs the commands the family asked for and the join check follows.
- The payload directory is named by the config, not found by the code: the template repo keeps
  its payload under `work/`, so `family.template` names that directory and the command reads
  AGENTS.md and `agent-data/` under it, with no heuristic about the template's shape.
- Names-only diff: one line per file, "differs", "only here", "only in payload", the project layer
  on one line, and a non-zero exit when the sets differ, since the question the command answers is
  whether a re-sync is a copy. A unified diff waits for a `-p` flag and a caller who wants it.
- custom.md by flag (wink, 2026-09-02): AGENTS.md says the project layer is never universal, and
  the commands keep that as their default, custom.md reported on one line and never copied. A
  family whose custom.md files are pointer-only, as the first adopters' are, wants them identical
  too, so `-c|--custom` includes it in both commands, and `--no-custom` overrides a config that
  sets it.
- The set directory is a positional operand (wink, 2026-09-02): `diff DIR` reads as `git diff
  <ref>` does, and `copy DIR` names its source the way `git pull <remote>` does, the `cp` idiom
  of a trailing destination answered by the command being inbound only and printing its resolved
  source first. The operand is optional, resolving positional, then config, then
  `family.template`, so the config keys are named for the operand, `agent-files.diff.dir` and
  `agent-files.copy.dir`, not for flags that no longer exist.
- Typed config tables over a list of default arguments (wink, 2026-09-02): `[agent-files.diff]`
  and `[agent-files.copy]` each hold `dir` and `custom`, resolved as the agent-session keys are,
  flag, then workspace config, then built-in, and `custom` adds a `bool` kind to the schema, which
  had none. A `diff = ["--custom"]` list of arguments merged into argv needs no new kind but
  escapes validation and the generated config's docs, and invites every future flag in as a
  string.
- Two operands, redone on the draft (wink, 2026-09-02): the diff and copy commands were pushed
  with one `DIR` operand and this workspace implicit. A bare `copy` then rewrote this repo's
  set when a guard was expected to refuse, and the implicit destination read as the cause: an
  explicit `SRC DST`, and `A B` for diff, is obvious and opens the outbound case, the maintainer
  folding an adopter's set into the payload from anywhere. The pushed diff rung was amended in
  place and re-described with its trailer kept, the bookmark force-pushed, and the copy rung
  finished on the new shape, the branch being a draft for exactly this. The versions kept their
  numbers.
- The At rest edit waits for its own cycle (wink, 2026-09-02): pointing AGENTS.md's "clean" at
  the command is an agent-file change, and Own commit, own cycle holds. It was drafted as a
  rung of its own, then folded into the status rung as a one-line pointer, then taken out again
  to run as a convention cycle after this one lands, with the other close-out entries in
  `## Todo`.
- The older entry **Add support subcommand status of the repos** is absorbed: this cycle's
  `status` is that entry, so it is deleted rather than left to be closed twice, and the
  squash-push entry that cited it now cites this cycle.
- 0.83.0: minor, a feature cycle.

#### Ladder details

##### feat: the status and agent-files commands opening

The cycle's setup commit: the bookmark, `## Closed` emptied, the Waiting entry promoted, the
Continuation notes acted on and reset, this block, and the artifact bumped to its `-0` under the
dev name.

##### feat: status, both repos' state in one call

Both repos' state takes two `jj st` invocations, and nothing prints the verdict At rest asks for.

* The facts `jj st` prints had no in-process reader.
  - The jj facade gains one, a working-copy status of the changed paths with their letter, the `@`
    and parent lines in `jj st`'s shape, and the two bits the verdict is made of, empty and
    described. It snapshots first, as every `@`-relative read does, so the answer is about the
    filesystem now. Renames show as a delete and an add, since nothing here tracks copies.
* The two repos are read one at a time and the verdict is in the reader's head.
  - `status`, alias `st`, takes a scope as a positional or `-s`, `work` by default, `agent`, or
    `both`, resolves the workspace from `-R` or the current directory, prints the work side under
    `work` and the agent side under its directory's name, and ends with one line: `clean`, or
    `dirty` naming each repo and why. The repos come from the shared scope resolver, so `work`
    needs no config and a plain jj repo answers for it, and `agent` outside a dual workspace is
    that resolver's error.
  - `both` joins the scope keywords (wink, 2026-09-02), the same set as `work,agent`, and every
    `-s` in the tool takes it, since the parser is one.
  - The root finder walks up past a nested `.jj`, so a plain repo under a workspace's tree, a
    scratch repo in `tmp/`, resolved to the workspace. `status` stops at the nearest jj repo
    unless it is one of the workspace's own sides, so a nested plain repo answers as itself and
    the agent dir still means the workspace. Found by the scratch run of the acceptance check.
  - Clean is both `@` empty and undescribed: the description is the second bit because an empty
    described `@` is an intent nothing has published, and the verdict says which bit failed.
  - The exit status is success either way. The squash-push entry that wants a machine-readable
    verdict gets a flag when it runs.
* At rest defines "clean" and names no command.
  - Left as it is: the pointer to `vc-x1 status` is the convention cycle's, entered in `## Todo`.

##### feat: status completes its scope keywords

`vc-x1 status <tab>` offered only flags, since a value parser written as a function declares no
values for the shell completer to offer.

* Clap's dynamic completer offers what a parser declares, and `parse_scope` declares nothing.
  - The scope module gains a parser that declares `work`, `agent`, and `both`, wrapping the same
    parse, and status's positional and `-s` use it, so the completer offers the three, a partial
    `b` completes to `both`, a bad value lists them, and the help shows them under `SCOPE`.
  - The other `-s` flags keep the function parser, since they also take the spelled-out
    `work,agent` forms and pin older error text. Sweeping them is the CLI consolidation
    entry's.
* README.md listed `status` on one line and had no section for it.
  - A `### status` section, with the scope, the labels, the verdict and its two reasons, the
    plain and nested repo rules, examples, and a sample output, so each rung's README change is
    the reader's test sheet.
* The bare listing offered `st` and not `status`.
  - The completer keeps one candidate per subcommand and takes the first by name, so a visible
    alias that sorts first hides the command. `st` becomes a hidden alias, named in the about
    line: the listing shows `status`, and `st<tab>` completes to it.
* Inserted by the user mid-cycle (wink, 2026-09-02) as its own rung, split from the config rung's
  working copy with the config work stashed beside the line and restored after this push.

##### feat: the agent-files config table

The diff and copy commands want per-workspace defaults for their operand and their custom.md
choice, and the schema has no boolean kind.

* The schema typed strings, sizes, and lists, and a yes-or-no key had no honest kind.
  - `bool` joins the kinds: the prototype accepts it and checks its default is a bare `true` or
    `false`, the generated constant is a Rust `bool`, the renderers print it bare, and
    validate-config flags a `bool` key holding anything else as a finding by shape, the way a
    scalar in a `str-list` key already is.
* The commands had nowhere to keep a workspace's defaults.
  - Four keys, `agent-files.diff.dir`, `agent-files.diff.custom`, `agent-files.copy.dir`, and
    `agent-files.copy.custom`, work-side only. The `dir` keys have examples and no default, since
    absent they defer to `family.template`, and the `custom` keys default to false, the rule's
    own reading of the project layer. The committed model config regenerated with the two
    tables.
  - The agent-files module reads the tables back typed, a missing config or table being the
    default and a `custom` that is not a bare bool an error naming the key. The diff and copy
    rungs consume it.
  - README's Workspace config tables section shows the two tables and how the flags and the
    operand override them.

##### feat: agent-files diff against a set directory

Which set an adopter holds is answerable only by three `diff` lines nobody types.

* Nothing compared a set against another copy of it.
  - `agent-files diff [A] [B]` lists the union of both sides' set files, AGENTS.md and the plain
    files under `agent-data/`, each as same, differs, only in A, or only in B, then `N of M
    differ`, and exits non-zero when anything differs, as `diff` does. Byte comparison, since a
    re-sync is a byte copy. custom.md rides along as the project layer, not compared, until
    `-c`/`--custom` compares it, with `--no-custom` overriding a config that says so.
  - A is the operand, else `agent-files.diff.dir`, else `family.template`, and B is the operand,
    else this workspace, the header line naming where each came from and the report showing
    the directories as written. Two operands need no workspace, so two peers compare from
    anywhere. The resolvers are shared with the copy rung, where the pair is its source and
    destination.
* `family.template` named the template repository, whose root holds the template's own
  AGENTS.md, not the payload.
  - This repo's config names the payload directory, `../vc-x1-template/work`, as the deliberation
    settled, and the key's prose and example in vc-config.md say so, the model regenerated with
    them.
* First run: against the payload, AGENTS.md and four agent-data files differ and the version
  file is only here, the v0.1.0 proposal set as expected. Against iiac-perf with `-c`, nothing
  differs, custom.md included, so the two adopters carry one set.
* README's agent-files section covers `version` and `diff`, with a sample report.

##### feat: agent-files copy from a set directory

A re-sync is a copy by hand, file by file, deletions easy to miss.

* Nothing made a set a copy of another.
  - `agent-files copy [SRC] [DST]` plans from the diff rung's comparison, a copy for each file
    that differs or is only in SRC and a delete for each that is only in DST, prints both ends
    and the steps, applies them, and leaves the result uncommitted, `jj diff` in DST being the
    review and the commit the user's. custom.md moves only with `-c`, TODO.md never. SRC
    resolves as diff's A does, from `agent-files.copy.dir` and `family.template`, and DST as
    its B, this workspace by default, so two operands copy between any two directories and the
    maintainer folds an adopter's set into the payload from anywhere.
  - The guard is on DST: when it sits in a jj repo whose working copy already changes a set
    file, the copy is refused naming the changes, so the copy's changes are the only ones in
    those paths. DST is located relative to that repo, since the payload is a subdirectory of
    the template repo. A DST outside any jj repo gets no guard and the run says so. Copying a
    directory onto itself is refused.
* README's agent-files section covers `copy`, with a sample run.

##### feat: the status and agent-files commands closing

Closing out the cycle: the acceptance check run and recorded, the block finalized and moved to
`## Closed`, the version bare, the dev name kept for Land to restore.

* Nothing in the block needs a `notes/` file of its own.
  - The commands' rules are in the README and the code, the nested-repo root rule with them,
    and the redo is the deliberation's to keep.
* No agent-file changed, so the size step has nothing to record.
  - No row added, by the user's say at the close, the Todo entry **Size is recorded only when an
    agent-file changed** holding the rule change.
* notes/README.md describes the notes directory, not the tool's commands, so it is unchanged.
* Close-out shape: trapezoid, the default, the ladder showing the redo in place.


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

### Check the transcript join on the landed proposal trapezoid

(wink, 2026-09-01) The `agent-files(proposal): v0.1.0` opening was re-described after its push, in
the work-repo only, so its title differs from its agent-repo pair's and its committer time is the
rewrite's, not the push's, and the docs rung's committer time moved with it as a rebased
descendant. The join the records rely on is: blame gives the commit, the commit's pair (by
`ochid:`, or by the pair whose time is the push) gives the transcript slice, a text search of the
slice gives the tool call, and the first hit being a tool call rather than a tool result says the
agent wrote the line. Two probes ran before the rewrite and behaved as predicted, an agent-written
line found in a Bash tool call and a hand-edited line found first in a read. The pair's slice ends
just before its own push call, which lands in the next pair's slice. Two more findings from the
same day: the transcript is the timeline, its lines appended with their own timestamps and the
push calls among them, so the agent-repo is durable storage for the file and its commit structure
carries no part of the join, with the caveat that attachment and queue lines land a millisecond or
two before the message they belong to (11 backward steps in one session), so a join sorts by
timestamp or reads message lines only. And compaction appends, it does not rewrite: ten earlier
sessions in the agent-repo each hold a `user` line flagged `isCompactSummary` mid-file with every
earlier line intact and its timestamp unchanged, so no tool call is lost, only the reasoning before
a post-compaction call may survive as summary alone. Re-run the probes on the landed trapezoid, add
a line from each rung, and decide whether the trailer is load-bearing or convenience, and what a
`vc-x1` command for the join needs. Promoted from `## Waiting` at the 2026-09-02 opening, its
condition met, and passed over for the commands cycle.

### Continuation notes leave the work-repo dirty after Land

(wink, 2026-09-01) Close-out step 7 has the agent write `## Continuation notes` before the exit,
and Land's last push has already gone, so the session ends with `TODO.md` modified in the
work-repo while the agent-repo is clean, and At rest's "both `@` empty" does not hold at the one
moment it is checked by eye. Seen at the first Land under the rule. Think about where the notes
belong: committed by a push of their own, which the hard stop after the final push forbids as
written, written before the closing rung so the closing carries them, kept in the agent-repo
whose session data is the same kind of ephemera, or accepted as the one dirt a restart is
allowed to leave, and say so in At rest.

### Land validates and installs before the main push

(wink, 2026-09-01) The first Land under the rule ran the full validation, with its install, after
the name restore and before the `main` push, rather than jj.md's step 4 after it, so the plain
binary came from the commit `main` was about to carry and the pre-push window with a stale `vc-x1`
closed. Write that order into Land, with the note that a single-step draft's validation installs
under the plain name and a later conversion to multi-step leaves that install behind until Land. A
convention change, paired with the entry above.

### At rest names vc-x1 status as the verdict's printer

(wink, 2026-09-02) AGENTS.md's At rest defines "clean" as both `@` empty and names no command
that answers it. The **feat: the status and agent-files commands** cycle gives the word a home,
`vc-x1 status`, and the one-line pointer in At rest is an agent-file change, so it runs as its
own cycle after that one lands, with the two close-out entries above if they are ready.

### config --merge folds new keys into a workspace config

(wink, 2026-09-02) `vc-x1 config` prints and validates, and a workspace whose config predates a
key learns of it only by reading the model. A `--merge` takes the model's tables and keys, adds
the ones the file lacks as commented lines with their default or example, leaves what the file
holds untouched, and writes the file back for review in the working copy. First use: dogfood it
on this repo's `.vc-config.md`, which the **feat: the status and agent-files commands** cycle
left without the `[agent-files.*]` tables on purpose.

### Global -R anchors the workspace for every command

(wink, 2026-09-01) `vc-x1 version -R vc-x1` and `vc-x1 -V -R vc-x1` from a parent directory are
refused, since `-R` is not global: seven subcommands declare their own, with their own defaults
and meanings, and the workspace root finder is anchored on the cwd. One global `-R <path>` on the
root command, as jj has it, anchors the root search for every subcommand, `version`, `-V`, and
the `agent-files` group included, so the report and the banner follow it, and the per-subcommand
flags retire or become its aliases. CLI-surface consolidation, paired with the nesting entry
below.

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

### Enhance squash-push

Display status of both repos and conditionally push if not clean if the status
changed display the final status. A --yes would mean do a push without prompting.

- The status display here is `vc-x1 status`'s output, once the **feat: the status and
  agent-files commands** cycle lands it.

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

[1]: #feat-the-status-and-agent-files-commands-opening
[2]: #feat-status-both-repos-state-in-one-call
[3]: #feat-the-agent-files-config-table
[4]: #feat-agent-files-diff-against-a-set-directory
[5]: #feat-agent-files-copy-from-a-set-directory
[6]: #feat-the-status-and-agent-files-commands-closing
[7]: #feat-status-completes-its-scope-keywords
[12]: /notes/forks-multi-user.md
