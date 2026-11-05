//! The window that fills the palettes of a pattern: a modal with no title,
//! with tabs on top for the two ways a palette is shown, a line and a block,
//! and the switch of the palette. "+" and "-" switch it too, in a circle.
//!
//! A pattern has several palettes, and a palette is a row of places. A colour
//! has an id, which is unique in the pattern, and a bead holds that id. So a
//! colour moves between places and palettes and its beads never change. A
//! removed colour with beads keeps its id and its place, with no rgb: the
//! place and the beads are hatched. A tap on that place offers its colour
//! again, or the delete of its beads, which leaves the place hatched. A hidden
//! colour keeps its rgb, and is hatched the same. A palette with nothing left
//! in its places goes, unless it is the only one.
//!
//! ```text
//! tap a colour ──► PlateMenu: the picker of the colour on top, then
//!                     │──accept──────────► the colour stays as picked
//!                     │──move to another─► one palette: a new one;
//!                     │                    more: a PlateMenu of them and a new one
//!                     └──remove──────────► its place and its beads hatched
//! tap a hatched place ──► the same, with "delete <n> beads" for "remove"
//!       closed with no entry ──► the colour goes back to what it was
//! drag a colour ──► the place under the pointer is lit and shows it ──let go──► it stays
//! tap "+" ──► the colour farthest from the others and the paper, its menu open
//! ```

use std::sync::{Arc, Mutex};

use egui::{Color32, Id, Key, Modal, Order, Pos2, Rect, Sense, vec2};
use palette::{FromColor, Oklab, Oklch, Srgb};
use plate_menu::{MenuItem, Notice, PlateMenu};

use super::{BUTTON, Lang, Mockup, button, delete_beads, named, named_rect};

/// The places of a palette, `MAX_COLOURS` of the plan.
const PLACES: usize = 9;
/// The ground the beads lie on, `PAPER` of the canvas: a new colour is
/// told from it too.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The lightness, chroma and hue steps a new colour is chosen among, in
/// OKLCh.
const LIGHTNESS: [f32; 5] = [0.25, 0.4, 0.55, 0.7, 0.85];
const CHROMA: [f32; 3] = [0.0, 0.08, 0.16];
const HUE_STEP: f32 = 30.0;
/// The swatch in the notice of the menu of a colour.
const NOTICE_SWATCH: egui::Vec2 = vec2(28.0, 20.0);
/// The places in a row of the block, as on a numpad.
const BLOCK_WIDTH: usize = 3;
/// The made up field under the line: its columns and rows.
const FIELD_SIZE: (usize, usize) = (14, 5);
/// The side of a bead of the made up field.
const FIELD_BEAD: f32 = 16.0;
/// The space between two lines of the hatch of a bead.
const HATCH_STEP: f32 = 5.0;

/// The room kept clear of the hatch around the number of a swatch.
const NUMBER_ROOM: egui::Vec2 = vec2(18.0, 22.0);

/// How a palette is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Line,
    Block,
}

/// A colour in a place. With no `rgb` it was removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Colour {
    id: u32,
    rgb: Option<Color32>,
    hidden: bool,
}

impl Colour {
    /// What it is drawn with: nothing means a hatch.
    fn seen(self) -> Option<Color32> {
        self.rgb.filter(|_| !self.hidden)
    }
}

/// The places of a block, by rows from the top: the first places stand at
/// the bottom, as on a numpad.
fn block(places: usize) -> Vec<Vec<usize>> {
    let all: Vec<usize> = (0..places).collect();
    all.chunks(BLOCK_WIDTH)
        .rev()
        .map(<[usize]>::to_vec)
        .collect()
}

/// The places the line shows: every place up to the last colour, then the
/// first place after it, which is the "+" of all the places left. A full
/// palette has no such place. Nothing here knows how many places there are.
fn line(coloured: &[bool]) -> Vec<usize> {
    let end = coloured
        .iter()
        .rposition(|colour| *colour)
        .map_or(0, |last| last + 1);
    (0..coloured.len().min(end + 1)).collect()
}

/// The palette `step` away from `active`, in a circle of `count`.
fn in_circle(active: usize, step: isize, count: usize) -> usize {
    (active as isize + step).rem_euclid(count as isize) as usize
}

/// The palettes and whether their window is open.
pub struct PaletteEditor {
    lang: Lang,
    view: View,
    palettes: Vec<Vec<Option<Colour>>>,
    /// The palette shown.
    active: usize,
    /// The id the next colour takes.
    next_id: u32,
    /// The id of each bead of the made up field, by rows.
    field: Vec<Vec<Option<u32>>>,
    open: bool,
    menu: PlateMenu,
    /// The place whose colour is being picked.
    menu_of: Option<usize>,
    /// What that place held before the menu opened.
    before: Option<Colour>,
    /// The place whose colour is going to another palette.
    moving: Option<usize>,
    /// The colour the picker in the notice of the menu changes; it goes
    /// to the palette in every frame the menu is open, to be seen.
    editing: Arc<Mutex<Color32>>,
    /// Each place shown and where it stood in the last frame.
    cells: Vec<(usize, Rect)>,
    /// Where the window stood in the last frame.
    window: Rect,
    /// The place of the colour being dragged.
    dragging: Option<usize>,
    /// The place it would stand in, let go now.
    drop_to: usize,
}

