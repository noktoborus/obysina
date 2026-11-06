# The picture under the pattern

A picture to trace a design from: chosen by the person, laid under the beads
over four corners, kept apart from the pattern. The store is the crate
`obusina-backdrop` (`crates/obusina-backdrop/README.md`); the window side is
`src/backdrop.rs`, `src/pick/` and `src/ui/backdrop_menu.rs`. Its tests are
`src/window_tests/backdrop.rs`.

## Decisions

1. **The picture is kept in the cache of the program, not in the pattern.**
   A copy of the file is `<n>.<ext>` in `backdrops/` of the cache, `<n>` a
   number no other picture of it had; `<n>.yaml` beside it says how it lies
   and names the pattern by its identifier (`PatternId`).
   - The cache: `ProjectDirs::cache_dir` on the desktop, `cache` beside the
     data of the program on android. The system may clear it: the picture is
     then gone, the pattern is not.
   - Why not in the pattern: a picture in the JSON makes the file large and
     its diff unreadable (`RULE pattern.diff`).
2. **A pattern has one picture.** A new one takes the place of the old one,
   and its corners.
3. **A pattern has an identifier,** a random UUID in its file (`id`). A file
   with none takes a new one when read, written at the next save. A copy of
   the file has the same one, and so the same picture.
4. **What an older version wrote to the store is lost** (`RULE
   config.legacy`): a description it cannot read is passed over. Its number
   is not given again.
5. **The picture is no part of undo.** The editing of the corners has its
   own "keep" (✔) and "put back" (✖); a removal asks first, for it deletes
   the copy.
6. **PNG, JPEG and WebP,** told by their first bytes, not by the name of the
   file. Another file is refused and nothing is kept. A picture with a side
   longer than `PICTURE_SIDE` is drawn smaller; the copy keeps all of it.
7. **The picture lies on the paper, under the shadows and the beads.** Its
   opacity is a slider in the notice of its menu, written when the menu
   closes.
8. **The four corners make any quadrilateral.** The picture is drawn
   through the projection they give, a mesh of `STEPS` by `STEPS` cells, so
   its lines stay straight and a photo taken at an angle comes out straight.
   - Two triangles were weighed and left: each is stretched on its own, and
     the lines of the picture break on the diagonal.
9. **The corners are on the field, in bead units.** The picture moves,
   zooms and turns with the beads.
10. **A new picture lies over the work area,** in its middle, as large as
    fits it, its sides as they are.
11. **The corners are edited on the field.** A drag that begins on the
    handle of a corner moves that corner; one inside the picture moves all
    of it; one outside moves the field, so a corner off the screen is
    reached. A press puts no bead while the corners are edited.

```text
picture ──► PlateMenu: the opacity on top, then
              │──choose a picture…─► the choice of the platform
              │──edit the corners──► handles; ✔ keeps and writes, ✖ puts back
              │──hide / show───────► kept, not drawn
              └──remove…───────────► asks; "remove" deletes the copy
```

## The choice of a file

| Platform | How | Code |
| --- | --- | --- |
| Linux, Windows | the dialog of the desktop (`rfd`), in a thread of its own | `src/pick/desktop.rs` |
| Android | `ACTION_OPEN_DOCUMENT` of `MainActivity`, read to bytes in Java, given to Rust by JNI | `src/pick/android.rs`, `android/` |
| Browser | the file input (`rfd`) | `src/pick/web.rs` |

```text
Picker::start ──► the platform asks ──► Sender ──► Picker::take, in a later frame
                                                    ──► App::take_picture
```

The answer is bytes and, when the platform gives one, a name. The window
keeps drawing while it waits.
