//! The window that fills a palette: a modal with no title. The colours
//! stand in a line, a "+" at the end. Two variants put a colour in another
//! place: "Move" of its menu, or a drag.
//!
//! ```text
//! tap a colour ──► PlateMenu ──pick──► the picker, under the line
//!                     │──move────► the next tap on a colour puts it there
//!                     └──remove──► gone
//! drag a colour ──► a mark between two colours ──let go──► it stands there
//! tap "+" ──► the colour farthest from the others and the paper, in the picker
//! ```

use egui::{Color32, Id, Modal, Order, Pos2, Rect, Sense, Stroke, vec2};
use palette::{FromColor, Oklab, Oklch, Srgb};
use plate_menu::{MenuItem, Notice, PlateMenu};

use super::{BUTTON, Lang, Mockup, button, named, named_rect};

/// The most colours a palette holds.
const MAX_COLOURS: usize = 9;
/// The ground the beads lie on, `PAPER` of the canvas: a new colour is
/// told from it too.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The lightness, chroma and hue steps a new colour is chosen among, in
/// OKLCh.
const LIGHTNESS: [f32; 5] = [0.25, 0.4, 0.55, 0.7, 0.85];
const CHROMA: [f32; 3] = [0.0, 0.08, 0.16];
const HUE_STEP: f32 = 30.0;
/// The height of the parts of the picker of egui besides its square: the
/// row of buttons, the colour and the strip of hues, with their gaps.
const PICKER_PARTS: f32 = 110.0;
/// The side of the square of the picker when the window is short.
const PICKER_SMALLEST: f32 = 100.0;
/// The swatch in the notice of the menu of a colour.
const NOTICE_SWATCH: egui::Vec2 = vec2(28.0, 20.0);

/// How a colour is put in another place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reorder {
    /// "Move" of the menu of the colour, then a tap where it goes.
    Menu,
    /// A drag of the colour; a mark shows where it goes.
    Drag,
}

/// What a tap on a colour does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Doing {
    /// Opens the menu of the colour.
    Nothing,
    /// The picker of this colour is open under the line.
    Picking(usize),
    /// This colour goes where the next tap is.
    Moving(usize),
}

/// A palette and whether its window is open.
pub struct PaletteEditor {
    reorder: Reorder,
    lang: Lang,
    colours: Vec<Color32>,
    open: bool,
    doing: Doing,
    menu: PlateMenu,
    /// The colour whose menu is open.
    menu_of: Option<usize>,
    /// Where each colour stood in the last frame.
    cells: Vec<Rect>,
    /// Where the window stood in the last frame.
    window: Rect,
    /// The colour being dragged.
    dragging: Option<usize>,
}

impl PaletteEditor {
    /// The mockup with one way to put a colour in another place.
    pub fn new(reorder: Reorder) -> PaletteEditor {
        PaletteEditor {
            reorder,
            lang: Lang::Ru,
            colours: vec![
                Color32::from_rgb(0x1b, 0x1b, 0x1b),
                Color32::from_rgb(0xf2, 0xef, 0xe6),
                Color32::from_rgb(0xc0, 0x30, 0x30),
                Color32::from_rgb(0x2f, 0x5f, 0xb3),
                Color32::from_rgb(0xe8, 0xc5, 0x47),
            ],
            open: true,
            doing: Doing::Nothing,
            menu: PlateMenu::new().with_id(Id::unique(("palette-editor-menu", reorder))),
            menu_of: None,
            cells: Vec::new(),
            window: Rect::NOTHING,
            dragging: None,
        }
    }
}

