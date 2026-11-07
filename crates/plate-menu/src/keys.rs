//! The keys the menu is walked by: one table, read by the menu and by a
//! caller that lists the keys of its window.

/// What a key does in the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Walk {
    /// Clears the query; with none, closes the menu.
    Close,
    /// The next entry, wrapping.
    Down,
    /// The entry before, wrapping.
    Up,
    /// Into the entries of the entry.
    StepIn,
    /// Out of the level, or the query cleared.
    StepOut,
    /// Forward by the plates shown.
    PageDown,
    /// Back by the plates shown.
    PageUp,
    /// The first entry; with a query, the caret to its start.
    First,
    /// The last entry; with a query, the caret to its end.
    Last,
    /// The character before the caret; with `Ctrl`, the whole query.
    Erase,
    /// Chooses the entry, with whatever is held.
    Choose,
    /// Nothing: taken so the toolkit does not hand the keyboard to a widget
    /// behind the menu.
    Stay,
}

/// Every key the menu takes, in the order it acts on them, and what each
/// does. Both the press and the release of each are taken out of the input.
pub const KEYS: &[(egui::Key, Walk)] = &[
    (egui::Key::Escape, Walk::Close),
    (egui::Key::ArrowDown, Walk::Down),
    (egui::Key::ArrowUp, Walk::Up),
    (egui::Key::ArrowRight, Walk::StepIn),
    (egui::Key::ArrowLeft, Walk::StepOut),
    (egui::Key::PageDown, Walk::PageDown),
    (egui::Key::PageUp, Walk::PageUp),
    (egui::Key::Home, Walk::First),
    (egui::Key::End, Walk::Last),
    (egui::Key::Backspace, Walk::Erase),
    (egui::Key::Enter, Walk::Choose),
    (egui::Key::Tab, Walk::Stay),
];
