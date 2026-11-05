# The palette

How the colours of a pattern are filled in. The design is drawn as the
mockups `palette`, `panel` and `numpad` of `experiments/mockups`; the window
does not have it yet.

## Decisions

1. **The palette is filled in a window with no title,** a modal over the
   field. It holds `MAX_COLOURS` places.
2. **A palette is a row of places, each with a colour or empty.** A number
   is a place, and stays with the colour until the colour is moved.
   - The line shows every place up to the last colour.
   - An empty place in the line is a "+".
   - One more "+" stands after the last colour. It stands for all the places
     left, and fills the first of them. A full palette does not have it.
   - The code treats every "+" as an empty place. It does not know the
     number of places.
   - A new colour takes the numbers in order: 1, 2, 3.
   - Was: a list, where a colour removed changed the numbers of the next ones.
3. **A tap on a colour opens a `PlateMenu` of it** (`RULE ui.menu`): the
   picker of the colour on top, then accept, move to another palette and
   remove.
   No button stands under a colour. A "+" adds a colour in its place and opens its menu.
   - Was: a "-" under each colour, which doubled the line and was read as
     a colour of its own.
4. **The picker is the notice of the menu,** as wide as the menu: a
   swatch, the number and `#rrggbb` over it, which change as it changes.
   The colour shows in the palette and on the field at once, and stays only
   by "Accept". A menu closed with no entry puts back what the place held, so
   a new colour not accepted is gone. The notice of a `PlateMenu`
   takes a widget of the caller (`Notice::widget`); a drag that begins
   on it and the keys while it holds the focus are its own, not the
   menu's.
   - Was: a picker under the line, for the colour chosen last; then one
     at the cell, too small; then one under the cell opened by "Pick a
     colour" of the menu, one step more than needed.
5. **A colour is moved by a drag.** It rises above the pointer. The place
   under the pointer is lit and shows the colour with its number. A colour
   in that place shows in the place the dragged one left; no other colour
   moves. Let go, it stays there. A drag onto a "+" leaves a "+" behind. A
   tap still opens the menu, which has no "Move".
   - Was: "Move" of the menu and a tap on the new place; then a drag that
     moved the other colours aside and numbered them anew.
6. **"+" adds the colour farthest from the others.** Of the steps of
   lightness, chroma and hue of OKLCh a screen can show, it takes the one
   whose nearest neighbour, among the colours and the paper of the field,
   is farthest in OKLab. Its menu opens, the picker on top.
   - Why: a fixed grey was the same every time and near the paper.
7. **A removed colour leaves its place and its beads.** Nothing asks. The
   colour keeps its id, with no rgb. Its place in the window and its beads on
   the field are hatched. It does not stand on the panel. A tap on the place
   opens the same menu, with "delete <n> beads" for "remove" while it has
   beads: a colour picked and accepted there colours the beads again; the
   delete takes only the beads, and the place stays hatched. With no beads the
   entry is "remove", which leaves a "+". Undo brings the colour back. A
   colour with no beads leaves a "+" at once. A hatched place is dragged and
   moved to another palette as any colour.
   - Was: a question that named the count of beads, and the beads gone; then
     a "+" in the place.
8. **One colour is in hand.** It is the one framed on the panel; no swatch of
   its own stands apart. It follows a colour that is moved. A colour removed
   leaves the empty place in hand.
   - Was: two colours in hand, one for each button, and a button that swapped
     them.
9. **A colour has an id, unique in the pattern, and a bead holds that id.**
   A row of the file is an array of ids, one for each cell; `0` is a cell with
   no bead. The place is the index in the array of its palette, and `null` is
   an empty place. A swap or a move between palettes changes no row
   (`RULE pattern.diff`). Format 1 changes itself before 1.0.0 (`EXC` of
   `RULE pattern.version`).
   - Was: a character for each bead: a letter, then the number of the place.

   ```json
   "palettes": [
     [{ "id": 1, "rgb": "#1b1b1b" }, null, { "id": 3 }],
     [{ "id": 2, "rgb": "#f2efe6" }]
   ],
   "rows": [
     [1, 1, 2, 0],
     [3, 2, 0, 0]
   ]
   ```

10. **The palette has two views, a line and a block.** Tabs on top of the
    window of the palette switch them. A new colour takes the first free place
    after the last colour, in the order of the numbers. A shake of the panel
    by its grip switches them too.
    - The line: `< 1 2 3 4 5 6 7 8 9 0 >`, with a narrow `0`; `<` and `>` while
      there are three palettes or more, one `↔` after `0` with two. An empty place is left
      out in the panel. It turns by the room it has and by the edge it stands
      at (`RULE ui.dock`), in the same order.
    - The block: `7 8 9 / 4 5 6 / 1 2 3`, with a wide `0` under `1 2`. Empty rows
      are folded in the panel, and an empty place in a row is a gap. In the
      window, every place stands, each empty one a "+".
    - The digits of the keyboard and the numpad choose the colour in hand, `0`
      the empty place; a field of the tools that holds the focus keeps its
      digits.
