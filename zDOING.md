# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

Kept by the acquaint of 2026-11-04:

- The test repo `~/tmp/wt5`, remotes `~/tmp/remotes`, sandbox on with `allowWrite
  /home/wink/tmp/remotes` in its `.claude/settings.local.json`: its cycle **chore: configure this
  workspace** has its opening and config rung backed up by `--covers`, agent `3ba90000`, every link
  `ok`, and its closing and Land are next, then its `TODO.md`'s other Todos.

## In Progress

### feat: close, land, and push compose with testing

Filed and opened the same day, wink's call (2026-11-02), from the anomaly the closing of **feat:
improve trailers** ran into. Folds in three Todos, moved here below.

#### Problem

(wink + agent, 2026-11-02) Closing a cycle should be obvious, and to the user's liking, and `vc-x1`
should help them do it properly. It does not:

- A local cycle's closing holds `ocid: .../xxxx` until `vc-x1 land` commits the agent `@`, and that
  rewrites all of the cycle's commits, 12 last time, so what was tested is not by SHA what lands.
- The way through took five steps, which wink called a non-starter, and the one-command answer
  gives up the test pause.
- `land` needs the agent repo, so a work-only land, a merge or rebase, or a share for others to test
  drags in a permanent link it does not need.
- Nothing says where a cycle stands or what may come next.
- The commands are already separate, so that is not the problem, wink's (2026-11-02). They do not
  compose: the user wants to test between any two of them, passively by `validate`, and actively by
  you, me, and others running a build named by `profile.<name>.suffix`. Today that is by hand and on
  the real repos, and full `validate` runs `cargo fmt`, which can dirty the `@` the next step needs
  empty.

#### The Todos it folds

##### commit --closing reads the nearest opening's title

(wink 2026-11-1: this these 2 `commit --closing` should be one cycle?)

(wink + agent, 2026-10-26) Without `--title`, `commit --closing` takes the cycle title from the
opening above the trunk, and with cycles stacked on one line it reads the oldest: the closing of
**feat: vc-x1 land** came out as **feat: vc-x1 commit takes a revision closing**, the stack's first
cycle, and was made with `--title`.

- The opening read is the nearest below the tip, the cycle's own, so a stack closes its top cycle.
- A test closes a stack of two cycles without `--title`.

##### commit --closing merges onto the cycle's base when it was opened above the trunk

(wink 2026-11-1: this seem convient, but not sure the why,when,downsides,dangers)

(wink + agent, 2026-10-26) `commit --closing` merges the trunk with the tip, so a cycle opened
above the trunk, on an unlanded series as **feat: vc-x1 land** was, gets a closing whose first
parent is the trunk, and the cycles below it leave `main`'s first parents for the second parent's
ladder. That closing was re-parented by hand onto the fix commit its opening sat on.

- The first parent is the opening's parent, where the cycle's bookmark was created, when that
  descends from the trunk, and the trunk otherwise, so `git log --first-parent` reads one commit
  per cycle.
- A test closes a cycle opened above an unlanded one and checks both parents.

##### land lands @- by default

(wink + agent, 2026-10-29) `land`'s `--source` defaults to "the one local bookmark besides
`--onto`", which depends on every bookmark in the repo and fails once there are two. The rest of
vc-x1 works from the line's `@-`, `push` moving its bookmark there and `commit --closing` naming it.

- `--source` left out is `@-`, with the work `@` empty: what lands is then the tree validation ran
  on, and nothing uncommitted is left behind. `commit --closing` without `--land` leaves exactly
  this, the closing at `@-` and an empty `@`.
- A non-empty work `@` is refused, naming `--source`, rather than warned about, since a land is
  pushed and permanent on the trunk. A merge `@`, whose `@-` is ambiguous, is refused too.
- The empty check is the work-repo's alone: the agent `@` holds the session and is never empty.
- An explicit `--source` lands what it names, with no check of `@`.
- The cycle's bookmark is found from `@-` as today: its own, else its nearest parent's above
  `--onto`, several refused with the list.
