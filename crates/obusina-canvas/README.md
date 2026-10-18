# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- The field drawn is the box around the start sheet, `View::sheet`, the
  cells under the screen when the pattern is first shown, and twice the
  active area (`Bounds::doubled`). Past it the ground is bare and a press
  does nothing. Only its cells on the screen are drawn.
- A bead is drawn at its proportions, with a small gap. An empty place in
  the active area is an outline; one outside it is a pale outline, and is
  not drawn below `PALE_MIN_BEAD`.
- Puts the middle of the active area, or cell (0, 0) when there is none, in
  the middle of the field the first time the field has a size.
- Draws the outlines of the empty places as one tile that repeats
  (`tile.rs`): loom repeats every cell, peyote every two columns. The tile
  is drawn by the distance to a rounded box, again only when the size of a
  bead or the grid changes; a column of the field on the screen is one quad
  of it, tinted pale or plain. The outlines cost the same at any size of the
  field.
- Keeps the beads as one tessellated mesh of the beads on the screen and
  half a screen more on every side. It is tessellated anew only when the
  pattern (its revision), the size of a bead, the turn or the pixels of the
  screen change, or the screen leaves those cells; otherwise a move of the
  field moves the mesh. `tessellated` in `CanvasResponse` says which frame
  built it.
- Turns the field by quarters, `View::turn`: 90, 180, 270, and 360, which is
  where it started. What is in the middle of the field stays there. Only the
  showing turns; the pattern is the same.
- Moves and zooms the field by its [`View`]:

  | Input | Result |
  | --- | --- |
  | press and let go in place | a press on the bead, by its button |
  | press and drag, any button, one finger | the field moves |
  | two fingers | the field moves and zooms |
  | wheel, ctrl + wheel | the field moves, zooms |

- Says which bead is hovered and which is pressed. egui tells a press from a
  drag: a press that moves or is held too long is a drag.

| Grid | Picture |
| --- | --- |
| Loom | ![Loom][picture-loom] |
| Peyote | ![Peyote][picture-peyote] |
| Peyote, turned 90° | ![Peyote turned][picture-peyote-turned] |

[picture-loom]: img/loom.png
[picture-peyote]: img/peyote.png
[picture-peyote-turned]: img/peyote-turned.png

## Boundaries

It changes nothing in the pattern: what a press does is the caller's. It holds
no text, so the caller names it for a screen reader. Where a bead stands is
asked of `Grid`, never computed here.

## Errors

None: it only draws.

## Usage

```rust,ignore
let mut view = View::default();
let field = Canvas::new(&pattern, &mut view).show(ui);
if let Some((cell, Press::Primary)) = field.pressed {
    pattern.set(cell, Some(colour));
}
```