impl PaletteEditor {
    /// The mockup with one palette of five colours, the fifth hidden.
    pub fn new() -> PaletteEditor {
        let mut places: Vec<Option<Colour>> = [
            [0x1b, 0x1b, 0x1b],
            [0xf2, 0xef, 0xe6],
            [0xc0, 0x30, 0x30],
            [0x2f, 0x5f, 0xb3],
            [0xe8, 0xc5, 0x47],
        ]
        .iter()
        .zip(1..)
        .map(|([red, green, blue], id)| {
            Some(Colour {
                id,
                rgb: Some(Color32::from_rgb(*red, *green, *blue)),
                hidden: id == 5,
            })
        })
        .collect();
        let colours = places.len();
        places.resize(PLACES, None);
        let (columns, rows) = FIELD_SIZE;
        let field = (0..rows)
            .map(|row| {
                (0..columns)
                    .map(|column| {
                        (column % 7 != 6).then_some(((column + row) % colours) as u32 + 1)
                    })
                    .collect()
            })
            .collect();
        PaletteEditor {
            lang: Lang::Ru,
            view: View::Line,
            palettes: vec![places],
            active: 0,
            next_id: colours as u32 + 1,
            field,
            open: true,
            menu: PlateMenu::new().with_id(Id::unique("palette-editor-menu")),
            menu_of: None,
            before: None,
            moving: None,
            editing: Arc::new(Mutex::new(Color32::BLACK)),
            cells: Vec::new(),
            window: Rect::NOTHING,
            dragging: None,
            drop_to: 0,
        }
    }
}

impl Mockup for PaletteEditor {
    fn name(&self) -> &'static str {
        "palette"
    }

    fn state(&self) -> serde_json::Value {
        let palettes: Vec<Vec<serde_json::Value>> = self
            .palettes
            .iter()
            .map(|places| {
                places
                    .iter()
                    .map(|place| {
                        place.map_or(serde_json::Value::Null, |colour| {
                            serde_json::json!({ "id": colour.id, "rgb": colour.rgb.map(hex) })
                        })
                    })
                    .collect()
            })
            .collect();
        serde_json::json!({
            "view": format!("{:?}", self.view),
            "active": self.active,
            "palettes": palettes,
            "dragging": self.dragging,
            "drop_to": self.drop_to,
            "open": self.open,
            "lang": self.lang.code(),
        })
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        self.lang = lang;
        if ui
            .button(lang.tr("Open the palette", "Открыть палитру"))
            .clicked()
        {
            self.open = true;
        }
        if !self.open {
            return;
        }
        let modal = Modal::new(Id::unique("palette-editor")).show(ui.ctx(), |ui| self.window(ui));
        self.window = modal.response.rect;
        named(ui.ctx(), modal.response.clone(), "palette window");
        if modal.should_close() && !self.menu.is_open() {
            self.open = false;
        }
        self.menu(ui.ctx());
    }
}

impl PaletteEditor {
    /// The places of the palette shown.
    fn places(&self) -> &[Option<Colour>] {
        &self.palettes[self.active]
    }

