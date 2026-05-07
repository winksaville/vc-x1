# Forking and multi-user coordination

Captures design discussion about two related but separable
concerns:

1. **Forking** a dual-repo workspace — working on an older
   code-side base while keeping the bot side append-only.
2. **Multi-user collaboration** on the same dual-repo —
   multiple humans, multiple bot sessions, code-side merges.

Both surface the same underlying tension: the code repo has
the classic git/jj DAG (branches, merges, rebases), while the
bot repo is fundamentally an append-only journal of
conversations that should never be rewritten.

For the *content* of the bot repo (vendor identity, format
versioning, multi-bot conventions, vendor-subdir directory
layout), see the companion file
[`bot-data-formats.md`](bot-data-formats.md).

This file is forward-looking: most of what it describes is
the bot thinks, not implemented. Treat as a design reference,
not a status doc.

## Forking with the bot side staying linear

### The asymmetric situation

When work needs to happen on an older code-side base — e.g.,
0.41.1 work branched off 0.41.0 while 0.42.0 is in flight on
main — the code repo can fork without difficulty. That's a
normal git/jj branch.

The bot repo can't fork the same way: each session is a linear
conversation, and rewriting an existing session line would
falsify the historical record. The conversation that happened
is the conversation that happened; it doesn't get re-played
from a different branch base.

### chid stability across rebase

jj's two-id model is built for this scenario. Each commit has
both:

- `commit_id` — git SHA, changes on rewrite.
- `change_id` — stable across `describe` / `rebase` /
  `squash-into-self`.

`ochid:` trailers carry change_ids. So when a code-side branch
later rebases on top of another branch, change_ids are
preserved and ochid trailers in both directions still resolve.

The bot thinks the only operations that break this are
`jj squash` (folding two commits into one — collapses two
change_ids into one) and `jj abandon` (removing a commit —
loses the change_id entirely). Pure rebase is safe.

### Why not time-based linkers

Time isn't a stronger primitive than change_id:

- Clock skew between processes / repos breaks ordering.
- Time still doesn't survive rewrites any better than
  change_id does — you'd still need a stable identifier baked
  into the trailer that matches some property of the
  counterpart commit.
- change_id is jj's intended stability primitive — that's
  literally the job it exists to do.

If the change_id scheme ever shows weakness, the bot thinks
the answer is jj-side hygiene (don't `squash`-collapse during
rebase) rather than switching primitives.

### Partner bookmarks

The chosen technique for forking the code side while keeping
the bot side linear: **partner bookmarks**.

When a code-side branch needs to advance independently, create
a bookmark of the same name on the bot side starting at the
*current* bot HEAD — not at the historical bot commit paired
with the code-side branch base. The two bookmarks share a name
and advance together via `vc-x1 push`, but each carries its
own repo's state.

Other names considered and rejected:

- **mirror** — implies symmetric content. The bookmarks are
  not mirrors; they hold different content with no
  point-by-point correspondence.
- **tandem** — captures cooperation, but slightly less common
  in software vocabulary.
- **paired** — bland; "pair" can imply matching halves.
- **disjoint** / **asymmetric** — describe a property of the
  relationship, not a label for the technique.
- **yoked** — accurate metaphor but obscure.

**partner** won because it conveys cooperation without
implying duplication. Always subject to change as we live
with the term.

The asymmetric ordering between code and bot history (rebased
main commits on the code side will eventually point at bot
commits that come *before* their partner-branch pairs in bot
history) is a cosmetic wrinkle, not a correctness problem —
change_ids resolve regardless of position.

### Recovery anchor

The original main HEAD — the commit being "branched away
from" — should stay put, not be rewritten or rewound. It's
the recovery anchor for the operation. If the partner-branch
work goes wrong, main is still where it was, and recovery is
`jj abandon` on the new branch's commits.

In the 0.41.1 init+clone refactor, this means main stays at
its current 0.42.0-4 docs-capture commit (`55eadc8`) for the
duration of the side-branch work. That commit becomes
redundant after the eventual rebase — its content has been
re-landed on `init-clone-refactor` — and gets dropped during
the rebase rather than being rewound now.

## Multi-user collaboration

