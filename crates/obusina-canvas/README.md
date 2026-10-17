# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- Draws every bead at its proportions, with a small gap. An empty place is an
  outline.
- Moves and zooms the field by its [`View`]: two fingers, the wheel and the
  middle button move it; a pinch and the wheel with ctrl zoom it.
- Says which bead is hovered and which is pressed, and by which button.
  A press of two fingers is a move, never a press on a bead.

| Grid | Picture |
| --- | --- |
| Loom | ![Loom][picture-loom] |
| Peyote | ![Peyote][picture-peyote] |
| Brick | ![Brick][picture-brick] |

[picture-loom]: img/loom.png
[picture-peyote]: img/peyote.png
[picture-brick]: img/brick.png

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
