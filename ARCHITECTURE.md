# Architecture

| Chapter | Subject |
| --- | --- |
| [docs/field.md](docs/field.md) | the kinds of grid, the endless field, the place of a pattern on it |
| [docs/panels.md](docs/panels.md) | where a panel stands, how it is laid out, how its place is kept |

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
```

| Crate | Purpose |
| --- | --- |
| `obusina-pattern` | the model: grid, palette, beads; the JSON file and its loaders |
| `obusina-canvas` | the field of beads: draws a pattern, moves and zooms, says which bead is pressed |
| `obusina-button` | a button of a fixed size, its label in the middle, plain, selected or filled |
| `plate-menu` | a menu of wide plates in the middle of the window; the kinds of grid are chosen in it |
| `obusina-panel` | a floating panel with a grip, turned by the edge it is dragged to and by the room it has |
| `obusina` | the window, the panels, the history of edits, the settings, the text of the user |
| `obusina-android` | the library android loads; calls `obusina::android` |

`obusina-canvas -> obusina-pattern` is the one dependency between library
crates (`RULE crate.deps`).

## The model

`Pattern` holds a `Grid`, the width of a bead to its height, a palette and
the beads put on a field with no end; every other cell is empty. Its active
area, `Pattern::bounds`, is the box around the beads. `Grid` answers where a
bead stands and which bead is under a point. Nothing else knows a rule of a
stitch.

The format of the file is `crates/obusina-pattern/README.md`.

## Edits

Every change goes through `edit::History` (`src/edit.rs`):

```text
press on a bead ──► App::press ──► History::paint(cells, colour) ──► one step
drag            ──► the field moves; the pattern does not change
```

A change of the kind of grid is one step too.

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
