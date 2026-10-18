# The field

The kinds of grid, the endless field and how a pattern is kept on it.

## Decisions

1. **Two kinds of grid: loom and peyote.** Brick stitch goes. A file that
   names `brick` is not read (`EXC` of `RULE pattern.version`).
2. **The field has no end.** A cell has signed coordinates; (0, 0) is the
   cell a new pattern is centred on. Only the filled cells are kept.
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
8. **A cell outside the active area is a pale outline:** the outline of an
   empty cell, fainter.
9. **The active area is the box itself,** with no margin around it. A bead
   can be drawn outside it all the same.
10. **The pale outlines are hidden when a bead is drawn small.** Below
    `PALE_MIN_BEAD` the ground stays bare outside the active area.
    - Why: a screen of small beads holds tens of thousands of outlines,
      costly to draw and of no use to the eye.
    - The cost: the mesh holds the cells of the screen and half a screen
      more on every side, and is built again when the screen leaves them.
      At `PALE_MIN_BEAD` in a window of 1000 by 700 points a build took
      about 40 ms of a release build; a frame that only moves the mesh
      takes well under one.
11. **A new pattern is an empty field,** shown on cell (0, 0). A pattern
    with beads opens on the middle of its active area.
12. **A turn keeps the middle of the screen.** What is seen in the middle
    stays there; a pattern has no fixed middle any more.
