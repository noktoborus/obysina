# AGENTS

LANG   this file uses the notation below. keep writing it in this notation.
FORM   RULE <topic> = <value>        hard rule
       RULE <topic> : <predicate>    hard rule, predicate form
       WHY  <reason>                 why the rule exists
       THEN <action>                 what to do when the rule triggers
       EX+  <good example>
       EX-  <bad example>
       REF  <path>                   where to look
       EXC  <exception>              where a rule is broken, and on whose word
READ   RULE is binding. WHY is context, not negotiable. EX is literal.

## Project

RULE product = Obusina, an editor of bead patterns
REF  README.md
REF  ARCHITECTURE.md

RULE version = `version` of `[workspace.package]` of Cargo.toml, set by
     `make VERSION=x.y.z`, which commits it and tags it `vx.y.z`
THEN the android package reads it from there; it is written nowhere else
REF  Makefile, target `version`

RULE release : a tag `vx.y.z` pushed is a release, by
     `.github/workflows/release.yml`: on the page of releases only, the
     site of the browser on GitHub Pages
THEN the builds are `.github/workflows/build.yml`, for every push and for a
     release; a step of a build is a target of the Makefile, so it runs here
     the same
EX-  a command of a build written in a workflow and nowhere in the Makefile
THEN an action is named by its latest major version
WHY  GitHub stops the runtime of Node 20 on its runners; an old major of an
     action runs on it
EX-  actions/checkout@v4, which runs on Node 20
REF  .github/workflows/

RULE app.id = ru.styxheim.obusina
WHY  the desktop matches a window to the program by one name. A window that
     names no entry gets the default icon and shows up twice in the task list.
THEN the window `app_id`, the desktop entry, its `StartupWMClass`, the icon
     file and the android package all carry it; `APP_ID` of `src/lib.rs` and
     `ID` in the Makefile hold it

RULE icon.source = `assets/icon.svg`, the one picture of the icon
THEN `build.rs` draws from it the icon of the window and, for windows, the
     `icon.ico` of the resource linked into `obusina.exe`
EX-  an `icon.ico` or a PNG of the icon committed beside the SVG

RULE build.lto : every build that leaves the machine is fat LTO in one unit of
     code: `[profile.release]`, and `[profile.mockups]` of
     `experiments/mockups`
THEN the desktop, windows, android and the browser build by `--release`;
     a profile of its own carries `lto = "fat"` and `codegen-units = 1`
WHY  a smaller program loads faster over a slow line and runs faster on a
     small machine
EX-  a build of a release, or of the mockups, by `dev`

RULE lang = rust, edition 2024
RULE toolkit = egui and eframe

RULE targets = linux, windows, android, a browser (wasm)
RULE platform_code : platform specific code is `cfg` gated
EX+  #[cfg(target_os = "android")] fn data_dir(app: &AndroidApp) -> PathBuf
EX-  two source trees, one per platform

RULE platform.files : a file is read and written through `unifs`, by async
     calls; the code above it is the same on every platform
WHY  a browser has no `std::fs`; its private file system answers in promises
THEN the window runs a call away from the frame (`src/work.rs`); its answer
     comes in a later frame
THEN a build script and a test read and write their files as they like
EX-  `std::fs::read` in the code of the program or of a crate above `unifs`

RULE android = eframe with `android-game-activity`; `android/` is the
     `cdylib` android loads, its `android_main` calls `obusina::android`
WHY  eframe gives accesskit on android only with the game activity
     (`RULE ui.name`)
THEN `NativeOptions::android_app` carries the `AndroidApp` it is given; the
     pattern lives in the internal data path of the program
RULE android.package = gradle in `android/`: the activity is
     `com.google.androidgamesdk.GameActivity` of `androidx.games:games-activity`,
     the library is built by cargo-ndk from a task of `android/app/build.gradle.kts`
THEN the version of games-activity is the one `android-activity` names in its
     README, not the latest
WHY  the native half of the activity is in `android-activity`, the java half in
     the library; two versions that do not match fail at run time
EX-  games-activity raised to 4.4.2 because it is newer
EXC  the prefab of games-activity is not used: `android-activity` carries its
     own native half

RULE android.cdylib : only `android/` is a `cdylib`, and it is empty on any
     other platform
WHY  a `cdylib` of the whole program exports every symbol, and the linker of
     mingw refuses that many
REF  android/README.md

