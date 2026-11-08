//! The menu of the settings, a `PlateMenu` in the middle of the window. Each
//! entry shows what it is now right of its label; what changes the pattern
//! is a level of its own.
//!
//! ```text
//! settings ──► grid <the kind>    › ──► ‹ grid │ loom ✔ │ peyote
//!              turn <0°>          › ──► ‹ turn │ 0° ✔ │ 90° │ 180° │ 270°
//!                  a choice: done at once, the menu stays in the level
//!              ruling <step>      › ──► the four numbers on top
//!              sheet <w×h>        › ──► the columns and the rows on top
//!              background <▇>     › ──► the picker and the swatches on top
//!                  a change shows on the field at once and stays;
//!                  │ ‹, Esc, the way back of android ──► the top, on the entry
//!                  └ put the old one back ──► the value the level was entered
//!                                             with; only once it changed
//!              palettes           ──► the window of the palettes
//!              background picture <none | shown | hidden>
//!                                 ──► the menu of the picture
//!                                     (`crate::ui::backdrop_menu`)
//!              panels           › ──► ‹ panels │ beads › │ tools ›
//!                  ‹ beads │ horizontal line ✔ │ vertical line │ block
//!                  a choice: kept for the posture of the window now
//!              glow <on | off>    ──► on or off, a setting of the program
//!              back to the list   ──► the pattern saved, the list
//!              close the settings
//! a locked pattern: background picture, panels, glow, back to the list,
//! close
//! a ruling, a size or a background left or closed over ──► one step of undo,
//!                                                          none when unchanged
//! ```
//!
//! A level that edits a value shows its editor as the notice of the level
//! (`MenuItem::notice`). The window follows the level the menu stands in
//! (`PlateMenu::level`): it takes the value when the level is entered, and
//! keeps the change as a step when the level is left or the menu closes.

use std::sync::{Arc, Mutex};

use egui::Color32;
use obusina_pattern::{Bounds, DEFAULT_PAPER, Grid, Rgb, Ruling};
use plate_menu::{Detail, MenuItem, Notice};
use rust_i18n::t;

use crate::app::App;
use crate::settings::PanelView;
use crate::ui::access::Named;

/// The most columns or rows from one line of the ruling to the next.
pub const MAX_RULING_STEP: u32 = 100;
/// The farthest a line of the ruling is set from cell (0, 0).
pub const MAX_RULING_OFFSET: i32 = 1000;
/// The turns of the field the menu offers, in degrees clockwise.
pub const TURNS: [u16; 4] = [0, 90, 180, 270];
/// The papers offered under the picker: the one of a new pattern, white,
/// linen and black, each with the key of its name.
const PAPERS: [(Rgb, &str); 4] = [
    (DEFAULT_PAPER, "paper.grey"),
    (Rgb([0xfa, 0xfa, 0xf7]), "paper.white"),
    (Rgb([0xe6, 0xdc, 0xc8]), "paper.linen"),
    (Rgb([0x1e, 0x1e, 0x1e]), "paper.black"),
];
/// The size of a swatch of a paper under the picker.
const SWATCH: egui::Vec2 = egui::vec2(44.0, 30.0);
/// The width of the swatch of the paper right of its entry.
const DETAIL_SWATCH: f32 = 40.0;

/// The levels that edit a value of the pattern.
const RULING: &str = "ruling";
const SHEET: &str = "sheet";
const PAPER: &str = "paper";
/// The entry of each of them that puts the old value back.
const REVERT: &str = "revert";

/// The editors of the levels, made when the menu opens, and the value a
/// level was entered with while the menu stands in it.
pub struct Editing {
    /// The ruling the numbers change.
    ruling: Arc<Mutex<Ruling>>,
    /// The size of the sheet (`src/ui/sheet.rs`); none with no sheet.
    sheet: Option<crate::ui::sheet::Sizing>,
    /// The paper the picker changes.
    paper: Arc<Mutex<Color32>>,
    /// The level stood in and the value it was entered with.
    entered: Option<Entered>,
    /// Whether the entry that puts the value back was offered at the last
    /// filling of the menu.
    revert_offered: bool,
}

/// The value a level was entered with.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Entered {
    Ruling(Ruling),
    Sheet(Bounds),
    Paper(Rgb),
}

