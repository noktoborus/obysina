# Panels

How a floating panel chooses where it stands and how it is laid out.

## Decisions

Decided, not built yet. Each item replaces the current behaviour of
`crates/obusina-panel`.

1. **The shape changes on release.** While the grip is dragged, the panel
   keeps its shape. A ghost outline shows where the panel will stand and in
   which orientation. On release the panel takes that place.
   - Why: a panel that changes shape under the finger jumps. It also
     vanishes for one frame while it is measured again.
2. **The pointer decides the edge, by a fixed distance.** An edge zone is
   `SNAP` points wide, cut to a part of a small window. In a corner the
   nearer edge wins. The edge already chosen holds until the other one is
   nearer by `HYSTERESIS`.
   - Why: zones made of a part of the window are huge on a monitor and
     uneven on a phone. A pointer on a border made the panel flip.
   - The pointer, not the panel: a wide panel still reaches an edge.
   - Only the edge strips are drawn, no diagonals.
3. **A panel keeps a place, not an orientation.** A place is one of two:
   - docked: an edge and a position along it, as a part of its length;
   - free: a position as a part of the window.

   A docked panel stands against its edge. Its orientation is the one of
   the edge. A free panel keeps its orientation. When the window changes its
   size, a docked panel stays on its edge, a free one keeps its part of the
   window.
4. **The content fits by length.** The content is given its orientation and
   the length it may take along the axis. The palette computes how many
   swatches stand in a line from that length; a column too tall for the
   window becomes two columns.
5. **A free panel does not turn.** A drag inside the centre keeps the
   orientation. There is no gesture that turns a panel in place.
6. **The place of each panel is remembered.** It is written to the
   configuration and read at the next start.
