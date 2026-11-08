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
   a level of the menu of the settings (decision 27). While it is open nothing else of the
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
13. **The size of the field is by the beads or set by hand.** It is kept
    in the pattern (`Pattern::field`): `None` by the beads, the default;
    a box when set by hand.

    | | by the beads | by hand |
    | --- | --- | --- |
    | work area, on paper | the box of the beads; none with no beads | the field |
    | a bead put past it | is put; the box grows | is refused |
    | beads past it | none | when the field was made smaller; they stay |
    | the count of the panel | every bead | the beads in the field |
    | delete the beads of a colour | all | all; the entry names those past the field |
    | the picture of the list | the box of the beads | the field |
    | the shadows of empty places | in the field and past it | in the field only |

    ```text
    ┌──────────── bounds of the view: the work area, doubled ───┐
    │  shade: the cells past the work area                      │
    │       ┌──── work area: paper ────────────────┐            │
    │       │                                      │            │
    │       └──────────────────────────────────────┘            │
    │                      ● by hand: a bead left past the field│
    └───────────────────────────────────────────────────────────┘
    ```

    - The view moves within the work area doubled: half its width on each
      side and half its height above and below (decision 23). With no work
      area it moves freely.
    - An empty pattern by the beads has no paper. The frame of the
      shadows shows where to draw.
    - Why the size by the beads is the default: a pattern grows where it is
      drawn; a size set by hand is for a work of a known size.
    - Was: a sheet, the first screen at the start, grown by a bead past it
      and shrunk to the beads when one went, never below the first screen
      (`View::sheet`, `History::fit_sheet`). The first screen went with it.
14. **The size of the field is chosen in the level "Field" of the menu
    of the settings** (`src/ui/settings_menu.rs`, `src/ui/field_size.rs`).
    The entry shows `columns×rows`, or "by the beads".

    ```text
    ‹ Field
      By the beads      ✔ ──► the field goes; the menu stays
      Set                 ──► the field it had, or the first screen at
                              zoom 1 grown to hold the beads; the menu stays
      [40] × [30]           on top, with "Set" only: about the middle
      As it was           ──► the size the level was entered with
    ```

    - The numbers show on the field at once and change it about its middle
      (`Bounds::resized`), from 1 up to `MAX_SIDE`. A field made smaller
      than the beads keeps them past it (decision 13). The field has no
      offset of its own to set. A number is dragged or typed, so a finger
      reaches it.
    - The first screen is `View::first_screen`: the cells the screen holds
      at zoom 1, about its middle, whatever the zoom now.
    - The field is part of the pattern (decision 17): a change of the size,
      by the beads or set, is one step of undo when the level is left.
    - Was: "Size of the sheet", the numbers alone, never smaller than the
      box of the beads (`Bounds::resized_over`).
    - Was: two fields in the tools, where changes one after another were
      one step, since a number dragged changed it in every frame.
15. **There are no outlines of empty places.** Decision 20 replaces them.
    - Was: a repeating texture of outlines, a quad of it for each column of
      the field.
16. **A bead is a sprite.** One white bead is drawn into a texture for the
    size of a bead; every bead is a quad of it, tinted its colour, and so is
    every shadow. Nothing is tessellated, so nothing is kept from frame to
    frame but the textures.
    - At a bead of 10 points in a window of 1000 by 700 points, a press, a
      step of a zoom and a move take about 0.1 ms each.
17. **The field set by hand is written to the file,** a part of the work,
    not of the window: `field` in the file, its origin and its size. By the
    beads it is left out. The work area is not written.
    - Was: the sheet, `sheet` in the file. No release counts yet, so format
      1 changed itself and reads no `sheet` (the plan of the field of the
      workspace).
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
    alike by the beads, in the field only by hand (decision 13), each
    fading to its edge. A shadow is a bead of faint black: `SHADOW` on a line
    of the ruling (decision 21), the lighter `SHADOW_CELL` elsewhere. The
    rules are in `crates/obusina-canvas/src/shadow.rs`.
    - The frame is a square of cells: as many columns as rows, as many as
      the screen at zoom 1 (`DEFAULT_BEAD`) holds along the side that holds
      more. It stands in the middle of the screen, its corners rounded. At
      zoom 1 and in, it covers the screen, and every empty place on the
      screen has a shadow.
    - Why a square of cells: it does not change with the shape of the
      window or a turn of the phone, and a square of a pattern is a square
      of cells. On the screen it has the shape of a bead.
    - Was: the screen at zoom 1, of the shape of the screen.
    - The round is `NEAR_RADIUS` cells around the pointer.
    - Neither ends at the edge of the field: a shadow stands past it too.
    - Why the frame: android gives no pointer while a stylus hovers. winit
      0.30 drops the events of a hover (`MotionAction::HoverMove`), so the
      round alone shows nothing under a finger or a stylus.
    - Why the round stays: zoomed out, it shows the shadows past the frame
      under a mouse.
    - Why the fade: a hard edge reads as a hole.
    - Zoomed out the frame holds as many cells as a square on the longer
      side of the screen at zoom 1, a few thousand quads.
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
    - "Ruling" of the menu of the settings is a level with the four
      numbers on top (decision 27). A number shows on the field at once and
      stays.
    - Why from cell (0, 0): it does not move. The box of the beads moves
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
    a new pattern has `DEFAULT_PAPER`. "Background colour" of the menu
    of the settings is a level with a picker and four papers on top
    (decision 27). A colour shows on the field at once and stays.
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
    of its label: the kind of grid, the turn, the step of the ruling, the
    size of the field, a swatch of the paper, on or off
    (`plate_menu::Detail`).
27. **What changes the pattern is a level of the menu of the settings**
    (`src/ui/settings_menu.rs`):

    ```text
    grid, turn, the size of the field ──► a value chosen is done at once;
        the menu stays in the level
    ruling, field set, background ──► the editor on top (MenuItem::notice);
        a change shows at once and stays
        ‹, Esc, the way back of android ──► the top, on the entry
        "Put the old … back", "As it was" ──► the value the level was
                                 entered with;
                                 weak until the value changed
    a level left, or the menu closed in it ──► one step of undo, none when
                                                the value is the one it had
    ```

    The top of the menu ends with "Palette settings", which opens the
    window of the palettes; "Volume"; "Return to the list of patterns";
    "Close the settings". A locked pattern has the last three alone.
    - Why the change stays and is put back by an entry, not accepted by
      one: the value is seen on the field while it changes; one more entry
      to keep it was a step nobody needs, and undo takes it back anyway.
    - Was: a menu of its own for each, with "Accept" and "Cancel"; "Cancel"
      closed the whole menu.
    - The turn left the tools for its level: two buttons of the panel for
      a thing set once. The pencil and the picker went with them: a press
      paints, and the colour in hand is taken from the panel of beads.
