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

Written 2026-10-30:

- `tmp/mistake-aside-todo.patch` and `tmp/mistake-aside-zDOING.md` hold **chore: carry setting a
  mistake aside into v0.3.1**, set aside when **WIP: cleanup** was dropped. They are gitignored and
  on this machine alone. The patch's context has moved, so its edits may need
  reapplying by hand: **Setting a mistake aside** added to **agent-files(proposal): v0.3.1**,
  **abandon and revert set a mistake aside** deleted from `## Todo`, and a reread note on
  **squash-push shows what it takes, and push follows jj**.
- `tmp/wflow.sh`, `tmp/wflow2.sh`, and `tmp/wflow3.sh` are the trials behind the matrix in **commit works at every scope, and --local keeps it on the
  machine**.

## In Progress

### feat: ochid trailers are URLs

Picked from `## Todo`, filed the same day, wink's call (2026-10-30).

#### Problem

(wink + agent, 2026-10-30) `vc-x1` stamps `/.claude/<chid>` on every agent trailer whatever the
directory is called, from the constant `OCHID_BOT_LABEL` in `src/desc_helpers.rs`, so this workspace's
`.agent-session` is never named, and nothing in a published commit says where its partner lives.
Anyone cloning just the work-repo can't resolve the agent side. Wink's decision: fix it correctly,
the trailer a URL in both directions. Folds in **ochid values may be URLs** and **ochid: agent-repo
location qualifier**, and the ochid-label bullet of **The docs and the code say .claude where they
mean the agent-repo directory**. The design is sketched in [Per-user bot repos via URL-shaped
ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid).

- The form, the repo's URL and the change id:
  - `ochid: https://github.com/winksaville/vc-x1.agent-session#qrxovvvqxnrq`
  - `ochid: https://github.com/winksaville/vc-x1#uzkpltwmukvx`
- The change id and not a SHA: a SHA changes on every squash and rewrite, and a clone keeps change
  ids. So the URL says where to look and `vc-x1` resolves the rest, and it is no clickable link to
  the commit, GitHub knowing only SHAs.
- Resolved locally first: a URL matching a repo of this workspace by its `origin` needs no network,
  and only a URL matching nothing fetches.
- The URL is read from `origin` at commit time and normalized to https, so ssh and https clones stamp
  the same value. Whether `[remote] agent-repo` in `.vc-config.md`, which already names the
  agent-repo's remote, should supply it is to be decided.
- No remote yet, `init`'s cross-links made before the remotes and `adopt` or a local-path remote:
  a fallback is defined, and we think it is the path form, `/.agent-session/<chid>`, the configured
  directory rather than the constant. A `file://` path is no answer, being one machine's.
- A renamed repo leaves its trailers stale, and rewriting them rewrites published history, so a small
  mapping of old URL to new in `.vc-config.md` is the likely answer. A deleted or private repo is the
  link rot the design note accepts: the code stays authoritative and only the context is lost.
- Published trailers stay valid and are never rewritten: `/.claude/<id>` and `/<id>` keep resolving.
  Today `validate_ochid` compares the prefix exactly, so it and `fix-desc` learn this, or they flag
  or rewrite all of history.
- Readers before writers: one rung teaches every reader the URL form, a later one stamps it, so no
  build writes a form it cannot read, across the family's repos on different versions too. The
  readers: `extract_bare_id`, `validate_ochid`, `fix-desc`, `lookup::partner`, the `dr-1` fixture,
  and push's pending-partner check.
- A cycle's own rungs carry the URL too, wink's call (2026-10-30), by the bends in the Deliberation.
- `jj.md`'s trailer section and the rules naming `/.claude/` follow, so **agent-files(proposal):
  v0.3.1** carries the change.
- Beyond a work-repo and its partner: the ARM session of 2026-08-31 made six commits in the sibling
  `vc-x1-messages` clone (`main` at `5cf8aad7`) while its transcript rode vc-x1's agent-repo with no
  work-repo commit to pair with, so nothing links the two histories. A URL-shaped ochid on the
  transcript commit could name the messages-repo commit, the same shape as a pull request whose real
  history lives in another repo.

#### Solution

Provisional, one rung per item in the Ladder, readers before writers.

#### Acceptance check

Provisional:

- A commit made by the landed build stamps the URL form in both directions, and `validate-desc`
  and `lookup` resolve it, with no network when the URL names a repo of the workspace.
