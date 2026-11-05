//! The panel of beads in the working mode: what stands on it, how the colour
//! in hand is marked, and the menu of a colour.
//!
//! ```text
//! tap a colour or `0` ──► it is in hand, a bar under it
//! tap what is in hand ──► PlateMenu: its title, then
//!                            │──hide / show──────► it and its beads hatched, it stays on the panel
//!                            │──delete beads─────► its beads gone, the colour stays
//!                            └──palette settings─► the window of the palette
//! `0` in hand has only the last entry. The key `P` opens the window too.
//! ```
//!
//! A removed colour, whose beads stay hatched, does not stand on the panel.

use egui::{Color32, Id, Sense};
use obusina_button::SquareButton;
use plate_menu::{MenuItem, PlateMenu};

use super::numpad::{block_rows, zero};
use super::palette_editor::{hatched, hatched_swatch};
use super::{BUTTON, Lang, Mockup, TEXT, delete_beads, in_hand, named};

/// The made up field: its columns and rows.
const FIELD_SIZE: (usize, usize) = (14, 5);
/// The side of a bead of the made up field.
const FIELD_BEAD: f32 = 16.0;
/// The ground of the made up field, `PAPER` of the canvas.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The space between two swatches, `SPACING` of the panel of beads.
const SPACING: f32 = 4.0;
/// The colours: the third was removed, and its beads are on the field.
const COLOURS: [Option<[u8; 3]>; 5] = [
    Some([0x1b, 0x1b, 0x1b]),
    Some([0xf2, 0xef, 0xe6]),
    None,
    Some([0x2f, 0x5f, 0xb3]),
    Some([0xe8, 0xc5, 0x47]),
];

/// A place of the palette.
struct Slot {
    /// With none, the colour was removed.
    rgb: Option<Color32>,
    hidden: bool,
}

/// How the panel stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Line,
    Block,
}

/// The panel and a made up field.
pub struct Panel {
    view: View,
    slots: Vec<Slot>,
    in_hand: Option<usize>,
    /// The place of each bead, by rows.
    field: Vec<Vec<Option<usize>>>,
    menu: PlateMenu,
    /// What the menu is of: a place, or `0`.
    menu_of: Option<Option<usize>>,
    /// The window of the palette would be open.
    settings: bool,
}

impl Panel {
    /// The panel with the first colour in hand.
    pub fn new() -> Panel {
        let slots: Vec<Slot> = COLOURS
            .iter()
            .map(|rgb| Slot {
                rgb: rgb.map(|[red, green, blue]| Color32::from_rgb(red, green, blue)),
                hidden: false,
            })
            .collect();
        let (columns, rows) = FIELD_SIZE;
        let count = slots.len();
        let field = (0..rows)
            .map(|row| {
                (0..columns)
                    .map(|column| (column % 7 != 6).then_some((column + row) % count))
                    .collect()
            })
            .collect();
        Panel {
            view: View::Line,
            slots,
            in_hand: Some(0),
            field,
            menu: PlateMenu::new().with_id(Id::unique("panel-menu")),
            menu_of: None,
            settings: false,
        }
    }