- The help: `--source <BM|REV>`, "The cycle's last commit, a bookmark or a revision [default: @-,
  with @ empty]".

#### Solution

Provisional. Three steps, each doing one thing, that compose with testing:

- Every step leaves a state that can be tested, nothing half-done.
- A step may repeat, so the user can test, tweak, and close again.
- Testing does not disturb the cycle or block the next step.
- Each tester's binary says what it is, its suffix the build and its version the commit.
- Exit codes can be trusted, so the steps and the tests chain in a plain shell, and `vc-x1` does not
  run the tests itself.

The steps:

- close makes the cycle's final shape, single-step or a trapezoid, local, with `--scope` defaulting to
  work. It may be run again, and the agent and the user keep iterating on it.
- land merges or fast-forwards onto any bookmark, local, and it may repeat. `--scope` defaults to work,
  and it takes any scope that applies.
- push to any other bookmark shares the work for backup or testing, with `xxxx` still in it. Push to a
  release bookmark with `-s b` seals: it commits the agent `@`, fills the `ocid:`, refreshes any stale
  `pcid:`, checks each tree unchanged, and publishes the agent repo first.
- A release bookmark is one the user treats as permanent: `[release] bookmarks`, or `main` and `master`
  when that key is absent. Work alone with an unsealed agent commit is an error there, unless `--yes`.

The links follow one general rule, for any mix of work and agent commits:

- `ochid:` is one line per related commit, on either side.
- Every `ocid:` is the SHA of an agent commit the same commit's `ochid:` lines name, matched by
  resolving it and not by line order.
- A work commit's `ocid:` is real only when its agent commit was made before it, wink's (2026-11-02),
  and otherwise stays `xxxx`. So most are `xxxx`, rungs included and on a release bookmark, and are
  never rewritten. In an N:1 cycle only the closing has a real one, the seal committing the
  agent repo before
  the closing's final form. In 1:1 pairs every commit has a real one.
- Each agent commit's SHA is carried by exactly one work commit, the last it names.

#### Acceptance check

By `vc-x1-dev`, on repos made by `init --test-repo <path>/<repo-name>`, in a table-driven test:

- An N:1 cycle: close leaves every `ocid:` at `xxxx`. A tweak and a second close work. `land` with no
  `--scope` leaves the agent repo alone, and a push to any other bookmark keeps `xxxx`, at any scope.
- A push to a release bookmark with `-s b` commits the agent `@`, fills only the closing's `ocid:`,
  leaves every rung's `xxxx` and SHA unchanged and the closing's tree identical, and pushes the agent
  repo first. Pushed to a second release bookmark, the commits are already sealed and nothing changes.
- A push to a release bookmark of work alone, with an agent commit no commit carries the SHA of, is an
  error naming `-s b` and `--yes`, and with `--yes` it publishes and warns. An unsealed closing naming
  another agent `@` is refused, and a release bookmark that moved is refused before anything is
  committed.
- The release bookmarks: with no key, `main` and `master`, so an old config works, and with `[release]
  bookmarks`, that list alone, a project's `main` then being an ordinary bookmark.
- A test between any two steps, `validate` or a suffixed build, changes nothing the next step depends
  on, and the next step runs.
- Two clones of one local remote give a moved trunk and another's commits, with no network.
- 1:1 pairs carry a real `ocid:` on every commit and never `xxxx`.
- `validate-desc` and `lookup` accept 1:1, N:1, 1:M, N:M, and mixed, and this repo's `main`, where every
  rung of **feat: improve trailers** carries an `ocid:`, reports what it reports today.
- close, land, and push each end by printing where the cycle stands and what may come next.
- `vc-x1 validate` passes.

#### Deliberation

- Close makes no agent commit, wink's (2026-11-02), reversing the agent's proposal: the agent and the
  user iterate after close, the conversation belongs in the cycle's own agent commit, and rework stays
  free since nothing on the agent side names a rung yet.