- Every published trailer on `main`, `/.claude/<id>` and `/<id>`, still passes `validate-desc`, and
  `fix-desc` proposes no rewrite of them.
- `vc-x1 validate` passes.

#### Deliberation

- Local and work only until the Land, wink's call (2026-10-30): every commit is made by `commit -s w
  --no-push`, the opening with `--skip remote` too, so every work commit carries the same `ochid:`,
  the agent `@` that `vc-x1 chid -s a` prints. This bends Opening step 2's publication and the
  per-rung flow's push until the Land, and the Land is outside the bend, its pushes approved as
  usual. No `squash-push -s a` and no agent commit runs until the Land, since either would leave
  those trailers dangling, a hazard **commit works at every scope, and --local keeps it on the
  machine** records.
- The Land is by `land -s b`, since `land -s w` is refused today: it commits the agent `@` as the
  partner, a trailer back to the closing, and publishes both repos, agent first.
- This cycle's own trailers are URLs, wink's call (2026-10-30), replacing `/.claude/` on the opening
  while it is local. Two bends follow, for this cycle's commits alone and until the Land:
  - No hand-written trailers: until [feat: commit and init stamp a URL-shaped ochid and ocid][4] stamps it,
    a commit's trailer is rewritten after the commit by `jj describe`, the value otherwise what
    `commit` stamped. Nothing is published, so no published commit is re-described.
  - The `ocid:` trailer, wink's (2026-10-30): the agent commit's SHA as a URL beside `ochid:`, on
    each commit as `.../commit/xxxx` until the Land, which commits the agent `@` first and replaces
    the placeholder, the agent-repo being neither committed nor pushed before then.
  - The binary: commits and the Land run by `vc-x1-dev`, built from the rung before, since the
    installed `vc-x1` reads `https://...#<chid>` as a change id named `<repo>#<chid>`, so its Land
    would see a changed partner and restamp the trailer.
  - The description review, wink's (2026-10-30): no separate review of each commit's description,
    since a local one can still be changed before the Land. The work review stays.
- Ladder labels are the rung's `-N`, `[0]`, `[1]`, and so on, as the specimen numbers them, wink's
  catch (2026-10-30), replacing `[cN]` from this commit on.
- The Land stamps a local cycle's commits, wink's and the agent's (2026-10-30): it commits the agent
  `@` first, with one `ochid:` URL per work commit, then writes `ochid:` and `ocid:` on each work
  commit from opening to closing, its trees unchanged, and pushes the agent repo first and the work
  repo after. So a Land pushes the agent commit as stamped and folds nothing into it, what arrived
  since staying in `@` for the next cycle, since a fold changes the SHA every `ocid:` names.
- No `squash-push -s a` from now on, wink's (2026-10-30): the session's tail is bookkeeping, and it
  rides the next cycle's agent commit. At rest's item 2 goes in **agent-files(proposal): v0.3.1**.
- What publishes, wink's (2026-10-30): `vc-x1 commit`, with or without `--land`, and `vc-x1 land`, which
  is exactly like it, are local and need no approval. `vc-x1 push`, exactly like `commit --push`, is
  what publishes, and it needs approval unless `--yes`. The approval's mechanism is open, and we think
  it is the Bash hook sending what publishes to the permission prompt, since an agent can type
  `--yes` as easily as anything else.
- A spawned Land, running after the agent goes quiet, is out of this cycle, wink's (2026-10-30), and
  is in **vc-x1 is an actor**.

#### Ladder

Provisional, the rungs after the opening named as the work is understood:

