# Panels

How a floating panel chooses where it stands and how it is laid out. The
widget itself is `crates/obusina-panel/README.md`.

## Decisions

1. **The pointer decides the edge, by a fixed distance.** An edge strip is
   `SNAP` points wide, cut to `SNAP_PART` of a small window. The edge held
   wins by `HYSTERESIS`.
   - Why: zones made of a part of the window are huge on a monitor and
     uneven on a phone. A pointer on a border made the panel flip.
   - The pointer, not the panel: a wide panel still reaches an edge.
2. **The content fits by length.** The beads count how many swatches stand
   in a line, so the swatches and the colours in hand fit.
   - Why not a wrapping layout of egui: a column of egui that wraps starts
     its next column after the whole width it was offered, and the panel
     grows every frame.
3. **The place of each panel is remembered.** `src/settings.rs` keeps the
   `Placement` of each panel by its name in `settings.yaml`, written when a
   panel is let go or folded.

## Decided, not built yet

The logic is as simple as it can be: three rules. They replace how the
panel is built now.

4. **The orientation is chosen in every frame of a drag:**

   ```text
   wanted = pointer in an edge strip ? the orientation of the edge : the current one
   wanted does not fit, the other fits → the other
   otherwise                           → wanted
   ```

   "Fits" means the panel, its grip under the pointer, lies inside the
   window. A panel turned in the middle does not turn back by itself. The
   same rule holds when the window changes its size.
   - Why: a panel pushed back inside loses the pointer from its grip.
5. **A place is a part of the window, nothing more.** Let go in an edge
   strip, the panel stands against that edge: its part on that axis is 0 or
   1, so it stays there when the window changes its size. There is no
   docked state.
6. **Nothing is drawn while a panel is dragged.** No ghost, no strips. The
   panel follows the pointer and turns at once; the turn is the hint.
7. **The tools do not break into lines.** A panel that does not fit turns
   instead.

## Open questions

- Two panels at two edges that meet overlap in the corner on a narrow
  window. They do not avoid each other. Left as it is for now.
