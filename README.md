# Obusina

An editor of bead patterns, for Linux, Windows and Android.

## What it does

- Draws a pattern on a field in a window. A bead is drawn at its
  proportions.
- The work area is on paper: the sheet, whose columns and rows are set in
  the tools and start at the size of the first screen, and every bead put
  past it. A bead put in the shade around grows the sheet. A bead taken
  away shrinks it to the box of the beads, never below the first screen.
  The shade reaches half the work area past it on every side. An empty
  place has no outline: only near the middle of the screen and under the
  pointer it shows the faint shadow of a bead, at every zoom. A button of
  the tools hides the shadows. A new pattern is an empty sheet.
- The field cannot be moved off the screen: at least ten beads of it stay
  on it, at the side it was moved to, also when the window changes its
  size.
- A picture under the beads to trace from: PNG, JPEG or WebP, its opacity
  and its four corners set by hand, so a photo taken at an angle lies
  straight. It is kept in the cache of the program, apart from the pattern.
- A ruling to count the beads by: every few columns and rows the shadows
  are darker. The step and the offset from cell (0, 0) are set in the menu
  of the grid and kept in the pattern.
- Two kinds of grid: loom (and square stitch) and peyote.
- Two floating panels with no title. The grip of dots moves a panel; a tap on
  it folds the panel. Dragged to the left or the right edge of the window, a
  panel turns into a column; to the top or the bottom, into a row. A panel
  that does not fit the window turns too. Let go at an edge, it stands
  against it. The place of each panel is kept for the next start. The
  panels:
  - **Tools**: undo, redo, a quarter turn of the field each way, the pencil,
    the picker, the kind of grid, the columns and the rows of the sheet,
    save;
  - **Beads**: the colours of the palette shown, each with the number of its
    place, and the empty place `0`. They stand as a line, which turns with
    the panel, or as a block, as the keys of a numpad; the tabs of the window
    of the palettes or a shake of the grip switch the two. With two palettes
    or more, buttons switch the palette. The one in hand is marked. A tap on
    it opens its menu: hide or show the colour, delete its beads, the
    settings of the palette.
- A pattern has one palette or more, each of nine places. The window of the
  palettes, opened from that menu or by `P`, shows the places as a line or
  as a block. A tap on a place opens the picker of its colour; the colour
  stays when it is accepted. A colour is moved to another palette, removed,
  or dragged onto another place, which swaps the two. A removed colour keeps
  its beads, hatched, until a colour is put in its place again. All that
  is changed while the window is open is one step of undo, made when it
  closes.
- A press on a bead switches it: to the colour in hand, or to empty when it
  has that colour already. The second button does nothing. On a touch screen
  a tap is the first button.
- The field turns by quarters, both ways: 90, 180, 270 and 360 degrees,
  which is where it started. Only the showing turns, not the beads, so a
  turn is no step of undo. The turn is saved with the pattern, which opens
  turned as it was.
- A drag moves the field, with any button or one finger. A pinch and the wheel
  with ctrl zoom it.
- A pattern is a JSON file. Every format the program ever wrote opens.

| Key | Action |
| --- | --- |
| ctrl+z | undo |
| ctrl+shift+z | redo |
| ctrl+s | save |
| 1 … 9 | the colour of that place of the palette shown |
| 0 | the empty place |
| + / - | the next and the previous palette |
| P | the window of the palettes |

## Running

```sh
cargo run -- my-pattern.json
```

A file that is not there is a new pattern, saved there. With no file the
pattern cannot be saved yet. On Android the pattern is kept in the data of the
program.

## Building

| Target | Command | Result |
| --- | --- | --- |
| Linux | `make build`, then `make install` | `target/release/obusina` |
| Windows | `make build-windows`, with mingw | `obusina.exe` |
| Android | `make apk`, `make apk-release`, `make aab` | `target/obusina.apk`, signed with the debug key; `target/obusina-release.apk`; the bundle in `android/app/build/outputs/` |

The icon is drawn from `assets/icon.svg` at build time. On Windows it is a
resource of `obusina.exe`, built with `windres` of mingw.

Android needs the NDK, the SDK, a JDK and `cargo-ndk`, named by
`ANDROID_HOME` and `ANDROID_NDK_HOME`. `.devcontainer/Dockerfile.claude`
installs every tool these need. A release is signed as `android/README.md`
says.
