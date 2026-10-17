# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- Draws every bead at its proportions, with a small gap. An empty place is an
  outline.
- Puts the pattern in the middle of the field the first time the field has a
  size.
- Keeps the beads as one tessellated mesh. It is tessellated anew only when
  the pattern (its revision), the size of a bead, the turn or the pixels of
  the screen change; a move of the field moves the mesh. `tessellated` in
  `CanvasResponse` says which frame built it.
- Turns the field by quarters, `View::turn`: 90, 180, 270, and 360, which is
  where it started. The middle of the pattern stays where it is. Only the
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
| Brick | ![Brick][picture-brick] |
| Peyote, turned 90° | ![Peyote turned][picture-peyote-turned] |

[picture-loom]: img/loom.png
[picture-peyote]: img/peyote.png
[picture-brick]: img/brick.png
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