impl Entered {
    /// The level of the value.
    fn level(&self) -> &'static str {
        match self {
            Entered::Ruling(_) => RULING,
            Entered::Sheet(_) => SHEET,
            Entered::Paper(_) => PAPER,
        }
    }
}

/// Opens the menu, no entry selected (`RULE ui.menu`).
pub fn open(app: &mut App) {
    app.settings_edit = Some(editors(app));
    let items = items(app);
    if let Err(error) = app.menu.open(items) {
        log::error!("the menu of the settings does not open: {error}");
    }
}

/// The editors of the levels, over the values the pattern has now.
fn editors(app: &App) -> Editing {
    Editing {
        ruling: Arc::new(Mutex::new(app.pattern.ruling())),
        sheet: app
            .pattern
            .sheet()
            .map(|sheet| crate::ui::sheet::Sizing::new(sheet, app.pattern.bounds())),
        paper: Arc::new(Mutex::new(colour(app.pattern.paper()))),
        entered: None,
        revert_offered: false,
    }
}

/// The entries of the menu, as the pattern and the levels are now.
fn items(app: &App) -> Vec<MenuItem> {
    let glow = match app.settings.glow() {
        true => t!("settings.on"),
        false => t!("settings.off"),
    };
    let mut items = match (app.pattern.locked(), &app.settings_edit) {
        (false, Some(editing)) => pattern_items(app, editing),
        _ => Vec::new(),
    };
    items.extend([
        MenuItem::new("backdrop", t!("backdrop.open")).detail(crate::ui::backdrop_menu::state(app)),
        panels_item(app),
        MenuItem::new("glow", t!("view.glow")).detail(glow),
        MenuItem::separator(),
        MenuItem::new("to-list", t!("settings.to_list")),
        MenuItem::new("close", t!("settings.close")),
    ]);
    items
}

/// The entries that change the pattern: none for a locked one.
fn pattern_items(app: &App, editing: &Editing) -> Vec<MenuItem> {
    let grid = app.pattern.grid();
    let kinds = Grid::ALL
        .iter()
        .map(|kind| MenuItem::new(kind_id(*kind), name(*kind)).detail(mark(*kind == grid)))
        .collect();
    let turn = app.pattern.turn();
    let turns = TURNS
        .iter()
        .map(|degrees| {
            MenuItem::new(turn_id(*degrees), turn_name(*degrees)).detail(mark(*degrees == turn))
        })
        .collect();
    let revert = editing.changed(app);
    let [columns, rows] = app.pattern.ruling().step;
    let mut items = vec![
        MenuItem::new("grid", t!("grid.kind"))
            .detail(name(grid))
            .opens_at(kind_id(grid))
            .children(kinds),
        MenuItem::new("turn", t!("turn.open"))
            .detail(turn_name(turn))
            .opens_at(turn_id(turn))
            .children(turns),
        MenuItem::new(RULING, t!("ruling.open"))
            .detail(format!("{columns}\u{d7}{rows}"))
            .notice(numbers(Arc::clone(&editing.ruling)))
            .children(revert_entry(revert, t!("ruling.revert").to_string())),
    ];
    if let (Some(sheet), Some(sizing)) = (app.pattern.sheet(), &editing.sheet) {
        items.push(
            MenuItem::new(SHEET, t!("sheet.open"))
                .detail(crate::ui::sheet::detail(sheet))
                .notice(sizing.numbers())
                .children(revert_entry(revert, t!("sheet.revert").to_string())),
        );
    }
    items.push(
        MenuItem::new(PAPER, t!("paper.open"))
            .detail(swatch(app.pattern.paper()))
            .notice(picker(Arc::clone(&editing.paper)))
            .children(revert_entry(revert, t!("paper.revert").to_string())),
    );
    items.push(MenuItem::new("palettes", t!("palette.settings")));
    items
}

/// The level of the panels: a level for each, its views, the one in use
/// marked.
fn panels_item(app: &App) -> MenuItem {
    let panel = |name: &str, label: String, views: &[PanelView]| {
        let used = app.settings.panel(app.posture, name).map(|(_, view)| view);
        let entries = views
            .iter()
            .map(|view| {
                MenuItem::new(view_id(name, *view), view_name(*view))
                    .detail(mark(used == Some(*view)))
            })
            .collect();
        let item = MenuItem::new(format!("panel-{name}"), label).children(entries);
        match used {
            Some(view) => item.opens_at(view_id(name, view)).detail(view_name(view)),
            None => item,
        }
    };
    MenuItem::new("panels", t!("panels.open")).children(vec![
        panel(
            crate::ui::palette::NAME,
            t!("panel.palette").to_string(),
            &crate::ui::palette::VIEWS,
        ),
        panel(
            crate::ui::tools::NAME,
            t!("panel.tools").to_string(),
            &crate::ui::tools::VIEWS,
        ),
    ])
}

