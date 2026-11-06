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

7. **The grid is chosen in a `PlateMenu`** (`RULE ui.menu`): the two kinds,
   then "Ruling…" (decision 21). While it is open nothing else of the
   window answers (`src/ui/blocker.rs`).
8. **The ground is in shade outside the work area** (decision 13). The
   paper of the work area is lighter. An empty cell has no outline; near the
   middle of the screen and the pointer it shows a shadow (decision 20).
   - Replaces the pale outlines outside the active area: one colour, and the
     active area is seen as a whole.
9. **The active area is the box itself,** with no margin around it. A bead
   can be drawn outside it all the same.
10. **The shadows are drawn at every size of a bead.** A zoom never hides
    them.
    - Was: outlines, hidden below a smallest bead, because tens of thousands
      of them cost too much to draw.
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
      and the field with it, in the same step of undo as the bead.
    - A bead taken away shrinks the sheet to the box of the beads left, in
      the same step of undo. It never shrinks below the first screen
      (`View::sheet`) and never grows: a sheet set smaller in the tools
      stays. It moves as little as it takes to hold the beads
      (`Bounds::resized_over`). `History::fit_sheet` does both.
    - Why the first screen: an empty sheet is still a place to work on.
    - Was: the sheet stayed as it grew when the bead went.
    - The work area is still the box around the sheet and the beads: the
      beads of a file can lie past its sheet.
    - Was before that: the sheet stayed, and only the work area grew; it was
      the sheet again when the bead went. One undo took the bead, not the
      size.
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
15. **There are no outlines of empty places.** Decision 20 replaces them.
    - Was: a repeating texture of outlines, a quad of it for each column of
      the field.
16. **A bead is a sprite.** One white bead is drawn into a texture for the
    size of a bead; every bead is a quad of it, tinted its colour, and so is
    every shadow. Nothing is tessellated, so nothing is kept from frame to
    frame but the textures.
    - At a bead of 10 points in a window of 1000 by 700 points, a press, a
      step of a zoom and a move take about 0.1 ms each.
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
    bead, as far as decision 23 lets it. A frame where the field has no size
    is skipped.
    - Why: the place of the pattern was counted from the top left corner of
      the field, so a narrower window, a phone turned or the keyboard of
      android carried the pattern off the screen.
    - The place and the zoom of the view are not written to the file: they
      belong to the window, not to the work.
    - Was: when no cell of the work area was on the screen after the change,
      the field was centred on it. Decision 23 replaces it: the field stays
      at the side it was moved to.
20. **An empty place shows the shadow of a bead only near the middle of the
    screen and the pointer,** on the paper, in the shade and past the field
    alike, each
    fading to its edge. A shadow is a bead of faint black: `SHADOW` on a line
    of the ruling (decision 21), the lighter `SHADOW_CELL` elsewhere. The
    rules are in `crates/obusina-canvas/src/shadow.rs`.
    - The frame is the screen at zoom 1 (`DEFAULT_BEAD`), in the middle of
      the screen, its corners rounded. Zoomed out it shows how much of the
      field one screen at zoom 1 holds. At zoom 1 and in, it is the screen
      or more, and every empty place on the screen has a shadow.
    - The round is `NEAR_RADIUS` cells around the pointer.
    - Neither ends at the edge of the field: a shadow stands past it too.
    - Why the frame: android gives no pointer while a stylus hovers. winit
      0.30 drops the events of a hover (`MotionAction::HoverMove`), so the
      round alone shows nothing under a finger or a stylus.
    - Why the round stays: zoomed out, it shows the shadows past the frame
      under a mouse.
    - Why the fade: a hard edge reads as a hole.
    - Zoomed out the frame holds as many cells as the screen at zoom 1, a
      few thousand quads.
    - Was: outlines over the whole field; then outlines over the paper,
      and in the shade only in the frame and the round. A soft spot with no
      cells was weighed and left: it does not show where a bead goes.
21. **The ruling: every few columns and rows the shadows are darker,** to
    count the beads by. `Ruling` of `obusina-pattern` holds the step, the
    columns and rows from one line to the next, and the offset of a line
    from cell (0, 0). A step of 0 draws no line along its axis. On peyote a
    row of lines goes up and down with the columns, as the beads of a row
    do.
    - The ruling is part of the pattern and its file (`ruling`), so a
      pattern opens with the lines it was counted by. A change is a step of
      undo. Format 1 changes itself (`EXC` of `RULE pattern.version`).
    - "Ruling…" of the menu of the settings opens a menu with the four
      numbers on top, then "Accept" and "Cancel", as the picker of a colour
      (`src/ui/settings_menu.rs`). A number shows on the field at once; only
      "Accept" keeps it. Cancel, Escape or a press beside the menu put it
      back.
    - Why from cell (0, 0): it does not move. The middle of the sheet moves
      when a bead grows or shrinks it, and the lines would jump while
      drawing; the middle of the screen moves with the field.
    - Why darker shadows: they mark the beads that are counted. Lines
      between the cells were weighed and left; on peyote a line across a
      row is a stair.
    - Was: every shadow of one strength, `SHADOW`.
22. **A button of the tools hides the shadows,** and shows them again. It
    stands beside the button of the settings. Whether they show is a setting of
    the program (`src/settings.rs`), not of the pattern: it is how the
    window shows, not part of the work.
23. **The field never leaves the screen.** At least `KEEP_BEADS` beads of
    it stand on the screen along each axis, or the whole field when it is
    smaller. A drag, a wheel, a zoom, a turn or a new size of the window
    that would take more away moves the field back as little as it takes,
    so it stays at the side it was moved to (`Canvas::keep_on_screen`).
    - Why: a field moved off the screen was lost, and found again only by
      a new size of the window, which put it in the middle.
    - Why not the middle: the field stays where the person put it.
24. **The paper is a colour of the pattern,** kept in its file (`paper`);
    a new pattern has `DEFAULT_PAPER`. "Colour of the paper…" of the menu
    of the settings opens a picker and four papers on top, then "Accept"
    and "Cancel", as the ruling does. A colour shows on the field at once;
    "Accept" keeps it as a step of undo.
    - The shade, the shadows and the hatch come from the paper
      (`Tones::of` of `obusina-canvas`): darker than a paper lighter than
      `LIGHT`, lighter than a darker one, so they are seen on both.
    - Why in the pattern: a pattern for black thread is drawn on black,
      and opens on it.
25. **The beads have volume:** each is drawn round, lighter in the middle
    and darker at its sides and its bottom. "Volume" of the menu of the settings
    turns it off and on. It is a setting of the program, as the shadows
    are: how the window shows, not part of the work.
    - Why it costs nothing: the round bead is a second white texture,
      tinted as the flat one; a bead is still one quad.
    - Was: flat beads only. A white spot of shine and a halo of the colour
      around the bead were weighed and left
      (`experiments/mockups/src/mock/glow.rs`).
26. **Each entry of the menu of the settings shows what it is now** right
    of its label: the kind of grid, the step of the ruling, a swatch of the
    paper, on or off (`plate_menu::Detail`).
