//! The window of the palettes: a modal with no title. Tabs on top show the
//! places as a line or as a block; `<` and `>`, or `↔` with two palettes,
//! and the keys "+" and "-", show the next palette in a circle. A tap on a place opens its menu, the
//! picker on top (`docs/palette.md`).
//!
//! ```text
//! tap a place ──► the picker shows the colour in the palette and on the field;
//!                 an empty place shows the colour farthest from the others
//!                    │──accept────────► one step of the history
//!                    │──move──────────► one palette: a new one; more: a menu of them
//!                    │──remove────────► with beads its place is hatched; with none, empty
//!                    └──delete beads──► the beads of a removed colour go, one step
//!      closed with no entry, or by Escape ──► the palettes as they were
//! drag a colour ──► the place under the pointer is lit ──let go──► the two places swap
//! ```

use std::sync::{Arc, Mutex};

use egui::{Color32, Id, Key, Modal, Order, Pos2, Rect, Sense, vec2};
use obusina_pattern::{Colour, Palette, Pattern, Rgb};
use plate_menu::{MenuItem, Notice, PlateMenu};
use rust_i18n::t;

use crate::app::App;
use crate::settings::PaletteView;
use crate::ui::access::Named;
use crate::ui::{button, icons};

/// The places in a row of the block, as on a numpad.
const BLOCK_WIDTH: usize = 3;
/// The swatch in the notice of the menu of a colour.
const NOTICE_SWATCH: egui::Vec2 = vec2(28.0, 20.0);

/// What the menu of the window is open for.
#[derive(Debug, Clone)]
enum MenuOf {
    /// A place, its colour shown by the picker; the palettes before it.
    Place { place: usize, before: Vec<Palette> },
    /// The place whose colour goes to another palette.
    Target { place: usize },
}

/// The state of the window of the palettes.
#[derive(Default)]
pub struct PaletteWindow {
    /// True while the window is open.
    pub open: bool,
    menu: PlateMenu,
    menu_of: Option<MenuOf>,
    /// The colour the picker in the notice changes.
    editing: Arc<Mutex<Color32>>,
    /// Each place shown and where it stood in the last frame.
    cells: Vec<(usize, Rect)>,
    /// The place of the colour being dragged.
    dragging: Option<usize>,
    /// The place it would stand in, let go now.
    drop_to: usize,
}

#[cfg(test)]
impl PaletteWindow {
    /// The layer of its menu.
    pub fn menu_layer(&self) -> egui::LayerId {
        self.menu.layer_id()
    }
}

/// The places the line shows: every place up to the last one that holds
/// anything, then the first after it, the "+" of all the places left. A full
/// palette has no such place.
fn line(held: &[bool]) -> Vec<usize> {
    let end = held
        .iter()
        .rposition(|held| *held)
        .map_or(0, |last| last + 1);
    (0..held.len().min(end + 1)).collect()
}

/// The places of a block, by rows from the top: the first places stand at
/// the bottom, as on a numpad.
pub fn block(places: usize) -> Vec<Vec<usize>> {
    let all: Vec<usize> = (0..places).collect();
    all.chunks(BLOCK_WIDTH)
        .rev()
        .map(<[usize]>::to_vec)
        .collect()
}

/// The palette `step` away from `active`, in a circle of `count`.
pub fn in_circle(active: usize, step: isize, count: usize) -> usize {
    (active as isize + step).rem_euclid(count.max(1) as isize) as usize
}

/// `Rgb` as a colour of egui.
pub fn color32(Rgb([red, green, blue]): Rgb) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// A colour of egui as `Rgb`.
fn rgb(colour: Color32) -> Rgb {
    Rgb([colour.r(), colour.g(), colour.b()])
}

/// The colour in a place of the palette shown.
fn colour_at(app: &App, place: usize) -> Option<&Colour> {
    app.pattern.palettes()[app.palette].places()[place].as_ref()
}

/// The form of a word after a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Form {
    One,
    Few,
    Many,
}

/// The form a word takes after `count`, by the rules of Russian or of
/// English.
fn form(count: usize, russian: bool) -> Form {
    let (ones, hundreds) = (count % 10, count % 100);
    if !russian {
        return if count == 1 { Form::One } else { Form::Many };
    }
    if ones == 1 && hundreds != 11 {
        Form::One
    } else if (2..=4).contains(&ones) && !(12..=14).contains(&hundreds) {
        Form::Few
    } else {
        Form::Many
    }
}

