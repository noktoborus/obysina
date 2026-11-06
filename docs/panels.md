# Panels

How a floating panel chooses where it stands and how it is laid out. The
rules themselves are `crates/obusina-panel/README.md`.

## Decisions

1. **The logic is as simple as it can be.** Three rules: a panel turns by
   the edge strip under the pointer and by the room it has; let go in a
   strip it stands against that edge; nothing is drawn while it is dragged.
2. **The pointer decides the edge, by a fixed distance.** An edge strip is
   `SNAP` points wide, cut to `SNAP_PART` of a small window. The edge held
   wins by `HYSTERESIS`.
   - Why: zones made of a part of the window are huge on a monitor and
     uneven on a phone. A pointer on a border made the panel flip.
   - The pointer, not the panel: a wide panel still reaches an edge.
3. **A panel turns at once.** It turns while it is dragged, its grip under
   the pointer. The turn is the hint: no ghost, no strips, no lines.
4. **A panel that does not fit turns.** In the middle, one that runs out of
   the window turns if the turned one fits. It does not turn back by itself,
   so it does not flip.
   - Why: a panel pushed back inside loses the pointer from its grip.
   - At an edge the panel slides along the edge when it is let go, so there
     it only has to be no larger than the window. Else a column too tall
     for the pointer at the middle of a side would turn back into a row.
   - Whether the turned one fits is told by its size. A panel never drawn
     turned lays its content out once more in that frame, invisible, to
     measure it (`FloatingPanel::show`).
   - Was: the size of the other orientation guessed by swapping the sides.
     The column of the tools is half as tall as their row is long, so a row
     longer than the window was high never turned into a column at a side.
5. **A place is a part of the window, nothing more.** 0 or 1 on an axis is
   against an edge. There is no docked state.
6. **The tools have two layouts, like the beads.** In a row, one line of
   groups. In a column, two buttons on a line, so the column is half as tall.
   They do not break into lines by length: a panel that does not fit turns.
7. **The beads fit by length.** They count how many swatches stand in a
   line, so the swatches fit: a palette is long either way.
   - Why not a wrapping layout of egui: a column of egui that wraps starts
     its next column after the whole width it was offered, and the panel
     grows every frame.
8. **The place of each panel is remembered.** `src/settings.rs` keeps the
   `Placement` of each panel by its name in `settings.yaml`, written when a
   panel is let go or folded. A turn made because the window changed its
   size is not written.

## Open questions

- Two panels at two edges that meet can overlap in the corner. They do not
  avoid each other. Left as it is for now.
