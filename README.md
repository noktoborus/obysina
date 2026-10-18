# Obusina

An editor of bead patterns, for Linux, Windows and Android.

## What it does

- Draws a pattern on a field in a window. A bead is drawn at its
  proportions.
- The work area is on paper: the sheet, whose columns and rows are set in
  the tools and start at the size of the first screen, and every bead put
  past it. A bead put in the shade around grows the work area; when it
  goes, the work area is the sheet again. The shade reaches half the work
  area past it on every side. An empty place is a faint outline at every
  zoom. A new pattern is an empty sheet.
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
  - **Beads**: the colours of the pattern with their counts, an empty place,
    and the two colours in hand.
- A press on a bead switches it: to the colour in hand, or to empty when it
  has that colour already. The first button uses the first colour, the second
  button the second. On a touch screen a tap is the first button, and the
  button with the turning arrows swaps the colours.
- The field turns by quarters, both ways: 90, 180, 270 and 360 degrees,
  which is where it started. Only the showing turns, not the pattern, so a
  turn is no step of undo.
- A drag moves the field, with any button or one finger. A pinch and the wheel
  with ctrl zoom it.
- A pattern is a JSON file. Every format the program ever wrote opens.

| Key | Action |
| --- | --- |
| ctrl+z | undo |
| ctrl+shift+z | redo |
| ctrl+s | save |

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
| Android | `make apk`, `make apk-release`, `make aab` | `android/app/build/outputs/` |

The icon is drawn from `assets/icon.svg` at build time. On Windows it is a
resource of `obusina.exe`, built with `windres` of mingw.

Android needs the NDK, the SDK, a JDK and `cargo-ndk`, named by
`ANDROID_HOME` and `ANDROID_NDK_HOME`. `.devcontainer/Dockerfile.claude`
installs every tool these need. A release is signed as `android/README.md`
says.