/// The entry that deletes `count` beads, its word fitted to the number.
pub fn delete_beads(count: usize) -> String {
    match form(count, rust_i18n::locale().starts_with("ru")) {
        Form::One => t!("palette.delete_beads_one", count = count),
        Form::Few => t!("palette.delete_beads_few", count = count),
        Form::Many => t!("palette.delete_beads_many", count = count),
    }
    .to_string()
}

/// Shows the window while it is open, and its menu.
pub fn show(app: &mut App, context: &egui::Context) {
    app.palette = app.palette.min(app.pattern.palettes().len() - 1);
    if !app.palette_window.open {
        return;
    }
    let escape =
        |input: &mut egui::InputState| input.consume_key(egui::Modifiers::NONE, Key::Escape);
    if app.palette_window.menu.is_open() && context.input_mut(escape) {
        app.palette_window.menu.close();
    }
    let modal = Modal::new(Id::unique("palette-window")).show(context, |ui| window(app, ui));
    modal.response.clone().named(&t!("palette.window"));
    if modal.should_close() && !app.palette_window.menu.is_open() {
        app.palette_window.open = false;
    }
    menu(app, context);
}

/// The content of the window.
fn window(app: &mut App, ui: &mut egui::Ui) {
    keys(app, ui);
    ui.horizontal(|ui| {
        let mut view = app.settings.palette_view();
        ui.selectable_value(&mut view, PaletteView::Line, t!("palette.line"));
        ui.selectable_value(&mut view, PaletteView::Block, t!("palette.block"));
        app.keep_palette_view(view);
        ui.separator();
        switch(app, ui);
    });
    ui.separator();
    let held: Vec<bool> = app.pattern.palettes()[app.palette]
        .places()
        .iter()
        .map(Option::is_some)
        .collect();
    let rows = match app.settings.palette_view() {
        PaletteView::Line => vec![line(&held)],
        PaletteView::Block => block(held.len()),
    };
    let mut tapped = None;
    let mut drags = Vec::new();
    app.palette_window.cells.clear();
    ui.vertical(|ui| {
        for row in rows {
            ui.horizontal(|ui| {
                for place in row {
                    cell(app, ui, place, &mut tapped, &mut drags);
                }
            });
        }
    });
    drag(app, ui, &drags);
    if let Some(place) = tapped {
        tap(app, place);
    }
}

/// "+" and "-" of the keyboard or the numpad, while no menu is open and no
/// colour is dragged.
fn keys(app: &mut App, ui: &egui::Ui) {
    let state = &app.palette_window;
    if state.menu.is_open() || state.dragging.is_some() {
        return;
    }
    step_by_keys(app, ui);
}

/// "+" and "-" show the next and the previous palette, in a circle. `=` is
/// the key of "+" with no shift.
pub fn step_by_keys(app: &mut App, ui: &egui::Ui) {
    let (next, previous) = ui.input(|input| {
        (
            input.key_pressed(Key::Plus) || input.key_pressed(Key::Equals),
            input.key_pressed(Key::Minus),
        )
    });
    if next {
        step(app, 1);
    }
    if previous {
        step(app, -1);
    }
}

/// Shows the palette `by` away, in a circle.
pub fn step(app: &mut App, by: isize) {
    app.palette = in_circle(app.palette, by, app.pattern.palettes().len());
}

/// The switch of the palette, the way to it by touch (`RULE ui.touch`): with
/// two palettes one button goes to the other, with more `<` and `>`.
fn switch(app: &mut App, ui: &mut egui::Ui) {
    let count = app.pattern.palettes().len();
    if count == 2 {
        let both = ui.button(icons::SWITCH);
        if both.named(&t!("palette.switch")).clicked() {
            step(app, 1);
        }
    } else {
        let previous = ui.add_enabled(count > 1, egui::Button::new("<"));
        if previous.named(&t!("palette.previous")).clicked() {
            step(app, -1);
        }
    }
    ui.label(t!(
        "palette.number",
        number = app.palette + 1,
        count = count
    ));
    if count != 2 {
        let next = ui.add_enabled(count > 1, egui::Button::new(">"));
        if next.named(&t!("palette.next")).clicked() {
            step(app, 1);
        }
    }
}

