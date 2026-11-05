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
5. **The file keeps the place on the field.** `rows_origin` is the cell of
   the field of the first bead of the first row; `rows` are the active area.
   A pattern opens where it was, and a peyote column keeps its parity.
   `width` and `height` leave the file: they are the size of `rows`. An
   empty pattern has no rows and `rows_origin` `[0, 0]`. Format 1 changes
   itself (`EXC` of `RULE pattern.version`). The whole format is
   `crates/obusina-pattern/README.md`.

```text
on the field, columns 1 2 3       the file
           ┌──┐
      ┌──┐ │B │ ┌──┐             "rows_origin": [1, 0],
      │A │ └──┘ │C │             "rows": ["ABC"]
      └──┘      └──┘
```

6. **There is no fill.** The tool goes, and with it `Pattern::region` and
   `Grid::neighbours`, which only the fill asked.
   - Why: the empty space of an endless field has no end to fill to.

7. **The grid is chosen in a `PlateMenu`** of two entries (`RULE ui.menu`).
8. **The ground is in shade outside the work area** (decision 13). The
   paper of the work area is lighter. An empty cell is the same faint outline
   everywhere in the field; where the outlines end, the field ends.
   - Replaces the pale outlines outside the active area: one tile, one
     colour, and the active area is seen as a whole.
9. **The active area is the box itself,** with no margin around it. A bead
   can be drawn outside it all the same.
10. **The outlines are drawn at every size of a bead.** A zoom never hides
    the grid.
    - Was: hidden below a smallest bead, because tens of thousands of
      outlines cost too much to draw. One repeating tile costs the same at
      any size (decision 15), so the reason is gone.
11. **A new pattern is an empty field,** shown on cell (0, 0). A pattern
    with beads opens on the middle of its active area.
12. **A turn keeps the middle of the screen.** What is seen in the middle
    stays there; a pattern has no fixed middle any more.
13. **The work area is the sheet and the beads; the field is twice it.**

    ```text
    ┌──────────── field: the work area, doubled ────────────┐
    │  shade: the cells past the work area                  │
    │       ┌──── work area: paper ────────────────┐        │
    │       │ ┌── sheet: the numbers of the tools ─┤        │
    │       │ │                                    │        │
    │       │ └────────────────────────────────────┤        │
    │       │            ● a bead past the sheet   │        │
    │       └──────────────────────────────────────┘        │
    └───────────────────────────────────────────────────────┘
    ```

    - The sheet is the start size of the work area: the numbers of the
      tools, the cells of the screen when the pattern is first shown.
    - A bead put past the sheet, in the shade, grows the sheet to hold it,
      and the field with it, in the same step of undo as the bead. When
      that bead goes, the sheet stays as it grew; undo takes it back.
    - The work area is still the box around the sheet and the beads: the
      beads of a file can lie past its sheet.
    - Was: the sheet stayed, and only the work area grew; it was the sheet
      again when the bead went. One undo took the bead, not the size.
    - The field is the work area grown by half its width on each side and
      half its height above and below. Past the field the ground is bare,
      and a press does nothing.
    - The work area is on paper; the rest of the field is in shade.
    - Replaces the field of the sheet and twice the active area, and the
      field with no end of decision 2.
14. **The size of the sheet is set in the tools.** Two fields, columns and
    rows, start at the size of the first screen and change the sheet around
    its middle (`Bounds::resized`), up to `MAX_SIDE`. A bead put past the
    sheet grows it, and the numbers with it (decision 13). A number is dragged or typed, so a finger reaches
    it. A number is never less than the box around the beads: the sheet
    takes no smaller size, and moves as little as it takes to hold the beads
    (`Bounds::resized_over`).
    - Was: any size from 1, the beads left past a smaller sheet.
    - The sheet is part of the pattern (decision 17): a change of its size
      is a step of undo. Changes one after another are one step, since a
      number dragged changes it in every frame.
15. **The outlines are one repeating texture, apart from the beads.** A tile
    of outlines is drawn once for the size of a bead; a column of the field
    is one quad of it. The beads are their own mesh.
    - Why: the outlines were most of one mesh and were built again on every
      press. At a bead of 10 points in a window of 1000 by 700 points, a
      press took about 40 ms of a release build; now a press, a
      step of a zoom and a move each take a few ms.
16. **A bead is a sprite.** One white bead is drawn into a texture for the
    size of a bead; every bead is a quad of it, tinted its colour. Nothing is
    tessellated, so nothing is kept from frame to frame but the two
    textures.
    - In the case of decision 15, a press, a step of a zoom and a move now
      take about 0.1 ms each.
17. **The sheet is written to the file.** It is the size of the work area
    set by hand, a part of the work, not of the window: `Pattern::sheet`,
    and `sheet` in the file, its origin and its size. A new pattern takes
    the cells of its first screen as its start value, no step of undo. A
    file with no sheet opens the same way. The work area is not written: it
    is the box around the sheet and the rows.
18. **The turn is written to the file.** `turn` holds how far the pattern is
    shown turned, 0, 90, 180 or 270; a pattern opens turned as it was. The
    beads do not move, and the turn stays no step of undo: an `EXC` of
    `RULE ui.undo`, by the user's word.
    - Turning the beads themselves was weighed and left: a quarter turn of
      peyote is brick stitch, which is no grid any more, and a bead not
      square would lie the other way.
19. **A change of the size of the window keeps the middle of the field.**
    What is in the middle of the field stays there, at the same size of a
    bead. When after the change no cell of the work area is on the screen,
    the field is centred on the work area, as when a pattern opens. A frame
    where the field has no size is skipped.
    - Why: the place of the pattern was counted from the top left corner of
      the field, so a narrower window, a phone turned or the keyboard of
      android carried the pattern off the screen.
    - The place and the zoom of the view are not written to the file: they
      belong to the window, not to the work.