## Layout

RULE binary = the root package `obusina`: a library with an entry for each
     platform, and `src/main.rs`, which starts it on the desktop
RULE platform_entry = `android/`, the one other package that starts the
     program
RULE library_code = crates/*
RULE experiments = experiments/*, a workspace member each, none of them
     depended on
EX-  a crate of `crates/*` carrying a `[[bin]]`

RULE crate.purpose : one crate is one purpose, builds and is tested on its own
THEN split a crate that grew a second purpose

RULE crate.layers : every crate of the workspace stands in one layer; a crate
     depends only on crates of the layers below its own
     0 platform       unifs
     1 model          obusina-pattern, obusina-backdrop
     2 drawing        obusina-canvas, obusina-panel, obusina-button,
                      plate-menu, egui-notify
     3 application    obusina
     4 entry          obusina-android
WHY  a dependency that goes down keeps each layer whole without the ones
     above it; one within a layer or up mixes them
THEN a new crate takes a layer, here and in ARCHITECTURE.md; a crate that
     needs one of its own layer or above is split, or the shared part moves
     to a layer below
EX+  obusina-backdrop -> unifs, obusina-canvas -> obusina-pattern
EX-  unifs -> obusina-pattern            up
EX-  obusina-panel -> obusina-button     within a layer
REF  ARCHITECTURE.md

RULE crate.knowledge : a library crate knows nothing of the interface, the
     configuration files or the types of crates not below it
     (`RULE crate.layers`)
EX+  obusina_pattern::Pattern, the crate's own type
EX+  a widget crate that knows egui and nothing of this application
EX-  obusina_pattern::Pattern reading `settings.yaml`
EX-  obusina_pattern depending on egui for `Color32`

RULE plate_menu.copy : `crates/plate-menu` is a copy of the crate of zyterm,
     moved to egui 0.37
WHY  zyterm is on egui 0.36, and a path to its crate brings a second egui
     whose types do not meet those of this one
THEN the copy goes once both share one crate on one egui
EXC  the copy, by the user's word, until then
EXC  work on obusina touches no other project, by the user's word: a change
     of this copy is not carried to zyterm, and the two drift apart

RULE notify.patch : `crates/egui-notify` is a copy of egui-notify 0.24.0 of
     crates.io with a button of an action; `[patch.crates-io]` of the
     workspace puts it in place of that crate
WHY  the toasts of egui-notify have no button; "undo" needs one
THEN depend on `egui-notify = "0.24"`, as on crates.io; the changes are
     listed in its README
THEN the copy goes once egui-notify of crates.io has an action
EXC  by the user's word: its version is the one of the crate it copies, its
     licence is its own MIT with the notice of its author (`LICENSE`), and
     its code keeps the shape of the original, `clippy::collapsible_if`
     allowed

RULE crate.readme : every crate has a README.md naming its scope, boundaries
     and errors
REF  crates/*/README.md

RULE license = MIT, declared once in `[workspace.package]`
THEN every member says `license.workspace = true`; none has a licence of its own

## Patterns

RULE pattern.format = JSON through serde_json
RULE pattern.lib : the model and its file live in `obusina-pattern`; it knows
     no toolkit
RULE pattern.write : a write is whole: `unifs::Files::write`
WHY  an interrupted write must not leave half a pattern

RULE pattern.loader : only the loader knows old formats
THEN one loader per format, `crates/obusina-pattern/src/load/v<N>.rs`; it
     turns its file into the current `Pattern`. The model and the writer know
     only the current format, `FORMAT`.
THEN a new shape raises `FORMAT`, changes the writer and adds a loader; the
     loaders before it are not changed
THEN a file of each format lies in `crates/obusina-pattern/tests/patterns/`,
     never changed; `tests/formats.rs` reads every one
EX-  a field of `Pattern` kept only because an old format had it
EX-  `v1.rs` edited to read a file of format 2
RULE pattern.newer : a file of a format newer than the program is refused with
     a message; it is never written over

RULE pattern.diff : the file is pretty printed, one row of beads on one line
WHY  a pattern kept in git shows a change as the rows that changed

RULE grid.model : a kind of grid is a type of `obusina-pattern`: where its
     beads stand and which bead is under a point
WHY  the canvas, the count and the word chart ask the grid; none of them holds
     a rule of a stitch
