//! The settings of the program: `settings.yaml` in the configuration
//! directory (`RULE config.dir`), read and written through `unifs`. They keep the place of each panel, how the
//! places of a palette stand, whether the empty places show their shadows
//! and whether the beads glow.
//!
//! A file this version cannot read is renamed to [`UNREADABLE`] beside it,
//! and the program starts from the defaults (`RULE config.legacy`).

use std::collections::BTreeMap;

use obusina_panel::{Orientation, Placement};
use serde::{Deserialize, Serialize};
use unifs::Files;

use crate::error::{AppError, Result};

/// The name of the file in the configuration directory.
pub const FILE: &str = "settings.yaml";
/// The name a file that cannot be read is kept aside under.
pub const UNREADABLE: &str = "settings.yaml.unreadable";

/// The settings, and the file they are kept in.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The configuration directory; `None` when the settings are not kept.
    #[serde(skip)]
    files: Option<Files>,
    /// The place of each panel, by its name.
    panels: BTreeMap<String, Panel>,
    /// How the places of a palette stand.
    palette_view: PaletteView,
    /// Whether the empty places of the field show their shadows.
    shadows: bool,
    /// Whether the beads are drawn round, lighter in the middle.
    glow: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            files: None,
            panels: BTreeMap::new(),
            palette_view: PaletteView::default(),
            shadows: true,
            glow: true,
        }
    }
}

/// How the places of a palette stand, on the panel of beads and in the
/// window of the palettes (decision 10 of `docs/palette.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaletteView {
    /// One line along the panel; turned with it.
    #[default]
    Line,
    /// Rows of three, as the keys of a numpad.
    Block,
}

/// The place of a panel, as the file holds it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Panel {
    /// The place in the room of the window, from 0 to 1 on each axis.
    at: [f32; 2],
    /// How its content is laid out.
    layout: Layout,
    /// Only its grip is shown.
    folded: bool,
}

/// An orientation, as the file names it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Layout {
    Row,
    Column,
}

impl Settings {
    /// The settings of the file in the configuration directory `files`, and
    /// what went wrong reading it. A file that is not there gives the
    /// defaults and no error.
    pub async fn load(files: Option<Files>) -> (Settings, Option<AppError>) {
        let (settings, error) = match &files {
            Some(files) => match read(files).await {
                Ok(settings) => (settings, None),
                Err(error) => (Settings::default(), Some(error)),
            },
            None => (Settings::default(), None),
        };
        (Settings { files, ..settings }, error)
    }

    /// The place kept for the panel `name`.
    pub fn panel(&self, name: &str) -> Option<Placement> {
        self.panels.get(name).map(|panel| (*panel).into())
    }

    /// Keeps the place of the panel `name`; gives the write of the file.
    pub fn keep_panel(&mut self, name: &str, placement: Placement) -> Option<Write> {
        self.panels.insert(name.to_owned(), placement.into());
        self.write()
    }

    /// How the places of a palette stand.
    pub fn palette_view(&self) -> PaletteView {
        self.palette_view
    }

    /// Keeps how the places of a palette stand; gives the write of the file
    /// when it changed.
    pub fn keep_palette_view(&mut self, view: PaletteView) -> Option<Write> {
        if view == self.palette_view {
            return None;
        }
        self.palette_view = view;
        self.write()
    }

    /// Whether the empty places of the field show their shadows.
    pub fn shadows(&self) -> bool {
        self.shadows
    }

    /// Keeps whether the empty places show their shadows; gives the write of
    /// the file when it changed.
    pub fn keep_shadows(&mut self, shadows: bool) -> Option<Write> {
        if shadows == self.shadows {
            return None;
        }
        self.shadows = shadows;
        self.write()
    }

    /// Whether the beads are drawn round, lighter in the middle.
    pub fn glow(&self) -> bool {
        self.glow
    }

    /// Keeps whether the beads glow; gives the write of the file when it
    /// changed.
    pub fn keep_glow(&mut self, glow: bool) -> Option<Write> {
        if glow == self.glow {
            return None;
        }
        self.glow = glow;
        self.write()
    }

    /// The write of the file as the settings are now; none when they are not
    /// kept.
    fn write(&self) -> Option<Write> {
        let files = self.files.clone()?;
        let text = serde_yaml_ng::to_string(self).expect("the settings are plain data");
        Some(Write { files, text })
    }
}