- The seal is the push to a release bookmark, wink's (2026-11-02), reversing the agent's land-as-seal:
  land may
  be a merge or a rebase on any bookmark, and sharing to test needs no permanent link. Only the release
  line is permanent.
  - The cost: no test at the final SHA before the release, only at an identical tree, and the binary's
    embedded SHA differs. A tester's SHA for the closing never reaches `main`, the rungs' do.
  - It bends **Only push publishes**, one push rewriting before it publishes, so it says so plainly and
    `--dry-run` shows every rewrite.
  - We think sealing is callable on its own too, since a pull request's merge is GitHub's and runs no
    `vc-x1 push`. It is built as a step of its own, whether or not it gets a flag now.
- `ocid:` is real only when the agent commit came first, wink's (2026-11-02), so a rung's stays `xxxx`
  rather than being absent or rewritten, and the commit that carries an agent commit's SHA is the last
  it names, wink's and the agent's, generalizing closing-only. A reader goes from a rung to its agent
  commit by `ochid:`. Wink's "next commit" is the fallback once the last is on the release line, a
  one-way link. Rejected as the norm: on a closing, "next" is the next cycle's opening, so
  `main`'s landmark
  would have no link to its conversation.
- Any mix is valid, wink's (2026-11-02): N:M, so readers and validators accept all of it and stop
  requiring links to be mutual. Writers stay at 1:1 and N:1 here, as **commit writes every mix of work
  and agent commits** files, and the agent's mid-cycle commit is **An agent commit mid-cycle backs up
  the transcript**.
- A test repo is `init --test-repo <path>/<repo-name>`, wink's (2026-11-02), with no config key: each
  is a path to `cd` into or name by `-R`, so there may be several.
  - It is local, and makes local bare remotes, as `--repo local=<parent>` does, since testing `push`
    needs a remote.
  - `--local` is the default rather than a flag to reach for, wink's (2026-11-02), so the name says what
    the repo is for, and what a bare `init` does stays with **init defaults to --local**.
- `--scope` defaults to work, always, wink's (2026-11-02), and any scope that applies may be named.
  - It reverses **commit works at every scope, and --local keeps it on the machine**'s `--land` requiring
    `--scope`, since a work-only land moves no agent commit.
  - This cycle does it for close, land, and push, and the rest of the commands stay with that entry.
- The tests run outside `vc-x1`, wink's and the agent's (2026-11-02): composing is the steps' job and a
  shell's, and `validate takes any profile` stays its own Todo.
- The agent repo may have branches someday, merged at a final Land, wink's heads-up (2026-11-02), so
  nothing here assumes the agent repo is linear, and the readers rely on the trailers alone.
- `xxxx` is an error only on a release bookmark, wink's (2026-11-02): it is fine on a test branch and
  probably a mistake on `main`, so the user defines the set, with `main` and `master` hard-coded for
  existing configs.
  - The key is `[release] bookmarks`, the agent's: a property of the repo and not a cycle, and the rules
    say bookmark. `[cycle] trunk` stays, where a cycle lands by default and not what is protected.
  - A key that is present replaces the two names rather than adding to them, the agent's: otherwise a
    project developing on `main` and releasing on `stable` could never take `main` off.
  - The check is per agent commit, the agent's: rungs keep `xxxx`, so a search for it would refuse every
    push. Each group of commits naming one agent commit needs one real `ocid:`.
  - Other commands read the same set, `trailers --update`'s refusal first, which names the trunk today.
  - `--yes` is no human check, an agent typing it as easily, so the agent passes it only when wink says
    so for that push. **Only push publishes** has the mechanism as open.
  - It catches a mistake made through `vc-x1` and no more: a raw `jj git push` or a pull request's merge
    goes around it, and branch protection is the tool for that.