EX-  `if peyote { y += 0.5 }` in the code that draws
REF  crates/obusina-pattern/src/grid.rs

RULE bead.shape : a bead is drawn at its proportions, not as a square
WHY  a seed bead is not as tall as it is wide; a picture drawn on squares comes
     out stretched in beads
THEN the proportion is a field of the pattern; a catalogue gives the default

RULE palette.catalog : the colours of a maker's catalogue are data files, read
     at run time, not code
EX+  catalogs/miyuki-delica-11.json
EX-  a `const DELICA: [(&str, [u8; 3]); 1700]`

## Files

RULE file.max_lines = 1000
THEN split into submodules

RULE module.subject : one module is one subject
EX+  error.rs, grid.rs, palette.rs
EX-  util.rs, helpers.rs, misc.rs

RULE docs.public : every public item has a documentation comment
RULE docs.deny = `#![deny(missing_docs)]` in every library crate

RULE format = `cargo fmt --all`
WHY  the shape of the code is nobody's opinion. A diff that only reformats is
     a diff nobody reads.
THEN run it over every change, before the change is read or committed
EX-  a line laid out by hand that rustfmt lays out another way

RULE format.tool = rustfmt with its defaults, no `rustfmt.toml`
WHY  `cargo fmt` knows the members of the workspace. `rustfmt` alone reaches
     only the modules of the files it is given.
THEN `--all`, never a file at a time
REF  `cargo fmt --all --check` says whether it was run, and changes nothing

## Documentation files

RULE doc.files = README.md, ARCHITECTURE.md, docs/*.md, crates/*/README.md
RULE doc.lang = simplified english (workspace `RULE text.lang`), technical

RULE doc.style : short sentences, one subject each; no metaphor
WHY  these files are read to find one answer, not from the first line to the
     last
THEN say the thing, then the reason in one clause, then stop
EX+  A peyote column is half a bead lower than its neighbour. The grid gives
     the offset; the canvas does not compute it.
EX-  A column of peyote lives in a world where every neighbour has slipped a
     little, and what comes of that is a drawing nobody asked for.

RULE doc.shape : a list, a table or a diagram wherever one fits; prose for
     what none of them holds
WHY  a table of eight rows is read in a glance; the same eight sentences are
     not
THEN a mapping is a table, a sequence of states is a diagram, a set of rules
     is a list
EX+  | grid | offset |
EX+  Saved ──edit──► Changed ──save ok──► Saved
EX-  eight sentences, each naming one grid and its offset

RULE doc.diagram = plain text inside a fenced block, ascii and box drawing
     characters only
WHY  a diagram is read in a terminal, in a diff, and by an agent as text
EX-  an image file drawn by hand, a mermaid block

RULE doc.picture : what a widget looks like is a picture its picture test
     rendered; how it behaves is a diagram
WHY  a rendered picture fails its test when the widget changes, so it cannot
     go out of date; a drawn one can
THEN `img/<name>.png` beside the crate README, named there by a label
     `[picture-<name>]: img/<name>.png`; `build.rs` writes the same labels as
     `data:` URLs before the README in `lib.rs`, because rustdoc copies no
     file and Markdown takes the first definition of a label
EX+  ![A peyote grid with a palette of three colours][picture-peyote]
EX-  a screenshot taken by hand, a picture with no test that renders it
EXC  the screenshots of README.md, in `img/`: drawn by the window test
     `screenshots`, written by `SCREENSHOTS=1 cargo test -p obusina
     screenshots`, compared to nothing; on the word of the user

RULE doc.settings : a documentation file never repeats what the settings page
     shows — a default, a unit, the entries of a list, the text of a hint
WHY  it is a second copy of the page, kept by hand, and it goes out of date in
     one of the two places
THEN name the setting and say what the page cannot: why it exists, what it
     costs, what breaks without it

RULE doc.numbers : a number that is a constant of the code is named, not
     written out
WHY  a number written twice goes out of date in one of the two places
EX+  `DEFAULT_BEAD_RATIO`
EX-  one and a quarter

RULE doc.subject : README is what the program does; ARCHITECTURE and its
     chapters are how it is built; a crate README is the surface of that crate
THEN a decision with a reason goes to the chapter of its subject; a feature
     goes to README; what a caller must know to use a crate goes to its README
REF  README.md, ARCHITECTURE.md, docs/

RULE doc.chapters : ARCHITECTURE.md is an index — the crates, the model, the
     file, the errors. Every other subject is a file of `docs/`.