    /// The content of the window: the tabs, the switch of the palette, the
    /// places, the made up field.
    fn window(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        self.keys(ui);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.view, View::Line, lang.tr("Line", "Линия"));
            ui.selectable_value(&mut self.view, View::Block, lang.tr("Block", "Блок"));
            ui.separator();
            self.switch(ui);
        });
        ui.separator();
        let mut tapped = None;
        let mut drags = Vec::new();
        self.cells.clear();
        let held: Vec<bool> = self.places().iter().map(Option::is_some).collect();
        let rows = match self.view {
            View::Line => vec![line(&held)],
            View::Block => block(held.len()),
        };
        ui.vertical(|ui| {
            for row in rows {
                ui.horizontal(|ui| {
                    for place in row {
                        self.cell(ui, place, &mut tapped, &mut drags);
                    }
                });
            }
        });
        ui.separator();
        self.draw_field(ui);
        self.drag(ui, &drags);
        if let Some(place) = tapped {
            self.tap(place);
        }
    }

    /// "+" and "-" of the keyboard or the numpad, while no menu is open and
    /// no colour is dragged. `=` is the key of "+" with no shift.
    fn keys(&mut self, ui: &egui::Ui) {
        if self.menu.is_open() || self.dragging.is_some() {
            return;
        }
        let (next, previous) = ui.input(|input| {
            (
                input.key_pressed(Key::Plus) || input.key_pressed(Key::Equals),
                input.key_pressed(Key::Minus),
            )
        });
        if next {
            self.step(1);
        }
        if previous {
            self.step(-1);
        }
    }

    /// Shows the palette `step` away.
    fn step(&mut self, step: isize) {
        self.active = in_circle(self.active, step, self.palettes.len());
    }

    /// The switch of the palette, the way to it by touch (`RULE ui.touch`).
    fn switch(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        let many = self.palettes.len() > 1;
        if ui.add_enabled(many, egui::Button::new("<")).clicked() {
            self.step(-1);
        }
        ui.label(format!(
            "{} {} / {}",
            lang.tr("Palette", "Палитра"),
            self.active + 1,
            self.palettes.len()
        ));
        if ui.add_enabled(many, egui::Button::new(">")).clicked() {
            self.step(1);
        }
    }

    /// One place: a colour with its number, or a "+".
    fn cell(
        &mut self,
        ui: &mut egui::Ui,
        place: usize,
        tapped: &mut Option<usize>,
        drags: &mut Vec<(usize, egui::Response)>,
    ) {
        let lang = self.lang;
        let number = (place + 1).to_string();
        let lit = self.dragging.is_some() && place == self.drop_to;
        let shown = self.shown(place);
        let swatch = match shown {
            Some(Colour {
                rgb: Some(rgb),
                hidden: false,
                ..
            }) => ui.add(button(&number).fill(rgb).selected(lit)),
            Some(_) => {
                let swatch = ui.add(button(&number).selected(lit));
                hatched_swatch(ui, swatch.rect, &number);
                swatch
            }
            None => ui.add(button("+").selected(lit)),
        };
        self.cells.push((place, swatch.rect));
        let held = self.places()[place].is_some().then(|| {
            ui.interact(
                swatch.rect,
                Id::unique(("palette-editor-drag", self.active, place)),
                Sense::click_and_drag(),
            )
        });
        let name = match self.places()[place] {
            Some(Colour { hidden: true, .. }) => {
                format!("{} {number}", lang.tr("hidden colour", "скрытый цвет"))
            }
            Some(Colour { rgb: Some(_), .. }) => {
                format!("{} {number}", lang.tr("colour", "цвет"))
            }
            Some(_) => format!("{} {number}", lang.tr("removed colour", "удалённый цвет")),
            None => format!("{} {number}", lang.tr("empty place", "пустое место")),
        };
        let swatch = named(ui.ctx(), swatch, &name);
        if swatch.clicked() || held.as_ref().is_some_and(egui::Response::clicked) {
            *tapped = Some(place);
        }
        drags.extend(held.map(|held| (place, held)));
    }

    /// What a colour of any palette is drawn with; nothing means a hatch.
    fn rgb_of(&self, id: u32) -> Option<Color32> {
        self.palettes
            .iter()
            .flatten()
            .flatten()
            .find(|colour| colour.id == id)
            .and_then(|colour| colour.seen())
    }

    /// The made up field: a bead of a removed or hidden colour is hatched.
    fn draw_field(&self, ui: &mut egui::Ui) {
        let (columns, rows) = FIELD_SIZE;
        let size = vec2(columns as f32, rows as f32) * FIELD_BEAD;
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0.0, PAPER);
        for (row, cells) in self.field.iter().enumerate() {
            for (column, id) in cells.iter().enumerate() {
                let Some(id) = id else {
                    continue;
                };
                let corner = area.min + vec2(column as f32, row as f32) * FIELD_BEAD;
                let bead = Rect::from_min_size(corner, vec2(FIELD_BEAD, FIELD_BEAD)).shrink(1.0);
                match self.rgb_of(*id) {
                    Some(rgb) => {
                        painter.rect_filled(bead, 3.0, rgb);
                    }
                    None => hatched(&painter, bead, Color32::from_gray(0x60)),
                }
            }
        }
        let name = self.lang.tr("field, made up", "поле, для примера");
        named_rect(ui.ctx(), area, name, Some(ui.layer_id()));
    }

    /// What a place shows: while a colour is dragged, the colour and the
    /// place under the pointer have changed places already.
    fn shown(&self, place: usize) -> Option<Colour> {
        match self.dragging {
            Some(from) if place == from => self.places()[self.drop_to],
            Some(from) if place == self.drop_to => self.places()[from],
            _ => self.places()[place],
        }
    }

    /// A drag of a colour: it rises above the pointer, and the line shows
    /// it in the place under the pointer, lit, with the number of that
    /// place. What is in that place shows in the place it left; nothing
    /// else moves. Let go, it stays there. The beads hold ids, so they do
    /// not change.
    fn drag(&mut self, ui: &egui::Ui, drags: &[(usize, egui::Response)]) {
        if let Some((place, _)) = drags.iter().find(|(_, drag)| drag.drag_started()) {
            self.dragging = Some(*place);
            self.drop_to = *place;
            self.menu.close();
        }
        let Some(from) = self.dragging else {
            return;
        };
        let Some(pointer) = ui.ctx().pointer_interact_pos() else {
            return;
        };
        if let Some(place) = self
            .place_at(pointer)
            .filter(|place| *place != self.drop_to)
        {
            self.drop_to = place;
            ui.ctx().request_repaint();
        }
        let painter = ui.ctx().layer_painter(egui::LayerId::new(
            Order::Tooltip,
            Id::unique("palette-editor-ghost"),
        ));
        if let Some(rgb) = self.places()[from].and_then(|colour| colour.rgb) {
            let ghost = Rect::from_center_size(pointer - vec2(0.0, BUTTON.y), BUTTON);
            painter.add(ui.visuals().popup_shadow.as_shape(ghost, 6.0));
            painter.rect_filled(ghost, 6.0, rgb);
            painter.text(
                ghost.center(),
                egui::Align2::CENTER_CENTER,
                (self.drop_to + 1).to_string(),
                egui::FontId::proportional(super::TEXT),
                obusina_button::contrast(rgb),
            );
        }
        if drags.iter().any(|(_, drag)| drag.drag_stopped()) {
            self.dragging = None;
            let to = self.drop_to;
            self.palettes[self.active].swap(from, to);
        }
    }

    /// The count of the beads of a colour.
    fn beads(&self, id: u32) -> usize {
        self.field
            .iter()
            .flatten()
            .filter(|bead| **bead == Some(id))
            .count()
    }

    /// The place of the line under a pointer: the one of the cell nearest
    /// to it.
    fn place_at(&self, pointer: Pos2) -> Option<usize> {
        self.cells
            .iter()
            .min_by(|(_, first), (_, second)| {
                first
                    .center()
                    .distance(pointer)
                    .total_cmp(&second.center().distance(pointer))
            })
            .map(|(place, _)| *place)
    }

    /// A tap on a place: its menu opens, the colour on top. An empty place
    /// shows a new colour; a hatched place keeps its id, so its beads show
    /// it too. Nothing stays until "Accept".
    fn tap(&mut self, place: usize) {
        let lang = self.lang;
        let before = self.places()[place];
        let had_rgb = before.and_then(|colour| colour.rgb);
        let all: Vec<Color32> = self
            .palettes
            .iter()
            .flatten()
            .flatten()
            .filter_map(|colour| colour.rgb)
            .collect();
        let rgb = had_rgb.unwrap_or_else(|| next_colour(&all));
        let id = before.map_or(self.next_id, |colour| colour.id);
        let hidden = before.is_some_and(|colour| colour.hidden);
        let beads_left = self.beads(id);
        let remove = MenuItem::new("remove", lang.tr("Remove", "Удалить"));
        let last = match before {
            Some(Colour { rgb: None, .. }) if beads_left > 0 => {
                MenuItem::new("beads", delete_beads(lang, beads_left))
            }
            Some(_) => remove,
            None => remove.enabled(false),
        };
        let items = vec![
            MenuItem::new("accept", lang.tr("Accept", "Принять")),
            MenuItem::new(
                "move",
                lang.tr("Move to another palette", "Переместить в другую палитру"),
            )
            .enabled(before.is_some()),
            last,
        ];
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of a colour does not open: {error}");
            return;
        }
        if before.is_none() {
            self.next_id += 1;
        }
        self.palettes[self.active][place] = Some(Colour {
            id,
            rgb: Some(rgb),
            hidden,
        });
        self.before = before;
        *self.editing.lock().expect("the colour is never poisoned") = rgb;
        let editing = Arc::clone(&self.editing);
        let name = format!("{} {}", lang.tr("Colour", "Цвет"), place + 1);
        let beads = format!("{}: {}", lang.tr("beads", "бусин"), self.beads(id));
        self.menu.notice(Notice::widget(move |ui| {
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
                let title = format!("{name} · {} · {beads}", hex(*colour));
                ui.label(egui::RichText::new(title).strong());
            });
            ui.spacing_mut().slider_width = ui.available_width();
            egui::color_picker::color_picker_color32(
                ui,
                &mut colour,
                egui::color_picker::Alpha::Opaque,
            );
        }));
        self.menu_of = Some(place);
    }

    /// Puts back what the place held before its menu opened.
    fn take_back(&mut self, place: usize) {
        self.palettes[self.active][place] = self.before.take();
    }

    /// A colour with beads keeps its id and its place, with no rgb; one
    /// with no beads leaves the place empty.
    fn remove(&mut self, place: usize) {
        let slot = &mut self.palettes[self.active][place];
        let Some(colour) = *slot else {
            return;
        };
        let has_beads = self
            .field
            .iter()
            .flatten()
            .any(|bead| *bead == Some(colour.id));
        *slot = has_beads.then_some(Colour {
            rgb: None,
            ..colour
        });
    }

    /// The beads of a colour go. The colour stays in its place.
    fn delete_beads_of(&mut self, place: usize) {
        let Some(colour) = self.palettes[self.active][place] else {
            return;
        };
        for bead in self.field.iter_mut().flatten() {
            if *bead == Some(colour.id) {
                *bead = None;
            }
        }
    }

    /// "Move to another palette": one palette makes a new one; with more, a menu
    /// asks which, or a new one.
    fn ask_where(&mut self, place: usize) {
        if self.palettes.len() == 1 {
            self.move_to(place, None);
            return;
        }
        let lang = self.lang;
        let mut items: Vec<MenuItem> = self
            .palettes
            .iter()
            .enumerate()
            .filter(|(palette, _)| *palette != self.active)
            .map(|(palette, places)| {
                let label = format!("{} {}", lang.tr("Palette", "Палитра"), palette + 1);
                MenuItem::new(format!("to-{palette}"), label).enabled(places.contains(&None))
            })
            .collect();
        items.push(MenuItem::new(
            "to-new",
            lang.tr("A new palette", "Новая палитра"),
        ));
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of the palettes does not open: {error}");
            return;
        }
        let question = lang
            .tr("Where does colour {} go?", "Куда переместить цвет {}?")
            .replace("{}", &(place + 1).to_string());
        self.menu.notice(question);
        self.moving = Some(place);
    }

    /// The colour goes to the first free place of a palette, or of a new
    /// one. Its id goes with it, so its beads do not change.
    fn move_to(&mut self, place: usize, palette: Option<usize>) {
        let target = palette.unwrap_or_else(|| {
            self.palettes.push(vec![None; PLACES]);
            self.palettes.len() - 1
        });
        let Some(free) = self.palettes[target].iter().position(Option::is_none) else {
            log::error!("palette {target} has no free place");
            return;
        };
        self.palettes[target][free] = self.palettes[self.active][place].take();
    }

    /// Takes away the palette shown if nothing is left in its places. The
    /// next one takes its turn in the circle. Hatched beads keep a palette,
    /// because a "+" there colours them again.
    fn drop_if_empty(&mut self) {
        let empty = self.places().iter().all(Option::is_none);
        if empty && self.palettes.len() > 1 {
            self.palettes.remove(self.active);
            self.active %= self.palettes.len();
        }
    }

    /// The menu of a colour, while it is open.
    fn menu(&mut self, context: &egui::Context) {
        let chosen = self.menu.show(context);
        if let Some(place) = self.menu_of
            && let Some(colour) = self.palettes[self.active][place].as_mut()
        {
            colour.rgb = Some(*self.editing.lock().expect("the colour is never poisoned"));
        }
        if let (true, Some(place)) = (self.menu.is_open(), self.menu_of.or(self.moving)) {
            let layer = self.menu.layer_id().id;
            if let Some(rect) = context.memory(|memory| memory.area_rect(layer)) {
                named_rect(
                    context,
                    rect,
                    &format!("menu of colour {}", place + 1),
                    Some(self.menu.layer_id()),
                );
            }
        }
        let Some(chosen) = chosen else {
            if !self.menu.is_open() {
                if let Some(place) = self.menu_of.take() {
                    self.take_back(place);
                }
                self.moving = None;
            }
            return;
        };
        if let Some(place) = self.moving.take() {
            let palette = chosen
                .id
                .strip_prefix("to-")
                .and_then(|palette| palette.parse().ok());
            self.move_to(place, palette);
            self.drop_if_empty();
            return;
        }
        let Some(place) = self.menu_of.take() else {
            return;
        };
        match chosen.id.as_str() {
            "accept" => self.before = None,
            "move" => {
                self.take_back(place);
                self.ask_where(place);
                self.drop_if_empty();
            }
            "remove" => {
                self.take_back(place);
                self.remove(place);
                self.drop_if_empty();
            }
            "beads" => {
                self.take_back(place);
                self.delete_beads_of(place);
            }
            _ => self.take_back(place),
        }
    }
}

