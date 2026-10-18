# mockups

Mockups of the design of the window, shown in a browser.

## Scope

- A mockup is a part of the window drawn with made up data: a panel, a
  window, a row of buttons. It is drawn with the widget crates of the
  workspace (`obusina-button`, `obusina-panel`), so it looks as the window
  will.
- One module of `src/mock/` is one mockup, a `Mockup`: a name and a function
  that draws it. `mock::all` lists them for the gallery.
- The gallery shows a strip of the names over the mockup chosen. The address
  `#<name>` opens that mockup first.
- `make mockups`, after `. /opt/env.sh`, serves the page on port 6080 of
  every address. trunk builds the crate for `wasm32-unknown-unknown`, runs
  `wasm-bindgen` and reloads the page when a file of the crate changes. A
  change of one mockup is rebuilt in about ten seconds.
- `cargo run -p mockups` shows the same gallery in a window of the desktop.
  It sends no note.

### Notes

A note says what is wrong with a part of a mockup. The switch "note" of the
strip turns the mockup into a surface for notes:

```text
off ──switch──► pointing ──tap or drag──► writing ──send──► capturing
                   ▲  ▲                      │                  │ picture
                   │  └────────cancel────────┘                  ▼
                   └──────────────answer of the receiver──── sending
```

- A tap takes the whole widget under it: the smallest one the mockup named
  with `mock::named`, lit while the pointer is over it. A drag draws a box,
  for a place between widgets or several of them.
- Ctrl+Enter in the text sends the note, Esc drops it.
- The note carries the text, the box, the named widgets it touches, the
  state of the mockup (`Mockup::state`) and a picture of the page with the
  box on it (`ViewportCommand::Screenshot`).
- trunk passes `/notes` to the receiver, `src/bin/notes`, on 127.0.0.1:6081.
  It keeps a note as `notes/<id>.json` and its picture as `notes/<id>.png`.
  Git ignores the directory.
- A note is read, acted on and deleted with its picture. What it decided
  goes to the plan or to the documentation; the note is not kept. The strip
  counts the notes waiting, of the mockup shown and of all, and the page
  draws their boxes.
- "waiting" opens the list of the notes waiting. The box of a note is lit
  while its line is under the pointer; the cross deletes it.
- The strip is drawn above the windows of a mockup: a modal window of egui
  blocks the layers under it.
- A press on the strip is taken out of the input in `raw_input_hook`,
  before egui reads it, and the strip reads it from there. The mockup never
  sees it, so a menu of the mockup that closes on a press beside it stays
  open. `egui_kittest` calls no hook: a test passes its presses through the
  same filter.
- While "note" or "waiting" is on, the mockup is frozen. Its first frame is
  taken as a picture, with its named widgets and its state. A browser
  takes the picture a frame or two later, so until it comes the mockup is
  still drawn, gets no press and no key, and nothing of the notes is drawn
  over it. From then on the mockup is not drawn and gets no event, and the
  picture stands in its place. Off again, the mockup is drawn as it was: a menu or a picker of it
  is still open.

| Part | What it is |
| --- | --- |
| `index.html` | the page: one canvas, and the line that tells trunk to build the crate |
| `Trunk.toml` | the port, the site in `target/mockups-site`, the profile, no downloads, the proxy to the receiver |
| `src/bin/notes` | the receiver of the notes |
| `[profile.mockups]` | of the workspace: `dev` with no debug information, the crates under it optimised |

## Boundaries

- It depends on the widget crates only. The panels of the window live in the
  root package, which knows files and settings and is not built for wasm: a
  mockup of a panel copies its layout, not its code.
- Nothing depends on it. It is no test of the window (`RULE test.window`);
  its tests check that every mockup draws and that a tap takes a widget.
- egui keeps no name of a widget that can be read back, so a note knows
  only the widgets a mockup names itself.
- Text is written in the code, in english or russian; it does not go through
  `rust_i18n`.

## Errors

The receiver answers a note it cannot keep with status 400 and the reason;
the strip shows it. A page with no receiver says "no receiver". trunk
refuses a `wasm-bindgen` of another version than the
crate `wasm-bindgen` of `Cargo.lock`; `/opt/packages.sh` installs that one,
and a new version of the crate is a new `WASM_BINDGEN` there.

## Usage

```text
. /opt/env.sh && make mockups     then  http://localhost:6080/#palette-editor
```
