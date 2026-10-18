# The field

The kinds of grid, the endless field and how a pattern is kept on it.

## Decided, not built yet

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

## Open questions

- The menu of grids: a `PlateMenu` of two entries, or one button that
  switches.
- How the cells outside are dimmed, and whether they are hidden when a bead
  is drawn small.
- The active area: the box itself, or a margin of a cell or two around it.