11. **A pattern has several palettes.** Together they hold more colours than
    the keys of one. "+" and "-" of the keyboard or the numpad show the next
    and the previous palette, in a circle. The digits choose in the palette
    shown. Buttons do it by touch (`RULE ui.touch`), in the window and on the
    panel: with two palettes one `↔` goes to the other, beside `0` in a block;
    with more, `<` and `>`, under `0` in a block.
    - Was: `<` and `>` with two palettes too. A palette goes when nothing is left in its places, and the next
    one is shown. Hatched beads keep a palette, because a "+" there colours
    them again. The last palette stays. A palette is a page of colours, not
    another colouring: the colours of all palettes show on the field.
12. **A colour is moved to another palette from its menu.** With one palette,
    the entry makes a new one. With more, a `PlateMenu` names the others and a
    new one; a full palette is off. The colour takes the first free place, and
    leaves its old place empty. The palette shown does not change.
13. **The colour in hand is marked by a bar under the swatch,** of one colour
    for every swatch. The bar lies on the ground of the panel, so no colour
    hides it. The swatch gets lower.
    - Was: a thick frame, black or white, whichever read on the swatch; a ring;
      two bars.
14. **A tap on the colour in hand opens its menu on the panel:** its title,
    then hide or show, delete <n> beads, and palette settings. A first tap
    takes the colour in hand. The right button does nothing on the panel or on the field. A hidden colour keeps its rgb and
    stands on the panel, hatched, and its beads are hatched; "show" brings it
    back. In the window it is hatched with its number. It can be drawn with,
    and the new beads are hatched. Delete takes only the beads.

15. **The window of the palette opens from the menu on the panel,** by the
    entry "palette settings", or by the key `P` while no field holds the focus.
    `0` is the way with no colours: a tap takes it in hand, a second tap opens a
    menu with that one entry.

```text
tap a colour ──► PlateMenu: the picker of the colour on top, then
                    │──accept──────────► the colour stays as picked
                    │──move to another─► one palette: a new one;
                    │                    more: a PlateMenu of them and a new one
                    └──remove──────────► its place and its beads hatched
tap a hatched place ──► the same, with "delete <n> beads" for "remove"
      closed with no entry ──► the colour goes back to what it was
drag a colour ──► the place under the pointer is lit and shows it ──let go──► it stays,
                  what stood there takes the place it left
tap a "+" ──► the colour farthest from the others and the paper, its menu open
"+", "-" ──► the next and the previous palette, in a circle
```

## Plan

The steps that bring the window to the design. Each is one commit, its
test of the window with it (`RULE test.window`).

6. **The hatch and the frame.** `obusina-canvas` draws a bead of a removed or
   hidden colour hatched; `SquareButton` draws the hatch and the mark of
   decision 13. Each with a picture test (`RULE test.picture`).
7. **The mockups.** When the window has a part of the design, its mockup
   goes, or is cut down to what is still open: a mockup is not kept beside
   the code it drew.

## Where the work stands

- Done in the window: decision 8, one colour in hand, and the second button
  does nothing.
- Done in `obusina-pattern`: the palettes, the ids, the file of decision 9 and
  the changes of the colours, each with its test. The panel numbers a swatch
  by its place.
- Done: decisions 1 to 7 and 10 to 15 as mockups, with no shake;
  `PlateMenu` takes a widget as its notice and leaves its drags and focused
  keys to it; the tools of the
  mockups: notes on a widget by its path, the list of notes, a comment to
  the chat, the mockup frozen while a note is written.
- No note waits on the desk `obusina` of showcase.
- Done in the history: `Edit::Palettes`, `History::change_colours`,
  `keep_palettes` for the picker, `delete_beads`, each with its test.
- Done in the window: `src/ui/palette_window.rs`, the window of decisions 1
  to 7, 10 to 12 and 15, with its tests (`src/window_tests/palette.rs`), and
  the menu of the colour in hand of decision 14. A hidden or removed colour
  is drawn with no fill, and its beads grey, until step 6.
- Done: decision 6 in `src/new_colour.rs`.
- Done on the panel of beads: the line and the block of decision 10, kept in
  the settings and turned by a shake of the grip (`PanelResponse::shaken` of
  `obusina-panel`); the switch of decision 11; the digits and "+" and "-".
- Next: step 6 of the plan.
