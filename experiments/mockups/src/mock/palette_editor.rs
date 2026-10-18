//! The window that fills a palette: a modal with no title. The colours
//! stand in a line, a "+" at the end.
//!
//! ```text
//! tap a colour ──► PlateMenu ──change──► the picker, under that colour
//!                     │──move────► the next tap on a colour puts it there
//!                     └──remove──► gone
//! tap "+" ──► the colour farthest from the others and the paper, in the picker
//! ```

use egui::{Color32, Id, Modal, Order, Rect};
use palette::{FromColor, Oklab, Oklch, Srgb};
use plate_menu::{MenuItem, PlateMenu};

use super::{Mockup, button, named, named_rect};

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

/// What a tap on a colour does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Doing {
    /// Opens the menu of the colour.
    Nothing,
    /// The picker of this colour is open under it.
    Picking(usize),
    /// This colour goes where the next tap is.
    Moving(usize),
}

/// A palette and whether its window is open.
pub struct PaletteEditor {
    colours: Vec<Color32>,
    open: bool,
    doing: Doing,
    menu: PlateMenu,
    /// The colour whose menu is open.
    menu_of: Option<usize>,
    /// Where each colour stood in the last frame, for the picker.
    cells: Vec<Rect>,
}

impl Default for PaletteEditor {
    fn default() -> Self {
        PaletteEditor {
            colours: vec![
                Color32::from_rgb(0x1b, 0x1b, 0x1b),
                Color32::from_rgb(0xf2, 0xef, 0xe6),
                Color32::from_rgb(0xc0, 0x30, 0x30),
                Color32::from_rgb(0x2f, 0x5f, 0xb3),
                Color32::from_rgb(0xe8, 0xc5, 0x47),
            ],
            open: true,
            doing: Doing::Nothing,
            menu: PlateMenu::new().with_id(Id::unique("palette-editor-menu")),
            menu_of: None,
            cells: Vec::new(),
        }
    }
}

impl Mockup for PaletteEditor {
    fn name(&self) -> &'static str {
        "palette-editor"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "colours": self.colours.iter().map(|colour| hex(*colour)).collect::<Vec<_>>(),
            "doing": format!("{:?}", self.doing),
            "open": self.open,
        })
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        if ui.button("open the palette").clicked() {
            self.open = true;
        }
        if !self.open {
            return;
        }
        let modal = Modal::new(Id::unique("palette-editor")).show(ui.ctx(), |ui| self.window(ui));
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
        let mut tapped = None;
        self.cells.clear();
        ui.horizontal_wrapped(|ui| {
            for (index, colour) in self.colours.iter().enumerate() {
                let key = (index + 1).to_string();
                let lifted = self.doing == Doing::Moving(index);
                let swatch = ui.add(button(&key).fill(*colour).selected(lifted));
                self.cells.push(swatch.rect);
                if named(ui.ctx(), swatch, &format!("colour {key}")).clicked() {
                    tapped = Some(index);
                }
            }
            let full = self.colours.len() >= MAX_COLOURS;
            let add = ui.add_enabled(!full, button("+"));
            if named(ui.ctx(), add, "add a colour").clicked() {
                self.colours.push(next_colour(&self.colours));
                self.doing = Doing::Picking(self.colours.len() - 1);
            }
        });
        if let Doing::Moving(from) = self.doing {
            ui.label(format!("tap where colour {} goes", from + 1));
        }
        if let Some(index) = tapped {
            self.tap(index);
        }
    }

    /// A tap on a colour: puts the colour being moved there, or opens the
    /// menu of the colour.
    fn tap(&mut self, index: usize) {
        if let Doing::Moving(from) = self.doing {
            let colour = self.colours.remove(from);
            self.colours.insert(index, colour);
            self.doing = Doing::Nothing;
            return;
        }
        let items = vec![
            MenuItem::new("change", "Change the colour"),
            MenuItem::new("move", "Move").enabled(self.colours.len() > 1),
            MenuItem::new("remove", "Remove"),
        ];
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of a colour does not open: {error}");
            return;
        }
        self.menu.notice(format!("Colour {}", index + 1));
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
            "change" => self.doing = Doing::Picking(index),
            "move" => self.doing = Doing::Moving(index),
            "remove" => {
                self.colours.remove(index);
                self.doing = Doing::Nothing;
            }
            _ => {}
        }
    }

    /// The picker of a colour, under its cell, above the window; a tap
    /// beside it closes it.
    fn picker(&mut self, context: &egui::Context) {
        let Doing::Picking(index) = self.doing else {
            return;
        };
        if index >= self.colours.len() {
            self.doing = Doing::Nothing;
            return;
        }
        let Some(cell) = self.cells.get(index) else {
            context.request_repaint();
            return;
        };
        let colour = &mut self.colours[index];
        let area = egui::Area::new(Id::unique("palette-editor-picker"))
            .order(Order::Tooltip)
            .fixed_pos(cell.left_bottom() + egui::vec2(0.0, 6.0))
            .constrain(true)
            .show(context, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
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
                |ui, mock: &mut PaletteEditor| mock.show(ui),
                PaletteEditor::default(),
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
}
