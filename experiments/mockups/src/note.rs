//! A note on a mockup: what the page sends, and what the receiver keeps
//! until the note is read.

use serde::{Deserialize, Serialize};

/// A widget a mockup names, and where it stands: left, top, right, bottom,
/// in points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    /// The name the mockup gave it.
    pub name: String,
    /// Where it stands.
    pub rect: [f32; 4],
}

/// A note as the page sends it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// The name of the mockup.
    pub mock: String,
    /// What the user wrote.
    pub text: String,
    /// The part of the page pointed at, in points: left, top, right,
    /// bottom. A tap is a box of no size.
    pub rect: [f32; 4],
    /// The size of the page, in points.
    pub window: [f32; 2],
    /// Pixels of the picture to a point.
    pub pixels_per_point: f32,
    /// The widgets the mockup named that the box touches.
    pub widgets: Vec<Widget>,
    /// What the mockup showed: its own data, from `Mockup::state`.
    pub state: serde_json::Value,
}

/// A picture of the page: its size in pixels, and its pixels, red, green,
/// blue and alpha, in base64.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    /// Width and height.
    pub size: [usize; 2],
    /// The pixels.
    pub rgba: String,
}

/// What the page posts to the receiver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Posted {
    /// The note.
    pub note: Note,
    /// The page with the box drawn on it, if the picture was taken.
    pub shot: Option<Shot>,
}

/// A note the receiver keeps, until it is read and deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kept {
    /// Its number, one more than the last one kept.
    pub id: u32,
    /// When it came, in seconds since 1970.
    pub at: u64,
    /// The note.
    #[serde(flatten)]
    pub note: Note,
    /// The file of the picture, beside the note.
    pub shot: Option<String>,
}