/// The id of the entry of `view` of the panel `name`.
fn view_id(name: &str, view: PanelView) -> String {
    format!("view:{name}:{}", view_key(view))
}

/// The key of a view, in the id of its entry.
fn view_key(view: PanelView) -> &'static str {
    match view {
        PanelView::Row => "row",
        PanelView::Column => "column",
        PanelView::TwoRows => "two_rows",
        PanelView::TwoColumns => "two_columns",
        PanelView::Block => "block",
    }
}

/// The name of a view.
fn view_name(view: PanelView) -> String {
    match view {
        PanelView::Row => t!("panels.row"),
        PanelView::Column => t!("panels.column"),
        PanelView::TwoRows => t!("panels.two_rows"),
        PanelView::TwoColumns => t!("panels.two_columns"),
        PanelView::Block => t!("panels.block"),
    }
    .to_string()
}

/// The panel and the view an entry of a view names.
fn view_of(id: &str) -> Option<(&'static str, PanelView)> {
    let rest = id.strip_prefix("view:")?;
    let (name, key) = rest.split_once(':')?;
    let name = [crate::ui::palette::NAME, crate::ui::tools::NAME]
        .into_iter()
        .find(|known| *known == name)?;
    let view = [
        PanelView::Row,
        PanelView::Column,
        PanelView::TwoRows,
        PanelView::TwoColumns,
        PanelView::Block,
    ]
    .into_iter()
    .find(|view| view_key(*view) == key)?;
    Some((name, view))
}

/// The entry of a level that puts its value back: drawn weak until the value
/// changed. A level shows it even then, so it is a level.
fn revert_entry(changed: bool, label: String) -> Vec<MenuItem> {
    vec![MenuItem::new(REVERT, label).enabled(changed)]
}

/// The mark of the value in use, right of its entry.
fn mark(used: bool) -> &'static str {
    if used { "\u{2714}" } else { "" }
}

/// The swatch of the paper right of its entry.
fn swatch(paper: Rgb) -> Detail {
    let fill = colour(paper);
    Detail::widget(DETAIL_SWATCH, move |ui| {
        let size = egui::vec2(DETAIL_SWATCH, ui.available_height());
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter().rect_filled(rect, 3.0, fill);
        ui.painter()
            .rect_stroke(rect, 3.0, stroke, egui::StrokeKind::Inside);
    })
}

impl Editing {
    /// Whether the value of the level stood in differs from the one it was
    /// entered with.
    fn changed(&self, app: &App) -> bool {
        match self.entered {
            Some(Entered::Ruling(before)) => app.pattern.ruling() != before,
            Some(Entered::Sheet(before)) => app.pattern.sheet() != Some(before),
            Some(Entered::Paper(before)) => app.pattern.paper() != before,
            None => false,
        }
    }
}

/// The numbers of the ruling: the step and the offset, across and down.
fn numbers(editing: Arc<Mutex<Ruling>>) -> Notice {
    Notice::widget(move |ui| {
        let mut ruling = editing.lock().expect("the ruling is never poisoned");
        egui::Grid::new("ruling-numbers").show(ui, |ui| {
            ui.label(t!("ruling.step"));
            for (step, name) in ruling
                .step
                .iter_mut()
                .zip([t!("ruling.step_x"), t!("ruling.step_y")])
            {
                let field = egui::DragValue::new(step).range(0..=MAX_RULING_STEP);
                ui.add(field).named(&name);
            }
            ui.end_row();
            ui.label(t!("ruling.offset"));
            for (offset, name) in ruling
                .offset
                .iter_mut()
                .zip([t!("ruling.offset_x"), t!("ruling.offset_y")])
            {
                let range = -MAX_RULING_OFFSET..=MAX_RULING_OFFSET;
                let field = egui::DragValue::new(offset).range(range);
                ui.add(field).named(&name);
            }
            ui.end_row();
        });
    })
}

