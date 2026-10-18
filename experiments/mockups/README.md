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

| Part | What it is |
| --- | --- |
| `index.html` | the page: one canvas, and the line that tells trunk to build the crate |
| `Trunk.toml` | the port, the site in `target/mockups-site`, the profile, no downloads |
| `[profile.mockups]` | of the workspace: `dev` with no debug information, the crates under it optimised |

## Boundaries

- It depends on the widget crates only. The panels of the window live in the
  root package, which knows files and settings and is not built for wasm: a
  mockup of a panel copies its layout, not its code.
- Nothing depends on it. It is no test of the window (`RULE test.window`);
  its one test checks that every mockup draws.
- Text is written in the code, in english or russian; it does not go through
  `rust_i18n`.

## Errors

None of its own. trunk refuses a `wasm-bindgen` of another version than the
crate `wasm-bindgen` of `Cargo.lock`; `/opt/packages.sh` installs that one,
and a new version of the crate is a new `WASM_BINDGEN` there.

## Usage

```text
. /opt/env.sh && make mockups     then  http://localhost:6080/#palette-editor
```