The bot thinks the same model — linear forest on the bot
side, multi-trailer reconciliation on the code side —
generalizes naturally to multi-user.

### Bot repo as a forest of linear chains

Each session is one append-only chain. The bot repo is a
forest of those chains; never merged. No bot-side merge
commits at all. Code-side has a DAG (merges, branches);
bot-side is a forest. The mapping is the multi-ochid trailer
set on each code commit, fanning out to the contributing
chain tips.

The bot thinks this is cleaner than "bot side mirrors code
side topology" because:

- Each conversation is a wire-format record of what was
  actually said. Merging two conversations into one commit
  would falsify that record.
- "Never rewrite bot commits" becomes an enforceable rule —
  change_ids on the bot side are automatically stable.
- Storage and sync stay simple — appending only, no merge
  conflict resolution on the bot side ever.

### Multi-line ochid trailer

Standard git trailer convention allows multiple values for
the same key. Just emit `ochid:` once per pointer:

```
ochid: /.claude/abc123
ochid: /.claude/def456
ochid: /.claude/ghi789
```

Single-pointer commits stay backward compatible — they're
just the N=1 case. No version bump needed; existing parsers
either already accept repeated trailers or are easy to
extend.

### When multi-ochid actually applies

- **Code-side merge with no extra bot work** — N ochids =
  the merge parents' contributing tip change_ids (one per
  parent). Possibly with de-duplication if parents share
  ancestry.
- **Code-side merge with conflict-resolution session** —
  N parents' ochids + the resolution session's tip ochid.
  If user A and user B merge into M and user C resolves the
  conflict, M has 3 ochids: A's tip, B's tip, C's session
  tip.
- **Single-user single-session** — still 1 ochid. Same as
  today.
- **Code rebase** — ochids unchanged (change_ids are
  stable). Same as today. Multi-user rebases work the same
  as single-user.

### Per-user vs shared bot repo

Open question worth deciding before code lands.