/// Draws a thing with no colour: an outline and diagonal lines.
pub(super) fn hatched(painter: &egui::Painter, rect: Rect, colour: Color32) {
    let line = egui::Stroke::new(1.0, colour);
    let clipped = painter.with_clip_rect(painter.clip_rect().intersect(rect));
    let lines = ((rect.width() + rect.height()) / HATCH_STEP).ceil() as usize;
    for step in 0..lines {
        let along = step as f32 * HATCH_STEP;
        clipped.line_segment(
            [
                rect.left_top() + vec2(along, 0.0),
                rect.left_top() + vec2(0.0, along),
            ],
            line,
        );
    }
    painter.rect_stroke(rect, 3.0, line, egui::StrokeKind::Inside);
}

/// Draws the hatch over a swatch, its number still on top.
pub(super) fn hatched_swatch(ui: &egui::Ui, rect: Rect, number: &str) {
    let painter = ui.painter();
    let ground = ui.visuals().widgets.inactive.weak_bg_fill;
    hatched(painter, rect, ui.visuals().text_color());
    painter.rect_filled(
        Rect::from_center_size(rect.center(), NUMBER_ROOM),
        3.0,
        ground,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        number,
        egui::FontId::proportional(super::TEXT),
        ui.visuals().strong_text_color(),
    );
}

