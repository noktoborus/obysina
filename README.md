# Obusina

An editor of bead patterns, for Linux, Windows and Android.

## What it does

- Draws a pattern on a field that fills the window. A bead is drawn at its
  proportions.
- Three kinds of grid: loom (and square stitch), peyote and brick stitch.
- Two floating panels that can be moved and folded:
  - **Tools**: undo, redo, the pencil, the fill, the picker, the kind of grid,
    save;
  - **Beads**: the colours of the pattern with their counts, an empty place,
    and the two colours in hand.
- The first button paints with the first colour, the second button with the
  second. On a touch screen the button with the turning arrows swaps them.
- Two fingers, the wheel and the middle button move the field. A pinch and the
  wheel with ctrl zoom it.
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

| Target | Command |
| --- | --- |
| Linux, Windows | `make build`, then `make install` on Linux |
| Android | not packaged yet; the library builds with the NDK |