impl Mockup for PaletteEditor {
    fn name(&self) -> &'static str {
        match self.reorder {
            Reorder::Menu => "palette-move",
            Reorder::Drag => "palette-drag",
        }
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "colours": self.colours.iter().map(|colour| hex(*colour)).collect::<Vec<_>>(),
            "doing": format!("{:?}", self.doing),
            "dragging": self.dragging,
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
        let modal = Modal::new(Id::unique(("palette-editor", self.reorder)))
            .show(ui.ctx(), |ui| self.window(ui));
        self.window = modal.response.rect;
        named(ui.ctx(), modal.response.clone(), "palette window");
        let busy = self.menu.is_open() || matches!(self.doing, Doing::Picking(_));
        if modal.should_close() && !busy {
            self.open = false;
            self.doing = Doing::Nothing;
        }
        self.picker(ui.ctx());
        self.menu(ui.ctx());
    }
}

impl PaletteEditor {
    /// The content of the window: the line of colours and the "+".
    fn window(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        let mut tapped = None;
        let mut drags = Vec::new();
        self.cells.clear();
        ui.horizontal_wrapped(|ui| {
            for (index, colour) in self.colours.iter().enumerate() {
                let key = (index + 1).to_string();
                let lifted = self.doing == Doing::Moving(index) || self.dragging == Some(index);
                let swatch = ui.add(button(&key).fill(*colour).selected(lifted));
                self.cells.push(swatch.rect);
                let held = (self.reorder == Reorder::Drag).then(|| {
                    ui.interact(
                        swatch.rect,
                        Id::unique(("palette-editor-drag", index)),
                        Sense::click_and_drag(),
                    )
                });
                let name = format!("{} {key}", lang.tr("colour", "цвет"));
                let swatch = named(ui.ctx(), swatch, &name);
                if swatch.clicked() || held.as_ref().is_some_and(egui::Response::clicked) {
                    tapped = Some(index);
                }
                drags.extend(held);
            }
            let full = self.colours.len() >= MAX_COLOURS;
            let add = ui.add_enabled(!full, button("+"));
            if named(ui.ctx(), add, lang.tr("add a colour", "добавить цвет")).clicked()
            {
                self.colours.push(next_colour(&self.colours));
                self.doing = Doing::Picking(self.colours.len() - 1);
            }
        });
        if let Doing::Moving(from) = self.doing {
            let hint = lang.tr(
                "Tap where colour {} goes",
                "Нажмите, куда поставить цвет {}",
            );
            ui.label(hint.replace("{}", &(from + 1).to_string()));
        }
        self.drag(ui, &drags);
        if let Some(index) = tapped {
            self.tap(index);
        }
    }

    /// A drag of a colour: the colour follows the pointer, a mark stands
    /// where it would go, and let go it stands there.
    fn drag(&mut self, ui: &egui::Ui, drags: &[egui::Response]) {
        if let Some(index) = drags.iter().position(egui::Response::drag_started) {
            self.dragging = Some(index);
            self.menu.close();
        }
        let Some(from) = self.dragging else {
            return;
        };
        let Some(pointer) = ui.ctx().pointer_interact_pos() else {
            return;
        };
        let place = self.place_at(pointer);
        let painter = ui.ctx().layer_painter(egui::LayerId::new(
            Order::Tooltip,
            Id::unique("palette-editor-ghost"),
        ));
        if let Some(mark) = self.mark(place) {
            painter.line_segment(
                [mark.center_top(), mark.center_bottom()],
                Stroke::new(3.0, ui.visuals().selection.bg_fill),
            );
        }
        if let Some(colour) = self.colours.get(from) {
            let ghost = Rect::from_center_size(pointer - vec2(0.0, BUTTON.y), BUTTON);
            painter.add(ui.visuals().popup_shadow.as_shape(ghost, 6.0));
            painter.rect_filled(ghost, 6.0, *colour);
            painter.text(
                ghost.center(),
                egui::Align2::CENTER_CENTER,
                (from + 1).to_string(),
                egui::FontId::proportional(super::TEXT),
                obusina_button::contrast(*colour),
            );
        }
        if drags.iter().any(egui::Response::drag_stopped) {
            self.dragging = None;
            let to = if place > from { place - 1 } else { place };
            if to != from && from < self.colours.len() {
                let colour = self.colours.remove(from);
                self.colours.insert(to.min(self.colours.len()), colour);
            }
        }
    }