- `vc-x1 close` is a subcommand, the agent's, to confirm: it pairs with `land` and `push`, and `commit
  --closing` shares its code.
- This cycle is local and work-only until its Land, per the v0.3.1 prototype `custom.md` names:
  - every commit is `commit -s w --no-push`, the opening with `--skip remote` too, bending Opening step
    2 and the per-rung flow's push. The description review goes and the work review stays. The Land is
    outside the bend, its pushes approved as usual.
  - commits and the Land run by `vc-x1-dev`, as `zDOING.md`'s Rule says.

#### Ladder

Provisional, the rungs after the opening named as the work is understood:

- [feat: close, land, and push compose with testing opening][0] (done)
- [feat: init --test-repo makes a test repo][1] (done)
- [feat: init drops --use-template][2] (done)
- [chore: vc-x1 adopts the agent-files v0.3.1-0 draft][3] (done)
- [feat: an agent commit covers its work commits][4] (done)
- [feat: an initial vc-dr-apply.sh][5] (done)
- [chore: vc-x1 adopts the agent-files v0.3.1-1 draft][16] (done)
- [feat: the cycle record returns to TODO.md, indexed by zTOC.md][17]
- [feat: a test repo converts an existing repo][6]
- [feat: the readers take any mix of ochid and ocid][7]
- [feat: ocid is real only when the agent commit came first][8]
- [fix: commit --closing reads the nearest opening and keeps the cycle's base][9]
- [feat: land takes @- and the work scope by default][10]
- [feat: a push to a release bookmark seals the cycle][11]
- [fix: testing leaves a cycle able to continue][12]
- [feat: close, land, and push say where the cycle stands][13]
- [docs: the rules and README name the three steps][14]
- [feat: close, land, and push compose with testing closing][15]

##### feat: close, land, and push compose with testing opening

Opens the cycle, its version at the `-0` form. Also carries the Todo work that led here: the reordered
`## Todo`, wink's, **lookup builds an index that correlates the two repos**, and the Todos this cycle
sets aside, filed. `tmp-durable/` is gitignored here, and its rule and the jj set-aside are drafted in
**agent-files(proposal): v0.3.1**.

##### feat: init --test-repo makes a test repo

The sequence has no safe place to run. `init --test-repo <REMOTES> --target <path>` makes a dual
workspace whose bare remotes are in REMOTES, named by the repo, `<name>.git` and
`<name>.agent-session.git`, so many test repos share one REMOTES apart from their workspaces, wink's
(2026-11-03). It first made the workspace at its value with the remotes beside it in `PATH.remotes`,
and `--repo local`'s fixed `remote-work.git` names would collide in a shared directory, so those stay
`--repo local`'s alone. It makes the `~/.claude/projects` symlink as any `init` does, wink's catch
(2026-11-02), since a session in a test repo needs it for its transcript to reach the agent repo, the
cost a dangling link once a test repo is deleted. REMOTES is always a path, so `tmp/remotes` needs no
`./`, wink's first try finding that too. It found `--repo local` could not make a parent that does not
hold the workspace, despite the code saying so, and that is fixed. Seeding one by hand found two more,
wink's (2026-11-02): `~` after `--use-template`'s comma reached `init` unexpanded, the shell expanding
it only at a word's start, and the two long paths left no completion for the second. Each path now
expands itself, and a lone path holding `work/` and `agent/` names both.

Today's sequence, run on one by `vc-x1-dev`, so the rungs below start from what was seen:

- Composes already: rungs, a topic push with `xxxx`, a tweak folded into the closing by `squash`, its
  `pcid:` lines still right, and a second clone of a test repo's remotes.
- A second close stacks a second merge on the first, 3 commits above the trunk, rather than remaking it.
  For the `commit --closing` rung.
- A fresh workspace cannot close without `--body`, having no `[closing] body`, and cannot `validate`,
  having no `[validate]`. For the same rung and the testing rung.
- `land` lands a rung with no closing, leaving `main` titled for that rung, and takes a dirty `@` with no
  word. For the land rung.
- After a topic push, `land` warns of the first commit going stale and not the others, and the topic stays
  on `origin`. For the rung that says where the cycle stands.
- `commit` and `squash` end by naming `vc-x1 show -s work @-`, which `show` refuses, its positional being
  retired. The same rung.