/// What a place shows: while a colour is dragged, it and the place under
/// the pointer have changed places already.
fn shown(app: &App, place: usize) -> Option<&Colour> {
    let state = &app.palette_window;
    let from = match state.dragging {
        Some(from) if place == from => state.drop_to,
        Some(from) if place == state.drop_to => from,
        _ => place,
    };
    colour_at(app, from)
}

/// One place: a colour with its number, a removed or hidden colour hatched
/// with its number, or a "+".
fn cell(
    app: &mut App,
    ui: &mut egui::Ui,
    place: usize,
    tapped: &mut Option<usize>,
    drags: &mut Vec<(usize, egui::Response)>,
) {
    let number = (place + 1).to_string();
    let state = &app.palette_window;
    let lit = state.dragging.is_some() && place == state.drop_to;
    let swatch = match shown(app, place) {
        Some(colour) => match colour.seen() {
            Some(seen) => button::swatch(ui, color32(seen), &number, lit, &number),
            None => button::hatched(ui, &number, lit, &number),
        },
        None => button::toggle(ui, lit, "+", "+"),
    };
    app.palette_window.cells.push((place, swatch.rect));
    let name = match colour_at(app, place) {
        Some(colour) if colour.rgb.is_none() => t!("palette.removed", number = number),
        Some(colour) if colour.hidden => t!("palette.hidden_colour", number = number),
        Some(_) => t!("palette.colour", number = number),
        None => t!("palette.place", number = number),
    };
    let held = colour_at(app, place).is_some().then(|| {
        ui.interact(
            swatch.rect,
            Id::unique(("palette-window-drag", app.palette, place)),
            Sense::click_and_drag(),
        )
    });
    let swatch = match &held {
        Some(held) => {
            held.clone().named(&name);
            swatch
        }
        None => swatch.named(&name),
    };
    if swatch.clicked() || held.as_ref().is_some_and(egui::Response::clicked) {
        *tapped = Some(place);
    }
    drags.extend(held.map(|held| (place, held)));
}

/// A drag of a colour: it rises above the pointer, the place under the
/// pointer is lit and shows it; let go, the two places swap as one step.
fn drag(app: &mut App, ui: &egui::Ui, drags: &[(usize, egui::Response)]) {
    if let Some((place, _)) = drags.iter().find(|(_, drag)| drag.drag_started()) {
        app.palette_window.dragging = Some(*place);
        app.palette_window.drop_to = *place;
    }
    let Some(from) = app.palette_window.dragging else {
        return;
    };
    if let Some(pointer) = ui.ctx().pointer_interact_pos() {
        if let Some(place) = place_at(&app.palette_window.cells, pointer)
            && place != app.palette_window.drop_to
        {
            app.palette_window.drop_to = place;
            ui.ctx().request_repaint();
        }
        ghost(app, ui, from, pointer);
    }
    if drags.iter().any(|(_, drag)| drag.drag_stopped()) {
        app.palette_window.dragging = None;
        let (palette, to) = (app.palette, app.palette_window.drop_to);
        if to != from {
            let swapped = app.history.change_colours(&mut app.pattern, |pattern| {
                pattern.swap_places(palette, from, to)
            });
            if let Err(error) = swapped {
                log::error!("the places do not swap: {error}");
            }
        }
    }
}

/// The dragged colour above the pointer, with the number of the place it
/// would take.
fn ghost(app: &App, ui: &egui::Ui, from: usize, pointer: Pos2) {
    let Some(seen) = colour_at(app, from).and_then(Colour::seen) else {
        return;
    };
    let fill = color32(seen);
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        Order::Tooltip,
        Id::unique("palette-ghost"),
    ));
    let ghost = Rect::from_center_size(pointer - vec2(0.0, button::SIZE.y), button::SIZE);
    painter.add(ui.visuals().popup_shadow.as_shape(ghost, 6.0));
    painter.rect_filled(ghost, 6.0, fill);
    painter.text(
        ghost.center(),
        egui::Align2::CENTER_CENTER,
        (app.palette_window.drop_to + 1).to_string(),
        egui::FontId::proportional(button::TEXT),
        obusina_button::contrast(fill),
    );
}

/// The place of the cell nearest to a pointer.
fn place_at(cells: &[(usize, Rect)], pointer: Pos2) -> Option<usize> {
    cells
        .iter()
        .min_by(|(_, first), (_, second)| {
            first
                .center()
                .distance(pointer)
                .total_cmp(&second.center().distance(pointer))
        })
        .map(|(place, _)| *place)
}

