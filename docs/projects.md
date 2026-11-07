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
`src/ui/projects/menu.rs`; the cards and the plates in `cards.rs`.

The picture of a card is the field: `obusina_canvas::still` gives the
shapes of the paper and the beads by the code that draws the field, and
`thumbs.rs` has the renderer of the window draw them once into a texture
of the graphics chip, at the size of the card in pixels. The card then
shows the texture only. It is drawn again when the file of the pattern,
the size of the card or the volume of the beads changes.

```text
frame N    still(): the shapes; the sprites of the beads become textures
frame N+1  once every texture of the shapes is on the chip: tessellate,
           render into a texture of the target format, register it
```

- Why the renderer of the window: its pipeline and its textures draw the
  field, so the picture is the same.
- The texture is shown through its own format: the renderer writes into it
  what it writes into the window.
- Where the renderer is not wgpu, a card shows its paper only.

## The choice

```text
a long press, the second button on a card ──► the choice, of that card
the choice ──a tap or a long press on a card──► chosen or cleared
           ──✖──► no choice
the bar ──export │ to the trash │ lock or unlock │ the menu
the menu ──one chosen──► the menu of a card
         ──more──► lock n │ unlock n │ export n │ move n to the trash
```

- Why a long press chooses and opens no menu: on a phone a pattern is
  mostly acted on with others; the menu is one tap further, on the bar.
- A finger chooses a card after `hold::SECONDS`, and may drift `hold::SLOP`
  points meanwhile; the list scrolls under it. Why not the long touch of
  egui: it lets the finger drift 6 points only, and a finger on a large
  card drifts more, so the choice was lost (`src/ui/projects/hold.rs`).
- The lock of the bar and of the menu of more locks all the chosen while
  one of them is not locked, else unlocks them.
- The export of more is one `.obusiny`; of one, a `.obusina`.
- The choice stays after a lock and an export; a move to the trash ends it.
- The screen stands inside the safe area of android (`docs/panels.md`): the
  root of eframe covers the whole window, so the list is drawn in
  `Context::content_rect`.

## A locked pattern

"Forbid changes" of the menu of a card or the lock of the bar of the
choice locks a pattern; "Unlock the card"
lets it change again. The lock is a field of the pattern and its file,
`locked` (`crates/obusina-pattern/README.md`), so a pattern exported locked
opens locked elsewhere.

- The card shows a red lock left of the name (`icons::LOCK`).
- The window shows the field and the tools only: the settings, the shadows,
  the underlay and the way back. The settings hold the volume alone. A
  press on the field does nothing. The keys are those of `Layer::Viewer`
  (`docs/keys.md`).
- Why the underlay stays: it is no part of the pattern; it is kept apart,
  in the cache (`docs/backdrop.md`).
