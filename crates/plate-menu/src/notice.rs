//! The line above the plates: a text, or a widget of the caller.

use std::sync::Arc;

/// What stands above the plates, what the menu is about.
///
/// A text is drawn strongly and wraps. A widget is the caller's: it is drawn
/// in the width of the menu, as high as it makes itself, on every frame the
/// menu is shown. Every text that converted into a `String` before converts
/// into a notice, so `notice("text")` reads as it always did.
#[derive(Clone)]
pub enum Notice {
    /// A text.
    Text(String),
    /// A widget of the caller.
    Widget(Arc<dyn Fn(&mut egui::Ui) + Send + Sync>),
}

impl PartialEq for Notice {
    fn eq(&self, other: &Notice) -> bool {
        match (self, other) {
            (Notice::Text(text), Notice::Text(other)) => text == other,
            (Notice::Widget(show), Notice::Widget(other)) => Arc::ptr_eq(show, other),
            _ => false,
        }
    }
}

impl Eq for Notice {}

impl Notice {
    /// A notice drawn by `show`.
    pub fn widget(show: impl Fn(&mut egui::Ui) + Send + Sync + 'static) -> Notice {
        Notice::Widget(Arc::new(show))
    }

    /// True for a text of nothing but blanks, which is no notice.
    pub(crate) fn is_blank(&self) -> bool {
        matches!(self, Notice::Text(text) if text.trim().is_empty())
    }

    /// Draws the notice and the line under it.
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        match self {
            Notice::Text(text) => {
                ui.add(egui::Label::new(egui::RichText::new(text).strong()).wrap());
            }
            Notice::Widget(show) => show(ui),
        }
        ui.separator();
    }
}

impl std::fmt::Debug for Notice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notice::Text(text) => formatter.debug_tuple("Text").field(text).finish(),
            Notice::Widget(_) => formatter.write_str("Widget"),
        }
    }
}

impl From<String> for Notice {
    fn from(text: String) -> Notice {
        Notice::Text(text)
    }
}

impl From<&str> for Notice {
    fn from(text: &str) -> Notice {
        Notice::Text(text.to_string())
    }
}

impl From<std::borrow::Cow<'_, str>> for Notice {
    fn from(text: std::borrow::Cow<'_, str>) -> Notice {
        Notice::Text(text.into_owned())
    }
}

impl From<&String> for Notice {
    fn from(text: &String) -> Notice {
        Notice::Text(text.clone())
    }
}