    /// The place among the colours a pointer points at: before the colour
    /// nearest to it, or after it when the pointer is right of its middle.
    fn place_at(&self, pointer: Pos2) -> usize {
        let nearest = self
            .cells
            .iter()
            .enumerate()
            .min_by(|(_, first), (_, second)| {
                first
                    .center()
                    .distance(pointer)
                    .total_cmp(&second.center().distance(pointer))
            });
        match nearest {
            Some((index, cell)) if pointer.x > cell.center().x => index + 1,
            Some((index, _)) => index,
            None => 0,
        }
    }

    /// The mark of a place: a thin box in the gap before its colour, or after
    /// the last one.
    fn mark(&self, place: usize) -> Option<Rect> {
        let gap = 4.0;
        match self.cells.get(place) {
            Some(cell) => Some(Rect::from_x_y_ranges(
                cell.left() - gap..=cell.left() - gap,
                cell.y_range(),
            )),
            None => self.cells.last().map(|cell| {
                Rect::from_x_y_ranges(cell.right() + gap..=cell.right() + gap, cell.y_range())
            }),
        }
    }

    /// A tap on a colour: puts the colour being moved there, or opens the
    /// menu of the colour, its swatch and number on top.
    fn tap(&mut self, index: usize) {
        let lang = self.lang;
        if let Doing::Moving(from) = self.doing {
            let colour = self.colours.remove(from);
            self.colours.insert(index, colour);
            self.doing = Doing::Nothing;
            return;
        }
        let mut items = vec![MenuItem::new(
            "pick",
            lang.tr("Pick a colour", "Выбрать цвет"),
        )];
        if self.reorder == Reorder::Menu {
            items.push(
                MenuItem::new("move", lang.tr("Move", "Переместить"))
                    .enabled(self.colours.len() > 1),
            );
        }
        items.push(MenuItem::new("remove", lang.tr("Remove", "Удалить")));
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of a colour does not open: {error}");
            return;
        }
        let colour = self.colours[index];
        let title = format!(
            "{} {} · {}",
            lang.tr("Colour", "Цвет"),
            index + 1,
            hex(colour)
        );
        self.menu.notice(Notice::widget(move |ui| {
            ui.horizontal(|ui| {
                let (swatch, _) = ui.allocate_exact_size(NOTICE_SWATCH, Sense::hover());
                ui.painter().rect_filled(swatch, 4.0, colour);
                ui.painter().rect_stroke(
                    swatch,
                    4.0,
                    ui.visuals().widgets.noninteractive.bg_stroke,
                    egui::StrokeKind::Inside,
                );
                ui.label(egui::RichText::new(&title).strong());
            });
        }));
        self.menu_of = Some(index);
    }

    /// The menu of a colour, while it is open.
    fn menu(&mut self, context: &egui::Context) {
        let chosen = self.menu.show(context);
        if let (true, Some(index)) = (self.menu.is_open(), self.menu_of) {
            let layer = self.menu.layer_id().id;
            if let Some(rect) = context.memory(|memory| memory.area_rect(layer)) {
                named_rect(context, rect, &format!("menu of colour {}", index + 1));
            }
        }
        let Some(chosen) = chosen else {
            return;
        };
        let Some(index) = self.menu_of.take() else {
            return;
        };
        match chosen.id.as_str() {
            "pick" => self.doing = Doing::Picking(index),
            "move" => self.doing = Doing::Moving(index),
            "remove" => {
                self.colours.remove(index);
                self.doing = Doing::Nothing;
            }
            _ => {}
        }
    }

    /// The picker of a colour, under its cell and as wide as the window,
    /// above it; a tap beside it closes it.
    fn picker(&mut self, context: &egui::Context) {
        let Doing::Picking(index) = self.doing else {
            return;
        };
        if index >= self.colours.len() {
            self.doing = Doing::Nothing;
            return;
        }
        let (Some(cell), true) = (self.cells.get(index), self.window.is_positive()) else {
            context.request_repaint();
            return;
        };
        let width = self.window.width();
        let top = cell.bottom() + 6.0;
        let room = context.content_rect().bottom() - top - PICKER_PARTS;
        let colour = &mut self.colours[index];
        let area = egui::Area::new(Id::unique(("palette-editor-picker", self.reorder)))
            .order(Order::Tooltip)
            .fixed_pos(Pos2::new(self.window.left(), top))
            .constrain(true)
            .show(context, |ui| {
                let frame = egui::Frame::popup(ui.style());
                let inner = width - frame.total_margin().sum().x;
                frame.show(ui, |ui| {
                    ui.set_width(inner);
                    ui.spacing_mut().slider_width = inner.min(room).max(PICKER_SMALLEST);
                    egui::color_picker::color_picker_color32(
                        ui,
                        colour,
                        egui::color_picker::Alpha::Opaque,
                    );
                });
            });
        named(context, area.response.clone(), "colour picker");
        let beside = context.input(|input| {
            input.pointer.any_pressed()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|pointer| !area.response.rect.contains(pointer))
        });
        let escape = context.input(|input| input.key_pressed(egui::Key::Escape));
        if beside || escape {
            self.doing = Doing::Nothing;
        }
    }
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

    fn window() -> egui_kittest::Harness<'static, PaletteEditor> {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_ui_state(
                |ui, mock: &mut PaletteEditor| mock.show(ui, Lang::En),
                PaletteEditor::new(Reorder::Menu),
            );
        harness.run();
        harness
    }

    /// The menu of a colour stands above the modal window and is chosen
    /// in.
    #[test]
    fn a_colour_is_removed_from_its_menu() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();

        harness.get_by_label("3").click();
        harness.run();
        harness.get_by_label("Remove").click();
        harness.run();

        assert_eq!(harness.state().colours.len(), 4);
    }

    /// A colour moved goes where the next tap is.
    #[test]
    fn a_colour_is_moved_by_a_tap() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window();
        let first = harness.state().colours[0];

        harness.get_by_label("1").click();
        harness.run();
        harness.get_by_label("Move").click();
        harness.run();
        harness.get_by_label("4").click();
        harness.run();

        assert_eq!(harness.state().colours[3], first);
    }

    /// The window of the variant `reorder`, in `lang`.
    fn window_of(reorder: Reorder, lang: Lang) -> egui_kittest::Harness<'static, PaletteEditor> {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_ui_state(
                move |ui, mock: &mut PaletteEditor| mock.show(ui, lang),
                PaletteEditor::new(reorder),
            );
        harness.run();
        harness
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

    /// A colour dragged past two others and let go stands there.
    #[test]
    fn a_colour_is_dragged_to_its_place() {
        use egui_kittest::kittest::Queryable;
        let mut harness = window_of(Reorder::Drag, Lang::En);
        let first = harness.state().colours[0];
        let from = harness.get_by_label("1").rect().center();
        let to = harness.get_by_label("3").rect().right_center() + egui::vec2(-4.0, 0.0);

        drag(&mut harness, from, to, true);

        assert_eq!(harness.state().colours[2], first);
        assert!(harness.state().dragging.is_none());
    }

    /// A press of the pointer on a colour, let go in place, opens its menu
    /// in both variants. The area a drag variant lays over a colour is on
    /// top, so it takes the tap as well as the drag.
    #[test]
    fn a_tap_of_the_pointer_opens_the_menu() {
        use egui_kittest::kittest::Queryable;
        for reorder in [Reorder::Menu, Reorder::Drag] {
            let mut harness = window_of(reorder, Lang::En);
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

            assert!(
                harness.state().menu.is_open(),
                "{reorder:?}: the menu opens"
            );
        }
    }
}