##### feat: init drops --use-template

`init --use-template` seeds a workspace from template directories, which a script beside vc-x1 does
instead, wink's call (2026-11-03), as **vc-dr-apply.sh applies vc-x1's rules to a dual workspace** in
`TODO.md` sets out. The flag goes, with its option type, the template copy and the README rewrite it
drove, their tests, and the README's word on it, and `init` makes a bare workspace, the repos holding
only `.vc-config.md` and `.gitignore`. The flag is gone outright, not kept hidden and refused by name
as the retired `--por` is, wink's call (2026-11-03): clap's completion offers a hidden flag once its
prefix matches nothing visible, so `--use<TAB>` would still complete it. The two Todos it answered retire, **init
--use-template takes an existing workspace as the template** and **Restructure templates: one repo +
a fixed agent seed manifest**, with the backlog's **`init` template ergonomics**, whose no-remote mode
`--test-repo` became.

Checked against the template Todo's overwrite rule, `init --adopt` holds to it: an existing
`.gitignore` is kept and given only the lines it lacks, a single-repo config is edited by adding lines
alone, and nothing else in the directory is written. The one quiet edit, the README's first line, was
`--use-template`'s and went with it.

##### chore: vc-x1 adopts the agent-files v0.3.1-0 draft

The last cycles have run on the proposal **agent-files(proposal): v0.3.1** while vc-x1's agent-files
still say v0.3.0, so the rules a session reads are not the ones it follows. vc-x1 takes the draft
`v0.3.1-0` in `../vc-x1-template-2/work` verbatim, wink's call (2026-11-03), by `vc-x1 agent-files
copy`, and `agent-files diff` against the draft reports every file the same. The proposal stays open
in `TODO.md`, noting the adoption, with the refinements found since kept there for `v0.3.1-1`. The
draft is not a repo, so this one copy is from a working tree, and vc-x1's commits are the source
after it.

##### feat: an agent commit covers its work commits

A cycle's rungs made by `commit -s w` share one agent commit, and nothing made it with one `ochid:`
per rung: `push -s a` takes the tip's description and links the tip alone, `commit -s a` links none,
and `commit -s b` refuses an empty work `@`. `commit -s a --covers <REVSET>` makes it, under the title
given, the cycle title by wink's (2026-11-03): one `ochid:` per work commit the revset names, oldest
first, the revset taken as written with no trunk assumed, wink's too, and every commit it names must
name the agent `@` already, or nothing is made. Then `trailers --update` gives every rung's `ocid:`
the agent commit's SHA, and `push -s b` publishes both.

Push needed no change: an agent commit naming several work commits, or partnering one that carries an
`ocid:`, is the `Landed` partner, which push publishes as it is, leaving the journal in `@`, so no
fold changes the SHA the links name.

Committing the agent side and publishing mid-cycle bends the adopted `v0.3.1-0`'s **A cycle stays
local until its Land**, wink's call (2026-11-03), and the proposal's **The user decides when each side
commits and publishes** makes it the rule in `v0.3.1-1`. The bend covers this cycle's mid-cycle agent
commits and pushes, each still approved on its own, and not its Land. The backup runs as one command
line, `commit -s a --covers` then `trailers --update` then `push -s b` joined by `&&`, a second bend,
of One command per invocation, wink's too: one invocation keeps the agent `@` still between the
steps, and `&&` stops at the first failure with its status.

##### feat: an initial vc-dr-apply.sh

A bare workspace from `init` has no rules, and nothing fills it now that `--use-template` is gone.
`scripts/vc-dr-apply.sh [--to DIR] [--rev REV] [--agent-rev REV] [--dry-run] SOURCE [AGENT_SOURCE]`
applies a source dual repo's committed rules to it, wink's shape (2026-11-03): SOURCE a dual workspace,
its agent repo found by its `.vc-config.md`, or SOURCE and AGENT_SOURCE given apart. From the work repo
at `--rev`, `AGENTS.md`, `agent-data/`, and `LICENSE-*`, from the agent repo at `--agent-rev`,
`LICENSE-*`, `README.md` with its first line renamed for the target's agent repo, wink's, and
`memory/MEMORY.md`, each when present, and from `scripts/vc-dr-apply/`, the work skeletons from
`vc-x1-template-2`, `{name}` filled, and the `.gitignore` lines. Both `@` must be empty, every path is
new, identical, or differs on disk, any that differs refuses the whole apply, and nothing is committed.
Template-2's `CLAUDE.md` is left out, Claude Code reading `AGENTS.md` itself.

Tried on `~/tmp/wt5`, made by `init --test-repo ~/tmp/remotes --target ~/tmp/wt5`, from vc-x1 at
`9f2d3444ce85`: the set identical to vc-x1's, the agent README renamed, and a second run refused by
the clean-start guard, which also caught a stray `custom.md`. The `differs` refusal and the two-path
form are not yet run, `wt5`'s Todo **vc-dr-apply refuses what differs**.

##### chore: vc-x1 adopts the agent-files v0.3.1-1 draft

The `v0.3.1-0` draft left out what the proposal found after it, so the rules a session read still
said a cycle stays local until its Land while this cycle backed up mid-cycle by a bend. vc-x1 writes
the draft `v0.3.1-1` itself, its own commits the source since the `-0` copy, wink's call
(2026-11-04), with all four refinements:

- **The user decides when each side commits and publishes**: **A cycle stays local until its Land**
  becomes **A cycle is local by default** in `AGENTS.md` and `rationale.md`, its index line with it,
  any mix of work and agent commits valid, and the mid-cycle backup its three commands. It ends this
  cycle's bend of the old rule.
- **Every unpublished rung's ocid is real before it is published**: in `AGENTS.md` and `jj.md` as
  those three commands, one command of its own name for them proposed and not built, since `push -s
  b` does not commit the agent side or restamp today.
- **New repos start from an adopter's committed rules**: the Terminology's Set, **Changing the
  agent-files**, its rationale, and `notes.md` name `scripts/vc-dr-apply.sh` and no template
  repository.
- **At rest leaves the tail for the next push**: already `-0`'s text and the tool's behavior, so only
  "the next cycle's agent commit" becomes "the next agent commit", one landing mid-cycle now.

Step 8 names `--opening` and `--closing`. The dev-build rule leaves this file's header for
`versioning.md`, where **Run your own build** becomes **Which build runs**, wink's (2026-11-04): the
user is asked when the dev build is newer than the plain name, and the plain name runs otherwise. A
default rather than a `custom.md` exception, wink's too, since it is simpler. The proposal entry
records `-1` and adds **Step 8 names the bookends** and **The user chooses the dev build**. The
marker file is `agent-files-v0.3.1-1`.

Also: a new Todo, **What a
session writes after its final push, sandbox on and off**, files the tail samples kept in
`tmp-durable/`, and the continuation notes keep only the `wt5` item. The retired `config --validate`
that misled the `wt5` agent stays in `cycle-model.md` and `vc-config.md`, wink's call (2026-11-04),
for the new **A hyphenated subcommand becomes a command and a flag**, which makes it `validate
--config`.

##### feat: the cycle record returns to TODO.md, indexed by zTOC.md

The trial of `zDOING.md` and `zCLOSED.md` ends, wink's call (2026-11-04): the cycle-record goes back
to `TODO.md > ## In Progress` and `## Closed`, and the continuation notes to `TODO.md`'s first
section, as the agent-files say. `zTOC.md` is new, `# TOC` with `## Ladder`, the running cycle's rung
titles each linking to its subsection in `TODO.md`, `_None._` between cycles, and `## The TOC`, the
references to the important files `TODO.md` lists now. The `[record]` keys, the default closing body,
and what `commit --closing` moves follow.

