# mockups

Mockups of the design of the window, shown in the gallery of `showcase`.

## Scope

- A mockup is a part of the window drawn with made up data: a panel, a
  window, a row of buttons. It is drawn with the widget crates of the
  workspace (`obusina-button`, `plate-menu`), so it looks as the window will.
- One module of `src/mock/` is one mockup, a `showcase::Mockup`. `mock::all`
  lists them for the gallery; `src/main.rs` hands them to `showcase::run`.
- The gallery, the notes on a mockup, the page of the pictures and the
  receiver are `/workspace/showcase`: `/workspace/showcase/README.md`.
- `make mockups`, after `. /opt/env.sh`, serves the page on port 6080 of
  every address, with the receiver of `showcase` on 6081. The notes are kept
  in `notes/`, the pictures the page shows are read from `shots/`; git
  ignores both.
- `cargo run -p mockups` shows the same gallery in a window of the desktop.
  It sends no note.

| Part | What it is |
| --- | --- |
| `index.html` | the page: one canvas, and the line that tells trunk to build the crate |
| `Trunk.toml` | the port, the site in `target/mockups-site`, the profile, no downloads, the proxy to the receiver |
| `[profile.mockups]` | of the workspace: `dev` with no debug information, the crates under it optimised |

## Boundaries

- It depends on the widget crates and on `showcase` only. The panels of the
  window live in the root package, which knows files and settings and is not
  built for wasm: a mockup of a panel copies its layout, not its code.
- Nothing depends on it. It is no test of the window (`RULE test.window`);
  its tests check that every mockup draws and what a mockup does when it is
  pressed.
- Text is written in the code in both languages, `Lang::tr`; it does not
  go through `rust_i18n`.

## Errors

A mockup that cannot open a menu says so in the log. What the receiver and
the gallery answer is in `/workspace/showcase/README.md`.

## Usage

```text
. /opt/env.sh && make mockups     then  http://localhost:6080/#palette-move
```
