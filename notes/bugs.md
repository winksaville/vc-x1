# Bugs

Known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` …) the same way
as `## Todo` in `todo.md`; run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. **`finalize::surface_previous_failures` is racy and
   bounded by "next vc-x1 run".** The current model has
   four gaps:
   - **Stale forever.** Markers sit on disk until the
     next `vc-x1 <anything>` runs. If the user abandons
     the workspace (e.g., CI / scheduled use), failures
     are never surfaced.
   - **Concurrent surface_previous_failures.** Two
     `vc-x1` runs racing: both `read_dir`, both print
     the marker, one deletes (the other's `remove_file`
     silently fails). The user sees the same failure
     printed twice.
   - **Mid-write torn read.** A `finalize --exec` child
     writing a marker while a sibling `vc-x1` is
     surfacing could read partial content. Atomic-rename
     on write would close this.
   - **No notify-at-failure path.** A detached
     `finalize --exec` failure only becomes visible when
     the user next runs *any* `vc-x1` command — fine for
     interactive use, invisible for CI / scheduled use
     where there may be no next run.

   The exec-child gate in `main.rs` (since 0.52.0-3)
   patches one related case — the detached child eating
   its own prior markers before the user can see them —
   but doesn't address any of the above. Holistic fix
   needs locking + atomic writes + maybe a
   notify-at-failure path for hands-off use.

# References
