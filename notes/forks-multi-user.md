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