/// `#rrggbb`.
fn hex(colour: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r(), colour.g(), colour.b())
}

/// A colour in OKLab, where a distance is near to how far apart two colours
/// look.
fn oklab(colour: Color32) -> Oklab {
    let srgb = Srgb::new(colour.r(), colour.g(), colour.b()).into_format::<f32>();
    Oklab::from_color(srgb.into_linear())
}

/// The colour a "+" adds: of the steps of lightness, chroma and hue, the
/// one whose nearest neighbour among the colours and the paper is farthest.
fn next_colour(colours: &[Color32]) -> Color32 {
    let taken: Vec<Oklab> = colours.iter().copied().chain([PAPER]).map(oklab).collect();
    let nearest = |candidate: &Oklab| {
        taken
            .iter()
            .map(|other| {
                let (l, a, b) = (
                    candidate.l - other.l,
                    candidate.a - other.a,
                    candidate.b - other.b,
                );
                (l * l + a * a + b * b).sqrt()
            })
            .fold(f32::INFINITY, f32::min)
    };
    candidates()
        .max_by(|first, second| nearest(&oklab(*first)).total_cmp(&nearest(&oklab(*second))))
        .unwrap_or(PAPER)
}

/// Every step of lightness, chroma and hue a screen can show; a grey once.
fn candidates() -> impl Iterator<Item = Color32> {
    let hues: Vec<f32> = (0..(360.0 / HUE_STEP) as usize)
        .map(|step| step as f32 * HUE_STEP)
        .collect();
    let mut steps = Vec::new();
    for lightness in LIGHTNESS {
        for chroma in CHROMA {
            let around: &[f32] = if chroma == 0.0 { &[0.0] } else { &hues };
            steps.extend(around.iter().map(|hue| Oklch::new(lightness, chroma, *hue)));
        }
    }
    steps.into_iter().filter_map(|oklch| {
        let srgb = Srgb::from_linear(palette::LinSrgb::from_color(oklch));
        let channels = [srgb.red, srgb.green, srgb.blue];
        channels
            .iter()
            .all(|channel| (0.0..=1.0).contains(channel))
            .then(|| {
                let [red, green, blue] =
                    channels.map(|channel: f32| (channel * 255.0).round() as u8);
                Color32::from_rgb(red, green, blue)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new colour is far from those there: next to black and white it is
    /// neither grey nor near either.
    #[test]
    fn a_new_colour_is_far_from_the_others() {
        let colours = [Color32::BLACK, Color32::WHITE];
        let new = next_colour(&colours);
        let nearest = colours
            .iter()
            .chain([&PAPER])
            .map(|colour| {
                let (first, second) = (oklab(new), oklab(*colour));
                ((first.l - second.l).powi(2)
                    + (first.a - second.a).powi(2)
                    + (first.b - second.b).powi(2))
                .sqrt()
            })
            .fold(f32::INFINITY, f32::min);

        assert!(nearest > 0.15, "{} is {nearest} from a colour", hex(new));
    }

    /// The window in English.
    fn window() -> egui_kittest::Harness<'static, PaletteEditor> {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_ui_state(
                |ui, mock: &mut PaletteEditor| mock.show(ui, Lang::En),
                PaletteEditor::new(),
            );
        harness.run();
        harness
    }

    /// Clicks the place with this label, then an entry of its menu.
    fn choose(
        harness: &mut egui_kittest::Harness<'static, PaletteEditor>,
        place: &str,
        entry: &str,
    ) {
        use egui_kittest::kittest::Queryable;
        harness.get_by_label(place).click();
        harness.run();
        harness.get_by_label(entry).click();
        harness.run();
    }

    /// The line is the places up to the last colour and one more; a hole
    /// is a place of the line like any other.
    #[test]
    fn the_line_ends_with_one_place_for_the_rest() {
        assert_eq!(line(&[false, false, false]), vec![0]);
        assert_eq!(line(&[true, true, false, false]), vec![0, 1, 2]);
        assert_eq!(line(&[true, false, true, false, false]), vec![0, 1, 2, 3]);
        assert_eq!(line(&[true, false, true]), vec![0, 1, 2]);
        assert_eq!(line(&[true, true, true]), vec![0, 1, 2]);
    }

    /// The block has the first places at the bottom, for any count.
    #[test]
    fn the_block_stands_as_a_numpad() {
        assert_eq!(block(9), vec![vec![6, 7, 8], vec![3, 4, 5], vec![0, 1, 2]]);
        assert_eq!(block(4), vec![vec![3], vec![0, 1, 2]]);
    }

    /// The palettes make a circle, both ways.
    #[test]
    fn the_palettes_make_a_circle() {
        assert_eq!(in_circle(0, 1, 3), 1);
        assert_eq!(in_circle(2, 1, 3), 0);
        assert_eq!(in_circle(0, -1, 3), 2);
        assert_eq!(in_circle(0, 1, 1), 0);
    }

    /// A removed colour leaves its place hatched, with its number, and asks
    /// nothing. Its beads keep its id with no rgb. "Accept" in that place
    /// colours them again.
    #[test]
    fn a_removed_colour_leaves_its_beads() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let id = harness.state().places()[2].unwrap().id;
        let beads = harness.state().beads(id);
        assert!(beads > 0);

        choose(&mut harness, "3", "Remove");

        assert_eq!(harness.state().rgb_of(id), None);
        assert!(harness.state().places()[3].unwrap().rgb.is_some());
        assert_eq!(harness.query_all_by_label("+").count(), 1);

        choose(&mut harness, "3", "Accept");
        assert!(harness.state().rgb_of(id).is_some());
        assert_eq!(harness.state().beads(id), beads);
    }

    /// A tap on a removed colour offers the delete of its beads. That leaves
    /// it in its place, and then its menu offers to remove it.
    #[test]
    fn a_removed_colour_deletes_only_its_beads() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let id = harness.state().places()[2].unwrap().id;
        let beads = harness.state().beads(id);
        choose(&mut harness, "3", "Remove");

        let entry = delete_beads(Lang::En, beads);
        harness.get_by_label("3").click();
        harness.run();
        assert!(harness.query_by_label("Remove").is_none());
        harness.get_by_label(&entry).click();
        harness.run();

        assert_eq!(harness.state().beads(id), 0);
        assert!(harness.state().places()[2].is_some());

        choose(&mut harness, "3", "Remove");
        assert!(harness.state().places()[2].is_none());
    }

    /// A hidden colour keeps its rgb, its place and its beads, and draws as
    /// a hatch.
    #[test]
    fn a_hidden_colour_is_hatched() {
        let harness = window();
        let hidden = harness.state().places()[4].unwrap();

        assert!(hidden.rgb.is_some());
        assert_eq!(harness.state().rgb_of(hidden.id), None);
        assert!(harness.state().beads(hidden.id) > 0);
    }

    /// A colour changed in the picker and not accepted goes back.
    #[test]
    fn a_colour_not_accepted_goes_back() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let before = harness.state().places()[0];

        harness.get_by_label("1").click();
        harness.run();
        *harness.state_mut().editing.lock().unwrap() = Color32::RED;
        harness.run();
        assert_eq!(harness.state().places()[0].unwrap().rgb, Some(Color32::RED));

        harness.state_mut().menu.close();
        harness.run();
        assert_eq!(harness.state().places()[0], before);
    }

    /// A colour changed in the picker and accepted stays.
    #[test]
    fn a_colour_accepted_stays() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();

        harness.get_by_label("1").click();
        harness.run();
        *harness.state_mut().editing.lock().unwrap() = Color32::RED;
        harness.run();
        harness.get_by_label("Accept").click();
        harness.run();

        assert_eq!(harness.state().places()[0].unwrap().rgb, Some(Color32::RED));
    }

    /// A new colour not accepted leaves the place empty.
    #[test]
    fn a_new_colour_not_accepted_is_gone() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();

        harness.get_by_label("+").click();
        harness.run();
        assert!(harness.state().places()[5].is_some());

        harness.state_mut().menu.close();
        harness.run();
        assert!(harness.state().places()[5].is_none());
    }

    /// The tab of the block shows every place: five colours and a "+" for
    /// each of the rest.
    #[test]
    fn the_tab_of_the_block_shows_every_place() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        assert_eq!(harness.query_all_by_label("+").count(), 1);

        harness.get_by_label("Block").click();
        harness.run();

        assert_eq!(harness.state().view, View::Block);
        assert_eq!(harness.query_all_by_label("+").count(), PLACES - 5);
    }

    /// With one palette, the entry makes a new one and the colour goes
    /// there with its beads; its old place is free.
    #[test]
    fn the_first_move_makes_a_new_palette() {
        let mut harness = window();
        let colour = harness.state().places()[1];
        let id = colour.unwrap().id;
        let beads = harness.state().beads(id);

        choose(&mut harness, "2", "Move to another palette");

        assert_eq!(harness.state().palettes.len(), 2);
        assert_eq!(harness.state().palettes[1][0], colour);
        assert!(harness.state().palettes[0][1].is_none());
        assert_eq!(harness.state().rgb_of(id), colour.unwrap().rgb);
        assert_eq!(harness.state().beads(id), beads);
    }

    /// With two palettes, the entry asks: the other one or a new one.
    #[test]
    fn a_later_move_asks_where() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        choose(&mut harness, "2", "Move to another palette");
        let colour = harness.state().places()[0];

        choose(&mut harness, "1", "Move to another palette");
        harness.get_by_label("Where does colour 1 go?");
        harness.get_by_label("A new palette");
        harness.get_by_label("Palette 2").click();
        harness.run();

        assert_eq!(harness.state().palettes.len(), 2);
        assert_eq!(harness.state().palettes[1][1], colour);
    }

    /// A palette left with nothing goes, and the one after it is shown.
    #[test]
    fn a_palette_left_with_nothing_goes() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        choose(&mut harness, "2", "Move to another palette");
        harness.key_press(Key::Plus);
        harness.run();
        assert_eq!(harness.state().active, 1);

        choose(&mut harness, "1", "Move to another palette");
        harness.get_by_label("Palette 1").click();
        harness.run();

        assert_eq!(harness.state().palettes.len(), 1);
        assert_eq!(harness.state().active, 0);
    }

    /// A palette with hatched beads stays: a "+" there colours them again.
    #[test]
    fn a_palette_with_hatched_beads_stays() {
        let mut harness = window();
        choose(&mut harness, "2", "Move to another palette");
        harness.state_mut().field[0][0] = harness.state().palettes[1][0].map(|colour| colour.id);
        harness.key_press(Key::Plus);
        harness.run();

        choose(&mut harness, "1", "Remove");

        assert_eq!(harness.state().palettes.len(), 2);
    }

    /// "+" and "-" switch the palette in a circle; a menu keeps them.
    #[test]
    fn the_keys_switch_the_palette() {
        let mut harness = window();
        choose(&mut harness, "2", "Move to another palette");
        assert_eq!(harness.state().active, 0);

        harness.key_press(Key::Plus);
        harness.run();
        assert_eq!(harness.state().active, 1);
        harness.key_press(Key::Plus);
        harness.run();
        assert_eq!(harness.state().active, 0);
        harness.key_press(Key::Minus);
        harness.run();
        assert_eq!(harness.state().active, 1);
    }

    /// The pointer pressed at `from`, moved to `to` in steps, and held
    /// there; `let_go` lets it go.
    fn drag(
        harness: &mut egui_kittest::Harness<'_, PaletteEditor>,
        from: Pos2,
        to: Pos2,
        let_go: bool,
    ) {
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        harness.event(egui::Event::PointerMoved(from));
        harness.event(button(from, true));
        harness.step();
        for step in 1..=10 {
            harness.event(egui::Event::PointerMoved(
                from + (to - from) * (step as f32 / 10.0),
            ));
            harness.step();
        }
        if let_go {
            harness.event(button(to, false));
        }
        harness.run();
    }

    /// A colour dragged onto another swaps with it; the rest and the beads
    /// do not change.
    #[test]
    fn a_colour_dragged_onto_another_swaps_with_it() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let before = harness.state().places().to_vec();
        let field = harness.state().field.clone();
        let from = harness.get_by_label("1").rect().center();
        let to = harness.get_by_label("3").rect().center();

        drag(&mut harness, from, to, true);

        let after = harness.state().places();
        assert_eq!(after[2], before[0]);
        assert_eq!(after[0], before[2]);
        assert_eq!(after[1], before[1]);
        assert_eq!(harness.state().field, field);
        assert!(harness.state().dragging.is_none());
    }

    /// A colour dragged onto the last "+" goes to the first place after the
    /// colours and leaves a hole, which shows as a "+".
    #[test]
    fn a_colour_dragged_onto_a_plus_leaves_a_hole() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let first = harness.state().places()[0];
        let from = harness.get_by_label("1").rect().center();
        let to = harness.get_by_label("+").rect().center();

        drag(&mut harness, from, to, true);

        assert_eq!(harness.state().places()[5], first);
        assert!(harness.state().places()[0].is_none());
        assert_eq!(harness.query_all_by_label("+").count(), 2);
    }

    /// A press of the pointer on a colour, let go in place, opens its menu.
    #[test]
    fn a_tap_of_the_pointer_opens_the_menu() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let at = harness.get_by_label("3").rect().center();
        harness.event(egui::Event::PointerMoved(at));
        for pressed in [true, false] {
            harness.event(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
        }
        harness.run();

        assert!(harness.state().menu.is_open());
    }

    /// While a colour is dragged, the place under the pointer shows it with
    /// the number of that place, before it is let go.
    #[test]
    fn a_dragged_colour_shows_its_new_number() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let before = harness.state().places().to_vec();
        let from = harness.get_by_label("1").rect().center();
        let to = harness.get_by_label("4").rect().center();

        drag(&mut harness, from, to, false);

        assert_eq!(harness.state().drop_to, 3);
        assert_eq!(harness.state().shown(3), before[0]);
        assert_eq!(harness.state().shown(0), before[3]);
        assert_eq!(
            harness.state().places(),
            before.as_slice(),
            "nothing moved before it is let go"
        );
    }

    /// Nine colours stand in one line, whatever the width of the window.
    #[test]
    fn nine_colours_stand_in_one_line() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let colour = Colour {
            id: 1,
            rgb: Some(Color32::RED),
            hidden: false,
        };
        harness.state_mut().palettes[0] = vec![Some(colour); PLACES];
        harness.run();

        let first = harness.get_by_label("1").rect();
        let last = harness.get_by_label("9").rect();
        assert_eq!(first.top(), last.top());
    }
}