##### feat: a test repo converts an existing repo

The likeliest real use of a template is converting an existing repo, and nothing rehearses it safely,
since `--test-repo` refuses `--adopt`. `init --test-repo <REMOTES> --target <copy> --adopt` converts a
copy of a repo with its remotes in REMOTES, wink's (2026-11-03). An adopted repo may already have an
`origin` on GitHub, which would break "nothing leaves this machine", so what `--adopt` does with an
existing `origin` is read first, and init refuses it or points it at REMOTES.

##### feat: the readers take any mix of ochid and ocid

The readers and validators assume one `ochid:` on a work commit and a link both ways. They learn the
general rule first, so no build writes a form it cannot read.

##### feat: ocid is real only when the agent commit came first

A Land rewrites every rung to fill its `ocid:`. It stops, so a rung's stays `xxxx` and only the commit
carrying the agent commit's SHA is written, and `validate-desc` accepts `xxxx` on the others.

##### fix: commit --closing reads the nearest opening and keeps the cycle's base

The two folded `commit --closing` Todos, wink asking if they are one cycle and the answer yes. A stack's
closing reads its own opening, and its first parent is the opening's parent when that descends from the
trunk. We think it refuses when a cycle below is not closed, which answers wink's question on dangers.

##### feat: land takes @- and the work scope by default