/// A write of the file of the settings, to be run away from the frame.
#[derive(Debug)]
pub struct Write {
    files: Files,
    text: String,
}

impl Write {
    /// Writes the file whole (`RULE config.write`).
    pub async fn run(self) -> Result<()> {
        self.files
            .write(FILE, self.text.as_bytes())
            .await
            .map_err(|source| AppError::SettingsWrite { source })
    }
}

/// Reads the file of `files`; the defaults when it is not there. A file
/// that does not parse is renamed to [`UNREADABLE`].
async fn read(files: &Files) -> Result<Settings> {
    let bytes = files
        .read(FILE)
        .await
        .map_err(|source| AppError::SettingsRead { source })?;
    let Some(bytes) = bytes else {
        return Ok(Settings::default());
    };
    match serde_yaml_ng::from_slice(&bytes) {
        Ok(settings) => Ok(settings),
        Err(source) => {
            if let Err(error) = files.rename(FILE, UNREADABLE).await {
                log::warn!("the unreadable settings stay in place: {error}");
            }
            Err(AppError::SettingsInvalid { source })
        }
    }
}

impl From<Placement> for Panel {
    fn from(placement: Placement) -> Panel {
        Panel {
            at: [placement.at.x, placement.at.y],
            layout: placement.orientation.into(),
            folded: placement.folded,
        }
    }
}

impl From<Panel> for Placement {
    fn from(panel: Panel) -> Placement {
        Placement {
            at: egui::vec2(panel.at[0], panel.at[1]),
            orientation: panel.layout.into(),
            folded: panel.folded,
        }
    }
}

impl From<Orientation> for Layout {
    fn from(orientation: Orientation) -> Layout {
        match orientation {
            Orientation::Horizontal => Layout::Row,
            Orientation::Vertical => Layout::Column,
        }
    }
}

impl From<Layout> for Orientation {
    fn from(layout: Layout) -> Orientation {
        match layout {
            Layout::Row => Orientation::Horizontal,
            Layout::Column => Orientation::Vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    /// The configuration directory of a scratch directory.
    fn config() -> (tempfile::TempDir, Files) {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let files = Files::at(directory.path().join("config"));
        (directory, files)
    }

    #[test]
    fn a_place_kept_is_read_back() {
        let (_directory, files) = config();
        let placement = Placement {
            folded: true,
            ..Placement::new(egui::vec2(0.25, 1.0), Orientation::Horizontal)
        };
        let (mut settings, error) = block_on(Settings::load(Some(files.clone())));
        assert!(error.is_none());
        let write = settings.keep_panel("tools", placement).expect("kept");
        block_on(write.run()).expect("written");

        let (again, error) = block_on(Settings::load(Some(files)));
        assert!(error.is_none());
        assert_eq!(again.panel("tools"), Some(placement));
        assert_eq!(again.panel("palette"), None);
    }

    #[test]
    fn a_file_that_does_not_parse_is_kept_aside() {
        let (_directory, files) = config();
        block_on(files.write(FILE, b"panels: [this is not a map")).expect("written");

        let (settings, error) = block_on(Settings::load(Some(files.clone())));

        assert!(matches!(error, Some(AppError::SettingsInvalid { .. })));
        assert_eq!(settings.panel("tools"), None);
        assert_eq!(block_on(files.read(FILE)).expect("it reads"), None);
        assert!(
            block_on(files.read(UNREADABLE))
                .expect("it reads")
                .is_some()
        );
    }

    /// A key the file does not carry reads as its default (`RULE config.default`).
    #[test]
    fn an_empty_file_gives_the_defaults() {
        let (_directory, files) = config();
        block_on(files.write(FILE, b"{}\n")).expect("written");

        let (settings, error) = block_on(Settings::load(Some(files)));

        assert!(error.is_none());
        assert_eq!(settings.panel("tools"), None);
    }

    #[test]
    fn the_view_of_the_palette_is_read_back() {
        let (_directory, files) = config();
        let (mut settings, _) = block_on(Settings::load(Some(files.clone())));
        assert_eq!(settings.palette_view(), PaletteView::Line);

        let write = settings
            .keep_palette_view(PaletteView::Block)
            .expect("kept");
        block_on(write.run()).expect("written");

        let (again, error) = block_on(Settings::load(Some(files)));
        assert!(error.is_none());
        assert_eq!(again.palette_view(), PaletteView::Block);
    }
}
