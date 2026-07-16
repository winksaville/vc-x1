# Support scripts

Helper scripts for maintaining this repo's docs and examples.

## Gen Example 1 - 3 output

[`gen-exmpl-1-3.sh`](gen-exmpl-1-3.sh) regenerates the transcripts
for the
[Testing the ochid-trailer guard](../README.md#testing-the-ochid-trailer-guard)
section of the top-level README:

- Scaffolds a throwaway init fixture under `/tmp`
  (`--repo local=...`: offline, local bare remotes) and runs the
  guard examples against its bot repo.
- Runs example 1 (refusal: trailer-bearing description on `@`) and
  example 2 (normal squash+push after clearing the description).
- Prints each `$ vc-x1 …` command followed by its actual output,
  separated by `==== … ====` banners, ready to paste into the
  README's code blocks.
- Removes the fixture (and its `~/.claude/projects` symlink) on
  exit.

Run it from the project root (or anywhere — it only touches `/tmp`
and the symlink directory):

```bash
support/gen-exmpl-1-3.sh
```
