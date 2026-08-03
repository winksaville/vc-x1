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
