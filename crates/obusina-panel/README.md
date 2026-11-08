# obusina-panel

A floating panel for egui with no title and no grip: any place of it moves
it. It never turns by itself: the caller gives its orientation, and may turn
it to the edge it is pulled to.

## Scope

- `FloatingPanel::new(id, name).placement(placement).orientation(orientation).show(context, |ui, room| …)`.
  `name` is what a screen reader reads for the panel.
- The caller keeps the place: the panel stands where `placement` says, but
  while it is held.
- How a press takes the panel (`hold.rs`):

```text
mouse   press ──moved past max_click_dist──► taken
finger  press ──still for max_click_duration──► taken
        press ──moved before that──► dropped: the touch is the content's
taken   ──moved──► follows the pointer, the point taken under it, the
                   panel whole in the window
        ──turned by the caller──► the point taken is the same part of the
                                  panel along and across
        ──let go after a move──► stands there; in an edge strip, against
                                 that edge
```

  A finger waits as long as egui waits before a touch is no click, so the
  button under it is not pressed when it lets go. A mouse moved past the
  click is no click either.
- A panel that is taken is lighter and has a deeper shadow (`LIFT_LIGHT`,
  `LIFT_SHADOW`). `PanelResponse::taken` is set in the frame it is taken;
  the caller may answer with a vibration.
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

- `Placement` holds the place as a part of the room of the window: 0 is
  against the left or the top, 1 against the right or the bottom. A panel
  against an edge stays there when the window changes its size. The room is
  the window less `MARGIN`, `room(window)`.
- `PanelResponse::edge` is the edge strip the pointer is in while the panel
  is taken: the edge it is pulled to.
- `PanelResponse::changed` is set in the frame the panel is let go after a
  move; `PanelResponse::placement` is then the place to keep.
- `FloatingPanel::content(key)`: the key of the shape of the content. When
  it or the orientation changes, the panel measures itself again; otherwise
  it keeps its size, and a content that got smaller would stand in a panel
  too large.
- `Room` gives the content its orientation and the length it may take along
  it. A line longer than that goes on in the next line; the content lays
  itself out so.

| Row | Column | Taken |
| --- | --- | --- |
| ![A row][picture-horizontal] | ![A column][picture-vertical] | ![Taken by the mouse][picture-lifted] |

[picture-horizontal]: img/horizontal.png
[picture-vertical]: img/vertical.png
[picture-lifted]: img/lifted.png

## Boundaries

It knows egui and nothing of the application. It holds no text: the caller
names the panel. Its state lives in the temporary memory of egui; the caller
keeps `Placement` and the orientation. The content lays itself out by the
`Room` it is given; the panel does not reflow it. Two panels do not avoid
each other.

## Errors

None: it only draws.
