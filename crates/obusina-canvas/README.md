# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- The work area is the box around the sheet and the beads. The sheet is the
  size set, `Pattern::sheet`; a pattern with none uses the cells under the
  screen when the field is first shown, `View::sheet`: the first screen,
  which the caller may give it. A bead past the sheet grows the work area;
  when it goes, the work area is the sheet again. The field
  drawn is the work area doubled (`Bounds::doubled`). Past it the ground is
  bare and a press does nothing. Only its cells on the screen are drawn.
- A bead is drawn at its proportions, with a small gap. The ground of the
  work area is paper, `PAPER`, which a new colour is told from; the rest
  of the field is in shade. An empty place has no outline. It shows the
  shadow of a bead only in a frame of the screen at zoom 1 (`DEFAULT_BEAD`)
  in its middle and within `NEAR_RADIUS` cells of the pointer, on the paper,
  in the shade and past the field alike, each fading over its edge
  (`FRAME_FADE`). With no
  pointer over the field, as under a finger, there is no round.
  `shadow.rs` holds the rules. A shadow on a line of the ruling of the
  pattern (`Pattern::ruling`) is `SHADOW`, any other the lighter
  `SHADOW_CELL`. `Canvas::shadows(false)` shows none.
- Puts the middle of the work area, or cell (0, 0) when there is none, in
  the middle of the field the first time the field has a size.
- Keeps the middle of the field where it is when the field changes its
  size, at the same size of a bead. A frame where the field has no size is
  skipped.
- Keeps `KEEP_BEADS` beads of the field on the screen along each axis, or
  all of it when it is smaller, after every move, zoom, turn and new size:
  the field moves back as little as it takes and stays at that side.
- Nothing is tessellated. One white bead is drawn into a texture by the
  distance to a rounded box, again only when the size of a bead changes
  (`sprite.rs`). A bead on the screen is one quad of it, tinted its colour;
  so is a shadow, tinted `SHADOW` or `SHADOW_CELL`. A bead of a colour not seen, removed or
  hidden (`Colour::seen`), is a quad of a second texture: its outline and
  diagonal lines.

  Every frame lays the quads of what is on the screen anew; a frame costs
  as many quads as there are columns of paper, beads and shadows on the
  screen.
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
| A colour removed, its beads hatched | ![Hatched][picture-hatched] |
| Loom, a line every 10 columns and 4 rows from (2, 1) | ![Ruling][picture-ruling] |
| Zoomed out: the shadows in the frame of the screen at zoom 1 and in the round under the pointer | ![Shadows][picture-shadows-zoomed-out] |
| Moved far off to the left: ten beads stay on the screen | ![Kept at the side][picture-kept-at-the-side] |

[picture-loom]: img/loom.png
[picture-peyote]: img/peyote.png
[picture-peyote-turned]: img/peyote-turned.png
[picture-hatched]: img/hatched.png
[picture-shadows-zoomed-out]: img/shadows-zoomed-out.png
[picture-kept-at-the-side]: img/kept-at-the-side.png
[picture-ruling]: img/ruling.png

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
