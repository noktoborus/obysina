//! The settings of the program: `settings.yaml` in the configuration
//! directory (`RULE config.dir`). They keep the place of each panel, how the
//! places of a palette stand, whether the empty places show their shadows
//! and whether the beads glow.
//!
//! A file this version cannot read is renamed to [`UNREADABLE`] beside it,
//! and the program starts from the defaults (`RULE config.legacy`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use obusina_panel::{Orientation, Placement};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// The name of the file in the configuration directory.
#[cfg(not(target_arch = "wasm32"))]
pub const FILE: &str = "settings.yaml";
/// The name a file that cannot be read is kept aside under.
pub const UNREADABLE: &str = "settings.yaml.unreadable";
/// The name a new file is written under before it takes the place of the
/// old one (`RULE config.write`).
const PARTIAL: &str = "settings.yaml.partial";

/// The settings, and the file they are kept in.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The file; `None` when the settings are not kept.
    #[serde(skip)]
    path: Option<PathBuf>,
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
            path: None,
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
    /// The settings of the file at `path`, and what went wrong reading it.
    /// A file that is not there gives the defaults and no error.
    pub fn load(path: Option<PathBuf>) -> (Settings, Option<AppError>) {
        let (settings, error) = match &path {
            Some(path) if path.exists() => match read(path) {
                Ok(settings) => (settings, None),
                Err(error) => (Settings::default(), Some(error)),
            },
            _ => (Settings::default(), None),
        };
        (Settings { path, ..settings }, error)
    }

    /// The place kept for the panel `name`.
    pub fn panel(&self, name: &str) -> Option<Placement> {
        self.panels.get(name).map(|panel| (*panel).into())
    }

    /// Keeps the place of the panel `name` and writes the file.
    pub fn keep_panel(&mut self, name: &str, placement: Placement) -> Result<()> {
        self.panels.insert(name.to_owned(), placement.into());
        self.write()
    }

    /// How the places of a palette stand.
    pub fn palette_view(&self) -> PaletteView {
        self.palette_view
    }

    /// Keeps how the places of a palette stand and writes the file.
    pub fn keep_palette_view(&mut self, view: PaletteView) -> Result<()> {
        if view == self.palette_view {
            return Ok(());
        }
        self.palette_view = view;
        self.write()
    }

    /// Whether the empty places of the field show their shadows.
    pub fn shadows(&self) -> bool {
        self.shadows
    }

    /// Keeps whether the empty places show their shadows and writes the
    /// file.
    pub fn keep_shadows(&mut self, shadows: bool) -> Result<()> {
        if shadows == self.shadows {
            return Ok(());
        }
        self.shadows = shadows;
        self.write()
    }

    /// Whether the beads are drawn round, lighter in the middle.
    pub fn glow(&self) -> bool {
        self.glow
    }

    /// Keeps whether the beads glow and writes the file.
    pub fn keep_glow(&mut self, glow: bool) -> Result<()> {
        if glow == self.glow {
            return Ok(());
        }
        self.glow = glow;
        self.write()
    }

    /// Writes the file, through a partial file and a rename.
    fn write(&self) -> Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let text = serde_yaml_ng::to_string(self).expect("the settings are plain data");
        let partial = path.with_file_name(PARTIAL);
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&partial, text))
            .and_then(|()| std::fs::rename(&partial, path));
        written.map_err(|source| AppError::SettingsWrite { source })
    }
}

/// Reads the file at `path`. A file that does not parse is renamed to
/// [`UNREADABLE`].
fn read(path: &Path) -> Result<Settings> {
    let text = std::fs::read_to_string(path).map_err(|source| AppError::SettingsRead { source })?;
    serde_yaml_ng::from_str(&text).map_err(|source| {
        if let Err(error) = std::fs::rename(path, path.with_file_name(UNREADABLE)) {
            log::warn!("the unreadable settings stay in place: {error}");
        }
        AppError::SettingsInvalid { source }
    })
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

    #[test]
    fn a_place_kept_is_read_back() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let path = directory.path().join("config").join(FILE);
        let placement = Placement {
            folded: true,
            ..Placement::new(egui::vec2(0.25, 1.0), Orientation::Horizontal)
        };
        let (mut settings, error) = Settings::load(Some(path.clone()));
        assert!(error.is_none());
        settings.keep_panel("tools", placement).expect("written");

        let (again, error) = Settings::load(Some(path));
        assert!(error.is_none());
        assert_eq!(again.panel("tools"), Some(placement));
        assert_eq!(again.panel("palette"), None);
    }

    #[test]
    fn a_file_that_does_not_parse_is_kept_aside() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let path = directory.path().join(FILE);
        std::fs::write(&path, "panels: [this is not a map").expect("written");

        let (settings, error) = Settings::load(Some(path.clone()));

        assert!(matches!(error, Some(AppError::SettingsInvalid { .. })));
        assert_eq!(settings.panel("tools"), None);
        assert!(!path.exists());
        assert!(directory.path().join(UNREADABLE).exists());
    }

    /// A key the file does not carry reads as its default (`RULE config.default`).
    #[test]
    fn an_empty_file_gives_the_defaults() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let path = directory.path().join(FILE);
        std::fs::write(&path, "{}\n").expect("written");

        let (settings, error) = Settings::load(Some(path));

        assert!(error.is_none());
        assert_eq!(settings.panel("tools"), None);
    }

    #[test]
    fn the_view_of_the_palette_is_read_back() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let path = directory.path().join(FILE);
        let (mut settings, _) = Settings::load(Some(path.clone()));
        assert_eq!(settings.palette_view(), PaletteView::Line);

        settings
            .keep_palette_view(PaletteView::Block)
            .expect("written");

        let (again, error) = Settings::load(Some(path));
        assert!(error.is_none());
        assert_eq!(again.palette_view(), PaletteView::Block);
    }
}
