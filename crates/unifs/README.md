# unifs

The files of a program by one async API: a directory of the disk, or the
private file system of the origin (OPFS) in a browser. The code that uses
it is the same on every platform.

## Scope

- `Files::at(dir)` on the disk, `Files::origin(dir)` in a browser: the
  files under one directory. `Files::sub(path)`: a directory under it.
- `read`, `write`, `remove`, `rename`, `list`, `modified`. Every call is
  `async`. A path is relative to the directory, its parts split by `/`.
- A write is whole: a reader sees the old bytes or the new ones, never a
  part.
- `persist()`: asks the platform to keep the files when the disk runs
  short.

| | On the disk | In a browser |
| --- | --- | --- |
| a call | `std::fs`, its work done at once | the promises of OPFS |
| a write | a temporary file beside its place, then a rename | a writable stream, seen once closed |
| `rename` | one rename | a write of the new file, then a removal of the old |
| the time of a file | of the file system | `File.lastModified` |

```text
read(path)          → Option<bytes>, None when it is not there
write(path, bytes)  → the directories are made
remove(path)        → a file that is not there is no failure
rename(from, to)    → a file `from` that is not there moves nothing
list(dir)           → [name, modified], files only; none when it is not there
modified(path)      → Option<time>
```

- Why a call on the disk does its work at once: the caller runs it on a
  thread of its own, away from the frame. No runtime of futures is needed.
- Why `rename` in a browser is no move: `FileSystemHandle.move` is no stable
  call of `web-sys`. A failure between the write and the removal leaves the
  file in both places.

## Boundaries

It knows files and directories only. It knows no format, no toolkit and
nothing of the program. It does not lock a file: two windows of a browser on
one origin write over each other.

## Errors

`Error`: a path out of the directory (`Path`), a file or a directory that
cannot be read (`Read`) or written (`Write`), no file system
(`Unavailable`). The text of the platform is kept in `detail`.
