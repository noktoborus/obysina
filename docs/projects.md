# The list of patterns

The window opens on the list of the patterns of the person. A pattern opens
from it in the editor and goes back to it saved. Nothing is saved by hand.

## Where the patterns are

`src/library/mod.rs` keeps each pattern as a file of its own in the data of
the program, its history beside it:

```text
<data>/patterns/<id>.json          the pattern (crates/obusina-pattern/README.md)
<data>/patterns/<id>.history.json  its history (src/history_file.rs)
<data>/trash/<id>.json, .history.json, .trashed (the seconds it went there)
```

| Platform | `<data>` |
| --- | --- |
| Linux, Windows | the data directory of `APP_ID` |
| Android | the internal data path; a `pattern.json` of the version before is taken in once |

- The time of the last change is the time of the file. The list is in that
  order, the latest first.
- A history is of the text of the pattern it was written with: it holds a
  hash of it. A pattern written alone after it opens with no history. The
  history is a help, not the work: a history this version cannot read is
  none.
- A file that cannot be read is passed over and logged; it is never written
  over.

## When a pattern is saved

```text
a change ──► AUTOSAVE_IDLE seconds with no other ──► save
         └─► AUTOSAVE_STEPS steps since the last save ──► save
back to the list, Escape, the window closed, android pauses ──► save
```

- Why eframe `persistence`: with it eframe calls `App::save` when android
  suspends the program, which is the only frame it gets before it may be
  killed. Its own state goes to `eframe/` beside the settings.
- A pattern that did not change since its last save is not written.

## The trash

A pattern moved to the trash keeps its history and its picture. It stays
`TRASH_DAYS` and goes when the list is next opened, with its picture. A move
to the trash shows a toast with "Undo" for `UNDO_SECONDS`
(`egui-notify`, `RULE notify.patch`). A pattern restored to a list that has
its identifier comes back under a new one.

## Files

`src/library/archive.rs`:

| File | What it holds |
| --- | --- |
| `<name>.obusina` | one pattern file, packed by xz |
| `<name>.obusiny` | a tar packed by xz, a pattern file `<id>.json` for each |
| `.json` | one pattern file, as it is |

- An export holds the patterns only: no history and no picture.
- The kind of a file is told by its bytes, not by its name.
- Why `lzma-rs`: it is Rust only, so it builds for android, windows and the
  browser with no C library. It packs less tightly than liblzma; a pattern
  file is small.

An import takes each pattern of a file:

| The list has | The pattern |
| --- | --- |
| no pattern of its identifier | added with it |
| the same work under it (`Pattern::same_work`: all but the name) | not added; its name is told |
| another work under it, or the trash has it | added under a new identifier |

A toast tells how many came new, how many as copies, and the names of
those already there. A file named on the command line is imported the same
way and opened.

## The screen

`src/ui/projects/` draws it, from the mockup
`experiments/mockups/src/mock/projects.rs`. The menus are in
`src/ui/projects/menu.rs`; the cards, the plates and the pictures of the
cards in `cards.rs`. A picture of a card is drawn once a change of its
file, one pixel a bead, two down so a peyote column stands half a bead
lower.
