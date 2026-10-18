# Panels

How a floating panel chooses where it stands and how it is laid out. The
widget itself is `crates/obusina-panel/README.md`.

## Decisions

1. **The shape changes on release.** While the grip is dragged, the panel
   keeps its shape. A ghost shows where the panel will stand and in which
   orientation.
   - Why: a panel that changes shape under the finger jumps. It also
     vanishes for one frame while it is measured again.
2. **The pointer decides the edge, by a fixed distance.** An edge strip is
   `SNAP` points wide, cut to `SNAP_PART` of a small window. The edge held
   wins by `HYSTERESIS`.
   - Why: zones made of a part of the window are huge on a monitor and
     uneven on a phone. A pointer on a border made the panel flip.
   - The pointer, not the panel: a wide panel still reaches an edge.
3. **A panel keeps a place, not an orientation.** A docked panel stands
   against its edge and is laid out by it. A free panel keeps its
   orientation. Both keep their part of the window when it changes its size.
4. **The content fits by length.** The content breaks itself into lines:
   - the tools break between groups of buttons (`src/ui/lines.rs`); a group
     is never split;
   - the beads count how many swatches stand in a line, so the swatches and
     the colours in hand fit.
   - Why not a wrapping layout of egui: a column of egui that wraps starts
     its next column after the whole width it was offered, and the panel
     grows every frame.
5. **A free panel does not turn.** A drag inside the middle keeps the
   orientation. No gesture turns a panel in place.
6. **The place of each panel is remembered.** `src/settings.rs` keeps the
   `Placement` of each panel by its name in `settings.yaml`, written when a
   panel is let go or folded.

## Decided, not built yet

These replace decisions 1 and 5 once they are built.

7. **A panel that does not fit turns.** While dragged in the middle, a
   panel that runs out of the window turns, if the turned panel fits with
   its grip under the pointer. It turns back when its first shape fits
   again. Both ways need `HYSTERESIS` more than the border, so it does not
   flip on it. The same holds when the window changes its size.
   - Why: a panel pushed back inside loses the pointer from its grip.
8. **The shape changes at once.** At an edge and when it does not fit, the
   panel turns while it is dragged, its grip under the pointer. No ghost,
   no strips, no lines are drawn.
   - Replaces decision 1: the turn is the hint.

## Open questions

- Two panels at two edges that meet overlap in the corner on a narrow
  window. They do not avoid each other.