    /// The numbers that stand on the panel: a removed colour does not.
    fn shown(&self) -> Vec<u8> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.rgb.is_some())
            .map(|(place, _)| place as u8 + 1)
            .collect()
    }

    /// The count of the beads of a place.
    fn beads(&self, place: usize) -> usize {
        self.field
            .iter()
            .flatten()
            .filter(|bead| **bead == Some(place))
            .count()
    }

    /// One swatch: marked in hand, hatched when hidden.
    fn swatch(&mut self, ui: &mut egui::Ui, lang: Lang, number: u8) {
        let place = usize::from(number) - 1;
        let label = number.to_string();
        let Some(rgb) = self.slots[place].rgb else {
            return;
        };
        let hidden = self.slots[place].hidden;
        let button = SquareButton::new(&label, BUTTON).text_size(TEXT);
        let response = if hidden {
            ui.add(button)
        } else {
            ui.add(button.fill(rgb))
        };
        if hidden {
            hatched_swatch(ui, response.rect, &label);
        }
        if self.in_hand == Some(place) {
            in_hand(ui, response.rect, (!hidden).then_some(rgb), &label);
        }
        if response.clicked() {
            self.tap(lang, Some(place));
        }
    }

    /// A tap: a colour or `0` goes in hand; what is in hand opens its menu.
    fn tap(&mut self, lang: Lang, what: Option<usize>) {
        if self.in_hand != what {
            self.in_hand = what;
            return;
        }
        let settings = MenuItem::new("settings", lang.tr("Palette settings", "Настройка палитры"));
        let (items, title) = match what {
            Some(place) => {
                let slot = &self.slots[place];
                let hide = if slot.hidden {
                    MenuItem::new("hide", lang.tr("Show", "Показать"))
                } else {
                    MenuItem::new("hide", lang.tr("Hide", "Скрыть"))
                };
                let beads = self.beads(place);
                let rgb = slot.rgb.unwrap_or(Color32::BLACK);
                let title = format!(
                    "{} {} · #{:02x}{:02x}{:02x}",
                    lang.tr("Colour", "Цвет"),
                    place + 1,
                    rgb.r(),
                    rgb.g(),
                    rgb.b()
                );
                let delete = MenuItem::new("beads", delete_beads(lang, beads)).enabled(beads > 0);
                (vec![hide, delete, settings], title)
            }
            None => (
                vec![settings],
                lang.tr("Empty place", "Пустое место").to_string(),
            ),
        };
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu does not open: {error}");
            return;
        }
        self.menu.notice(title);
        self.menu_of = Some(what);
    }

    /// The menu, while it is open.
    fn menu(&mut self, context: &egui::Context) {
        let Some(chosen) = self.menu.show(context) else {
            return;
        };
        let Some(what) = self.menu_of.take() else {
            return;
        };
        match (chosen.id.as_str(), what) {
            ("settings", _) => self.settings = true,
            ("hide", Some(place)) => self.slots[place].hidden = !self.slots[place].hidden,
            ("beads", Some(place)) => {
                for bead in self.field.iter_mut().flatten() {
                    if *bead == Some(place) {
                        *bead = None;
                    }
                }
            }
            _ => {}
        }
    }

    /// The empty place, with its mark while it is in hand.
    fn empty_place(&mut self, ui: &mut egui::Ui, lang: Lang, wide: bool) {
        let response = zero(ui, wide);
        if self.in_hand.is_none() {
            let ground = ui.visuals().widgets.inactive.weak_bg_fill;
            in_hand(ui, response.rect, Some(ground), "0");
        }
        if response.clicked() {
            self.tap(lang, None);
        }
    }

    /// The made up field: a bead of a removed or hidden colour is hatched.
    fn draw_field(&self, ui: &mut egui::Ui, lang: Lang) {
        let (columns, rows) = FIELD_SIZE;
        let size = egui::vec2(columns as f32, rows as f32) * FIELD_BEAD;
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0.0, PAPER);
        for (row, cells) in self.field.iter().enumerate() {
            for (column, place) in cells.iter().enumerate() {
                let Some(place) = place else {
                    continue;
                };
                let corner = area.min + egui::vec2(column as f32, row as f32) * FIELD_BEAD;
                let bead = egui::Rect::from_min_size(corner, egui::vec2(FIELD_BEAD, FIELD_BEAD))
                    .shrink(1.0);
                let slot = &self.slots[*place];
                match slot.rgb.filter(|_| !slot.hidden) {
                    Some(rgb) => {
                        painter.rect_filled(bead, 3.0, rgb);
                    }
                    None => hatched(&painter, bead, Color32::from_gray(0x60)),
                }
            }
        }
        super::named_rect(
            ui.ctx(),
            area,
            lang.tr("field, made up", "поле, для примера"),
            Some(ui.layer_id()),
        );
    }
}

