# obusina-canvas

The field of beads: an egui widget that draws an
[`obusina_pattern::Pattern`] and says which bead the pointer is on.

## Scope

- The work area is the box around the sheet and the beads. The sheet is the
  size set, `Pattern::sheet`; a pattern with none uses the cells under the
  screen when it is first shown, `View::sheet`, which the caller may give
  it. A bead past the sheet grows the
  work area; when it goes, the work area is the sheet again. The field
  drawn is the work area doubled (`Bounds::doubled`). Past it the ground is
  bare and a press does nothing. Only its cells on the screen are drawn.
- A bead is drawn at its proportions, with a small gap. The ground of the
  work area is paper, `PAPER`, which a new colour is told from; the rest
  of the field is in shade. An empty place is
  a faint outline, at every size of a bead: every one on the paper; in the
  shade only those in a frame of the screen at zoom 1 (`DEFAULT_BEAD`) in
  its middle and within `NEAR_RADIUS` cells of the pointer, each fading
  over its edge (`FRAME_FADE`). With no pointer over the field, as under a
  finger, there is no round. `shade.rs` holds the rules.
- Puts the middle of the work area, or cell (0, 0) when there is none, in
  the middle of the field the first time the field has a size.
- Keeps the middle of the field where it is when the field changes its
  size, at the same size of a bead. When then no cell of the work area is
  on the screen, it centres the work area again. A frame where the field
  has no size is skipped.
- Nothing is tessellated. Two textures are drawn by the distance to a
  rounded box, again only when the size of a bead or the grid changes:
  - a tile of outlines that repeats (`tile.rs`): loom every cell, peyote
    every two columns; a column of the work area on the screen is one quad
    of it, an outline in the shade one quad of its own;
  - one white bead (`sprite.rs`); a bead on the screen is one quad of it,
    tinted its colour. A bead of a colour not seen, removed or hidden
    (`Colour::seen`), is a quad of a second texture: its outline and
    diagonal lines.

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
| A colour removed, its beads hatched | ![Hatched][picture-hatched] |
| Zoomed out: the frame of the screen at zoom 1, the round under the pointer | ![Shade][picture-shade-zoomed-out] |

[picture-loom]: img/loom.png
[picture-peyote]: img/peyote.png
[picture-peyote-turned]: img/peyote-turned.png
[picture-hatched]: img/hatched.png
[picture-shade-zoomed-out]: img/shade-zoomed-out.png

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
