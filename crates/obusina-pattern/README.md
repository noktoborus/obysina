# obusina-pattern

A bead pattern: its grid, its palette, its beads and its JSON file.

## Scope

- `Pattern`: the beads put on a field with no end; every other cell is
  empty. A `Cell` has a signed column and row.
- `Pattern::bounds`: the active area, the box around the beads; `None` when
  there are none. `Bounds` grows, meets, joins and doubles.
- `Pattern::revision`: changes with every change of the pattern; no other
  pattern has it unless it is a clone. A drawing of the pattern is kept by it.
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
  "grid": "peyote",
  "sheet": {
    "origin": [-4, 1],
    "size": [8, 4]
  },
  "bead_ratio": 1.0,
  "palette": [
    { "key": "A", "rgb": "#1b1b1b", "catalog": "miyuki-delica-11", "code": "DB-0010" },
    { "key": "B", "rgb": "#f2efe6" }
  ],
  "rows_origin": [-3, 2],
  "rows": ["AAB.BB", "ABABA."]
}
```

The writer puts each number of an array on a line of its own; the example
keeps them on one.

| Field | Meaning |
| --- | --- |
| `format` | the format of the file, `FORMAT` when written |
| `grid` | `loom` or `peyote` |
| `sheet` | the size of the work area set by hand; left out when none is set |
| `sheet.origin` | column and row of the field of its top left cell |
| `sheet.size` | columns and rows, at least one each |
| `bead_ratio` | the width of a bead to its height |
| `palette[].key` | one character; never `.` |
| `palette[].rgb` | `#rrggbb` |
| `palette[].catalog`, `code` | where the colour comes from; may be left out |
| `rows_origin` | column and row of the field of the first bead of the first row; `[0, 0]` with no rows |
| `rows` | the active area, the box around the beads; every row as long as the first |

A cell of the field has a signed column and row. The rows and the sheet are
both placed on the field by their own origin, so a pattern opens where it
was, and a peyote column keeps its parity. The work area is not written: it
is the box around the sheet and the rows.

```text
on the field                          the file
        column -4         3           "sheet": origin [-4, 1], size [8, 4]
row 1   ┌──── sheet ──────┐
        │ ┌── rows ──┐    │           "rows_origin": [-3, 2]
row 2   │ │AAB.BB    │    │           "rows": ["AAB.BB",
row 3   │ │ABABA.    │    │                    "ABABA."]
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
`Newer`, `UnknownFormat`, `RowWidth`, `UnknownKey`, `Outside`, `DuplicateKey`,
`Colour`, `Ratio`.