impl Mockup for Panel {
    fn name(&self) -> &'static str {
        "panel"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "view": format!("{:?}", self.view),
            "in_hand": self.in_hand,
            "settings": self.settings,
            "shown": self.shown(),
            "hidden": self.slots.iter().map(|slot| slot.hidden).collect::<Vec<_>>(),
        })
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.label(lang.tr(
            "Instead of the shake of the panel:",
            "Вместо жеста «потрясти панель»:",
        ));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.view, View::Line, lang.tr("Line", "Линия"));
            ui.selectable_value(&mut self.view, View::Block, lang.tr("Block", "Блок"));
        });
        if ui.input(|input| input.key_pressed(egui::Key::P)) && !self.menu.is_open() {
            self.settings = true;
        }
        if self.settings {
            ui.label(lang.tr(
                "Here the window of the palette opens: the mockup `palette`.",
                "Здесь открывается окно палитры: макет `palette`.",
            ));
        }
        ui.separator();
        let shown = self.shown();
        let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACING, SPACING);
            match self.view {
                View::Line => {
                    ui.horizontal(|ui| {
                        for number in &shown {
                            self.swatch(ui, lang, *number);
                        }
                        self.empty_place(ui, lang, false);
                    });
                }
                View::Block => {
                    ui.vertical(|ui| {
                        for row in block_rows(&shown) {
                            ui.horizontal(|ui| {
                                for cell in row {
                                    match cell {
                                        Some(number) => self.swatch(ui, lang, number),
                                        None => {
                                            ui.allocate_exact_size(BUTTON, Sense::hover());
                                        }
                                    }
                                }
                            });
                        }
                        self.empty_place(ui, lang, true);
                    });
                }
            }
        });
        named(ui.ctx(), frame.response, "panel of beads");
        ui.separator();
        self.draw_field(ui, lang);
        self.menu(ui.ctx());
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use super::*;

    fn window() -> egui_kittest::Harness<'static, Panel> {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_ui_state(|ui, mock: &mut Panel| mock.show(ui, Lang::En), Panel::new());
        harness.run();
        harness
    }

    /// A removed colour does not stand on the panel.
    #[test]
    fn a_removed_colour_is_not_on_the_panel() {
        let harness = window();

        assert_eq!(harness.state().shown(), vec![1, 2, 4, 5]);
        assert!(harness.query_by_label("3").is_none());
    }

    /// A tap takes a colour in hand; a tap on the colour in hand opens the
    /// menu.
    #[test]
    fn the_colour_in_hand_opens_the_menu() {
        let mut harness = window();

        harness.get_by_label("2").click();
        harness.run();
        assert_eq!(harness.state().in_hand, Some(1));
        assert!(!harness.state().menu.is_open());

        harness.get_by_label("2").click();
        harness.run();
        assert!(harness.state().menu.is_open());
    }

    /// The entry of the menu of a colour opens the settings.
    #[test]
    fn the_menu_of_a_colour_opens_the_settings() {
        let mut harness = window();

        harness.get_by_label("1").click();
        harness.run();
        harness.get_by_label("Palette settings").click();
        harness.run();

        assert!(harness.state().settings);
    }

    /// `0` is the way there with no colours: a second tap opens a menu of
    /// one entry.
    #[test]
    fn the_empty_place_opens_the_settings() {
        let mut harness = window();

        harness.get_by_label("0").click();
        harness.run();
        assert_eq!(harness.state().in_hand, None);
        assert!(!harness.state().menu.is_open());

        harness.get_by_label("0").click();
        harness.run();
        assert!(harness.query_by_label("Hide").is_none());
        harness.get_by_label("Palette settings").click();
        harness.run();

        assert!(harness.state().settings);
    }

    /// The key `P` opens the settings.
    #[test]
    fn the_key_opens_the_settings() {
        let mut harness = window();

        harness.key_press(egui::Key::P);
        harness.run();

        assert!(harness.state().settings);
    }

    /// A hidden colour stays on the panel, and its beads are still there.
    #[test]
    fn a_hidden_colour_stays_on_the_panel() {
        let mut harness = window();
        let beads = harness.state().beads(0);

        harness.get_by_label("1").click();
        harness.run();
        harness.get_by_label("Hide").click();
        harness.run();

        assert!(harness.state().slots[0].hidden);
        assert!(harness.query_by_label("1").is_some());
        assert_eq!(harness.state().beads(0), beads);
    }

    /// Deleting the beads leaves the colour.
    #[test]
    fn deleting_the_beads_leaves_the_colour() {
        let mut harness = window();
        let beads = harness.state().beads(0);

        harness.get_by_label("1").click();
        harness.run();
        let entry = delete_beads(Lang::En, beads);
        harness.get_by_label(&entry).click();
        harness.run();

        assert_eq!(harness.state().beads(0), 0);
        assert!(harness.state().slots[0].rgb.is_some());
    }
}