/// The picker of the paper, as wide as the menu, and the swatches of
/// [`PAPERS`] under it.
fn picker(editing: Arc<Mutex<Color32>>) -> Notice {
    Notice::widget(move |ui| {
        let mut paper = editing.lock().expect("the paper is never poisoned");
        ui.spacing_mut().slider_width = ui.available_width();
        egui::color_picker::color_picker_color32(ui, &mut paper, egui::color_picker::Alpha::Opaque);
        ui.horizontal(|ui| {
            for (rgb, key) in PAPERS {
                let fill = colour(rgb);
                let name = t!(key);
                let button = obusina_button::SquareButton::new("", SWATCH)
                    .fill(fill)
                    .selected(*paper == fill)
                    .name(&name);
                if ui.add(button).clicked() {
                    *paper = fill;
                }
            }
        });
    })
}

/// Draws the menu while it is open, and does what is chosen. While nothing
/// else of the window answers (`src/ui/blocker.rs`), the ruling, the size
/// and the paper show on the field as they change.
pub fn show(app: &mut App, context: &egui::Context) {
    app.menu.nothing_found(t!("menu.nothing_found"));
    if app.menu.is_open() {
        crate::ui::blocker::under(context, "settings-menu", &app.menu);
    }
    let chosen = app.menu.show(context);
    if let Some(chosen) = chosen {
        choose(app, context, &chosen.id);
    }
    follow(app);
}

/// Does what an entry chosen does; the menu closed on it, and opens again
/// where it stood when it is to stay.
fn choose(app: &mut App, context: &egui::Context, id: &str) {
    match id {
        REVERT => revert(app),
        "glow" => {
            app.keep_glow(!app.settings.glow());
            reopen_at(app, "glow");
        }
        "palettes" => app.palette_window.open = true,
        "backdrop" => crate::ui::backdrop_menu::open(app),
        "to-list" => {
            leave(app);
            let time = context.input(|input| input.time);
            app.back_to_list(context, time);
        }
        "close" => {}
        view if view_of(view).is_some() => {
            if let Some((name, chosen)) = view_of(view) {
                crate::ui::panels::set_view(app, context, name, chosen);
            }
            reopen_at(app, view);
        }
        kind if Grid::ALL.iter().any(|grid| kind_id(*grid) == kind) => {
            if let Some(grid) = Grid::ALL.into_iter().find(|grid| kind_id(*grid) == kind) {
                app.set_grid(grid);
            }
            reopen_at(app, kind);
        }
        turn => {
            if let Some(degrees) = TURNS.into_iter().find(|degrees| turn_id(*degrees) == turn) {
                app.turn_to(degrees);
                reopen_at(app, turn);
            }
        }
    }
}

/// Opens the menu again on the entry `id`, in its level.
fn reopen_at(app: &mut App, id: &str) {
    let items = items(app);
    if let Err(error) = app.menu.open_at(items, id) {
        log::error!("the menu of the settings does not open again: {error}");
    }
}

/// Puts back the value the level was entered with, and opens the menu again
/// in that level.
fn revert(app: &mut App) {
    let Some(editing) = &app.settings_edit else {
        return;
    };
    let Some(entered) = editing.entered else {
        return;
    };
    match entered {
        Entered::Ruling(before) => {
            *editing.ruling.lock().expect("the ruling is never poisoned") = before;
            app.pattern.set_ruling(before);
        }
        Entered::Sheet(before) => {
            if let Some(sizing) = &editing.sheet {
                *sizing.size.lock().expect("the size is never poisoned") =
                    [before.width(), before.height()];
            }
            app.pattern.set_sheet(Some(before));
        }
        Entered::Paper(before) => {
            *editing.paper.lock().expect("the paper is never poisoned") = colour(before);
            app.pattern.set_paper(before);
        }
    }
    let items = items(app);
    if let Err(error) = app.menu.open_in(items, entered.level()) {
        log::error!("the menu of the settings does not open again: {error}");
    }
}

