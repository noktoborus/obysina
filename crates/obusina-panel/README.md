# obusina-panel

A floating panel for egui with no title: a grip of dots moves it, a tap on the
grip folds it, and the edge of the window it is let go at docks it.

## Scope

- `FloatingPanel::new(id, name).placement(placement).show(context, |ui, room| …)`.
  `name` is what a screen reader reads for the grip. `placement` is where the
  panel stands the first time; later it keeps its own.
- The grip stands at the start of the panel: on the left of a row, on top of
  a column. Dragging it moves the panel; a tap folds the panel to the grip and
  a second tap opens it.
- The pointer chooses the edge, by a fixed distance (`edge.rs`):

```text
┌──────────────────────────┐
│           top            │   each strip reaches SNAP into the window,
│    ┌────────────────┐    │   or SNAP_PART of a small window
│left│      free      │right
│    └────────────────┘    │   in a corner the nearer edge wins; the
│          bottom          │   edge held wins by HYSTERESIS
└──────────────────────────┘
```

- A drag goes through three states:

```text
held ──moved──► dragged ──let go at an edge──► docked: against the edge,
  │                │                            laid out by it
  │                └─────let go in the middle─► free: where it is, in the
  │                                             orientation it had
  └──let go, not moved──► folded or opened
```

  While dragged, the panel keeps its shape and follows the pointer. The
  strips are drawn, the one under the pointer filled, with a ghost of the
  panel where it will stand. The shape changes once, on release.
- `Placement` holds the place as a part of the room of the window, so a
  docked panel stays on its edge and a free one keeps its part of the window
  when the window changes its size. The room is the window less `MARGIN`.
- `PanelResponse::changed` is set in the frame the panel is let go or
  folded; `PanelResponse::placement` is then the place to keep for the next
  start.
- `Room` gives the content its orientation and the length it may take along
  it. The content breaks itself into lines to fit it.

| Row | Column | Folded | Dragged to the left |
| --- | --- | --- | --- |
| ![A row][picture-horizontal] | ![A column][picture-vertical] | ![Folded][picture-folded] | ![The strips and the ghost][picture-zones] |

[picture-horizontal]: img/horizontal.png
[picture-vertical]: img/vertical.png
[picture-folded]: img/folded.png
[picture-zones]: img/zones.png

## Boundaries

It knows egui and nothing of the application. It holds no text: the caller
names the grip. Its state lives in the temporary memory of egui; the caller
keeps `Placement` if it should outlive the program. The content lays itself
out by the `Room` it is given; the panel does not reflow it. Two panels do not
avoid each other.

## Errors

None: it only draws.
