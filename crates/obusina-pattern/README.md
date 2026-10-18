# obusina-pattern

A bead pattern: its grid, its palette, its beads and its JSON file.

## Scope

- `Pattern`: the beads put on a field with no end; every other cell is
  empty. A `Cell` has a signed column and row.
- `Pattern::bounds`: the active area, the box around the beads; `None` when
  there are none.
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

JSON, pretty printed. The rows are the active area, and `origin` is the cell
of the field of their first bead. A row of beads is one line: one key per
bead, `.` for an empty place.

```json
{
  "format": 1,
  "grid": "peyote",
  "origin": [
    -3,
    2
  ],
  "bead_ratio": 1.0,
  "palette": [
    { "key": "A", "rgb": "#1b1b1b", "catalog": "miyuki-delica-11", "code": "DB-0010" },
    { "key": "B", "rgb": "#f2efe6" }
  ],
  "rows": ["AAB.BB", "ABABA."]
}
```

| Field | Meaning |
| --- | --- |
| `format` | the format of the file, `FORMAT` when written |
| `origin` | column and row of the field of the first bead of the first row; `[0, 0]` with no rows |
| `bead_ratio` | the width of a bead to its height |
| `palette[].key` | one character; never `.` |
| `palette[].catalog`, `code` | where the colour comes from; may be left out |
| `rows` | the active area; every row as long as the first |

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
