# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, and reset to `_None._` by the reader.

- Asked and not answered: whether the flag-name rule gets written down. A settable key's leaf is
  its flag's long name, in nine pairs (`--account`, `--debug`, `--repo`, `--result-lines`,
  `--col-width`, `--custom` twice, `--yes`, and the `[DIR]` operand), with no exception among the
  keys that have a flag. It is written nowhere, which is how `remote.agent-name` came to be proposed
  and corrected. The candidate home is one line in the "Shape:" prose of `vc-config.md`, in rung 2.
- Asked three times and not answered: whether to write a `## Todo` entry for using "dual-repo
  workspace" consistently and defining it in `README.md`. The spellings in the tree today are
  dual-repo, dual workspace, POR, single-repo workspace, and `is_work_only()`, which `prose.md > One
  spelling per term` forbids.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

_No cycle currently in progress._

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

### squash-push and status take a SCOPE, and push resolves its own bookmarks

(wink, 2026-09-17) `squash-push -R .agent-session` bakes in a path the config already knows, and
the path differs by project, `.agent-session` here and `.claude` historically, so the invocation
goes stale when it moves. `vc-x1 squash-push agent` is shorter, cannot go stale, and reads as what
it means. The design, decided in conversation:

- `SCOPE` is the positional on `squash-push`, with `--scope` as its flag form, the shape
  [`status`](#status-prints-a-verdict-per-repo-and-exits-with-a-bit-per-side) already has. `BOOKMARK`
  moves to `-b`/`--bookmark`, since once the default is the line's own bookmark, naming one is the
  rare override and the rare thing belongs on a flag. This is a breaking CLI change, and the callers
  are the user and `push`'s stage, which builds params directly and never parses argv.
- `-R`'s meaning shifts from "the repo to operate on" to "the workspace root to resolve the scope
  against", which is what `status`'s `-R` already means. A second behavior change in one flag, so it
  wants saying out loud in the docs.
- `push` takes no `SCOPE`: it is always both repos, as its own help says, and a flag with one legal
  value is documentation pretending to be an option. What `push` loses instead is `[BOOKMARK]`,
  resolving each repo's bookmark the way `squash-push` now does. That collapses its hardcoded
  asymmetry, the work repo's `BOOKMARK` against the agent repo's literal `main`, into one rule, and
  retires the warning in [jj.md](agent-data/jj.md#vc-x1-push-what-it-does-and-does-not-do) against
  mirroring a work bookmark into the agent repo.
- A `trunk()` tie-break is needed before `push` can resolve. At a cycle's opening the topic bookmark
  is created at `main`'s commit, so the first push's nearest bookmarked ancestor is one commit
  carrying two bookmarks, which today's resolution refuses. Dropping candidates that are `trunk()`
  when others remain reads as "the trunk sitting coincidentally at your commit is not your line", and
  leaves a genuine tie of two topic bookmarks still refused.
- `both` on `squash-push` raises what `status` never had to answer: with the work side pushed and the
  agent side failing, what the exit code is and whether anything is undone. The instinct is no
  rollback and a bit per side, the shape the status entry already proposes.

Sequenced with **status prints a verdict per repo and exits with a bit per side**, which is already
about scope handling and per-side exit codes, so the scope resolution is written once rather than
twice. Either one cycle covering both, or two adjacent with status first.

### A squash-push test reads the live repo's bookmarks

(wink, 2026-09-20) `squash_push::tests::the_default_is_to_act_without_asking` resolves a bare
`squash-push`'s default bookmark against the vc-x1 repo the tests run in, not a fixture. At a
cycle's opening the topic bookmark sits on `main`'s commit, the resolution refuses the two-bookmark
tie, and the test fails until the cycle's first push moves the bookmark off `main`. The test should
resolve against a fixture, or check only the `yes` default without resolving a bookmark. The
`trunk()` tie-break in [squash-push and status take a
SCOPE](#squash-push-and-status-take-a-scope-and-push-resolves-its-own-bookmarks) would also clear
it, but a test should not depend on the live repo's state either way.

### Commit titles carry no scope, the declared types aside

(wink, 2026-09-17) [Conventional commit scopes](agent-data/prose.md#conventional-commit-scopes)
asks a title's scope for a component name, and [Conventional-commit
shape](agent-data/prose.md#conventional-commit-shape-ladder--commit) offers `feat(push): ...` as
its example. A scope on a plain type repeats what the description already says, and a cycle is
collected by the greppable stem its titles share rather than by the scope. The slot earns its place
only in a declared type, where it carries the declaration's own vocabulary, today
`agent-files(proposal)` and `agent-files(adoption)`. The change: a plain type carries no scope, a
declared type keeps its, and the shape section's example loses one. `prose.md` is a universal file,
so this runs as an `agent-files(proposal)` cycle that bumps the set version.

### lookup narrows a write's work window to its lines and flags a renamed partner

(wink, 2026-09-12) Found trying **feat: vc-x1 lookup for dual repos** by hand before its Land.
`lookup agent` on a transcript write narrows the work window to the files the call wrote, and
that is too wide: session line 846 of `2b15dd23` wrote TODO.md and notes/bugs.md, and the window
printed nine regions, 188 added lines, while the call's own text carries about 67 of them. Most
regions were written partly by that call and partly by other calls in the same rung, so dropping
whole regions removes only two of nine.

- Line narrowing: keep the added lines the call's text carries, a heredoc body, a Write's content,
  an Edit's new string, with a line or two of context, and say how many lines the commit added
  that the call did not.
- A renamed partner: `lookup TODO.md:100` printed a commit and its partner with different titles,
  "fix: clone says the right dir and stops on a rejected config" and "fix: clone's dry run and
  warning name the right files", though each trailer names the other. A push gives both the same
  title, so one side was re-described after its push. Print one line saying so when the titles
  differ, so the pair does not read as a wrong link.

### lookup --history steps a region through the commits that changed it

(wink, 2026-09-12) The time dimension of a lookup: from a line, step older and newer through the
commits that changed its region, each step showing the region there, its partner, and the
transcript write, as `git log -L` does for one side. The interactive form, clicking a line and
arrowing through time, is the session viewer's, the one the `## Waiting` entry names, so vc-x1
computes the chain and the viewer draws it.

- Two axes. The work axis is the region's commit history. The agent axis is the session timeline,
  stepping between writes to the same file or between push windows, and is the easier, since the
  timeline and windows exist.
- Older: blame names the last commit to touch the lines, and mapping the range through that
  commit's line diff gives the region in its parent, repeated to the region's first appearance.
  The reach back covers a moved line.
- Newer: a commit can have several descendants, and a trapezoid is that shape. Follow the line
  that reaches `main`, with a ladder's rungs a detour to step into.
- An amended step can show its predecessor, the first reader for the partner module's evolution
  walk.
- Shape: `lookup --history FILE:LINE` prints the chain oldest first, commit, range, partner, and
  write per step, with a JSON form for the viewer. Each step uses the line narrowing of the entry
  above. The `dr-1` fixture has a line edited across two cycles, a real move, and an amend to test
  it over.

### feat: vc-x1 msg, the messages protocol as code

(wink, 2026-09-11) The README's write actions run by hand, take, edit, release, commit, and the
title rule has a gap the hand cannot close: a commit that closes a thread and also carries lines
in other threads, or closes two threads, has no title under v0.3.2, since the close form names
one thread. The title is a convenience, not a store, so the gap costs a skim of `git log`, but
the answer is the subcommand the messages-rules note names under Further out: `vc-x1 msg`, with
open, reply, done, close, pending, and status, the repo found through `[family] messages` in
`.vc-config.md`, so the id allocation, the guards, the pending query, and the commit are code
and the by-hand steps go. We think a tool commits at each release, so a commit carries what one
take wrote and a batch never crosses threads. Found at **chore: update vc-x1-messages to
v0.3.2**, asking what a batch across threads is titled, and **chore: update vc-x1-messages to
v0.3.3** answered it, the ids of the threads the commit touches, so the tool titles by that rule
and its own question is whether it commits at each release. Its own cycle, multi-step, and next.

### sync clones a declared but absent agent-repo

(wink, 2026-09-06) A dual clone that stops because the cloned config declares no agent side
leaves a POR, and today the recovery after fixing the field is a by-hand clone of the derived
`.claude` remote into the directory the field names, then `vc-x1 symlink`. Sync already resolves
its repo set from the config on disk, so a declared `repos.agent` whose directory does not exist
is a fourth state beside up-to-date, behind, ahead and diverged, "absent", whose act is the
second half of `clone_dual`: a colocated clone from the work remote's derived URL, then the
symlink. The config need not be committed, since sync reads the file, so the user fixes the
field, runs `vc-x1 sync .`, and the agent that then starts has both sides and commits the fix
as a cycle. Two guards: the work repo must have a remote to derive from, a clear error otherwise,
and "absent" means the directory is missing, not present and not a repo, since a stray
directory at that name is the user's. When this lands, the clone error's second line says "then
`vc-x1 sync .`" instead of leaving the clone to the hand. The scenario is a new vc-x1 meeting an
old repo, and #18 in [bugs.md](notes/bugs.md) is where it was found.

### clone takes --agent for the agent-repo's source

(iiac-perf, 2026-09-06) The clone's local location now follows the cloned config, and what is
still fixed by convention is the remote: `derive_bot_url` appends `.claude` to the work-repo's
source, so clone can only fetch an agent-repo that sits in the same namespace under the same
owner. That is the piece the multi-contributor case breaks, since a second contributor's
agent-repo lives under their own owner, and the URL cannot come from the work-repo's config for
the reason the design note gives. The general shape is a second source, `vc-x1 clone <work-url>
--agent <url>`, with the derivation as the default when the flag is absent: one flag, and the
onboarding path from the note, cloning someone else's work-repo with your own agent-repo, is one
command. After the sync entry above, since sync cloning an absent agent-repo needs the same URL
and decides where it is stored, a flag or a per-user config.

### clone from a path refuses a dirty source

(wink, 2026-09-06) A clone fetches the source's bookmarks and checks out `main`, never the
source's working copy, which is right for a URL and a surprise for a path: the user is looking
at the directory, edits `.vc-config.md` there, clones, and gets the old key, with the tool
blamed. Found running the #18 fixture. For a path source, a working copy with changes is an
error, not a warning, since the path form is rare and deliberate, the cost is one commit, and
`jj git clone` is there for anyone who wants the draft left behind. The message names what
would be checked out and where it sits against `@-`, so the user knows whether committing is
enough or `main` must move too:

```
error: ../src/work has uncommitted changes, and a clone checks out main at bc7a4366, which is @-.
Commit and move main, or use jj git clone to check out main as it is.
```

with "two commits behind @-" when they differ. A clean working copy with `main` behind `@-` is
a feature bookmark in progress and is not refused.

Rides with this cycle, from iiac-perf's review of the landed clone fix: a test that a config
declaring the agent-repo outside the work-repo's tree, `../name.claude`, clones there, since the
resolution already follows the config but the symlink then points outside the target directory
and nothing exercises it.

### clone takes -b for the work-repo's bookmark

(wink, 2026-09-06) `jj git clone -b` picks which fetched bookmark the working copy sits on, and
vc-x1 clone always picks `main`. Reviewing a cycle branch on another machine, which is how the
stale binary on `7600x` was found, wants the branch checked out. `-b <bookmark>` on the work
side, `main` by default, the agent side pinned to `main` as always, since it is a linear
journal. The cloned config is then read from the chosen bookmark's tree, which is right: a
branch that renamed the agent directory clones to the renamed one. The other `jj git clone`
flags stay out: `--depth` breaks the `ochid:` cross-links, which need history on both sides,
and `--remote`, `--tag` and `--object-hash` are not choices a workspace should differ on. The
dirty-source error above names the chosen bookmark in place of `main`.

### Continuation notes leave the work-repo dirty after Land

(wink, 2026-09-01) Close-out step 7 has the agent write `## Continuation notes` before the exit,
and Land's last push has already gone, so the session ends with `TODO.md` modified in the
work-repo while the agent-repo is clean, and At rest's "both `@` empty" does not hold at the one
moment it is checked by eye. Seen at the first Land under the rule. Think about where the notes
belong: committed by a push of their own, which the hard stop after the final push forbids as
written, written before the closing rung so the closing carries them, kept in the agent-repo
whose session data is the same kind of ephemera, or accepted as the one dirt a restart is
allowed to leave, and say so in At rest.

- Tried at the **docs: check the transcript join on two landed trapezoids** close-out
  (2026-09-03), the third candidate: the notes went into the closing commit before its push, and
  the work-repo ended clean and stays clean through Land. The cost is notes written before the
  Land they describe, which suits a close-out and would not suit a mid-cycle stop.

### Land validates and installs before the main push

(wink, 2026-09-01) The first Land under the rule ran the full validation, with its install, after
the name restore and before the `main` push, rather than jj.md's step 4 after it, so the plain
binary came from the commit `main` was about to carry and the pre-push window with a stale `vc-x1`
closed. Write that order into Land, with the note that a single-step draft's validation installs
under the plain name and a later conversion to multi-step leaves that install behind until Land. A
convention change, paired with the entry above.

### push refuses when no full validate passed on the current tree

(wink, 2026-09-03) Two rules in [Before any push](AGENTS.md#before-any-push) have no check and both
fail in silence: that validation ran and passed after the last edit, and that the full run, the one
that installs, is what a review is owed. This cycle broke the second for three rungs. The installed
`vc-x1-dev` reported the opening's `0.83.1-0` while the manifest had reached `0.83.1-4`, and nothing
said so until wink ran `vc-x1-dev -V` by hand. A doc-only rung makes it likeliest, since `--fast`
looks sufficient there and the install looks like waste.

- The proposal: a successful full `vc-x1 validate` writes a stamp holding the working copy's tree
  identity and the manifest version, and `vc-x1 push` refuses when the current tree does not match
  it. The message names the version gap, which is the legible form of the failure.
- The artifact needs no inspection: `cargo install --path . --locked` is the last element of
  `[validate] full`, so a full run that passed on this tree installed by construction. Recording
  that the run passed is stronger than checking what it installed, and it carries to an adopter
  whose artifact has no `-V` and no cargo.
- Rejected, `~/.cargo/.crates2.json` (wink, 2026-09-03): it holds the name, version, and source path
  such a check would want, and it is a cargo internal. It succeeded the `[v1]` `.crates.toml`, which
  cargo still writes beside it, so the format has churned once already, and the newer file carries
  no version marker of its own, so the next change misparses rather than announces itself.
- Rejected, the running binary's own version: the flow invokes `vc-x1 push`, the stable binary an
  earlier Land installed from this same path, whose name and version differ from a mid-cycle
  manifest's every time. The check would need a paired rule that the flow runs `vc-x1-dev`, and an
  escape hatch for a dev build too broken to push itself.
- The counter-evidence, and why this is an entry and not a decision (wink, 2026-09-03): a tracking
  file has burned us here before, in [bugs.md](notes/bugs.md) item 8, where `push` adopted a stale
  `.vc-x1/push-state.toml` from an earlier invocation, resumed at its final stage, squashed a new
  session's transcript into an already-published bot commit, and force-pushed it sideways, leaving
  permanent residue in iiac-perf's repo. Whoever takes this weighs that first. A stamp holds facts
  that are compared rather than stages that are resumed, which is the smaller thing, and the same
  hazard still applies: a stamp that cannot be trusted has to fail closed and demand a validate.
- wink's reading (2026-09-03): the root problem may be that the instructions are too complex, in
  which case a gate patches over the complexity and the cheaper fix is fewer or clearer rules. [The
  per-rung flow](AGENTS.md#the-per-rung-flow) step 5 is one clause requiring the full run and one
  granting `--fast` for iteration, and the grant is the half that got read.
- Pairs with **Land validates and installs before the main push**, the same install-currency
  question at the other end of a cycle.

### At rest names vc-x1 status as the verdict's printer

(wink, 2026-09-02) AGENTS.md's At rest defines "clean" as both `@` empty and names no command
that answers it. The **feat: the status and agent-files commands** cycle gives the word a home,
`vc-x1 status`, and the one-line pointer in At rest is an agent-file change, so it runs as its
own cycle after that one lands, with the two close-out entries above if they are ready.

### The cycle-record's items are an intro and bullets, and a pronoun names its noun

(wink, 2026-09-03) A cycle-record's Problem came out as three sentences with thirteen commas, a
wall of prose, with an "it" whose referent was four words from a different "its". The Prose form
already asks for a short intro and bullets and names the cycle-record as a surface, but the
specimen every block copies shows paragraphs, so the paragraph wins. Fix and unify four places
in one `agent-files` proposal cycle:

- cycle-model.md: rewrite the specimen's Problem, Solution, and Acceptance check as an intro
  sentence and bullets, the shape the **docs: check the transcript join on two landed trapezoids**
  opening settled on, since "copy the shape, not the words" is the specimen's own instruction.
- notes.md, The In Progress block: each item is in the Prose form, an intro sentence that states
  the claim and bullets that carry the detail, and the acceptance check is one runnable check per
  bullet. Replaces "a sentence or two".
- prose.md, a pronoun rule beside Semicolons: a pronoun whose referent is not the sentence's
  subject is replaced by its noun, and two referents in one sentence are both named.
- prose.md, a density heuristic a reader can run: a sentence with more than three commas, or a
  paragraph with more than three sentences of detail, becomes an intro and bullets.
- prose.md, a term rule: use the Terminology section's term when one exists, since the family
  shares words across projects and "records" already means iiac-perf's `--records` option to its
  owner, where the cycle-record was meant.
- AGENTS.md and jj.md, "Land" as a proper noun: the Terminology entry and jj.md's heading make a
  verb into a name, and a reader is left to guess whether it means the push or the arrival on
  `main`. Say "land on `main`", or on whichever bookmark, wherever the name is used, and retire the
  entry.

Evidence, 2026-09-20: rung 2 of **feat: init adopts an existing tree** took four review rounds on
its `Ladder details`, and every round found a record-prose defect rather than a code one. The `*`
facets stated answers where problems go, "the name" left its referent a paragraph away, the text
assumed the reader knew `.vc-config.md` and the markdown-as-config carrier, and "this workspace
records" gave a workspace agency it has not got.

### The ladder heads the rung subsections

(wink, 2026-09-21) The In Progress block puts `#### Ladder` after the Acceptance check and the rung
subsections under a separate `#### Ladder details` heading, with the whole Deliberation between
them. The **feat: init adopts an existing tree** block moved the ladder below the Deliberation,
where it heads the subsections as their index, and dropped the second heading. Carry it into the
agent-files in one `agent-files` proposal cycle:

- notes.md, The In Progress block: the `Ladder details` area paragraph and the single-step case,
  which then has no subsections under its `#### Ladder`.
- cycle-model.md: the specimen's order and headings.
- AGENTS.md: the Cycle-record items list, and step 4 of The per-rung flow, which names the area.
- The cost to weigh: the ladder carries `(current)` and now sits past the Deliberation, outside the
  60-line acquaint read. The Continuation notes name the rung in flight at a restart, which may be
  enough, or the acquaint read may look for `(current)` instead.

### repos.agent becomes repos.agent-dir, and a command brings a config up to date

(wink, 2026-09-02, 2026-09-21) Two changes that need each other: a key rename every adopter must
apply, and a command that applies it.

- `repos.agent` becomes `repos.agent-dir`. It holds the agent-repo's directory while `[remote]
  agent-repo` holds its name on the remote, and the flag that sets the directory is `--agent-dir`.
  A key's leaf is its flag's long name everywhere else.
  - Breaking, unlike the `[remote]` key: every adopter's config carries `repos.agent`, and
    workspace-root discovery reads it. The old key is rejected with a fix-it, as `repos.bot` was
    at 0.80.0 (`reject_old_agent_keys` in `src/legacy_vc_config.rs`), never read as an alias.
- A command brings a config up to date, and the fix-it names it. Today `vc-x1 config` prints and
  validates, and a workspace whose config predates a change learns of it only by reading the model.
  - It renames every old key found, across both sides' config files, from one old-to-new table that
    also carries `repos.bot`.
  - It adds the model's tables and keys the file lacks, as commented lines with their default or
    example, and leaves what the file holds untouched.
  - It edits in place and keeps the prose: in a `.vc-config.md` only the key lines inside the `toml`
    fences change, and prose that still names an old key is reported rather than rewritten.
  - A dry run by default, as `fix-desc` and `fix-todo` are, and the result is reviewed in the
    working copy.
  - First use: this repo's `.vc-config.md`, which the **feat: the status and agent-files commands**
    cycle left without the `[agent-files.*]` tables on purpose.
- The name is undecided. iiac-perf has `update-config FILE`, which does the same job by
  regenerating the file from its template, keeping the values and losing the prose, and fails on a
  key it no longer knows rather than renaming it. The same spelling across the family would be
  nice, not required. The candidates are `update-config`, `config --update`, and `config update`
  under **Nest the validate and fix commands**.

### init adopt takes a git-only repo

(wink, 2026-09-21) **feat: init adopt takes a POR** took a jj repo colocated with git, with or
without a single-repo config, and points a git-only repo, `.git` with no `.jj`, at
`jj git init --colocate`, which the user runs before adopting. Adopt could run it: the facade's
colocated init creates a new `.git`, and attaching to an existing one is jj's other init path,
which the facade lacks.

### --repo takes a URL, and a separate flag takes the path for local remotes

(wink, 2026-09-21) `--repo local=<dir>` reads as the local work repo, where it names the directory
init creates the bare origins in, standing in for GitHub. The two categories of one flag also
overlap, since `remote=` takes a path prefix too, meaning bare repos someone else created. Split
them by what they take:

- `--repo <url>` names a remote on a server, a URL and never a path.
- A separate flag, `--repo-local <path>` or `--repo-test <path>`, the name for this cycle to settle,
  takes a path and only a path, and init creates the bare repos there. The two conflict.
- Init creates the path when it is missing. Today a missing one fails with jj's "Could not open
  data at", and only at the publish step, after an adopt has already committed the work repo, so
  preflight checks it before anything is written.
- The bares are named after the project, `<path>/<name>.git` and `<path>/<name>.agent-session.git`,
  not the fixed `remote-work.git`, so one path holds several projects' remotes as one GitHub account
  holds several repos. The fixtures and `tests/cli_sync.rs` are built on the fixed names.
- The user config's account keys, `repo.default` and `repo.category.<cat>`, were shaped for one
  flag with categories (0.41.1-4). Two flags want a key each, named by their long names, and the
  old keys rejected with a fix-it, which is work for [repos.agent becomes repos.agent-dir, and a
  command brings a config up to
  date](#reposagent-becomes-reposagent-dir-and-a-command-brings-a-config-up-to-date).
- `local=` and `remote=` as category values are rejected with a fix-it, not kept as aliases.

### The code says agent where it still says bot

(wink, 2026-09-21) 0.80.0 renamed the bot side to the agent side in the config and on the CLI, and
the code kept the old word: about 314 `bot_` and `Side::Bot` identifiers across `src`. A reader
meets both words for one repo, and output a user reads still carries the old one.

- Output first, since a user reads it:
  - `clone`'s summary prints `Bot repo:` (`src/clone.rs`).
  - `push`'s stages are `commit-bot` and `squash-push-bot`, and it prints `restored bot repo`.
  - `push` prints `.claude had no pending changes` whatever the agent repo's directory is, which is
    wrong as well as old.
  - `fix-desc` says `no bot side`.
- Then the identifiers: `Side::Bot`, `is_bot_dir`, `write_bot_config`, `bot_dir` and its kin, and
  the `bot_session` and `validate_bot` modules.
- Out of scope: the old-config code in `src/legacy_vc_config.rs`, whose `bot` spellings are the old
  names it rejects, and the `--scope bot` rejection, which names the old value on purpose.
- `create_dual` in `src/init.rs` was renamed by **feat: init defaults to .agent-session**, as the
  first function its rung rewrote.

### validate-anchors fails a cross-file link whose file is absent

(iiac-perf, 2026-09-02) Nothing checks that a cross-file markdown link's target file exists.
`validate-anchors` recognizes cross-file targets and skips them, counting them in its report,
and `validate-config` resolves only a `vc-config.md#<anchor>` fragment against the schema, so a
link to a file that is not there passes both. The concrete case is `.vc-config.md`, the file the
family copies between repos: zc-ring-x1's links `vc-config.md` and `vc-config-test.md`, neither
in that repo, and `vc-x1 validate-config` (0.82.0) on a copy of it reports six problems with
neither missing file among them. The cheapest check: a cross-file target's file half is a path,
and "does the file exist" needs no slugging of the other file, so fail a link whose file is
absent, relative to the file holding the link, while still skipping the fragment. The fragment
half stays the crawl the backlog already plans. Reported by iiac-perf's message
**2026-09-02T17:26:18.543Z Cross-file links go unchecked** in `../vc-x1-messages`, which asks
for a reply naming it and linking where this landed, so the reply goes out once this entry's
commit is pushed.

### The opening deletes the closed block's reference definitions with it

(wink, 2026-09-04) A closed block's ladder links its rungs as `- [<title>][N]`, and the `[N]:
#<slug>` definitions live outside the block, in the file's `# References`, pointing at its
`Ladder details` subsections. The opening's "delete whatever `## Closed` holds" takes the block
and leaves the definitions, so every opening since the ladder form arrived left dead definitions
behind, seven at the `agent-files(adoption): v0.2.0` opening. Nothing said so, because
`validate-anchors` is not in `.vc-config.md`'s `[validate]` table. Two edits: [The In Progress
block](agent-data/notes.md#the-in-progress-block) says the definitions go with the block at the
next opening, and `validate-anchors` joins the validate table once its three known failures are
fixed, so a dead definition fails the push that made it.

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

### status prints a verdict per repo and exits with a bit per side

(wink, 2026-09-03) `vc-x1 status` prints each scoped repo's `jj st` block and a summary line,
more than the At rest check needs. The redesign: the default output is one line per scoped repo,
`<label>: clean` or `<label>: dirty: <why>`, the why being the `@ has changes` and `@ is
described` the verdict already names, and the global `-v` restores today's blocks. What `-vv`
adds is left open until a use shows up. The exit code is a bit per side, `work` 1 and
`.agent-session` 2, so `both` exits 0 clean, 1, 2, or 3, and the code means the same repo
whatever the scope, since the scope is one keyword and `both` runs work then agent. Errors exit
outside 0 to 3, so `$?` is never ambiguous, which means the command returns its own exit code
rather than the runner's Ok-or-1 mapping, as `agent-files diff` does. The per-repo verdict is
exposed as a function, since **Enhance squash-push** calls it for its precheck and after-check,
and it carries the bookmark's publish state beside the working-copy verdict, since that entry's
"clean" needs both. The docs follow: the command's help, the README's status section, and the
At rest pointer entry above, whose wording describes the output.

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

### Probe a full re-describe of a landed ladder, or retire the question

(wink, 2026-09-03) Deferred from the **docs: check the transcript join on two landed trapezoids**
cycle, where it was a rung and was dropped at the amend probe's review. The case is a re-describe
of every rung of a pushed ladder, titles and bodies only, with the bookmark renamed and the line
force-published: the heaviest rewrite the work-to-transcript lookup can meet.

- Its four predictions were written before the deferral: every `ochid:` trailer still resolves,
  since change ids do not move, every work committer time moves to the redo so the push-call key
  fails for every rung, every rung's title diverges from its partner's, since agent-repo commits
  on `main` are not rewritten, and the transcript writes stay in their old time-windows.
- Rank it low or retire it: the proposal cycle's opening was re-described after its push and
  evidences all four at one rung, and the amend probe evidences the committer-time half again. The
  open question is only whether eight rungs behave as one did.
- If it runs, it wants a cycle whose ladder is expendable, not a cycle about to land. Re-describing
  pushed commits, renaming a bookmark, and force-publishing just before Land was judged the wrong
  risk for a confirmation.

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

### A workspace anchor, so `[repos]` is shareable

(wink, 2026-09-04) `[repos]` values resolve against the config file's own directory, which is why
the two sides' blocks must differ in every entry, and why a `.vc-config.md` copied between repos
carries values that are wrong in its new home. Define `workspace` as the offset from a config to
the workspace root and anchor `[repos] work` and `[repos] agent` to that instead, so the table is
shareable and only the one `workspace` line is per-clone. It also collapses two rules into one:
root-finding (the nearest config's `[repos] work`) and side detection (the entry resolving to its
own directory) are separate traversals over the same three values today, and four defects in
**agent-files(proposal): v0.2.2** came from prose describing that pair and confusing it.

- `workspace` must be readable with no other knowledge, since it is what tells a reader which side
  it is on, so it cannot depend on side detection. We think that means it replaces `[repos] work`
  rather than sitting beside it, keeping both being a way to keep both rules under new names.
- Ship it with a `[repos] work` fallback as a migration window, converting all three family repos
  in the same cycle, and delete the fallback in a second one. A fallback left standing makes three
  rules where there were two, which is the surface the change exists to shrink.
- The gain is not identical configs: a file has to know where it sits, so one line always differs.
  The gain is that it is one named line and the rest of the table is shared.
- Related: [ochid values may be URLs](#ochid-values-may-be-urls). An anchored `[repos]` would let a
  trailer prefix be derived rather than the constant it is now.

### ochid values may be URLs

(wink, 2026-09-04) An `ochid:` is a side label and a chid, resolvable only inside the workspace
that wrote it. Letting the value be a URL would let a commit point at a counterpart in a repo the
reader holds no checkout of, which is what a family member reading another member's history
needs. Surfaced at the **agent-files(proposal): v0.2.2** close-out, from wink's direction that it
is coming soon, and that cycle's wording was kept side-based rather than path-based so it does not
have to be undone when it arrives.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's copy
of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores) and
[notes/done.md](notes/done.md).

### feat: init adopts an existing tree

#### Problem

Today `vc-x1 init` creates a workspace from nothing and refuses any target that already exists, so a
directory holding work cannot become a dual workspace. The recovery is by hand on both sides, two
colocated `jj git init` runs, two `.vc-config.md` files, two `.gitignore` files, the remotes, and
the symlink, and the two initial commits end up with no `ochid:` cross-link, since a trailer is
never hand-written. Three inputs want the one command: a plain directory that is no repo at all, a
repo with no `.vc-config.md`, and a repo whose config declares only the work side. Beside that, the
agent side a fresh init creates lands at `<project>/.claude`, the mount collision this repo left
when it moved to `.agent-session`, and the agent remote's name is derived by appending `.claude` to
the work source rather than read from anywhere, so nothing records what an existing workspace's
agent-repo is called.

#### Solution

`vc-x1 init --adopt` grows an existing directory into a dual workspace, reading which state it is
in first:

- A plain directory gets both repos around it, its content, large files included, the work repo's
  first commit.
- A jj repo colocated with git, with no config or a single-repo one, keeps its history and gains
  one commit on top, its config edited in place when it has one. With an `origin`, the agent
  repo's remote is derived beside it and the work side pushes nothing.
- A dual workspace, a missing target, and a git-only repo are refused, the last pointed at
  `jj git init --colocate`.

Beside it, init's agent side defaults to `.agent-session` on the directory and the remote, with
`--agent-dir`, `--agent-repo`, and `--agent-suffix` to name them, and the work config records the
remote's name as `[remote] agent-repo`, which `clone` reads, an absent key still meaning `.claude`.
The field runs along the way fixed four more things in the same code: the step narration numbered
1 to N in run order, shell completion for path arguments, a `clone` destination normalized before
it names the symlink, and init's `ochid:` trailer written with the agent side's label rather than
its directory's name.

#### Acceptance check

The tree at `../iiac-perf-expr-1`, a plain directory of experiment records shared with iiac-perf,
becomes a dual workspace in one `vc-x1 init . --adopt`: its `pins/` and `smooth/` records tracked in
the first commit, `.agent-session` on both sides, two public repos under `winksaville` pushed over
https, the Claude Code symlink live, and `vc-x1 status` clean on both sides. In a fixture, `--adopt`
on a POR keeps its history and adds the agent side, on an already-dual workspace it refuses,
`--agent-repo` with `--agent-suffix` is an error, a suffix opening with neither `.` nor `-` is an
error, and a workspace whose config carries no `[remote] agent-repo` still clones, taking
`.claude`. `cargo test` passes and `vc-x1 config work` lists `remote.agent-repo`.

#### Deliberation

- Two `## Todo` entries are consumed rather than one. **init turns a POR into a dual-repo** is the
  cycle's subject, and **`init` still seeds `.claude` as the agent directory** folds in because
  every adopt rung would otherwise be written against the default it asks to change, and a workspace
  adopted under `.claude` would be born into the mount collision.
  - The folded entry's open question, whether the GitHub repo suffix follows the directory, is
    answered yes (wink, 2026-09-20). Repos already published as `<name>.claude` keep their names,
    and the `[remote] agent-repo` key is what lets them.
- The plain-directory case is new work beyond the entry, which named a POR and a single-repo config
  only. Neither describes `../iiac-perf-expr-1`, and that directory is the acceptance check, so the
  third state is in scope from the start.
- The opt-in is `--adopt` rather than init detecting an existing target by itself. The entry's
  wording offered the bare `vc-x1 init .` and `notes/vc-x1-init.md` leaned the other way, and a path
  target that happens to exist should not silently change what the command does.
- Two remote-name flags, rather than one value read as a name or a suffix by its first character
  (wink, 2026-09-20). A full name goes to `--agent-repo`, a suffix to `--agent-suffix`, which errors
  unless it opens with `.` or `-`, and the two conflict. One polymorphic value would hide the
  discriminator inside a parser.
- The config key holds the agent-repo's name, not a URL. The owner and host keep coming from the
  work remote, so a fork under another owner still resolves, and `notes/forks-multi-user.md` reads
  the config as the home for repos the project already tracks and a URL as the form for a one-off
  external contributor.
- An absent `[remote] agent-repo` means `.claude`, rather than a probe of both names. Absence is
  what says the workspace predates the key, so the fallback is a fact about the file instead of a
  network guess, and it retires once the key is everywhere.
- Only init writes the key, since init writes the config anyway. A fetch that edits a tracked file
  would leave the tree dirty with an edit the user then owes a commit, so `clone` and `sync` suggest
  the key instead, and `validate-config` carries the suggestion, its job being the config files.
- The neighbouring entries stay and narrow. Both **sync clones a declared but absent agent-repo**
  and **clone takes --agent for the agent-repo's source** rank above this one: sync's act becomes a
  call into what adopt builds, and clone's flag stays the per-contributor URL the config key
  deliberately does not carry.
- The entry [repos.agent becomes repos.agent-dir, and a command brings a config up to
  date](#reposagent-becomes-reposagent-dir-and-a-command-brings-a-config-up-to-date) stays too,
  holding what was **config --merge folds new keys into a workspace config**. Adopt patches the
  `[repos]` table it owns rather than growing a general merge.
- Multi-step, with the two name rungs ahead of the four adopt rungs. Each adopt rung would otherwise
  be written against a default and a derivation it is about to change.
- The greppable stem is `init`, carried by every rung, since each changes what one command does. The
  bookends carry the bare cycle title, so the pair brackets the ladder on its own grep.
- The repos are public and the remote is https (wink, 2026-09-20), passed at the acceptance run as
  `--repo remote=` with the https namespace. The user config's ssh default is left alone, being the
  user's own file.
- The records are tracked data (wink, 2026-09-20). The 28M of `.jsonl` under `pins/` and `smooth/`
  goes into the first commit rather than into a gitignore, since the records are what the experiment
  is.
- The blocking test fix is folded into the opening rather than inserted as a rung (wink,
  2026-09-20). Publishing the bookmark is what breaks the test, so no rung could validate ahead of
  the fix, and an inserted rung would have had to push before the opening it depends on.
- The `## Waiting` entry's condition is unmet, `vc-x1 closed` not landed, so nothing promotes.

#### Ladder

- [feat: init adopts an existing tree opening][1] (done)
- [fix: squash-push tests never read the terminal][9] (done)
- [feat: init records the agent-repo's name][2] (done)
- [feat: init defaults to .agent-session][3] (done)
- [feat: init adopt detects the target's state][4] (done)
- [feat: init adopt takes a plain directory][5] (done)
- [feat: init adopt takes a POR][6] (done)
- [test: init adopt runs end to end][7] (done)
- [feat: path arguments complete in the shell][10] (done)
- [fix: clone names its symlink from a normalized path][11] (done)
- [fix: init's ochid trailer names the agent side by its label][13] (done)
- [feat: init adopts an existing tree closing][8] (done)

##### feat: init adopts an existing tree opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, move the
two Todo entries into this block, bump the version-of-record, and rename the package to its dev
name. Publishing the bookmark broke a test, so the fix rides here too.

* A test built its params against the developer's checkout.
  - `the_default_is_to_act_without_asking` resolved against `.`, and `try_from` now reads the
    line's bookmark, so publishing the cycle's bookmark put both `main` and it on one line and the
    resolution became ambiguous. The test moved onto a fixture, where its sibling
    `try_from_canonicalizes_and_defaults` already sits.
  - Every opening would have met it, since putting a second bookmark on the line is what an
    opening does. The previous cycle moved the sibling for the same reason and missed this one.
* A false positive in `validate-anchors` was filed rather than fixed.
  - A heading's code-span text is dropped when its slug is computed, so the correct link at
    `TODO.md:515` reads as a break and the check cannot be run to zero. It has nothing to do with
    adopt, so it went to `notes/bugs.md` as #19.

##### fix: squash-push tests never read the terminal

Inserted ahead of the rung in flight, whose edits wait as a patch. `vc-x1 validate` run from a
terminal failed one squash-push test and hung on another, since both reached the real prompt.

* The tests assumed `cargo test` gives them no tty, and it does not: the test binary inherits the
  stdin `cargo test` was started with, which is the terminal when a person runs it.
  - `asking_without_a_tty_errors_rather_than_hangs` prompted, read an empty answer, and failed on
    the decline. `pushs_stage_never_asks` prompted at rest and waited for an answer.
  - The agent's own validation runs with no terminal, so it passed there and the gap went unseen.
* Whether a person is there to ask is now read once, into `Context`, and a test states it.
  - `Context::new` fills `stdin_is_tty` from the real stdin, `test_ctx` sets it false, and
    squash-push's `confirm` takes it as an argument rather than reading stdin itself.
  - The whole suite now passes under a pseudo-terminal, which is how the fix was checked.
* `push` keeps its own four tty checks, since no test reaches them. They move to the `Context`
  field when a test does.
* This block's `#### Ladder` now follows `#### Deliberation` and heads the rung subsections, in
  place of a separate `#### Ladder details` heading.
  - The rungs' list sits beside the subsections it links, as their index.
  - A rule bent by wink's say-so: the layout differs from the one `notes.md`, `cycle-model.md`, and
    `AGENTS.md` describe. The bend covers this block's layout only. The agent-file text is left for
    [The ladder heads the rung subsections](#the-ladder-heads-the-rung-subsections), its own cycle.

##### feat: init records the agent-repo's name

Nothing records what a workspace's agent-repo is called, so every reader that needs the name appends
`.claude` to the work source, and a workspace naming it anything else cannot be found.

* There is nowhere in a workspace to record the agent-repo's name.
  - A workspace is two repos ([the dual-repo model](AGENTS.md#the-dual-repo-model)), and the
    agent-repo has a remote of its own. Where it sits locally is `repos.agent`'s answer, a path
    that defaults to a directory inside the work-repo's and may name anywhere in the tree. What it
    is called on its remote was nobody's answer.
  - The work-side `.vc-config.md` gains a `[remote]` table whose `agent-repo` key holds the last
    segment of that remote's URL. The file is markdown whose `toml` fences are the configuration
    and whose prose reaches no parser, which is [how the carrier is
    read](vc-config.md#how-this-file-is-read).
  - A table of its own, not a second `[repos]` entry: `[repos]` registers local paths, and the two
    keys a letter apart were misread at review the day the key was drafted.
  - README.md's [Workspace config tables](README.md#workspace-config-tables) documents the table
    beside the others, now as one bullet per table where it was one paragraph for all of them.
  - Our own config now carries `[remote] agent-repo = "vc-x1.claude"`, the name GitHub holds this
    project's agent-repo under, while `repos.agent` puts it at `.agent-session`. The two differ
    here, which is why neither can be derived from the other.
* Every workspace that already exists records no agent-repo name, and that cannot be an error.
  - An absent key reads as the `.claude` suffix, and `validate-config` suggests the key without
    counting a finding, so an old workspace still validates clean while the suggestion spreads it.
* Clone needed the agent-repo's name before it had the config that holds it.
  - The URL derivation moved to after the work clone, beside the `repos.agent` read already waiting
    there. The legacy branch is not asked for the key, since it is reached only when the `[repos]`
    config was rejected.
* Init had the project's name and the remote's both in reach, and recorded the project's.
  - The value written is the last segment of the agent-repo's origin URL, not the plan's `bot_name`,
    which is the local directory's project name. The `cli_sync` fixture caught it: a project called
    `tr` whose agent bare is `remote-work.claude.git`.

##### feat: init defaults to .agent-session

A workspace init creates puts its agent-repo at `<project>/.claude`, where the harness's bind mounts
land inside the agent repo, and the name is a constant no caller can override.

* The directory is a constant, and `.claude` is where the harness's bind mounts land.
  - The default is `.agent-session`, and `--agent-dir` names another. The work config records it
    as `repos.agent`, and the work `.gitignore` ignores it.
  - It is one name, not a path: the agent side's config reaches the work repo as `..`, and `.git`
    and `.jj` are refused as the work repo's own.
* The remote name was the work source plus `.claude`, derived in two places.
  - One function now builds a dual plan's agent side for every provisioner. The remote name is
    `--agent-repo`, else the work URL's last segment plus `--agent-suffix`, else plus
    `.agent-session`, and the URL keeps the work repo's owner and host.
  - The plan holds that side as one `AgentPlan`, present exactly when the workspace is dual, where
    it was six optional fields. The dual steps take it whole, so the four `unwrap()` calls that
    read its parts are gone.
  - A suffix must begin with `.` or `-` and name something after it, and the two remote-name flags
    conflict. A suffix like `-agent` is a value, not a flag.
  - The directory and the remote name are chosen apart, so `--agent-dir` alone leaves the remote
    name at its default.
* Existing workspaces keep `.claude`.
  - Their configs record no `[remote] agent-repo`, and an absent key still means `.claude`, so they
    clone as before. `derive_bot_url` stays for that fallback alone.
* The `--por` shape has no agent repo.
  - The three flags are refused with it.
* Left as it is: `--use-template`'s default agent template is still the `<CODE>.claude` sibling,
  since it names a template on disk rather than a workspace's directory.

##### feat: init adopt detects the target's state

An existing target is refused before init looks at it, so the four states it could be in are
indistinguishable to the command that has to grow them.

* Init refused any existing target with one message, whatever it held.
  - `--adopt` asks init to grow an existing target into a dual workspace, and init now reads which
    of four states the target is in before it decides: a plain directory with no repo, a repo with
    no workspace config, a single-repo workspace, or a dual-repo workspace.
  - A repo is a `.git` or a `.jj` in the target itself, and the config's `repos.agent` separates
    single-repo from dual. A dual workspace's agent repo declares `agent = "."`, so it reads as
    dual.
* The refusal did not say what would work.
  - Without `--adopt`, an existing target is refused by its state and pointed at `--adopt`.
  - A dual workspace is refused either way, having nothing to grow, and `--adopt` on a target that
    does not exist is refused too, since a missing directory is likelier a typo than a request.
  - The three states adopt will take are refused by name until their rungs land.
* Some targets fit no state, and init must not guess at them.
  - A file, a config with no repo beside it, and a config with no `repos.work` are errors, and a
    legacy config gets its existing fix-it.
  - A target inside another repo is not refused. Checking the ancestors would block every adopt
    under a home directory kept in git, and the one nesting that matters, the agent repo of a dual
    workspace, is caught by its config.
* `--adopt` grows an agent side, so it is refused with `--por`.
* Riding along, unrelated to adopt: this repo's `[validate] fast` is now the same list as `full`,
  on trial. `cargo test --bins` saved about four seconds and skipped clippy and the integration
  tests, where this cycle's failures turned up. The flag and the key stay, for a project whose
  full run is slow.

##### feat: init adopt takes a plain directory

A directory that is no repo at all has to gain both repos at once, with its own content as the
work-repo's first commit.

* `--adopt` refused a plain directory as not taken yet.
  - It is taken now: init creates both repos around the directory as a fresh init would, and the
    work repo's first commit is everything its `.gitignore` does not exclude. The rest of the
    fresh init, the configs, the cross-linked `ochid:` trailers, the remotes, and the symlink, is
    unchanged.
  - A directory already holding the agent directory is refused before anything is written, and
    `--use-template` is refused with `--adopt`, since a template would overwrite files of the same
    name (wink, 2026-09-21).
* jj leaves a new file over 1MiB out of a snapshot, and the facade dropped the report that said so.
  - The first commit tracks every file whatever its size and lists the ones over the limit (wink,
    2026-09-21). The directory was adopted to be committed, and a `.gitignore` is how to keep a
    file out. Only new files are limited, so once tracked, later snapshots take their changes.
  - The snapshot now returns what it left out for size, and `commit_any_size` snapshots once to
    learn that and again with the limit lifted. Every other caller still ignores the report, which
    is [bugs.md #20](notes/bugs.md), since `push` can leave a new large file out the same way.
* An adopted directory may already have a `.gitignore`.
  - It is kept, and given the agent directory's line when it lacks it (wink, 2026-09-21). With
    none, init writes its usual one.
* Init's narration numbered its steps apart from the order they ran (wink, 2026-09-21).
  - A real run printed Step 7, 8, 7, 9, 11 and prose for the rest, the dry run listed eleven steps
    against the retired design, among them a `git clean -xdf` that would read as a threat to an
    adopted directory's content, and a step 10 that no longer runs.
  - One list in `src/init/steps.rs` now names each step a plan runs, numbered 1 to N in run order.
    The dry run prints it whole and the real run prints `Step N: <title>` as each step starts, so
    the two cannot disagree, and a step a plan does not run is left out rather than printed as
    skipped.
  - A dual run is ten steps, the last the symlink, and a single-repo run four. The helpers' own
    lines that repeated a step's title became debug lines, and what they add, a commit's chid and
    the files over the size limit, prints indented under its step.

##### feat: init adopt takes a POR

A repo already carries history, a `.gitignore`, and possibly a single-repo config, none of which the
create path's unconditional writes may clobber.

* `--adopt` refused a repo as not taken yet.
  - A jj repo colocated with git and with no workspace config is taken now. Its history stays, and
    one commit goes on top, titled "Adopt as a dual-repo workspace", carrying `.vc-config.md`, the
    `.gitignore` line, and the `ochid:` trailer to the agent repo's first commit.
  - A single-repo workspace, the shape `init --por` makes, is taken too (wink, 2026-09-21): its
    first run in the field was refused, since every POR vc-x1 makes carries a config. Its config is
    edited in place, `agent =` into `[repos]` and `agent-repo =` under `[remote]`, a header added
    when the file has none, every other line kept, and the result read back and restored when it
    does not declare both keys.
  - A git-only repo stays refused, pointed at `jj git init --colocate`, and is filed as [init adopt
    takes a git-only repo](#init-adopt-takes-a-git-only-repo).
* A repo usually has a remote already, which a fresh init would try to create.
  - With an `origin`, that is the work repo's remote, and the agent's is derived beside it, the
    provisioner read off the origin's URL. The work side creates and pushes nothing, its commit the
    user's to land (wink, 2026-09-21), and `--repo` and `--account` are refused.
  - With no origin, the remotes come from `--repo` as a fresh init's do, and both are pushed.
  - Resolving `--repo` needs the user config, which a repo with an origin does not. Under `--adopt`
    the chain's error is held and raised only when the target has no origin to use.
* A repo may hold uncommitted work, which adopt's commit would take in.
  - A working copy with changes or a description is refused before anything is written (wink,
    2026-09-21).
* The steps follow the repo's shape: no prepare step, the commit titled as it is and called the
  adopt commit where the cross-link names it, and no work publish when an origin exists.
  - A commit's detail line reads "work commit" rather than "work initial commit", which an adopt
    commit is not.
  - The preflight's two work-side `unwrap()` calls became `if let`, since an adopted repo's plan
    has no work slug or bare to check.

##### test: init adopt runs end to end

The adopt rungs test `init()` in process, with the symlink turned off, so nothing that runs again
drives the `vc-x1` binary through an adopt, and the symlink step is checked by hand alone. The rung
adds CLI integration tests to `tests/cli_init.rs`, the binary as a subprocess with `HOME`
redirected (wink, 2026-09-21).

* No test drove the binary through an adopt, and the symlink step had no test at all.
  - `tests/cli_init.rs` runs the built `vc-x1` as a subprocess with `HOME` in the fixture, so the
    symlink lands there, and reads the log it prints.
  - A plain directory: all ten steps print and no eleventh, a file over jj's new-file limit is
    named, both repos and both bare origins appear, and the symlink points at the agent repo.
  - The single-repo workspace `init --por` makes, adopted in a second run: step 1 edits its config
    in place, no work publish step runs, the origin's `main`, read with the real git, is where it
    was, the agent bare sits beside the origin, and the symlink points at the agent repo.
  - The refusals: an existing target without `--adopt` is told to pass it, `--adopt` on a missing
    target is told it does not exist, and neither writes anything.

##### feat: path arguments complete in the shell

Inserted after the adopt rungs (wink, 2026-09-21). An argument that takes a path completes in the
shell only when it is a `PathBuf`, so `init`'s TARGET and seven others typed as a `String` or
parsed by a function of their own complete nothing under `COMPLETE=bash`.

* clap's completion engine completes a path only for an argument it knows takes one, a `PathBuf`
  or an argument with a `value_hint`, and eight path arguments were neither.
  - `init` and `clone` TARGET take `AnyPath`, since each also takes a URL, and `symlink` TARGET
    and `--use-template` take `DirPath`.
  - `lookup`'s `FILE:LINE`, in either position, `--config none|PATH`, and the `config` and
    `validate-config` TARGET take `FilePath`. The path completes and the rest, `:LINE` or a side
    keyword, is typed.
* Nothing checked what the binary offers.
  - `tests/cli_complete.rs` asks the built binary for candidates the way the shell hook does,
    over the fish protocol, which prints one per line, and checks each argument offers a path in a
    scratch tree. `validate-anchors`, a `PathBuf` that completed already, rides along as the
    control. The bash protocol gives the same answer by hand.

##### fix: clone names its symlink from a normalized path

Inserted after the completion rung (wink, 2026-09-21), from a clone in the field. `clone` joins its
NAME to the working directory as given, so `./dtdrvvx1` makes `…/experiments/./dtdrvvx1`, and the
symlink encodes that path to `-home-…-experiments---dtdrvvx1`, a name Claude Code never looks
for, so a session there keeps its history outside the agent repo.

* `clone` joined its NAME to the working directory without normalizing it.
  - It now collapses `.` and `..` the way `init`'s path targets already did. `normalize_path`
    moved from `init.rs` to `common.rs` so both call the one copy.
* The symlink's name is the path encoded character by character, so any `.` or `..` a caller let
  through changed it.
  - `SymLink::new` normalizes the working directory and the target before encoding, so every
    caller, `clone`, `init`, and `symlink`, names the link as Claude Code derives it from the real
    directory.
* Nothing covered a dotted destination.
  - `tests/cli_clone.rs` clones into `./cl` through the binary and checks the output has no `/./`
    and the one symlink is named from `<base>/cl`. A symlink unit test names the link from
    `/home/user/./project` and `/home/user/x/../project` as `-home-user-project`.

##### fix: init's ochid trailer names the agent side by its label

Inserted after the symlink fix (wink, 2026-09-21), from `validate-desc` in the field. The agent
side's ochid label is `/.claude` whatever its directory is called, which `push` writes and
`validate-desc` checks, but init's cross-link spells the prefix from the directory's name. Since
init defaults to `.agent-session`, every workspace it makes starts with a work commit whose
trailer `validate-desc` rejects.

* `cross_ref_ochids` spelled the agent side's prefix from its directory, `/<dir>/`, where its own
  doc comment said `/.claude/`.
  - It writes the sides' canonical labels, `OCHID_BOT_LABEL` and `OCHID_WORK_LABEL`, the ones
    `push` writes and `validate-desc` checks. The two differed only once rung 3 moved the default
    off `.claude`.
* Two init tests asserted `ochid: /.agent-session/`, pinning the bug as the behavior.
  - They assert `/.claude/`, and a CLI test makes a fresh workspace and adopts a plain directory,
    both with `--agent-dir .sess`, and runs `validate-desc` on all four repos.

##### feat: init adopts an existing tree closing

Closing out the cycle.

* Acceptance check: pass (2026-09-22).
  - `vc-x1-dev init ./iiac-perf-expr-1 --adopt` ran ten steps. The 21 records in `claim/`,
    `pins/`, and `smooth/` are tracked in the first commit, 23 files with the config and
    `.gitignore`, the three records over the 5MiB limit named and tracked. Both repos are public
    under `winksaville` with `https://` origins, the directory and the remote are `.agent-session`,
    the symlink is named from the normalized path, `vc-x1 status both` is clean, and
    `validate-desc` passes on both sides.
  - The fixture half runs in `cargo test`: a POR keeps its history, a dual workspace is refused,
    the two remote-name flags conflict, a bad suffix is refused, an absent `[remote] agent-repo`
    clones as `.claude`, and `config work` lists `remote.agent-repo`.
* The Problem's three inputs are all taken: the single-repo workspace, planned for later, came in
  when its first run in the field was refused.
* Close-out shape: trapezoid (wink, 2026-09-22).
* What the field taught: four of the cycle's twelve rungs came from runs in the user's shell rather
  than from the plan, and each found what the in-process tests could not see, a terminal on stdin,
  a symlink named from a dotted path, a trailer spelled from a directory, and ssh from a user
  config. Running the built binary on real directories before the closing was worth the rungs it
  added.
* Filed for later: the missing `--repo local=<dir>` parent fails at the agent's publish, after the
  work repo is committed, so a preflight check joins the `--repo` entry.

# References

[1]: #feat-init-adopts-an-existing-tree-opening
[2]: #feat-init-records-the-agent-repos-name
[3]: #feat-init-defaults-to-agent-session
[4]: #feat-init-adopt-detects-the-targets-state
[5]: #feat-init-adopt-takes-a-plain-directory
[6]: #feat-init-adopt-takes-a-por
[7]: #test-init-adopt-runs-end-to-end
[8]: #feat-init-adopts-an-existing-tree-closing
[9]: #fix-squash-push-tests-never-read-the-terminal
[10]: #feat-path-arguments-complete-in-the-shell
[11]: #fix-clone-names-its-symlink-from-a-normalized-path
[13]: #fix-inits-ochid-trailer-names-the-agent-side-by-its-label
[12]: /notes/forks-multi-user.md