WHY  a documentation file is read whole for one answer
THEN write the chapter in `docs/<subject>.md`, name it in the table at the
     head of ARCHITECTURE.md, and leave in the index only what the chapters
     are read against
EX+  docs/grids.md, named in the table as the kinds of grid
EX-  a tenth section added to ARCHITECTURE.md

RULE doc.once : one thing is written down in one file; the others name it and
     say where it is
WHY  two copies of a paragraph go out of date in one of the two places
THEN what a crate answers for goes to its README, what the application decides
     with it goes to the chapter, and the chapter names the README by path
EX+  The file format itself is `crates/obusina-pattern/README.md`.
EX-  the table of JSON fields in a chapter and in the crate README

RULE doc.crate : a crate README carries scope, boundaries and errors, in that
     order, and the signatures a caller starts from

RULE doc.current : a documentation file says what the code does now
WHY  a file that describes what was taken out is read as a description of
     what is there
THEN change the file with the code, in the same commit; delete what is gone

## Comments

RULE comment.place : comments document items with `///` and `//!`
RULE comment.body : no inline comments inside a function body
WHY  a line that needs an explanation needs a name instead
THEN name the value, or split the function

RULE comment.lang = simplified english

## Errors

RULE error.enum : every crate defines one error enum in error.rs and a
     `Result<T>` alias

RULE error.shape : the enum is closed and machine readable, one variant per
     failure case, carrying what is needed to act on it

RULE error.source : an error of a lower layer is wrapped in a matching variant
     with `#[source]`
THEN add a variant instead of widening an existing one
EX+  #[error("pattern file error")] Pattern { #[source] source: PatternError }
EX-  Err(format!("{error}"))

RULE error.message : error messages are english and diagnostic only, never
     shown to the user

RULE error.user_text : user facing text lives in the binary, in
     locales/app.yml, selected by `AppError::message_key`
REF  src/error.rs

## Text and language

RULE i18n = rust_i18n, the `t!` macro
RULE locales = en, ru
RULE locale.coverage : every key in locales/app.yml has an en and a ru value
WHY  a missing value would show the key to the user
THEN a test enforces it; run it
REF  tests/locales.rs

RULE icon.glyph : an icon is a code point the fonts of egui carry, declared in
     src/ui/icons.rs
WHY  a code point no font carries is drawn as a box
THEN add it to `icons::ALL`; the test there asks the toolkit and fails if no
     font has it
EX+  pub const UNDO: &str = "\u{21b6}";
EX-  "\u{25a8}", a square with lines no default font carries

RULE icon.search : an icon is picked from `cargo run -p glyphs`, never guessed
WHY  the set the fonts carry is small and not obvious. `Fonts::has_glyph`
     says no for a code point whose first face is the one the replacement
     glyph comes from.
THEN take a code point that stands in the list the tool writes, with the face
     that carries it
EX-  dropping a code point because `has_glyph` said no
THEN the emoji faces come with the feature `monochrome_emoji_fonts` of egui;
     without it the toolkit carries no icon at all
REF  experiments/glyphs/README.md

RULE ui.mockup : a design discussed before it is built is shown as a mockup
     of `experiments/mockups`, built by `make mockups` into the module
     `mockups` of the desk `obusina` of showcase
THEN one module of `src/mock/` per mockup, drawn with the widget crates and
     made up data; the address `#obusina/mockups/<name>` opens it
THEN the gallery, the notes and the server are `/workspace/showcase`, a
     project of its own; `experiments/mockups` holds only the mockups of
     obusina and depends on it by path
REF  /workspace/showcase/README.md
THEN a mockup is no test and no part of the window: the window is built and
     tested on its own (`RULE test.window`)
RULE ui.mockup.notes : a note left on a mockup is read with
     `../showcase/target/debug/showcase notes --desk obusina`, acted on, then
     deleted with its picture
THEN what it decided goes to the plan or to the documentation; a note is not
     kept
THEN while the user works on mockups, `../showcase/target/debug/showcase wait
     --desk obusina` runs in the background; when it ends, the notes are read
     and it is started again
REF  experiments/mockups/README.md

RULE log.lang = english, through the `log` crate

## Interface

