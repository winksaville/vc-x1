# Commit model

Two commit bodies from the `config loader` cycle in [cycle-model.md](cycle-model.md), shown as
specimens rather than a skeleton: a work rung's body and a bookend's. Copy the shape, not the
words. The rules are in [Commit-body form](prose.md#commit-body-form).

## A work rung

Title: `refactor: split loader from parser`

```
Config loading and parsing are one function, so the CLI cannot load a
config without parsing it and a test cannot parse one without a file on
disk. The move into its own module is what forces the split.

Solution: a loader finds and reads the file and a parser takes text and
returns the schema, so a test hands the parser a string, and `main` gets
the schema from one call.

See details in TODO.md:
##### refactor: split loader from parser
```

The first paragraph states this commit's problem and defines "split", the word the title assumes.
The second opens "Solution:", nothing names a file that the diff already lists, and the pointer
is the rung's subsection heading, a line of its own, which a search of `TODO.md` finds at any
commit.

## A bookend

Title: `refactor: extract config loader opening`

```
Opens the cycle "refactor: extract config loader".

See details in TODO.md:
### refactor: extract config loader
```

A bookend resolves no problem of its own, so its body names the cycle by its title and points at
its record, the cycle's heading. The closing's body is the same shape with "Closes".