/// A tap on a place: its menu opens, the picker on top. An empty place shows
/// a new colour, a removed one a colour again for its beads. Nothing stays
/// until "Accept".
fn tap(app: &mut App, place: usize) {
    let before = app.pattern.palettes().to_vec();
    let colour = colour_at(app, place).cloned();
    let shown = match colour.as_ref().and_then(|colour| colour.rgb) {
        Some(rgb) => rgb,
        None => new_colour(&app.pattern),
    };
    let id = match app.pattern.put_colour(app.palette, place, shown) {
        Ok(id) => id,
        Err(error) => {
            log::error!("the place does not take a colour: {error}");
            return;
        }
    };
    let beads = app.pattern.count(id);
    let last = match &colour {
        Some(colour) if colour.rgb.is_none() && beads > 0 => {
            MenuItem::new("beads", delete_beads(beads))
        }
        Some(_) => MenuItem::new("remove", t!("palette.remove")),
        None => MenuItem::new("remove", t!("palette.remove")).enabled(false),
    };
    let items = vec![
        MenuItem::new("accept", t!("palette.accept")),
        MenuItem::new("move", t!("palette.move")).enabled(colour.is_some()),
        last,
    ];
    let state = &mut app.palette_window;
    if let Err(error) = state.menu.open(items) {
        log::error!("the menu of a colour does not open: {error}");
        put_back(&mut app.pattern, before);
        return;
    }
    *state.editing.lock().expect("the colour is never poisoned") = color32(shown);
    state
        .menu
        .notice(picker(Arc::clone(&state.editing), place + 1, beads));
    state.menu_of = Some(MenuOf::Place { place, before });
}

/// The picker in the notice: the swatch, the number, `#rrggbb` and the beads
/// over it, which change as it changes.
fn picker(editing: Arc<Mutex<Color32>>, number: usize, beads: usize) -> Notice {
    Notice::widget(move |ui| {
        let mut colour = editing.lock().expect("the colour is never poisoned");
        ui.horizontal(|ui| {
            let (swatch, _) = ui.allocate_exact_size(NOTICE_SWATCH, Sense::hover());
            ui.painter().rect_filled(swatch, 4.0, *colour);
            ui.painter().rect_stroke(
                swatch,
                4.0,
                ui.visuals().widgets.noninteractive.bg_stroke,
                egui::StrokeKind::Inside,
            );
            let title = t!(
                "palette.title",
                number = number,
                rgb = rgb(*colour),
                count = beads
            );
            ui.label(egui::RichText::new(title).strong());
        });
        ui.spacing_mut().slider_width = ui.available_width();
        egui::color_picker::color_picker_color32(
            ui,
            &mut colour,
            egui::color_picker::Alpha::Opaque,
        );
    })
}

/// The colour farthest from those of every palette and from the paper.
fn new_colour(pattern: &Pattern) -> Rgb {
    let taken: Vec<Rgb> = pattern
        .palettes()
        .iter()
        .flat_map(Palette::places)
        .flatten()
        .filter_map(|colour| colour.rgb)
        .collect();
    crate::new_colour::next_colour(&taken, rgb(obusina_canvas::PAPER))
}

/// Puts back palettes kept before a menu opened.
fn put_back(pattern: &mut Pattern, palettes: Vec<Palette>) {
    if let Err(error) = pattern.set_palettes(palettes) {
        log::error!("the palettes are not put back: {error}");
    }
}