RULE ui.name : every widget carries a name a screen reader can read
WHY  egui names a widget for accesskit by the text drawn on it. An icon is
     read as a code point. A switch whose label stands beside it is read as
     nothing. A hint shown on hover is not given to accesskit at all.
THEN `ui::access::Named`: `tip` for an icon with a hint, `named` for the
     rest; a widget drawn by hand gives `Response::widget_info`
EX+  ui.button(icons::UNDO).tip(&t!("edit.undo"))
EX-  ui.button(icons::UNDO).on_hover_text(t!("edit.undo"))
REF  src/ui/access.rs

RULE ui.touch : every action is reachable by touch
WHY  on android there is one button, no hover, no wheel and often no keyboard
THEN a right click, a hover, a key or a wheel is a second way to an action,
     never the only one; two fingers zoom and move the field
EX-  a second colour reachable only by the right button
EXC  F1, the plate of the keys (`src/ui/help.rs`), by the user's word: it
     shows the keys, which help only a keyboard, so a key is its one way

RULE ui.layout : the field of beads fills the window; the tools and the
     palette are `obusina_panel::FloatingPanel`s
THEN no side panel, no menu bar, no title and no grip on a panel: any place
     of it moves it, a mouse moved or a finger held still
EX-  a `SidePanel` holding the palette
EX-  an `egui::Window` with its title bar
REF  crates/obusina-panel/README.md

RULE ui.dock : a panel pulled to an edge strip takes the view of that edge
     when that view fits the length of the edge whole; it turns nowhere
     else; the menu of the settings chooses its view too; let go in an edge
     strip it stands against that edge
THEN nothing but the panel, lighter, is drawn while it is dragged
THEN the content of a panel lays itself out by the `Room` it is given; a
     line goes on in the next line where the window ends
THEN the place and the view of each panel are kept in the settings, by the
     posture of the window and the name of the panel; with none kept, the
     panels are stood by `src/ui/arrange.rs`
REF  docs/panels.md

RULE ui.button : a button of a panel is made by `ui::button`, all of one
     size, `button::SIZE`, the size of a swatch; each is an
     `obusina_button::SquareButton`, its label in the middle
WHY  a row or a column of buttons of one size reads as one row, and each is
     large enough for a finger
EX+  button::icon(ui, icons::UNDO, &t!("edit.undo"), can_undo)
EX-  ui.add(egui::Button::new(icons::UNDO)) in a panel
REF  src/ui/button.rs

RULE ui.menu : a choice among variants is a `plate_menu::PlateMenu` in the
     middle of the window, opened by a button of a panel
EX+  the kinds of grid, a level of `src/ui/settings_menu.rs`
EX-  an `egui::Popup` or a `ComboBox` under the button
THEN a menu opens by `open`, with no entry selected
WHY  the user's word: a tap must not meet an entry lit for the keys
THEN `open_at` only opens a menu again on the entry just chosen in it;
     `open_in` again in the level it stood in
THEN a value of several settings is a level of the menu, its editor the
     notice of the level; the way back out of it is `‹`, Esc, the way back
     of android, never an entry "Cancel"

RULE ui.keys : every key is a row of a table: `Binding`s of the layer,
     beside its code, or `keys::GLOBAL`; `keys::layer` says which layer is on
     top, and only it acts on a key
THEN a layer reads its table with `keys::take`; a new layer is a variant
     of `keys::Layer`, its table named in `keys::keymap`
THEN `clippy.toml` refuses `key_pressed`, `consume_key` and their kin; only
     `keys::take` and the menu of `plate-menu` read a key
THEN `UPDATE_KEYS=1 cargo test -p obusina keys` writes `docs/keys.md` from
     the tables; a test fails while the file says other
EX+  Binding { keys: &[command(Key::Z)], action: FieldAction::Undo, name: "edit.undo" }
EX-  if ui.input(|input| input.key_pressed(Key::P)) in the code of a panel
REF  src/keys.rs

RULE ui.narrow : the window works at the width of a phone
THEN a floating panel is kept inside the window when the window is smaller

RULE ui.render = `render::native_options` starts the window, on every platform
     but a browser; `render::web_options` starts the page in a browser
WHY  a modest chip ends up drawn by the processor unless the adapter is
     picked by hand and the limits asked are cut to what it grants; the same
     code runs the window of zyterm. A browser picks the adapter itself, but
     the defaults of eframe ask a texture of 8192, and a Raspberry Pi grants
     4096
