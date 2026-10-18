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

- Moving a colour: a variant of decision 4.
- What happens to the beads of a colour removed.
- A palette as a list, where a colour removed moves the next ones, or as
  nine places, where it leaves a hole.
- The key of a colour in the file, and the hint of the numpad on a swatch.
