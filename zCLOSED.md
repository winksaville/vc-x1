# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### chore: the validation-failure Todo retires to the per-rung flow change

(wink + agent, 2026-10-15) The Todo "A validation failure is investigated before it is run again"
has two halves: the failure it records, whose cause was the `HOME` race, and a rule for the
Validate step of [The per-rung flow](AGENTS.md#the-per-rung-flow). The cycle `fix: no test sets
HOME` fixed the first, without citing the entry, which its search missed, and the second is the
next cycle's, so the entry has nothing left to wait for in `## Todo`.

The case, from the entry:

- A full validation failed once in the closing of the cycle **docs: the session lock is settled
  as flock** (2026-10-11), with no code changed, and the agent ran it again, saw it pass, and
  reported that. A pass on the second run says nothing of why the first failed, and wink wanted
  the failure looked into first.
- `init::tests::adopt_keeps_a_gitignore_that_has_the_line` and
  `fix_desc::tests::por_workspace_noops` each died making a commit, on "Couldn't load public key
  /h/.ssh/git-commits-wink@saville.com.pub". The runs before and after passed all 650.
- The cause: a test in `src/init/tests.rs` set `HOME` to `/h` for the whole test process while it
  checked `expand_vars`, and put it back, and a test signing a commit in that moment looked for
  the key under `/h`. Fixed by `fix: no test sets HOME`, which reproduced it with a sleep in the
  old test's window.
- Seen again 2026-10-15, before that fix. That session named the cause before running validation
  again, as the rule would have it, and the second run passed.

#### Deliberation

- The entry retires, and the next cycle is the rule, wink's call at review. The rule's text and
  the order to open that cycle first are in the continuation notes of [zDOING.md](zDOING.md),
  and the case is above, so nothing of the entry is lost.
  - The cycle first kept the entry, citing the fix, its rule open. That read as owing the rule to
    a later cycle no one had named, where wink meant this one to complete it.
  - The rule cannot join this cycle: an agent-file change is its own commit, and convention work
    its own cycle ([Changing the agent-files](AGENTS.md#changing-the-agent-files)).
- The pushed commit was amended and re-described in the work-repo, wink's go, its `ochid:`
  trailer kept, and the bookmark force-pushed: it had not landed. The bookmark keeps the name
  from the first title, as it is deleted at Land, and the agent-repo commit keeps the first
  title, since it is on the agent-repo's `main` already.
- Run now as its own cycle, wink's call over folding it into the next opening.
- Single-step, one Todo entry's retirement.
- Wink's go at the work review covered the description review and the first push, a bend of
  [The per-rung flow](AGENTS.md#the-per-rung-flow)'s steps 8 and 9. Land is outside it, wink
  reviewing before it. The amend and its force-push are outside it too, each with its own go.

#### Solution

The entry leaves `## Todo`. The continuation notes name the next cycle, the `agent-files` proposal
adding the rule to the Validate step, with the rule's text, and this record holds the case.

#### Acceptance check

- `TODO.md` has no entry "A validation failure is investigated before it is run again", and
  `zDOING.md`'s continuation notes name the per-rung flow change as the next cycle. Passed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- chore: the validation-failure Todo retires to the per-rung flow change (done)

Close-out shape: none to choose, single-step. Land fast-forwards `main` over the one commit.

# References
