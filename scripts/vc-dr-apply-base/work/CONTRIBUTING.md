# Contributing

How to build, test, and change {name}, for a person or an agent. What {name} is is in
[README.md](README.md), and how a change is made is in [AGENTS.md](AGENTS.md#cycle-protocol).

## Tasks

The project's tasks, in the order a change uses them, each named by its first column, the name the
agent-files use for it.

| Task | What it does | When | Command |
|---|---|---|---|
| bump | sets the version-of-record ([Suffix scheme](agent-data/versioning.md#suffix-scheme)) | first, at each commit of a cycle | <command> |
| validate | <what it checks> | before each review, and before every commit | <command> |

## Agents

What an agent working here reads beyond the agent-files, read at acquaint after `custom.md`
([Acquaint](AGENTS.md#acquaint)). It is the project's layer over the agent-files, committed, so
every clone has it, and a workspace's `custom.md`, local and gitignored, overrides it.

- Agent-files: the version this project carries, named by `agent-data/agent-files-v*`.
