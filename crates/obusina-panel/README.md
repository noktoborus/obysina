# obusina-panel

A floating panel for egui with no title: a grip of dots moves it, a tap on the
grip folds it, and the zone of the window it is dragged to lays it out.

## Scope

- `FloatingPanel::new(id, name).place(corner, offset).show(context, |ui, orientation| …)`.
  `name` is what a screen reader reads for the grip.
- The grip stands at the start of the panel: on the left of a row, on top of
  a column. Dragging it moves the panel; a tap folds the panel to the grip and
  a second tap opens it.
- While the grip is dragged, the zone under the pointer lays the panel out:

```text
┌──────────────────────────┐
│╲          top           ╱│      left, right   a column
│  ╲ ┌────────────────┐ ╱  │      top, bottom   a row
│left│     centre     │right      centre        as it was
│  ╱ └────────────────┘ ╲  │
│╱         bottom         ╲│
└──────────────────────────┘
```

  The centre is the window less `EDGE` of its size on every side; the
  diagonals from the corners split the rest. The pointer decides the zone,
  not the panel: a panel wider than an edge zone still reaches it.
- While a panel is dragged, the zones are drawn over the content and under
  the panels, and the zone under the pointer is filled.
- The point of the panel taken by the pointer stays under it: the panel is
  placed from the pointer before it is drawn, so it neither lags a frame nor
  loses the first points of a drag.
- The panel is kept inside the window, also when the window shrinks.

| Row | Column | Folded | Dragged to the left |
| --- | --- | --- | --- |
| ![A row][picture-horizontal] | ![A column][picture-vertical] | ![Folded][picture-folded] | ![The zones][picture-zones] |

[picture-horizontal]: img/horizontal.png
[picture-vertical]: img/vertical.png
[picture-folded]: img/folded.png
[picture-zones]: img/zones.png

## Boundaries

It knows egui and nothing of the application. It holds no text: the caller
names the grip. Its place, its fold and its orientation live in the temporary
memory of egui, so they are lost when the program ends. The content lays
itself out by the `Orientation` it is given; the panel does not reflow it.

## Errors

None: it only draws.
