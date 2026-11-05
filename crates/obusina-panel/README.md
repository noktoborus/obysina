# obusina-panel

A floating panel for egui with no title: a grip of dots moves it, a tap on the
grip folds it. It turns by the edge it is dragged to and by the room it has.

## Scope

- `FloatingPanel::new(id, name).placement(placement).show(context, |ui, room| …)`.
  `name` is what a screen reader reads for the grip. `placement` is where the
  panel stands the first time; later it keeps its own.
- The grip stands at the start of the panel: on the left of a row, on top of
  a column. Dragging it moves the panel; a tap folds the panel to the grip and
  a second tap opens it.
- The pointer chooses the edge strip, by a fixed distance (`edge.rs`):

```text
┌──────────────────────────┐
│           top            │   each strip reaches SNAP into the window,
│    ┌────────────────┐    │   or SNAP_PART of a small window
│left│     middle     │right
│    └────────────────┘    │   in a corner the nearer edge wins; the
│          bottom          │   edge held wins by HYSTERESIS
└──────────────────────────┘
```

- In every frame of a drag the panel chooses its orientation, and turns at
  once, its grip under the pointer:

```text
wanted = pointer in an edge strip ? a row at the top or the bottom,
                                    a column at the sides
                                  : the orientation it has
wanted does not fit, the other fits → the other
otherwise                           → wanted
```

  In the middle a panel fits when its far end lies inside the window. At an
  edge, or at rest, it fits when it is not larger than the window. The same
  rule turns a panel at rest when the window changes its size.
- Let go in an edge strip, the panel stands against that edge. Nothing is
  drawn while a panel is dragged.
- `Placement` holds the place as a part of the room of the window: 0 is
  against the left or the top, 1 against the right or the bottom. A panel
  against an edge stays there when the window changes its size. The room is
  the window less `MARGIN`.
- `PanelResponse::changed` is set in the frame the panel is let go or
  folded; `PanelResponse::placement` is then the place to keep for the next
  start.
- `FloatingPanel::content(key)`: the key of the shape of the content. When
  it changes, the panel measures itself again; otherwise it keeps its size,
  and a content that got smaller would stand in a panel too large.
- `PanelResponse::shaken` is set in the frame the held grip is shaken: the
  pointer turns three times within a short time (`shake.rs`). One drag is
  one shake at most. The panel gives it no meaning.
- `Room` gives the content its orientation and the length it may take along
  it.

| Row | Column | Folded |
| --- | --- | --- |
| ![A row][picture-horizontal] | ![A column][picture-vertical] | ![Folded][picture-folded] |

[picture-horizontal]: img/horizontal.png
[picture-vertical]: img/vertical.png
[picture-folded]: img/folded.png

## Boundaries

It knows egui and nothing of the application. It holds no text: the caller
names the grip. Its state lives in the temporary memory of egui; the caller
keeps `Placement` if it should outlive the program. The content lays itself
out by the `Room` it is given; the panel does not reflow it. Two panels do not
avoid each other.

## Errors

None: it only draws.
