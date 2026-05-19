#!/usr/bin/env python3
r"""Renumber `notes/todo.md` entries in `## Todo` and `## Bugs`.

- Matches any leading `N. ` prefix (not only `1. `), so the script
  closes gaps after deletions and absorbs insertions equally well.
- Shifts continuation-line indent by the delta in prefix width
  (e.g. 4→3 when a 2-digit entry becomes 1-digit, or +1 the other
  way). Empty lines pass through.
- Idempotent on already-renumbered content.
- Convention: intro paragraphs in `## Todo` and `## Bugs` begin
  every line with exactly 1 leading space so they don't match
  the entry regex (`^\d+\.\s`). Minimum is the principled choice
  here — the convention's job is to dodge the column-0 anchor,
  not to add visual emphasis. CommonMark strips up to 3 leading
  spaces on paragraphs, so the indent is invisible in the
  rendered output.

Slated for replacement by a Rust `vc-x1 fix-todo` / `validate-todo`
subcommand pair in a future cycle (see notes/todo.md).
"""

import re

PATH = "/home/wink/data/prgs/rust/vc-x1/notes/todo.md"

# Regex for a leading `N. ` on a line — `\d+` is one or more digits,
# `\.` is a literal dot, `\s` is a whitespace character.
NUM_RE = re.compile(r"^(\d+)\.\s")


def renumber(content: str) -> str:
    """Walk `## Todo` and `## Bugs`, renumber entries, shift continuations."""
    # Split the file into lines, walk them, build the new file line-by-line
    # in `out`, then rejoin at the end.
    lines = content.split("\n")
    out: list[str] = []

    # State carried across the loop:
    section: str | None = None  # current `## Heading`, or None outside any
    entry_num = 0  # running entry count within `section`
    shift = 0  # for the most-recent entry's continuation
    # lines: how many spaces to add (positive)
    # or remove (negative) from their indent so
    # they stay aligned under the new prefix.

    for line in lines:
        if line.startswith("## "):
            # Section heading — entering a new section; reset counters.

            # Strip trailing spaces
            section = line.strip()

            # Number start with 0 pre-incremented below
            entry_num = 0

            # Actual value calcuated below  on a match
            # If it doesn't match no shifting will be performed
            shift = 0

            # Output section line as is even trailing spaces(?)
            # I wonder if this should be "section" instead?
            out.append(line)
            continue

        # Only the two numbered sections need renumbering.
        if section in ("## Todo", "## Bugs"):
            m = NUM_RE.match(line)
            if m:
                # Entry first line: starts with `N. `. Renumber to the next
                # sequential value and record the prefix-width delta for the
                # entry's continuation lines that follow.
                old_prefix = m.group(0)  # e.g. "9. "  or "10. "
                entry_num += 1
                new_prefix = f"{entry_num}. "  # e.g. "10. " or "9. "
                out.append(new_prefix + line[len(old_prefix) :])
                shift = len(new_prefix) - len(old_prefix)
                continue

            if shift != 0 and line.strip() != "":
                # Assumes no empty lines between numbered items(?)

                # Continuation line under the current entry. If the entry's
                # prefix changed width (shift != 0), pad or trim the leading
                # spaces by the same amount so the text stays aligned under
                # the new prefix. Blank lines pass through (no indent to fix).
                leading = len(line) - len(line.lstrip(" "))
                new_leading = max(0, leading + shift)
                out.append(" " * new_leading + line.lstrip(" "))
                continue

        # Everything else (blank lines, lines outside the two numbered
        # sections, continuation lines when shift == 0) passes through
        # unchanged.
        out.append(line)

    return "\n".join(out)


def main() -> None:
    with open(PATH, encoding="utf-8") as f:
        content = f.read()
    content = renumber(content)
    with open(PATH, "w", encoding="utf-8") as f:
        f.write(content)
    print("Renumbered todo.md")


if __name__ == "__main__":
    main()
