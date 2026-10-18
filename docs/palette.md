# The palette

How the colours of a pattern are filled in. The design is drawn as the
mockup `palette-editor` of `experiments/mockups`; the window does not have
it yet.

## Decisions

1. **The palette is filled in a window with no title,** a modal over the
   field. The colours stand in a line, a "+" at its end. A palette holds
   at most nine colours; at nine the "+" is off.
2. **A tap on a colour opens a `PlateMenu` of it:** change the colour,
   move, remove (`RULE ui.menu`). No button stands under a colour.
   - Was: a "-" under each colour, which doubled the line and was read as
     a colour of its own.
3. **The picker opens at the colour it changes,** under its cell. A tap
   beside it or Esc closes it.
   - Was: a picker under the line, for the colour chosen last.
4. **A colour is moved by a tap.** "Move" lifts the colour; the next tap on
   a colour puts it in that place.
5. **"+" adds the colour farthest from the others.** Of the steps of
   lightness, chroma and hue of OKLCh a screen can show, it takes the one
   whose nearest neighbour, among the colours and the paper of the field,
   is farthest in OKLab. The picker opens on it.
   - Why: a fixed grey was the same every time and near the paper.

```text
tap a colour ──► PlateMenu ──change──► the picker, under that colour
                    │──move────► the next tap on a colour puts it there
                    └──remove──► gone
tap "+" ──► the colour farthest from the others and the paper, in the picker
```

## Open

- What happens to the beads of a colour removed.
- A palette as a list, where a colour removed moves the next ones, or as
  nine places, where it leaves a hole.
- The key of a colour in the file, and the hint of the numpad on a swatch.
