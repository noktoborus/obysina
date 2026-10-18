# The palette

How the colours of a pattern are filled in. The design is drawn as the
mockups `palette-move` and `palette-drag` of `experiments/mockups`; the
window does not have it yet.

## Decisions

1. **The palette is filled in a window with no title,** a modal over the
   field. The colours stand in a line, a "+" at its end. A palette holds
   at most nine colours; at nine the "+" is off.
2. **A tap on a colour opens a `PlateMenu` of it** (`RULE ui.menu`): the
   picker of the colour on top, then move and remove. No button stands
   under a colour. "+" adds a colour and opens its menu.
   - Was: a "-" under each colour, which doubled the line and was read as
     a colour of its own.
3. **The picker is the notice of the menu,** as wide as the menu: a
   swatch, the number and `#rrggbb` over it, which change as it changes.
   The colour changes in the palette at once. The notice of a `PlateMenu`
   takes a widget of the caller (`Notice::widget`); a drag that begins
   on it and the keys while it holds the focus are its own, not the
   menu's.
   - Was: a picker under the line, for the colour chosen last; then one
     at the cell, too small; then one under the cell opened by "Pick a
     colour" of the menu, one step more than needed.
4. **How a colour is moved is open between two variants,** each a mockup:
   - `palette-move`: "Move" lifts the colour; the next tap on a colour puts
     it in that place.
   - `palette-drag`: the colour is dragged and rises above the pointer.
     The line already shows it in the place under the pointer, with the
     number of that place, the others moved aside and numbered anew; let
     go, it stays there. A tap still opens the menu, which has no "Move".
     - Was: a mark between two colours, which showed the place but not the
       number the colour would take.
5. **"+" adds the colour farthest from the others.** Of the steps of
   lightness, chroma and hue of OKLCh a screen can show, it takes the one
   whose nearest neighbour, among the colours and the paper of the field,
   is farthest in OKLab. Its menu opens, the picker on top.
   - Why: a fixed grey was the same every time and near the paper.

```text
tap a colour ──► PlateMenu: the picker of the colour on top, then
                    │──move────► the next tap on a colour puts it there
                    └──remove──► gone
drag a colour ──► the line shows it in its new place, renumbered ──let go──► it stays
tap "+" ──► the colour farthest from the others and the paper, its menu open
```

## Open

Each of these is decided before the step of the plan that needs it.

1. **Moving a colour:** `palette-move` or `palette-drag` (decision 4). With
   a drag the menu has no "Move".
2. **The beads of a colour removed:** gone with it as one step of undo
   (proposed), or "Remove" off while the colour has beads, or a question
   first.
3. **A list or nine places:**
   - a list: a colour removed or moved moves the numbers of the next ones,
     and the beads are numbered anew;
   - nine places (proposed): a number is a place, 1 to 9; a colour removed
     leaves a hole, which "+" fills first; a move swaps two places or goes
     to an empty one. The drag then lights a place instead of moving the
     others aside.
4. **The key of a colour in the file:** the letters of now, or the number
   of its place, `1`…`9`, with `.` for an empty cell (proposed). Format 1
   may change itself before 1.0.0 (`EXC` of `RULE pattern.version`).
5. **The hint of the numpad on a swatch,** the second subject of this
   chapter, waits for 3 and 4:
   - a number is a key of the numpad: in a column of two, `7 8 / 4 5 /
     1 2`; in a row, `1 2 4 5 7 8`; a seventh colour adds the third column,
     `3 6 9`;
   - proposed: the numbers stay as they are given, `1 2 4 5 7 8`, then
     `3 6 9`; a row in rising order; empty rows of a column folded; the
     empty place is `0`, wide, under `1 2`, first in a row;
   - the digits of the keyboard and the numpad choose the first colour,
     with shift the second, `0` the empty place; a field of the tools that
     holds the focus keeps its digits.
6. **The colours in hand** after a colour is removed or moved: they follow
   it, and one removed is the empty place (proposed).

## Plan

The steps that bring the window to the design. Each is one commit, its
test of the window with it (`RULE test.window`).

1. **The model.** `Pattern` holds at most `MAX_COLOURS` (9) colours; a file
   with more is refused, `PatternError::Palette`. A colour is added,
   changed, moved and removed through `Pattern`; a bead keeps its colour,
   not its number. By open 2, 3 and 4.
2. **The history.** `Edit` grows a step for each: add, change, move,
   remove. The changes of a colour while its picker is dragged are one
   step, as the size of the sheet is (`History::resize_sheet`).
3. **The window of the palette.** `src/ui/palette_window.rs`: a `Modal`
   with no title, the line of colours and "+", opened by a button of the
   panel of beads. A tap on a colour opens its `PlateMenu`, the picker in
   its notice (`Notice::widget`), then the entries of open 1. Every text in
   `locales/app.yml`, en and ru.
4. **The new colour.** `next_colour` of the mockup moves to the binary,
   with the crate `palette` and its test.
5. **The numpad.** The swatches of the panel of beads show the numbers
   and stand as on a numpad (open 5); the keys choose a colour.
6. **The mockups.** When the window has a part of the design, its mockup
   goes, or is cut down to what is still open: a mockup is not kept beside
   the code it drew.

## Where the work stands

- Done: decisions 1 to 5 as mockups; `PlateMenu` takes a widget as its
  notice and leaves its drags and focused keys to it; the tools of the
  mockups: notes on a widget by its path, the list of notes, a comment to
  the chat, the mockup frozen while a note is written.
- No note waits in `experiments/mockups/notes/`.
- Next: answers to open 1 to 4, then step 1 of the plan.
