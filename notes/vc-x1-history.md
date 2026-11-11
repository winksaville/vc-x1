# vc-x1's history: the links that do not resolve

The known list of the cycle **chore: complete vc-x1, ready for vc-x2**: every commit in vc-x1's
history whose link to its partner cannot be made to resolve, with why. `vc-x1 validate-desc` over a
fresh clone reports an `err` or a `miss` for these and for nothing else, and the same after a
whole-history `vc-x1 trailers --update`, which vc-x1's history has not been given. Each is named by
its change id, which a rewrite keeps.

Found by the rehearsals of 2026-11-06, in a fresh clone of the rehearsal's remotes, and checked on
the unrewritten history on 2026-11-09. In wink's work-repo the 3 drafts below resolve, since it
holds them.

## Errors

### The work-repo

- `mnonktwnuywp` 2026-07-08 chore: prep pin-bot-to-main cycle (0.68.0-0): its `ochid:` names
  `mzovqnqsmnlk`, which is in no agent-repo, the original's included.

### The agent-repo

The partner is a vc-x1 work commit that is not on `main`, each `ochid:` naming it correctly. Each
session's `vc-x1 push` log, in the transcript, names the pair and the repo, `work_chid=<id>,
bot_chid=<id>` and `commit -R /home/wink/data/prgs/rust/vc-x1`, and the bookmark pushed.

A cycle that never landed, **feat: the family directory**, on the bookmark
`feat-the-family-directory`, deleted after it. Its work commits remain in wink's work-repo, in jj's
operation history and not on any bookmark: at operation `b1bd5d35580e` each names its agent
partner, and its tip is `d0c396883a58`.

- `opqrntkrtxrk` 2026-10-17 feat: the family directory closing, naming `uwsznqssswzt`
- `mkxpzrvwvzvy` 2026-10-16 docs: touched files' prose loses its semicolons, naming `ovyutsvwlysp`
- `nnkyvxoxyvqz` 2026-10-16 docs: custom.md follows the family directory, naming `yslmonknrloo`
- `qtumtlnrmknt` 2026-10-16 feat: config finds the family directory by home, naming
  `psywsvrqluxo`
- `xmkzlkvlsuuq` 2026-10-16 feat: add the family directory, naming `mnvttxkntyqv`
- `ypsvtsslrkss` 2026-10-16 feat: the family directory opening, naming `oonqmttzlwtk`

The rungs of **agent-files(adoption): from iiac-perf, zc-ring-x1, 2026-08-31**, on the bookmark
`agent-filesadoption-from-iiac-perf-zc-ring-x1-2026-08-31`. We think the ladder was squashed when it
landed, as `oxqznutxpxvs`, titled with the bare cycle title and naming another agent commit, which
predates the rule that a ladder is never squashed. The rungs are in no repo, and the work-repo's
operation history begins at 2026-09-28.

- `vputsqvzvstx` 2026-08-31 docs: re-sync our copy from the payload, naming `ummxpxvkqpuu`
- `ltqplmmzmsqn` 2026-08-31 docs: copy the folded set into the payload, naming `xspxusqnlsxp`
- `zxspvztyuwkv` 2026-08-31 docs: triage the three-way diff and the members' proposals, naming
  `ruvsvzoxplrk`
- `sqoptttmvkyo` 2026-08-31 docs: define the agent-files type and title grammar, naming
  `smnyzxztklus`
- `mmxnuskzyqvt` 2026-08-31 agent-files(adoption): from iiac-perf, zc-ring-x1, 2026-08-31
  opening, naming `ulnvkqsqkmrs`

The partner is a draft local to wink's work-repo, on no bookmark and never pushed, replaced by
what landed, wink's call 2026-11-06 to list them rather than publish them:

- `qulkqsywytxm` 2026-08-04 refactor: trapezoid-push + body-intro validation opening, naming
  `yqxnsurlwsrr`
- `wzrymrkvuxmk` 2026-07-20 refactor: extract push/state.rs, naming `qknonomzmytm`, which landed as
  `zmvmrxslqxsm`
- `kxnnttnrmwoo` 2026-07-20 chore: open merge close-out cycle, naming `ozvukmlnmyuu`

The `ochid:` is malformed: its value contains digits, so it is not a jj change id, and read as a git
SHA or its prefix it names no commit in either repo, the original work-repo's included:

- `mkqtvoknqzlp` 2026-04-13 fix: remove submodule from init/clone (0.31.1), naming `34b6c258817b`

Its partner is the work commit `utxrqywnsplo` of the same title, whose `ochid:` names this agent
commit by its SHA, `3743deed09c0`, from when both sides linked by SHA. `trailers` makes that the
change id, so the link from the work side resolves, and only this one, from the agent side, is
broken. We think `34b6c258817b` is the work commit's SHA before it was rewritten, authored
2026-04-06 and committed 2026-04-13, and this clone holds no record of the earlier SHA.

## No trailer

Each is a `miss`, which `validate-desc` does not count as an error. A rewrite would give the work
commits their `pcid:` lines alone.

- the work-repo's `ltwkxykzutws` 2026-08-13 docs: messages specimen for the vc-x1-messages README
- the work-repo's `lxonplwlulwq` 2026-04-01 feat: Add design-for-symlink-new-clone-cmd.md
- the agent-repo's `surlunmtosqn` 2026-05-23 chore: post-push session capture (0.57.0)

## Older id forms

Not on the list, since they resolve: an 8 or 32 character change id. `trailers --update` makes a
work commit's into its 12, and `validate-desc` gives an agent commit's a note, the agent-repo never
being rewritten. Six agent commits carry one, and five work commits did before the rewrite.
