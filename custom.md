# custom.md - <project>'s project layer

The one agent-editable instruction file (see [AGENTS.md](AGENTS.md#custommd-the-project-layer)).
Loaded after AGENTS.md; on conflict, this file wins.

## Medium and validation

<what the artifact is; manifest and package name; see agent-data/versioning.md>

- **Full validation**
  - when: per-commit checklist step 4; skip-able for notes-only commits, mandatory at close-out
  - <the medium's commands, run as separate invocations, each exit status checked>
- **Fast validation**
  - when: ladder checklist step 3
  - <the medium's quick check>

## Project conventions and overrides

<project-local conventions; overrides of the pinned files, each naming the section it
supersedes. Empty at birth is fine, except the mailbox parameters below, which AGENTS.md's
acquaint-time practice and the pinned files' template pointers rely on.>

- **Mailbox parameters**: member name `<member>`; the template repository is at
  `<path-to-template>` (mailbox `messages/<member>.md` there, protocol in its `MESSAGES.md`)
- **Commit description: problem + solution, no version, no file list.** Supersedes
  cycle-protocol.md "Commit description" and cycle-checklists.md per-commit step 7 (their
  title rules stand except length; trailers unchanged). Adopted from iiac-perf's
  cycle-protocol.md "Commit description" 2026-08-06, dogfooded from the 0.78.3 close-out
  onward; template proposal pending at the 20260803 baseline review.
  - title <=50 cols; body is a problem statement then a solution statement, both broad,
    prose form, wrapped <=72
  - no version in title or body (the manifest records it; a renumber would falsify
    immutable text), no file list (the diff and `git show --stat` are the mechanical
    record), no deliberation (chores, todo, and the ochid-named session hold it)
  - the evidence: the 0.78.3 file-by-file body needed five redrafts across the cycle's
    riders and restated a diff already reviewed at the ready-to-commit gate
