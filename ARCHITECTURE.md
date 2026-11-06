# Architecture

| Chapter | Subject |
| --- | --- |
| [docs/field.md](docs/field.md) | the kinds of grid, the endless field, the place of a pattern on it |
| [docs/panels.md](docs/panels.md) | where a panel stands, how it is laid out, how its place is kept |
| [docs/palette.md](docs/palette.md) | how the colours of a pattern are filled in |

## Crates

```
obusina-android (cdylib, android only) ──► obusina
obusina (binary and library)
 ├── obusina-button
 ├── obusina-canvas ──► obusina-pattern
 ├── obusina-panel
 ├── obusina-pattern
 └── plate-menu         (a copy of the crate of zyterm)
experiments/glyphs     (nothing depends on it)
experiments/mockups    (nothing depends on it) ──► ../showcase
```

| Crate | Purpose |
| --- | --- |
| `obusina-pattern` | the model: grid, palettes, colours, beads; the JSON file and its loaders |
| `obusina-canvas` | the field of beads: draws a pattern, moves and zooms, says which bead is pressed |
| `obusina-button` | a button of a fixed size, its label in the middle, plain, selected or filled |
| `plate-menu` | a menu of wide plates in the middle of the window; the kinds of grid are chosen in it |
| `obusina-panel` | a floating panel with a grip, turned by the edge it is dragged to and by the room it has |
| `obusina` | the window, the panels, the history of edits, the settings, the text of the user |
| `obusina-android` | the library android loads; calls `obusina::android` |

`obusina-canvas -> obusina-pattern` is the one dependency between library
crates (`RULE crate.deps`).

## The model

`Pattern` holds a `Grid`, the width of a bead to its height, its palettes
and the beads put on a field with no end. A colour has an id, unique in the
pattern, and a bead holds that id; every other cell is empty. Its active
area, `Pattern::bounds`, is the box around the beads. `Grid` answers where a
bead stands and which bead is under a point. Nothing else knows a rule of a
stitch.

The format of the file is `crates/obusina-pattern/README.md`.

## Edits

Every change goes through `edit::History` (`src/edit.rs`):

```text
press on a bead ──► App::press ──► History::paint(cells, colour) ──► one step
                    a bead past the sheet: the sheet grows, in the same step;
                    a bead taken away: the sheet shrinks, in the same step
drag            ──► the field moves; the pattern does not change
window of the palettes open ──► History::begin ... closed ──► History::end
                    every edit between is one step: `Edit::Many`
```

A change of the kind of grid is one step too, and so is a ruling accepted
in its menu (`History::keep_ruling`). While a batch is open there
is nothing to undo or redo: its edits are no step yet.

## Start

| Platform | Entry | Pattern |
| --- | --- | --- |
| Linux, Windows | `src/main.rs` → `obusina::desktop` | the file named on the command line |
| Android | `android_main` of `android/` → `obusina::android` | `pattern.json` in the internal data path |

The settings are `settings.yaml` in the configuration directory of `APP_ID`,
on Android in the internal data path (`src/settings.rs`).

## Rendering

`src/render.rs` starts the window on `wgpu` the way zyterm does: a graphics
chip before a software adapter, and the limits of the device cut to what the
adapter grants, so a small chip still gets its window.

## Errors

Each library crate has one error enum. `AppError` of `src/error.rs` wraps them
and names the key of the text the user reads (`locales/app.yml`).
