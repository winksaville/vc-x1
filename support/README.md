# Support scripts

Helper scripts for maintaining this repo's docs and examples.

## Gen Example 1 - 3 output

[`gen-exmpl-1-3.sh`](gen-exmpl-1-3.sh) regenerates the transcripts
for the
[Testing the ochid-trailer guard](../README.md#testing-the-ochid-trailer-guard)
section of the top-level README:

- Scaffolds a scratch jj repo under `/tmp` (offline; no remote
  needed).
- Runs example 1 (synchronous refusal), example 2 (normal squash),
  example 3 (detached race → failure marker), and the example-3
  resolution (retry succeeds, then surfaces the marker after the
  current run's output).
- Prints each `$ vc-x1 …` command followed by its actual output,
  separated by `==== … ====` banners, ready to paste into the
  README's code blocks.
- Refuses to start if failure markers are already pending — the
  resolution step would surface (and delete) them.
- Removes the scratch repo on exit.

Run it from the project root (or anywhere — it only touches
`/tmp` and the marker directory):

```bash
support/gen-exmpl-1-3.sh
```

Note: in a terminal, example 3's detached child inherits
`/dev/tty`, so its refusal may also print ~5 s after the detach
line. The marker block in the resolution output is the durable
record of that same failure.
