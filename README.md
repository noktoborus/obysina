# Obusina

An editor of bead patterns, for Linux, Windows and Android.

## What it does

- Draws a pattern on a field that fills the window. A bead is drawn at its
  proportions.
- Three kinds of grid: loom (and square stitch), peyote and brick stitch.
- Two floating panels with no title. The grip of dots moves a panel; a tap on
  it folds the panel. At the left or the right edge of the window a panel
  stands in a column, at the top or the bottom in a row. The panels:
  - **Tools**: undo, redo, a quarter turn of the field each way, the pencil,
    the fill, the picker, the kind of grid, save;
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

Android needs the NDK, the SDK, a JDK and `cargo-ndk`, named by
`ANDROID_HOME` and `ANDROID_NDK_HOME`. `.devcontainer/Dockerfile.claude`
installs every tool these need. A release is signed as `android/README.md`
says.
