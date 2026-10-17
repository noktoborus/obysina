# Architecture

## Crates

```
obusina-android (cdylib, android only) ──► obusina
obusina (binary and library)
 ├── obusina-canvas ──► obusina-pattern
 ├── obusina-panel
 └── obusina-pattern
experiments/glyphs     (nothing depends on it)
```

| Crate | Purpose |
| --- | --- |
| `obusina-pattern` | the model: grid, palette, beads; the JSON file and its loaders |
| `obusina-canvas` | the field of beads: draws a pattern, moves and zooms, says which bead is pressed |
| `obusina-panel` | a floating panel with a grip, laid out by the zone of the window it is dragged to |
| `obusina` | the window, the panels, the history of edits, the text of the user |
| `obusina-android` | the library android loads; calls `obusina::android` |

`obusina-canvas -> obusina-pattern` is the one dependency between library
crates (`RULE crate.deps`).

## The model

`Pattern` holds a `Grid`, a size, the width of a bead to its height, a palette
and a colour or nothing for every cell. `Grid` answers where a bead stands,
which bead is under a point and which beads touch it. Nothing else knows a
rule of a stitch.

The format of the file is `crates/obusina-pattern/README.md`.

## Edits

Every change goes through `edit::History` (`src/edit.rs`):

```
press ──► History::paint ──► … ──► release ──► History::finish ──► one step
             │
             └── a second finger comes ──► History::cancel ──► nothing
```

A change of the kind of grid is one step too.

## Start

| Platform | Entry | Pattern |
| --- | --- | --- |
| Linux, Windows | `src/main.rs` → `obusina::desktop` | the file named on the command line |
| Android | `android_main` of `android/` → `obusina::android` | `pattern.json` in the internal data path |

## Rendering

`src/render.rs` starts the window on `wgpu` the way zyterm does: a graphics
chip before a software adapter, and the limits of the device cut to what the
adapter grants, so a small chip still gets its window.

## Errors

Each library crate has one error enum. `AppError` of `src/error.rs` wraps them
and names the key of the text the user reads (`locales/app.yml`).