THEN the glyph atlas is cut to `render::ATLAS_SIDE` in `App::raw_input_hook`
REF  src/render.rs

RULE ui.undo : every change to a pattern can be undone
THEN a change goes through one edit type of the binary; nothing writes the
     model past it
EXC  the turn the pattern is shown at, by the user's word: it is kept in the
     pattern for its file and moves no bead, so it is written past the
     history and is no step of undo

## Configuration

RULE config.dir = the platform configuration directory, resolved with the
     `directories` crate from `app.id`; on android, the internal data path of
     the `AndroidApp`
RULE config.format = yaml through serde_yaml_ng
RULE config.write : a write is whole: `unifs::Files::write`
WHY  an interrupted write must not leave half a settings file

RULE config.legacy : no code reads what an older version wrote to the
     configuration
WHY  a shape that is read is a shape that is kept, in the types, the tests and
     the head of whoever reads them next. The settings are the program's own
     file. Patterns are not: `RULE pattern.version`.
THEN change the file, the type and what writes it, and let the old shape go.
     A file the current version cannot read is kept aside, and the program
     starts from the defaults.
EX-  a field read under the name it used to have
EX-  a `legacy_*` field, a `migrate`

RULE config.default : a key a file does not carry reads as the default of
     that setting
WHY  this is how a new setting reaches a file written without it
EX+  #[serde(default = "default_bead_ratio")]

## Dependencies

RULE dep.version = the latest release on crates.io
EX-  a git or pre-release dependency

RULE dep.search : before writing code for a general problem, look for a crate
     that solves it
EX+  `palette` for the distance between two colours

RULE dep.declare : versions are declared in `[workspace.dependencies]`
EX+  serde.workspace = true
EX-  serde = "1" in a crate manifest

RULE unix.calls = `rustix`, not `libc`
WHY  `rustix` gives a system call as a safe function with typed flags and an
     `Errno`. Through `libc` the same call is `unsafe`.

RULE unsafe : no `unsafe` where a safe wrapper or a safe shape of the code
     does the same
WHY  an `unsafe` block is a promise the compiler cannot check
THEN look for a safe crate or another shape first; where there is none, the
     block is as small as the call

## Window tests

RULE test.window = `egui_kittest` drives the window of `obusina`: a `Harness`
     over `App`, keys, text, clicks and touches given as egui events, what is
     drawn read through the tree of accesskit
WHY  what a change does to the window is seen, not guessed, by every run of
     `cargo test`: no display, no `sleep`, the same on every machine
THEN a change to what the window shows or how it answers comes with a test of
     the window: open it, act on it, check the tree and the pattern
THEN the test is a module under `src/`, so it reads the private state of
     `App`; a pattern is saved to a scratch directory, never one of a person
THEN `egui_kittest` with `eframe`, `wgpu` and `snapshot` is a dev-dependency
     of the root package
EX+  the picker picked by its name, a click on the canvas, the count of the
     colour read from the panel and the cells read from the pattern
EX-  a step that waits by `sleep`: `harness.run()` until the window settles,
     or steps until a condition, with a bound
EX-  an element found by where it was drawn, when it has a label

RULE test.window.look : what a change does to how the window looks is looked
     at
THEN `harness.render()` to a PNG in a scratch directory, opened before saying
     what it shows; that PNG is not committed and is no reference
     (`RULE test.picture`)

RULE test.window.limits : `egui_kittest` hands egui its events, so what lies
     between the system and egui is outside it
THEN these are checked on a display (`RULE gui.display`) when a change touches
     them: a key from the layout through winit to `egui::Key`, the clipboard,
     files dropped, the file dialog, focus and size given by the desktop, the
     scale of the screen, the identity of the window on the desktop
THEN a touch gesture is not one of them: the test gives egui the events
     egui-winit gives it — `Event::Touch`, and for the first finger the
     pointer too

## Picture tests

RULE test.picture : a widget crate has picture tests; nothing else has
WHY  a widget is drawn the same way wherever it is used, so its pixels are its
     behaviour. The window changes with every locale, theme and setting.
THEN the window is tested by its tree (`RULE test.window`), not by its pixels
EX+  `obusina-canvas`: one picture per kind of grid
EX+  `obusina-panel`: a row, a column, a panel taken
EX+  `obusina-button`: every kind of button in one row
EX-  a picture of the main window of `obusina` committed as a reference
EXC  the screenshots of README.md (`RULE doc.picture`, its EXC); no
     reference, nothing compares them