`land` requires `--scope`, refuses work alone, and finds its source from every bookmark. Its `--scope`
defaults to work, leaving the agent repo alone, any scope that applies may be named, and it takes the
folded Todo's `@-` default.

##### feat: a push to a release bookmark seals the cycle

Nothing makes the link permanent once `land` stops. `[release] bookmarks` names the set, in
`vc-config.md` and the generated model too. A push to one with `-s b` commits the agent `@`, fills the
`ocid:`, checks each tree unchanged, and publishes, and it refuses a bookmark that moved and an `xxxx`
naming another agent `@`. Work alone leaving an agent commit unsealed is an error unless `--yes`.

##### fix: testing leaves a cycle able to continue

A test between the steps can leave the next one refused, full `validate`'s `cargo fmt` dirtying the
`@` that `land` needs empty. A test changes nothing the next step depends on, or the step says what it
found and how to go on.

##### feat: close, land, and push say where the cycle stands

Nothing says what state a cycle is in. Each command ends by printing it and what may come next, and
refuses what may not, and `vc-x1 close` arrives.

##### docs: the rules and README name the three steps

The README and **agent-files(proposal): v0.3.1**: **A cycle stays local until its Land**, **Land is the
sequence, and `vc-x1 land` its integrate step**, **Only push publishes**, and **Work commits link by
URL** are rewritten, and the Terminology gets a word for what makes a cycle permanent, we think
"release". The rules themselves change in that proposal's cycle.

##### feat: close, land, and push compose with testing closing

Closing out the cycle.

# References

[0]: #feat-close-land-and-push-compose-with-testing-opening
[1]: #feat-init---test-repo-makes-a-test-repo
[2]: #feat-init-drops---use-template
[3]: #chore-vc-x1-adopts-the-agent-files-v031-0-draft
[4]: #feat-an-agent-commit-covers-its-work-commits
[5]: #feat-an-initial-vc-dr-applysh
[6]: #feat-a-test-repo-converts-an-existing-repo
[7]: #feat-the-readers-take-any-mix-of-ochid-and-ocid
[8]: #feat-ocid-is-real-only-when-the-agent-commit-came-first
[9]: #fix-commit---closing-reads-the-nearest-opening-and-keeps-the-cycles-base
[10]: #feat-land-takes---and-the-work-scope-by-default
[11]: #feat-a-push-to-a-release-bookmark-seals-the-cycle
[12]: #fix-testing-leaves-a-cycle-able-to-continue
[13]: #feat-close-land-and-push-say-where-the-cycle-stands
[14]: #docs-the-rules-and-readme-name-the-three-steps
[15]: #feat-close-land-and-push-compose-with-testing-closing
[16]: #chore-vc-x1-adopts-the-agent-files-v031-1-draft
[17]: #feat-the-cycle-record-returns-to-todomd-indexed-by-ztocmd
