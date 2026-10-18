# The field

The kinds of grid, the field drawn and how a pattern is kept on it.

## Decisions

1. **Two kinds of grid: loom and peyote.** Brick stitch goes. A file that
   names `brick` is not read (`EXC` of `RULE pattern.version`).
2. **A cell has signed coordinates.** (0, 0) is the cell a new pattern is
   centred on. Only the filled cells are kept. The model has no edge; the
   field drawn has one (decision 13).
3. **The active area is the box around the filled cells.** It is computed,
   never set. An empty pattern has none. An empty cell outside it is drawn
   dimmed.
4. **The parity of a peyote column is the one of its column on the field.**
   An odd column is half a bead lower; for a negative column the parity is
   `rem_euclid(2)`.
   - Why: a shift by an odd number of columns lifts every odd column a whole
     bead against the even ones. The same rows then show another picture,
     where other beads touch. Nothing tells: the file reads and the counts
     match.
5. **The file keeps the place on the field.** `origin` is the cell of the
   field of the first bead of the first row; `rows` are the active area. A
   pattern opens where it was, and a peyote column keeps its parity.
   `width` and `height` leave the file: they are the size of `rows`. An
   empty pattern has no rows and `origin` `[0, 0]`. Format 1 changes itself
   (`EXC` of `RULE pattern.version`).

```text
on the field, columns 1 2 3       the file
           ┌──┐
      ┌──┐ │B │ ┌──┐             "origin": [1, 0],
      │A │ └──┘ │C │             "rows": ["ABC"]
      └──┘      └──┘
```

6. **There is no fill.** The tool goes, and with it `Pattern::region` and
   `Grid::neighbours`, which only the fill asked.
   - Why: the empty space of an endless field has no end to fill to.

7. **The grid is chosen in a `PlateMenu`** of two entries (`RULE ui.menu`).
8. **The ground is in shade outside the active area.** The paper of the
   active area is lighter. An empty cell is the same faint outline
   everywhere in the field; where the outlines end, the field ends.
   - Replaces the pale outlines outside the active area: one tile, one
     colour, and the active area is seen as a whole.
9. **The active area is the box itself,** with no margin around it. A bead
   can be drawn outside it all the same.
10. **The outlines are hidden when a bead is drawn small.** Below
    `OUTLINE_MIN_BEAD` only the ground is drawn: the paper and the shade.
    - Why: a screen of small beads is a dense net of lines, of no use to the
      eye.
11. **A new pattern is an empty field,** shown on cell (0, 0). A pattern
    with beads opens on the middle of its active area.
12. **A turn keeps the middle of the screen.** What is seen in the middle
    stays there; a pattern has no fixed middle any more.
13. **The field drawn is the start sheet and twice the active area.**

    ```text
    ┌──────────────── field: the box around both ────────────────┐
    │ ┌──── start sheet: the cells of the screen ────┐           │
    │ │                    ┌── twice the area ──────┼──────────┐ │
    │ │                    │      ┌─ active ─┐      │          │ │
    │ │                    │      └──────────┘      │          │ │
    │ └────────────────────┼────────────────────────┘          │ │
    │                      └───────────────────────────────────┘ │
    └────────────────────────────────────────────────────────────┘
    ```

    - The start sheet is the cells under the screen when the pattern is
      first shown. It stays for the session.
    - Twice the active area is the area grown by half its width on each
      side, and by half its height above and below.
    - Outside the field the ground is bare, and a press does nothing. A bead
      put near the edge grows the active area, and the field with it.
    - Replaces the field with no end of decision 2.
14. **The size of the sheet is set in the tools.** Two fields, columns and
    rows, start at the size of the first screen and change the sheet around
    its middle (`Bounds::resized`), up to `MAX_SIDE`. A number is dragged or
    typed, so a finger reaches it.
    - The size lives for the session, like the sheet. It is no step of
      undo: it changes no bead.
15. **The outlines are one repeating texture, apart from the beads.** A tile
    of outlines is drawn once for the size of a bead; a column of the field
    is one quad of it. The beads are their own mesh.
    - Why: the outlines were most of one mesh and were built again on every
      press. At the smallest bead with outlines in a window of 1000 by 700
      points, a press took about 40 ms of a release build; now a press, a
      step of a zoom and a move each take a few ms.
16. **A bead is a sprite.** One white bead is drawn into a texture for the
    size of a bead; every bead is a quad of it, tinted its colour. Nothing is
    tessellated, so nothing is kept from frame to frame but the two
    textures.
    - In the case of decision 15, a press, a step of a zoom and a move now
      take about 0.1 ms each.