RULE test.picture.tool = `egui_kittest` with `wgpu` and `snapshot`, a
     dev-dependency of the widget crate
THEN `tests/snapshots.rs`, one test a picture, output path `img`; a fixed
     window size and `Theme::Dark`, so a picture changes only with the widget

RULE test.picture.renderer = lavapipe, the software Vulkan of mesa
WHY  the container has no GPU; the same renderer draws the same pixels on
     every machine
THEN `mesa-vulkan-drivers` in `.devcontainer/Dockerfile.claude`; a machine
     without a Vulkan driver cannot run `cargo test --workspace`

RULE test.picture.update : a picture changes only together with the widget,
     and is looked at before it is committed
THEN `UPDATE_SNAPSHOTS=1 cargo test -p <crate> --test snapshots`, then open
     every PNG that changed; the `.new`, `.diff` and `.old` files a failed run
     leaves are ignored by git
EX-  a picture updated so a failing test passes, without opening it

## Checks before finishing

RULE checks : all six are clean before the work is committed
THEN cargo fmt --all
     cargo clippy --workspace --all-targets
     cargo build --workspace
     cargo test --workspace
     . /opt/env.sh && make build-windows
     cargo audit
WHY  code gated by `cfg` for windows is compiled only for windows; the tests
     run on the linux build
WHY  `/opt/env.sh` puts mingw on the path; `windres` of it compiles the icon

RULE checks.web : the page of the browser is built when a change touches
     `web/`, `Trunk.toml`, the code gated for wasm or what it calls
THEN make web

RULE checks.android : the android package is built when a change touches
     `android/`, the code gated for android or what it calls, or when the
     user asks
THEN make apk, after `. /opt/env.sh`

## Commits

RULE commit : finished work is committed, after the checks above are clean
THEN one commit per subject; the subject line says what the work does, and
     the body says why

RULE commit.clean : a message carries nothing of the machine it was written on
WHY  the history is read by people who were never on that machine, for as long
     as the repository lives
THEN name the file, the type, the rule — the things the repository has
EX+  `crates/obusina-pattern/src/grid.rs`, `Grid::cell_at`,
     `RULE pattern.loader`
EX-  a session or request identifier, a password, a token, the name of a
     local user, a path under a home directory, the address of a machine

## Tools

RULE api.inspect : inspect a crate API instead of guessing it
THEN cargo brief, ruskel

RULE gui.display : optional — the window run on a display with none attached:
     Xvfb for X11, cage in headless mode for Wayland
WHY  it is the one way to see what `RULE test.window.limits` names
THEN used when a change touches what `RULE test.window.limits` names, or when
     the user asks; either display is enough unless the change is of one of
     them
THEN build, run the window, act on it, take a picture after each step and
     look at every picture before saying what it shows; the configuration is a
     scratch directory, never one of a person
THEN X11, with no window manager, so the window is given the focus:
       sh -c 'Xvfb :99 -screen 0 1280x800x24 -nolisten tcp & sleep 1
         env -u WAYLAND_DISPLAY DISPLAY=:99 HOME=<scratch> XDG_CONFIG_HOME=<scratch>/cfg target/debug/obusina &' &
       W=$(DISPLAY=:99 xdotool search --name . | head -1)
       DISPLAY=:99 xdotool windowfocus --sync $W
       DISPLAY=:99 import -window root shot.png
THEN Wayland, as the user `headless`: cage refuses to run as root. Its runtime
     directory is that user's, mode 0700, and the program renders on the
     processor:
       runuser -u headless -- env -u DISPLAY XDG_RUNTIME_DIR=<run> HOME=<home> \
         WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman \
         cage -- /workspace/obusina/target/debug/obusina &
       runuser -u headless -- env XDG_RUNTIME_DIR=<run> WAYLAND_DISPLAY=wayland-0 grim shot.png
THEN stop what was started by its name: `pkill -x obusina`, `pkill -x cage`,
     `pkill -x Xvfb`
EX+  wtype -s 300 -k Return -s 300
EX-  wtype -k Return: wtype is gone, and its keyboard with it, before the
     window takes the key
EX-  pkill -f obusina: it matches the shell that runs it
REF  .devcontainer/Dockerfile.claude

## Exceptions

RULE exception : a rule is broken only when the user asks for it
