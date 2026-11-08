# Panels

Where a floating panel stands and how its buttons stand. The rules of the
panel itself are `crates/obusina-panel/README.md`.

## Decisions

1. **A panel has no grip and no title.** Any place of it moves it: a mouse
   pressed and moved, a finger held still for a long touch and then moved.
   - Why: a grip took room and a finger missed it.
   - Why a finger waits: a swipe on a panel must not move it by chance.
   - Was: a grip of dots; a tap on it folded the panel, a shake of it
     turned the line of beads into a block.
2. **A panel that is taken shows it.** It is lighter, with a deeper
   shadow; on android the phone gives a short vibration (`tick` of
   `MainActivity`). Nothing else is drawn while a panel is dragged.
3. **A panel is never folded.** It can be moved to an edge.
4. **A panel never turns by itself.** Neither the edge it is dragged to nor
   a change of the size of the window turns it. Its view is chosen in the
   menu of the settings, level `Panels`:

   | panel | views |
   | --- | --- |
   | beads | a row, a column, a block |
   | tools | a row, a column, two rows, two columns |

   - Was: a panel turned by the edge strip under the pointer, and by the
     room it had. A panel turned by itself where the person did not want it.
5. **Let go in an edge strip, a panel stands against that edge.** The
   pointer decides the edge, by a fixed distance: `SNAP` points, cut to
   `SNAP_PART` of a small window. The edge held wins by `HYSTERESIS`.
6. **A line goes on in the next line where the window ends.** The row and
   the column of the beads and of the tools count how many buttons stand in
   a line of the `Room`. The two lines of the tools stand in pairs, and do
   not go on.
   - Why not a wrapping layout of egui: a column of egui that wraps starts
     its next column after the whole width it was offered, and the panel
     grows every frame.
7. **A place is a part of the window, nothing more.** 0 or 1 on an axis is
   against an edge. There is no docked state.
8. **The place and the view of each panel are kept apart for a window that
   stands and one that lies.** `Posture` is told by the sides of the
   window: taller than wide stands, else it lies. A square window lies.
   `src/settings.rs` keeps both in `settings.yaml`, `layout.portrait` and
   `layout.landscape`, by the name of the panel.
   - Why by the sides and not by a sensor: it is the same on every
     platform, and a window of a desktop made narrow is a window that
     stands.
   - What an older version kept, `panels`, is not read (`RULE
     config.legacy`): the panels are stood again.
9. **When no place is kept for the posture, both panels are stood**
   (`src/ui/arrange.rs`), and kept. The first way in which both fit and do
   not overlap:

   ```text
   1. both in one line at the first short edge
   2. one at each short edge
   3. both in one line at the first long edge
   4. one at each long edge
   5. 1 to 4, the tools in two lines, the beads a block
   none: the beads a column at the right, the tools a row at the top
   ```

   The short edges of a window that stands are the top and the bottom, of
   one that lies the left and the right. The size of a view is counted from
   `button::SIZE` and the spacing, not measured: the panels are stood
   before they are drawn.
10. **On android the panels stand in the part of the window the system
    leaves free.** egui-winit gives egui the insets of the system bars and
    of the cut out of the camera on iOS only; on android `MainActivity`
    reads them from the window (`WindowInsetsCompat`, the system bars and
    the cut out) and `raw_input_hook` gives them to egui
    (`src/activity.rs`). egui lays the panels out in `Context::content_rect`,
    which leaves them out. The field is drawn under them, edge to edge.
    - Why: on a phone with a cut out, a panel at the top stood under the
      status bar, and it could not be taken.
    - Was: the insets of the content rectangle of the activity. Android 15
      draws a window of `targetSdk` 35 or more edge to edge, so that
      rectangle is the whole window and gave none.

## Open questions

- Two panels moved by hand can overlap. They do not avoid each other. Left
  as it is for now.
