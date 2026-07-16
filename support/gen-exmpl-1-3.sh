#!/usr/bin/env bash
#
# gen-exmpl-1-3.sh — regenerate the example transcripts for
# README.md's "Testing the ochid-trailer guard" section.
#
# - Scaffolds a throwaway init fixture under /tmp
#   (`--repo local=...`: offline, local bare remotes) and runs the
#   guard examples against its bot repo.
# - Runs example 1 (refusal: trailer-bearing description on @) and
#   example 2 (normal squash+push after clearing the description).
# - Prints each `$ vc-x1 ...` command followed by its actual
#   output, separated by `==== ... ====` banners, ready to paste
#   into the README's code blocks.
# - Removes the fixture (and its ~/.claude/projects symlink) on
#   exit.

set -eu

parent=$(mktemp -u /tmp/vc-x1-guard-XXXXXX)
symlink="$HOME/.claude/projects/$(echo "$parent/work" | tr / -)"
trap 'rm -rf "$parent" "$symlink"' EXIT

hr() { printf '\n==== %s ====\n\n' "$1"; }

hr "scaffold: $parent"
vc-x1 init "$parent/work" --repo local="$parent"
bot="$parent/work/.claude"

hr "example 1 — refusal (expect exit 1)"
echo b >> "$bot/notes.md"
jj describe -R "$bot" -m 'new journal

ochid: /abc123abc123'
printf '$ vc-x1 squash-push -R "%s"\n' "$bot"
rc=0
vc-x1 squash-push -R "$bot" || rc=$?
echo "exit=$rc"

hr "example 2 — cleared description squashes and pushes (expect exit 0)"
jj describe -R "$bot" -m ''
printf '$ vc-x1 squash-push -R "%s"\n' "$bot"
rc=0
vc-x1 squash-push -R "$bot" || rc=$?
echo "exit=$rc"

hr "done (fixture removed on exit)"
