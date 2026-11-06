//! What stands right of the label of an entry: a text, or a widget of the
//! caller.

use std::sync::Arc;

/// What stands right of the label: a key, a value, a count, or a widget.
///
/// A text is drawn weak. A widget is the caller's: it is drawn in a box
/// `width` wide and as high as the text of the plate, on every frame the
/// entry is shown. It shows a value, a colour say; a click on it is a click
/// on the plate, so it holds nothing that takes one. Every text that
/// converted into a `String` before converts into a detail, so
/// `detail("text")` reads as it always did.
#[derive(Clone)]
pub enum Detail {
    /// A text.
    Text(String),
    /// A widget of the caller and the width of its box.
    Widget {
        /// The width of the box.
        width: f32,
        /// What draws into the box.
        show: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>,
    },
}

impl Detail {
    /// A detail drawn by `show` in a box `width` wide.
    pub fn widget(width: f32, show: impl Fn(&mut egui::Ui) + Send + Sync + 'static) -> Detail {
        Detail::Widget {
            width,
            show: Arc::new(show),
        }
    }

    /// True for an empty text, which is no detail.
    pub fn is_empty(&self) -> bool {
        matches!(self, Detail::Text(text) if text.is_empty())
    }
}

impl Default for Detail {
    fn default() -> Detail {
        Detail::Text(String::new())
    }
}

/// Two widgets are equal when they are one closure of one width.
impl PartialEq for Detail {
    fn eq(&self, other: &Detail) -> bool {
        match (self, other) {
            (Detail::Text(one), Detail::Text(other)) => one == other,
            (
                Detail::Widget { width, show },
                Detail::Widget {
                    width: other_width,
                    show: other_show,
                },
            ) => width == other_width && Arc::ptr_eq(show, other_show),
            _ => false,
        }
    }
}

impl Eq for Detail {}

impl std::fmt::Debug for Detail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Detail::Text(text) => formatter.debug_tuple("Text").field(text).finish(),
            Detail::Widget { width, .. } => formatter
                .debug_struct("Widget")
                .field("width", width)
                .finish_non_exhaustive(),
        }
    }
}

impl From<String> for Detail {
    fn from(text: String) -> Detail {
        Detail::Text(text)
    }
}

impl From<&str> for Detail {
    fn from(text: &str) -> Detail {
        Detail::Text(text.to_string())
    }
}

impl From<std::borrow::Cow<'_, str>> for Detail {
    fn from(text: std::borrow::Cow<'_, str>) -> Detail {
        Detail::Text(text.into_owned())
    }
}

impl From<&String> for Detail {
    fn from(text: &String) -> Detail {
        Detail::Text(text.clone())
    }
}