- [feat: ochid trailers are URLs opening][0] (done)
- [feat: the readers take a URL-shaped ochid][1] (done)
- [feat: an ochid URL comes from the repo's origin][2] (done)
- [fix: commit stays local unless --push asks][3] (done)
- [feat: commit and init stamp a URL-shaped ochid and ocid][4]
- [feat: land stamps a local cycle's trailers][5]
- [feat: land sets the ocid trailer to the agent commit's SHA][6]
- [docs: the rules and README name the URL form][7]
- [feat: ochid trailers are URLs closing][8]

##### feat: ochid trailers are URLs opening

Opens the cycle, its version at the `-0` form. Also carries the Todo work that led here: **commit
works at every scope, and --local keeps it on the machine** filed, folding **commit and land take
the same scopes**, and this entry's own folds.

##### feat: the readers take a URL-shaped ochid

Every reader of a trailer now takes `<scheme>://<repo>#<chid>` beside the path form, and no writer
stamps it yet, so nothing on any line changes.

- `extract_bare_id` splits at the last `/` or `#`. Everything that reads a trailer goes through it:
  `pair`, `validate-desc`, `fix-desc`, and `lookup`.
- `validate_ochid` accepts the side's label or any URL, a prefix with `://` ending in `#`.
  - The label stays the check, so a `/` on the agent's side still fails, and a published
    `/.claude/<id>` passes as it did.
  - An scp-style `git@host:...#<id>` is not a URL here, since stamps are normalized to https.
- `fix-desc` keeps a trailer's URL when it fixes the ID, where it would have replaced it with the
  label.
- `list`'s ochid column shows the URL form as `#<chid>`, since a whole URL pushes the column out.
- Not yet: a URL naming the wrong repo passes when its ID resolves, the comparison with `origin`
  belonging to [feat: an ochid URL comes from the repo's origin][2]. A URL matching nothing reports
  not found and fetches nothing.
- Also files **vc-x1 is an actor** in `## Todo`, from the deliberation on how a Land runs.

##### feat: an ochid URL comes from the repo's origin

A repo now has a canonical URL, and a trailer's URL is checked against it, while nothing stamps one
yet.

- `url::canonical_repo_url` turns `https`, `http`, `ssh`, and scp-style remotes into
  `https://<host>/<path>`, so ssh and https clones agree. It drops a user, password, and port, since
  the value goes into published commits, and a trailing `/` and `.git`. A local path and `file://`
  have none.
- `OchidForms`, by `ochid_forms_for`, is what a trailer naming a repo may be: the published label,
  the path form by the directory's own name, `/.agent-session/`, the fallback for a repo with no URL,
  and the URL from its `origin`.
- `validate_ochid` checks against it, so a URL naming another repo is now wrong, in any spelling of
  the right one being accepted, and with no hosted `origin` to compare a URL passes on its ID.
- `fix-desc` reports a wrong URL and does not rewrite it, since a renamed repo would otherwise have
  all its history rewritten. The mapping of old URL to new stays open, not in this cycle.

##### fix: commit stays local unless --push asks

Inserted, wink's call (2026-10-30), since **A cycle stays local until its Land** relies on it, and
forgetting `--skip remote` published the bookmark. It grew into the default itself.

- A bare `vc-x1 commit` publishes nothing, `--opening`'s bookmark included. `--push` is the one thing
  that publishes, a rung's pair or the opening with its bookmark.
- `--local` and its visible alias `--no-push` say the default outright, so what already says
  `--no-push`, `jj.md` among it, stays true until **agent-files(proposal): v0.3.1** runs.
- `[commit] push` is gone, wink's (2026-10-30): "always require `--push`" means no workspace setting
  changes what a bare `commit` does. `commit` no longer reads the key, and `validate-config` reports
  one still in a config as an unknown key.
- `--opening --no-advance --push` publishes the bookmark at the start and keeps the opening local, the
  one way left to publish a cycle's name alone.
- With nothing to push, the check that `@-` is on a remote does not run, so a cycle can open on a
  line that is not published.
- Also files **bookmark takes create, delete, and move** and **init defaults to --local** in `## Todo`.
- Not yet: `--land` still pushes. That goes in [feat: land stamps a local cycle's trailers][5], with
  `vc-x1 land`. `build-dr-1.py` asks for `--push`, since it builds a published ladder.

##### feat: commit and init stamp a URL-shaped ochid and ocid

##### feat: land stamps a local cycle's trailers

##### feat: land sets the ocid trailer to the agent commit's SHA

##### docs: the rules and README name the URL form

##### feat: ochid trailers are URLs closing

[0]: #feat-ochid-trailers-are-urls-opening
[1]: #feat-the-readers-take-a-url-shaped-ochid
[2]: #feat-an-ochid-url-comes-from-the-repos-origin
[3]: #fix-commit-stays-local-unless---push-asks
[4]: #feat-commit-and-init-stamp-a-url-shaped-ochid-and-ocid
[5]: #feat-land-stamps-a-local-cycles-trailers
[6]: #feat-land-sets-the-ocid-trailer-to-the-agent-commits-sha
[7]: #docs-the-rules-and-readme-name-the-url-form
[8]: #feat-ochid-trailers-are-urls-closing