/// Follows the level the menu stands in: a level left, or the menu closed,
/// keeps its change as a step; a level entered takes its value; the value
/// of the level stood in shows on the field. The entry that puts the value
/// back is offered once the value changed.
fn follow(app: &mut App) {
    let level = app.menu.level();
    let standing = level.first().map(String::as_str);
    let entered = app
        .settings_edit
        .as_ref()
        .and_then(|editing| editing.entered);
    if let Some(entered) = entered
        && standing != Some(entered.level())
    {
        leave(app);
    }
    if !app.menu.is_open() {
        app.settings_edit = None;
        return;
    }
    if let Some(standing) = standing {
        enter(app, standing);
    }
    show_on_field(app);
    let Some(editing) = &app.settings_edit else {
        return;
    };
    let changed = editing.changed(app);
    if changed != editing.revert_offered {
        if let Some(editing) = &mut app.settings_edit {
            editing.revert_offered = changed;
        }
        let items = items(app);
        if let Err(error) = app.menu.refill(items) {
            log::error!("the menu of the settings is not filled again: {error}");
        }
    }
}

/// Takes the value of the level `level` when it is entered.
fn enter(app: &mut App, level: &str) {
    let ruling = app.pattern.ruling();
    let sheet = app.pattern.sheet();
    let paper = app.pattern.paper();
    let Some(editing) = &mut app.settings_edit else {
        return;
    };
    if editing.entered.is_some() {
        return;
    }
    editing.entered = match level {
        RULING => {
            *editing.ruling.lock().expect("the ruling is never poisoned") = ruling;
            Some(Entered::Ruling(ruling))
        }
        SHEET => sheet.map(|sheet| {
            editing.sheet = Some(crate::ui::sheet::Sizing::new(sheet, app.pattern.bounds()));
            Entered::Sheet(sheet)
        }),
        PAPER => {
            *editing.paper.lock().expect("the paper is never poisoned") = colour(paper);
            Some(Entered::Paper(paper))
        }
        _ => None,
    };
    editing.revert_offered = false;
    if editing.entered.is_some() && level == SHEET {
        // The level shows the numbers of the new sizing.
        let items = items(app);
        if let Err(error) = app.menu.refill(items) {
            log::error!("the menu of the settings is not filled again: {error}");
        }
    }
}

/// Puts the value of the editor of the level stood in on the pattern, so
/// it shows on the field.
fn show_on_field(app: &mut App) {
    let Some(editing) = &app.settings_edit else {
        return;
    };
    match editing.entered {
        Some(Entered::Ruling(_)) => {
            let shown = *editing.ruling.lock().expect("the ruling is never poisoned");
            app.pattern.set_ruling(shown);
        }
        Some(Entered::Sheet(_)) => {
            if let Some(sizing) = &editing.sheet {
                let shown = sizing.sheet();
                app.pattern.set_sheet(Some(shown));
            }
        }
        Some(Entered::Paper(_)) => {
            let shown = *editing.paper.lock().expect("the paper is never poisoned");
            app.pattern.set_paper(rgb(shown));
        }
        None => {}
    }
}

/// Leaves the level stood in: its change is kept as one step of undo, none
/// when the value is the one it was entered with.
fn leave(app: &mut App) {
    let Some(entered) = app
        .settings_edit
        .as_mut()
        .and_then(|editing| editing.entered.take())
    else {
        return;
    };
    match entered {
        Entered::Ruling(before) => app.history.keep_ruling(&app.pattern, before),
        Entered::Sheet(before) => app.history.keep_sheet(&app.pattern, before),
        Entered::Paper(before) => app.history.keep_paper(&app.pattern, before),
    }
    if app.menu.is_open() {
        let items = items(app);
        if let Err(error) = app.menu.refill(items) {
            log::error!("the menu of the settings is not filled again: {error}");
        }
    }
}

/// The identifier of the entry of a kind.
fn kind_id(grid: Grid) -> &'static str {
    match grid {
        Grid::Loom => "loom",
        Grid::Peyote => "peyote",
    }
}

/// The name of a kind of grid.
fn name(grid: Grid) -> String {
    match grid {
        Grid::Loom => t!("grid.loom"),
        Grid::Peyote => t!("grid.peyote"),
    }
    .to_string()
}

/// The identifier of the entry of a turn.
fn turn_id(degrees: u16) -> String {
    format!("turn-{degrees}")
}

/// The name of a turn: none, or its degrees.
fn turn_name(degrees: u16) -> String {
    match degrees {
        0 => t!("turn.none").to_string(),
        degrees => format!("{degrees}\u{b0}"),
    }
}

/// A colour of the pattern as the toolkit draws it.
fn colour(Rgb([red, green, blue]): Rgb) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// A colour of the toolkit as the pattern keeps it.
fn rgb(colour: Color32) -> Rgb {
    Rgb([colour.r(), colour.g(), colour.b()])
}
