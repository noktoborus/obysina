# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- The work area is the box around the sheet, `View::sheet`, and the beads.
  The sheet is the size set: the cells under the screen when the pattern is
  first shown, or what the caller sets. A bead past the sheet grows the
  work area; when it goes, the work area is the sheet again. The field
  drawn is the work area doubled (`Bounds::doubled`). Past it the ground is
  bare and a press does nothing. Only its cells on the screen are drawn.
- A bead is drawn at its proportions, with a small gap. The ground of the
  work area is paper; the rest of the field is in shade. An empty place is
  a faint outline, the same on both, and is not drawn below
  `OUTLINE_MIN_BEAD`.
- Puts the middle of the active area, or cell (0, 0) when there is none, in
  the middle of the field the first time the field has a size.
- Nothing is tessellated. Two textures are drawn by the distance to a
  rounded box, again only when the size of a bead or the grid changes:
  - a tile of outlines that repeats (`tile.rs`): loom every cell, peyote
    every two columns; a column of the field on the screen is one quad of it;
  - one white bead (`sprite.rs`); a bead on the screen is one quad of it,
    tinted its colour.

  Every frame lays the quads of what is on the screen anew; a frame costs
  as many quads as there are columns and beads on the screen.
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