- **Shared bot repo** (today's model extended): one
  `.claude/` repo holds all sessions across all users. Push
  from any user appends a chain. Pros: viewer sees
  everything in one place; cross-references resolve locally.
  Cons: every user pulls every other user's session data on
  sync — could be heavy. Permissions are coarse.
- **Per-user bot repos**: each user has their own
  `.claude-<user>/` repo, ochid trailers carry a user
  qualifier (`ochid: /.claude-alice/abc123`). Pros:
  lightweight sync, natural permissions. Cons: viewer needs
  to know all the bot-repo URLs to show full context;
  offline viewing breaks if you don't have the relevant
  user's repo.

The bot thinks **shared** is the default with a per-project
escape hatch — most teams want the cross-references to
"just work." The qualifier in ochid (the path component
after the leading `/`) already encodes either model:
`/.claude/<chid>` for shared, `/.claude-alice/<chid>` for
per-user. The trailer format itself is forward-compatible.

### Per-user bot repos via URL-shaped ochid

Per-user bot repos become practical for distributed
projects (open-source, multi-org) when ochid trailers can
qualify by URL rather than only workspace-relative path.
This generalizes the per-user model in the section above
from "co-located teams sharing a workspace" to "anyone on
the public internet."

**Trailer shape.** Existing path form is
workspace-root-relative (`/.claude/<chid>` for shared,
`/.claude-alice/<chid>` for per-user-in-shared-workspace).
URL form generalizes the qualifier to a remote repo:

```
ochid: https://github.com/alice/bot-sessions#<chid>
ochid: git@github.com:alice/bot-sessions.git#<chid>
```

Parser dispatch is one rule: if the qualifier parses as a
URL, fetch over the network; otherwise resolve relative
to workspace root. Existing path-form trailers stay the
N=1 backward-compatible case. No schema versioning
needed.

**Link rot is the load-bearing problem.** A contributor
can publish their bot repo, get merged, then later make
it private or delete it. The ochid URL 404s; bot context
is lost. The bot thinks this is a real-but-not-
catastrophic leak — the commit's code and
`Signed-off-by:` line remain authoritative; bot context
is a nice-to-have for understanding patch development,
not a load-bearing semantic. Direct analogue: dead URLs
in academic citations, mailing-list links in older
kernel commit messages. Annoying for archaeology, not
fatal.

**Mitigations, in order of practical adoption:**

1. **Project-side mirroring at merge time.** When a
   maintainer accepts a PR, project tooling fetches the
   contributor's bot repo at the referenced chid and
   mirrors to a project-namespace archive (e.g.
   `bot-archives/<contributor>-<date>-<chid>.jsonl`).
   Future ochid resolution falls back to the mirror if
   the original URL 404s. The bot thinks this is the
   only mitigation most projects would actually adopt —
   cheap, obvious upside, no friction for contributors.
   Same pattern as LKML's permanent archive.
2. **Cryptographic stapling.** Embed a hash of the bot
   repo's tip at submission time:
   `ochid: <url>#<chid>;sha256=<hash>`. Tamper-detection
   stays sound even if retrieval fails; doesn't recover
   data. Adds maintenance burden to trailer parsing.
3. **CI-enforced live ochid at merge.** Project CI
   fetches the URL, verifies it resolves with parseable
   content, before allowing merge. Doesn't prevent
   later lock/delete but ensures the merge moment was
   sound. Contentious in open-source culture (gates
   contributors on infra they don't own).

**Opt-out via project policy.** A no-bot-context commit
isn't a leak — it's a project policy choice. A project
that wants bot transparency mandates each commit carry
≥1 `ochid:` trailer or an explicit
`No-bot-context: <reason>` trailer. Enforceable in CI,
identical pattern to existing `Signed-off-by:`.
Contributors who want privacy decline that project's
contribution norms; their commits go elsewhere or omit
the trailer where the project allows.

**Cross-bot interop bonus.** Once URL-shaped, the
trailer doesn't have to point at a Claude Code repo.
Codex, Cursor, Aider, custom in-house tools — anyone
whose bot leaves a publishable session record can host
their own bot repo and emit ochid trailers in the same
format. The trailer becomes "where to find the AI
session that produced this commit," not "where to find
the Claude Code session." Strictly bigger ecosystem
play with no extra design cost.

**Implication for shared-central scaling.** URL-shaped
per-user repos potentially short-circuit the
size/threshold problem documented below. Each
contributor pays for their own bot repo; project pays
for an opt-in mirror archive sized per its policy. The
"50 devs share one ~15G bot repo" problem becomes "50
devs each maintain their own ~300M bot repo, project
mirrors selectively."

Trade-off: shared-central keeps cross-references local
and fast; per-user-via-URL needs a network fetch (or a
warm mirror) to resolve any non-local trailer. Pick by
project shape — small co-located teams favor shared;
distributed open-source favors per-user-via-URL.

### Bot-side commit immutability

Codify "never rewrite bot commits" as a hard rule. The
reason: change_ids on the bot side need to be permanently
stable for ochid trailers from the code side to resolve
forever. Even `jj describe` (description-only) changes the
commit_id, which doesn't break change_ids but does signal
that bot commits are mutable in a way they shouldn't be.

Trade: any commit-level metadata fix has to be done by
appending a new commit, not amending the existing one.
Awkward but correct.

### Bot-repo size and scaling thresholds

Concrete measurement on a single-developer vc-x1 project
(2026-05-07, after `cargo clean`):

- Code repo (excluding `.claude`): ~4M
- Bot repo (`.claude`): 311M
- Ratio: ~78:1 bot-to-code

Cause: every Claude Code session writes a UUID-named
`.jsonl` file. Content is mostly unique per session —
git/jj packs each file's append history well, but
cross-file dedup is weak. So bot-repo bytes scale roughly
linearly with total session count.

Linear extrapolation to shared-central multi-user (the bot
thinks this is a reasonable upper bound; actual savings
from cross-session dedup are small):

- 5 devs: ~1.5G
- 20 devs: ~6G
- 50 devs: ~15G

Rough operational thresholds:

- **≤ ~10 users:** shared-central is fine.
- **~10-50 users:** shared-central starts hurting
  (multi-minute initial clone; noticeable `vc-x1 sync`
  pulls).
- **> ~50 users or > ~10G bot repo:** mitigations become
  necessary.

### Monotonic-growth asymmetry

Bot-repo size only grows. Once a session jsonl is
committed, the bot-side immutability rule keeps it
forever. This is by design (historical record), but it
means the bot repo has no analog of `cargo clean`:
code-repo working bytes can be reclaimed (build artifacts
come back on `cargo build`); bot-repo bytes are permanent.

Sizing assumptions for shared-central scaling are a
one-way ratchet, not a steady-state cost.

### Mitigation menu (when thresholds approach)

None worth building preemptively. Listed so trigger points
have known responses:

1. **Bot-repo pruning/archiving.** Move closed-out
   sessions (older than N months, or tied to commits no
   longer in any active branch) to an archive repo.
   Active bot repo stays small; archive fetched on demand
   when older history is referenced. The bot thinks this
   is the cheapest big win — recent sessions are the hot
   set, old ones are cold.
2. **Sparse fetch by trailer-referenced sessions.** When
   fetching the code repo, derive the set of bot-side
   change_ids referenced by ochids in the fetched range
   and fetch only those bot commits. Requires git/jj
   sparse-fetch + a trailer-walking step. Heavier to
   build but decouples clone cost from bot-repo size.
3. **Per-user bot repos.** The structural alternative —
   see "Per-user vs shared bot repo" above. Viable when
   shared-central pruning/sparse-fetch isn't enough.
4. **LFS for `.jsonl`.** Probably overkill — LFS targets
   binary blobs, not append-only text. Listed only
   because it's a known git escape hatch.

### Tracking trigger

Concretely measurable: the 311M baseline is the current
benchmark. When the shared-central bot repo crosses ~1G
or starts dominating clone time noticeably, that's the
design trigger for pruning. User count alone is not the
right signal; bot-repo size is.

## Future-direction notes

Two alternatives to the partner-bookmark technique that may
be worth considering once we live with the current scheme:

### Always keep the bot repo on main

Instead of creating a partner bookmark on the bot side for
each code-side branch, the bot side could be locked to a
single bookmark — `main` — for the entire lifetime of the
project. Every push would advance bot main, regardless of
what bookmark the code side is on.

The bot thinks this aligns with "bot history is one linear
journal" more strongly than partner bookmarks do. Partner
bookmarks introduce *parallel* bot bookmarks even though the
bot history under them is still linear; an always-main
discipline removes that parallelism entirely.

Trade: requires push / finalize / sync to support asymmetric
bookmark names per side (see next note).

### Multi-bookmark-name support in push / finalize / sync

Today, `vc-x1 push <bookmark>` advances the same bookmark
name on both code and bot sides. The "always-on-main"
discipline above (and other asymmetric workflows) requires
push / finalize / sync to take separate bookmark arguments
per side, e.g.:

```
vc-x1 push --code-bookmark init-clone-refactor --bot-bookmark main
vc-x1 push -B init-clone-refactor              # bot defaults to main
vc-x1 push init-clone-refactor                  # both, today's behavior
```

The bot thinks the right shape lands once `--scope` is
fully wired across the dual-repo commands — `--scope`
already handles "which side does this command apply to,"
and adding per-side bookmark arguments is a natural
extension.

## Open questions worth thinking about

- **Conflict resolution sessions: how do they get
  authored?** When user C resolves a merge conflict, does
  their bot session start a fresh chain rooted "from
  nowhere," or branch off one of the contributing chains?
  The chain-rooting question affects how the viewer renders
  it.
- **De-duplication on octopus merges.** If parents share
  ancestry, you'll get redundant ochids. Trim to unique
  tips? Keep all? The bot thinks trim, but worth deciding.
- **Bot-side bookmark scheme.** Per-session bookmark?
  Per-user-per-feature? Naming convention matters because
  bookmarks are how `jj git push` decides what to send.
- **Viewer concerns.** "Show me the sessions that
  contributed to this commit" is straightforward (read the
  trailers, walk back in each chain to the relevant range).
  "Show me the code commits this session contributed to" is
  the inverse and needs an index, since change_ids don't
  carry forward references. Probably build the index from a
  one-time scan of code-side trailers.
- **Migration story for existing repos** with single-ochid
  trailers — they're already valid in the multi-ochid
  scheme (N=1 case), so probably no migration needed. Worth
  confirming on the parser side.
