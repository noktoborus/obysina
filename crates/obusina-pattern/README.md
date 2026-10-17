# obusina-pattern

A bead pattern: its grid, its palette, its beads and its JSON file.

## Scope

- `Pattern`: a grid of `width` by `height` cells; each holds a colour of the
  palette or nothing.
- `Grid`: where a bead stands, which bead is under a point, which beads touch
  it.
- `read`, `write`, `Pattern::to_json`, `Pattern::from_json`: the file.

| Grid | Where a bead stands |
| --- | --- |
| `loom` | rows and columns line up |
| `peyote` | an odd column is half a bead lower |
| `brick` | an odd row is half a bead to the right |

## The file

JSON, pretty printed. A row of beads is one line: one key per bead, `.` for an
empty place.

```json
{
  "format": 1,
  "grid": "peyote",
  "width": 6,
  "height": 3,
  "bead_ratio": 1.0,
  "palette": [
    { "key": "A", "rgb": "#1b1b1b", "catalog": "miyuki-delica-11", "code": "DB-0010" },
    { "key": "B", "rgb": "#f2efe6" }
  ],
  "rows": ["AAB.BB", "ABABAB", "......"]
}
```

| Field | Meaning |
| --- | --- |
| `format` | the format of the file, `FORMAT` when written |
| `bead_ratio` | the width of a bead to its height |
| `palette[].key` | one character; never `.` |
| `palette[].catalog`, `code` | where the colour comes from; may be left out |

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
`Newer`, `UnknownFormat`, `RowCount`, `RowWidth`, `UnknownKey`, `DuplicateKey`,
`Colour`, `Size`.
