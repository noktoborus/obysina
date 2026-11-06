# obusina-pattern

A bead pattern: its grid, its palettes, its beads and its JSON file.

## Scope

- `Pattern`: the beads put on a field with no end; every other cell is
  empty. A `Cell` has a signed column and row.
- `PatternId`: a random identifier of a pattern, kept in its file; files
  kept apart from the pattern name it by. A copy of the file has the same
  one.
- `Palette`: [`MAX_COLOURS`] places, each a `Colour` or empty. A pattern has
  one palette or more.
- `Colour`: an id, unique in the pattern, and what it looks like. A bead holds
  the id, so a colour moves between places and palettes and no bead changes.
  A colour with no `rgb` was removed and keeps its beads; a hidden one keeps
  its `rgb`. `Colour::seen` is `None` for both: they are drawn hatched.
- `Pattern::put_colour`, `swap_places`, `move_colour`, `remove_colour`,
  `set_hidden`, `delete_beads`: the changes of the colours; `set_palettes`
  puts back palettes kept before, for an undo
  (`docs/palette.md`). A palette left with nothing goes, unless it is the
  last one or holds a removed colour.
- `Pattern::bounds`: the active area, the box around the beads; `None` when
  there are none. `Bounds` grows, meets, joins and doubles.
- `Pattern::revision`: changes with every change of the pattern; no other
  pattern has it unless it is a clone. A drawing of the pattern is kept by it.
- `Ruling`: a line every `step` columns and rows, `offset` from cell (0, 0),
  to count the beads by; `Ruling::marks` says whether a cell is on one.
  `Pattern::ruling` and `set_ruling`; a new pattern has
  `DEFAULT_RULING_STEP`.
- `Grid`: where a bead stands, which bead is under a point, which cells lie
  under a span (`Grid::cells_in`) and the span of a box (`Grid::span`).
- `read`, `write`, `Pattern::to_json`, `Pattern::from_json`: the file.

| Grid | Where a bead stands |
| --- | --- |
| `loom` | rows and columns line up |
| `peyote` | an odd column of the field is half a bead lower |

The parity of a peyote column is the one of its column on the field, also
left of zero. A shift by an odd number of columns would show another
picture (`docs/field.md`).

## The file

JSON, pretty printed. A row of beads is one line: one key per bead, `.` for
an empty place.

```json
{
  "format": 1,
  "id": "6f0d2c7e-9a41-4b8e-8d3a-2f5b1c9e7a10",
  "grid": "peyote",
  "turn": 90,
  "sheet": {
    "origin": [-4, 1],
    "size": [8, 4]
  },
  "ruling": { "step": [10, 4], "offset": [2, -1] },
  "bead_ratio": 1.0,
  "palettes": [
    [
      { "id": 1, "rgb": "#1b1b1b", "catalog": "miyuki-delica-11", "code": "DB-0010" },
      { "id": 2, "rgb": "#f2efe6" }
    ],
    [null, { "id": 3 }, { "id": 4, "rgb": "#c03030", "hidden": true }]
  ],
  "rows_origin": [-3, 2],
  "rows": [
    [1, 1, 2, 0, 2, 2],
    [1, 2, 1, 2, 1, 3]
  ]
}
```

The writer puts each number of an array on a line of its own, but a row of
beads on one line; the example keeps them all on one.

| Field | Meaning |
| --- | --- |
| `format` | the format of the file, `FORMAT` when written |
| `id` | the identifier of the pattern (`PatternId`), a UUID; a new one when left out |
| `grid` | `loom` or `peyote` |
| `turn` | how far the pattern is shown turned clockwise: 0, 90, 180 or 270; the beads do not move; 0 when left out |
| `sheet` | the size of the work area set by hand; left out when none is set |
| `sheet.origin` | column and row of the field of its top left cell |
| `sheet.size` | columns and rows, at least one each |
| `ruling` | the lines to count the beads by (`Ruling`); `DEFAULT_RULING_STEP` and no offset when left out |
| `ruling.step` | columns and rows from one line to the next; `0` draws no line along that axis |
| `ruling.offset` | column and row of a line, counted from cell (0, 0) |
| `bead_ratio` | the width of a bead to its height |
| `palettes` | one array or more, each of at most `MAX_COLOURS` places; the index is the place, `null` an empty one; empty places at the end are left out |
| `palettes[][].id` | a number, unique in the file, never `0` |
| `palettes[][].rgb` | `#rrggbb`; left out for a removed colour |
| `palettes[][].hidden` | `true` for a hidden colour; left out otherwise |
| `palettes[][].catalog`, `code` | where the colour comes from; may be left out |
| `rows_origin` | column and row of the field of the first bead of the first row; `[0, 0]` with no rows |
| `rows` | the active area, the box around the beads; one id per cell, `0` for a cell with no bead; every row as long as the first |

A cell of the field has a signed column and row. The rows and the sheet are
both placed on the field by their own origin, so a pattern opens where it
was, and a peyote column keeps its parity. The work area is not written: it
is the box around the sheet and the rows.

```text
on the field                          the file
        column -4         3           "sheet": origin [-4, 1], size [8, 4]
row 1   ┌──── sheet ──────┐
        │ ┌── rows ──┐    │           "rows_origin": [-3, 2]
row 2   │ │112022    │    │           "rows": [[1, 1, 2, 0, 2, 2],
row 3   │ │121213    │    │                    [1, 2, 1, 2, 1, 3]]
row 4   │ └──────────┘    │
        └─────────────────┘
```

A write goes to a temporary file beside the target, then a rename.

## Formats

```text
text ──► load::load ──► "format": 1 ──► load/v1.rs ──► Pattern
                     ├─ newer than FORMAT ──► PatternError::Newer
                     └─ none ──► PatternError::NoFormat
```

Only the loaders know old formats. The writer writes `FORMAT` only. A file of
each format is in `tests/patterns/`.

## Boundaries

No toolkit and no interface. It says nothing to the user: its errors are for
the program.

## Errors

`Result<T, PatternError>`: `Read`, `Write` (with the path), `Json`, `NoFormat`,
`Newer`, `UnknownFormat`, `RowWidth`, `UnknownId`, `Outside`, `DuplicateId`,
`Palette`, `NoPalette`, `Colour`, `Ratio`, `Turn`, `Sheet`; for a change of
the colours, `Place`, `Full` and `Lost`.
