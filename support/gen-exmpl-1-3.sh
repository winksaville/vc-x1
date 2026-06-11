#!/usr/bin/env bash
#
# gen-exmpl-1-3.sh — regenerate the example transcripts for
# README.md's "Testing the ochid-trailer guard" section.
#
# - Scaffolds a scratch jj repo under /tmp (offline; no remote).
# - Runs example 1 (synchronous refusal), example 2 (normal
#   squash), example 3 (detached race -> failure marker), and the
#   example-3 resolution (retry succeeds, then surfaces the
#   marker after the current run's output).
# - Prints each `$ vc-x1 ...` command followed by its actual
#   output, separated by `==== ... ====` banners, ready to paste
#   into the README's code blocks.
# - Refuses to start if failure markers are already pending — the
#   resolution step would surface (and delete) them.
# - Removes the scratch repo on exit.

set -eu

status_glob="$HOME/.cache/vc-x1/finalize-status/*.status"

if compgen -G "$status_glob" >/dev/null; then
    echo "error: pending finalize failure markers exist — run any" >&2
    echo "vc-x1 command to surface them, then re-run this script" >&2
    exit 1
fi

repo=$(mktemp -u /tmp/vc-x1-guard-XXXXXX)
trap 'rm -rf "$repo"' EXIT

hr() { printf '\n==== %s ====\n\n' "$1"; }

hr "scaffold: $repo"
jj git init "$repo"
echo a > "$repo/file.txt"
jj commit -m 'prev journal' -R "$repo"

hr "example 1 — synchronous refusal (expect exit 1)"
echo b >> "$repo/file.txt"
jj describe -R "$repo" -m 'new journal

ochid: /abc123abc123'
printf '$ vc-x1 finalize --repo "%s" --squash --delay 0\n' "$repo"
rc=0
vc-x1 finalize --repo "$repo" --squash --delay 0 || rc=$?
echo "exit=$rc"

hr "example 2 — cleared description squashes normally (expect exit 0)"
jj describe -R "$repo" -m ''
printf '$ vc-x1 finalize --repo "%s" --squash --delay 0\n' "$repo"
rc=0
vc-x1 finalize --repo "$repo" --squash --delay 0 || rc=$?
echo "exit=$rc"

hr "example 3 — detached race (child refuses, drops a marker)"
echo c >> "$repo/file.txt"
printf '$ vc-x1 finalize --repo "%s" --squash --delay 5 --detach\n' "$repo"
vc-x1 finalize --repo "$repo" --squash --delay 5 --detach
jj describe -R "$repo" -m 'late journal

ochid: /def456def456'

# Wait for the child's failure marker (up to 15s).
found=
for _ in $(seq 1 30); do
    if compgen -G "$status_glob" >/dev/null; then
        found=1
        break
    fi
    sleep 0.5
done
[ -n "$found" ] || { echo "error: no failure marker appeared" >&2; exit 1; }

hr "resolution of example 3 — retry succeeds, then surfaces the marker"
jj describe -R "$repo" -m ''
printf '$ vc-x1 finalize --repo "%s" --squash --delay 0\n' "$repo"
rc=0
vc-x1 finalize --repo "$repo" --squash --delay 0 || rc=$?
echo "exit=$rc"

hr "done (scratch repo removed on exit)"