/// The menu of the window, while it is open.
fn menu(app: &mut App, context: &egui::Context) {
    app.palette_window
        .menu
        .nothing_found(t!("menu.nothing_found"));
    if app.palette_window.menu.is_open() {
        crate::ui::blocker::under(context, "palette-window", &mut app.palette_window.menu);
    }
    let chosen = app.palette_window.menu.show(context);
    if let Some(MenuOf::Place { place, .. }) = app.palette_window.menu_of {
        let picked = *app
            .palette_window
            .editing
            .lock()
            .expect("the colour is never poisoned");
        let now = colour_at(app, place).and_then(|colour| colour.rgb);
        if now != Some(rgb(picked))
            && let Err(error) = app.pattern.put_colour(app.palette, place, rgb(picked))
        {
            log::error!("the picked colour is not shown: {error}");
        }
    }
    let Some(chosen) = chosen else {
        if !app.palette_window.menu.is_open()
            && let Some(MenuOf::Place { before, .. }) = app.palette_window.menu_of.take()
        {
            put_back(&mut app.pattern, before);
        }
        if !app.palette_window.menu.is_open() {
            app.palette_window.menu_of = None;
        }
        return;
    };
    let palette = app.palette;
    match app.palette_window.menu_of.take() {
        Some(MenuOf::Place { place, before }) => match chosen.id.as_str() {
            "accept" => app.history.keep_palettes(&app.pattern, before),
            "move" => {
                put_back(&mut app.pattern, before);
                ask_where(app, place);
            }
            "remove" => {
                put_back(&mut app.pattern, before);
                change(app, |pattern| pattern.remove_colour(palette, place));
            }
            "beads" => {
                put_back(&mut app.pattern, before);
                if let Some(id) = colour_at(app, place).map(|colour| colour.id) {
                    app.history.delete_beads(&mut app.pattern, id);
                }
            }
            _ => put_back(&mut app.pattern, before),
        },
        Some(MenuOf::Target { place }) => {
            let target = chosen
                .id
                .strip_prefix("to-")
                .and_then(|target| target.parse().ok());
            change(app, |pattern| pattern.move_colour(palette, place, target));
        }
        None => {}
    }
}

/// A change of the colours as one step; a failure is logged.
fn change(app: &mut App, change: impl FnOnce(&mut Pattern) -> obusina_pattern::Result<()>) {
    if let Err(error) = app.history.change_colours(&mut app.pattern, change) {
        log::error!("the colours do not change: {error}");
    }
    app.palette = app.palette.min(app.pattern.palettes().len() - 1);
}

/// "Move to another palette": with one palette, a new one is made; with
/// more, a menu names them, a full one off, and a new one.
fn ask_where(app: &mut App, place: usize) {
    let palette = app.palette;
    if app.pattern.palettes().len() == 1 {
        change(app, |pattern| pattern.move_colour(palette, place, None));
        return;
    }
    let mut items: Vec<MenuItem> = app
        .pattern
        .palettes()
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != palette)
        .map(|(index, found)| {
            MenuItem::new(format!("to-{index}"), t!("palette.to", number = index + 1))
                .enabled(found.first_free().is_some())
        })
        .collect();
    items.push(MenuItem::new("to-new", t!("palette.new")));
    let state = &mut app.palette_window;
    if let Err(error) = state.menu.open(items) {
        log::error!("the menu of the palettes does not open: {error}");
        return;
    }
    state.menu.notice(t!("palette.where", number = place + 1));
    state.menu_of = Some(MenuOf::Target { place });
}

/// The id of the colour in hand stays one the pattern has.
pub fn keep_in_hand(app: &mut App) {
    if app.colour.is_some_and(|id| {
        app.pattern
            .colour(id)
            .and_then(|colour| colour.rgb)
            .is_none()
    }) {
        app.colour = None;
    }
}

#[cfg(test)]
mod tests {
    use obusina_pattern::MAX_COLOURS;

    use super::*;

    #[test]
    fn the_line_ends_with_one_place_for_the_rest() {
        assert_eq!(line(&[false, false, false]), vec![0]);
        assert_eq!(line(&[true, true, false, false]), vec![0, 1, 2]);
        assert_eq!(line(&[true, false, true, false]), vec![0, 1, 2, 3]);
        assert_eq!(line(&[true, true, true]), vec![0, 1, 2]);
    }

    #[test]
    fn the_block_stands_as_a_numpad() {
        assert_eq!(
            block(MAX_COLOURS),
            vec![vec![6, 7, 8], vec![3, 4, 5], vec![0, 1, 2]]
        );
        assert_eq!(block(4), vec![vec![3], vec![0, 1, 2]]);
    }

    #[test]
    fn the_palettes_make_a_circle() {
        assert_eq!(in_circle(0, 1, 3), 1);
        assert_eq!(in_circle(2, 1, 3), 0);
        assert_eq!(in_circle(0, -1, 3), 2);
        assert_eq!(in_circle(0, 1, 1), 0);
    }

    #[test]
    fn the_word_for_the_beads_fits_the_number() {
        assert_eq!(form(1, true), Form::One);
        assert_eq!(form(3, true), Form::Few);
        assert_eq!(form(11, true), Form::Many);
        assert_eq!(form(12, true), Form::Many);
        assert_eq!(form(21, true), Form::One);
        assert_eq!(form(25, true), Form::Many);
        assert_eq!(form(1, false), Form::One);
        assert_eq!(form(21, false), Form::Many);
    }
}
