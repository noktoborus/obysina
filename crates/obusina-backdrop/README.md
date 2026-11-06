# obusina-backdrop

The pictures under bead patterns, each with how it lies over the field of its
pattern, kept in a directory of their own.

## Scope

- `Store::new(dir)`: the directory. It is made when the first picture is put.
- `Store::put(bytes, backdrop)`: copies a picture as it is under the next
  free number, writes its description and takes away the picture the pattern
  had. A pattern has one picture.
- `Store::find(pattern)`, `Store::read(kept)`, `Store::save(kept)`,
  `Store::remove(kept)`.
- Every call is `async`, so a store of the browser (OPFS), whose files
  answer in futures, stands behind the same calls. Here a call does its
  work on the files at once; the caller runs it away from the frame. A
  picture is named by its file in the store, `Kept::file`, not by a path.
- `Kind::of(bytes)`: PNG, JPEG or WebP, told by the first bytes, not by the
  name of the file. `Kind::EXTENSIONS` for a dialog of files.

```text
<dir>/
    1.png      the picture, the bytes it was given
    1.yaml     pattern: <id of the pattern>
               corners: [[x, y], ...]  top left, top right, bottom right,
                                       bottom left, in bead units of the field
               opacity: 0.6
               hidden: false
               source: photo.png       for a person
```

A file is written beside its place and renamed into it.

## Boundaries

It knows files and YAML. It knows no toolkit and no pattern: the pattern is
named by a string, the picture is bytes, the corners are numbers. It does
not decode a picture.

A description this version cannot read is passed over, so what an older
version wrote is lost. Its number is not given again.

## Errors

`BackdropError`: a file that cannot be read or written, a description that
cannot be written, bytes of another kind (`Kind`).
