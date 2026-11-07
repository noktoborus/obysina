//! A pattern: a grid, a palette and the beads put on a field with no end.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{PatternError, Result};
use crate::grid::{Bounds, Cell, Grid};
use crate::id::PatternId;
use crate::palette::{Colour, ColourId, NO_BEAD, Palette, Rgb};
use crate::ruling::Ruling;

/// The width of a bead to its height, for a pattern that says nothing else.
pub const DEFAULT_BEAD_RATIO: f32 = 1.0;

/// The paper of a new pattern, the ground of its sheet: a neutral grey, so
/// that a colour looks the same in every theme and a black bead is seen.
pub const DEFAULT_PAPER: Rgb = Rgb([0xd6, 0xd4, 0xce]);

/// The turns a pattern is shown at, in degrees.
pub const TURNS: [u16; 4] = [0, 90, 180, 270];

/// The next revision any pattern takes: one counter for all of them, so that
/// two patterns that are not clones never share a revision.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

/// A revision no pattern had before.
fn next_revision() -> u64 {
    NEXT_REVISION.fetch_add(1, Ordering::Relaxed)
}

/// A bead pattern.
///
/// Only the cells that hold a bead are kept; every other cell of the field
/// is empty. Two patterns are equal when they have the same identifier and
/// name and hold the same work ([`Pattern::same_work`]); the revision is not
/// compared.
#[derive(Debug, Clone)]
pub struct Pattern {
    id: PatternId,
    name: String,
    grid: Grid,
    ratio: f32,
    palettes: Vec<Palette>,
    beads: BTreeMap<Cell, ColourId>,
    sheet: Option<Bounds>,
    ruling: Ruling,
    paper: Rgb,
    turn: u16,
    locked: bool,
    revision: u64,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.locked == other.locked
            && self.same_work(other)
    }
}

impl Pattern {
    /// True when two patterns hold the same work: every field but the
    /// identifier, the name and the revision is the same. A pattern renamed
    /// elsewhere is the same work.
    pub fn same_work(&self, other: &Pattern) -> bool {
        self.grid == other.grid
            && self.ratio == other.ratio
            && self.palettes == other.palettes
            && self.beads == other.beads
            && self.sheet == other.sheet
            && self.ruling == other.ruling
            && self.paper == other.paper
            && self.turn == other.turn
    }
}

impl Pattern {
    /// A pattern with no beads and one palette of these colours.
    pub fn new(grid: Grid, ratio: f32, colours: Vec<Colour>) -> Result<Pattern> {
        let places = colours.into_iter().map(Some).collect();
        Pattern::with_palettes(grid, ratio, vec![Palette::new(places)?])
    }

    /// A pattern with no beads and these palettes, at least one.
    pub fn with_palettes(grid: Grid, ratio: f32, palettes: Vec<Palette>) -> Result<Pattern> {
        if !(ratio.is_finite() && ratio > 0.0) {
            return Err(PatternError::Ratio { ratio });
        }
        check(&palettes)?;
        Ok(Pattern {
            id: PatternId::new(),
            name: String::new(),
            grid,
            ratio,
            palettes,
            beads: BTreeMap::new(),
            sheet: None,
            ruling: Ruling::default(),
            paper: DEFAULT_PAPER,
            turn: 0,
            locked: false,
            revision: next_revision(),
        })
    }

    /// The identifier, kept in the file.
    pub fn id(&self) -> PatternId {
        self.id
    }

    /// Takes the identifier a file gives.
    pub fn set_id(&mut self, id: PatternId) {
        self.id = id;
    }

    /// The name the person gave it; empty when none was given.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gives it a name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        if self.name != name {
            self.name = name;
            self.revision = next_revision();
        }
    }

    /// The kind of grid.
    pub fn grid(&self) -> Grid {
        self.grid
    }

    /// Changes the kind of grid; the beads keep their cells.
    pub fn set_grid(&mut self, grid: Grid) {
        if self.grid != grid {
            self.grid = grid;
            self.revision = next_revision();
        }
    }

    /// The width of a bead to its height.
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    /// A number that changes with every change of the pattern, and that no
    /// other pattern has unless it is a clone of this one: what a drawing of
    /// the pattern can be kept by.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The palettes, at least one.
    pub fn palettes(&self) -> &[Palette] {
        &self.palettes
    }

    /// The colour with this id, in any palette.
    pub fn colour(&self, id: ColourId) -> Option<&Colour> {
        self.palettes
            .iter()
            .flat_map(|palette| palette.places())
            .flatten()
            .find(|colour| colour.id == id)
    }

    /// The sheet: the size of the work area set by hand, where it stands on
    /// the field. `None` until one is set.
    pub fn sheet(&self) -> Option<Bounds> {
        self.sheet
    }

    /// Sets the sheet.
    pub fn set_sheet(&mut self, sheet: Option<Bounds>) {
        if self.sheet != sheet {
            self.sheet = sheet;
            self.revision = next_revision();
        }
    }

    /// The lines to count the beads by.
    pub fn ruling(&self) -> Ruling {
        self.ruling
    }

    /// Sets the lines to count the beads by.
    pub fn set_ruling(&mut self, ruling: Ruling) {
        if self.ruling != ruling {
            self.ruling = ruling;
            self.revision = next_revision();
        }
    }

    /// The colour of the paper, the ground of the sheet.
    pub fn paper(&self) -> Rgb {
        self.paper
    }

    /// Sets the colour of the paper.
    pub fn set_paper(&mut self, paper: Rgb) {
        if self.paper != paper {
            self.paper = paper;
            self.revision = next_revision();
        }
    }

    /// How far the pattern is shown turned clockwise: 0, 90, 180 or 270
    /// degrees. The beads do not move; it is kept so the pattern opens as it
    /// was shown.
    pub fn turn(&self) -> u16 {
        self.turn
    }

    /// True when the pattern is only to be looked at: the window shows no
    /// palette and no tool that changes it.
    pub fn locked(&self) -> bool {
        self.locked
    }

    /// Locks the pattern against changes, or lets it be changed again.
    pub fn set_locked(&mut self, locked: bool) {
        if self.locked != locked {
            self.locked = locked;
            self.revision = next_revision();
        }
    }

    /// Sets the turn it is shown at, in degrees; a turn that is not a
    /// quarter is refused.
    pub fn set_turn(&mut self, degrees: u16) -> Result<()> {
        if !TURNS.contains(&degrees) {
            return Err(PatternError::Turn { degrees });
        }
        if self.turn != degrees {
            self.turn = degrees;
            self.revision = next_revision();
        }
        Ok(())
    }

    /// The work area: the box around the sheet and the beads. A bead past
    /// the sheet grows it; when that bead goes, the work area is the sheet
    /// again. `None` with no sheet and no beads.
    pub fn work_area(&self) -> Option<Bounds> {
        match (self.sheet, self.bounds()) {
            (Some(sheet), Some(beads)) => Some(sheet.union(beads)),
            (sheet, beads) => sheet.or(beads),
        }
    }

    /// The active area: the box around the beads. `None` when there are none.
    pub fn bounds(&self) -> Option<Bounds> {
        let mut cells = self.beads.keys().copied();
        let first = cells.next()?;
        Some(cells.fold(Bounds::of(first), Bounds::with))
    }

    /// Every bead and its colour.
    pub fn beads(&self) -> impl Iterator<Item = (Cell, ColourId)> + '_ {
        self.beads.iter().map(|(cell, colour)| (*cell, *colour))
    }

    /// The colour of a bead; `None` for an empty cell.
    pub fn get(&self, cell: Cell) -> Option<ColourId> {
        self.beads.get(&cell).copied()
    }

    /// Puts a colour, or nothing, in a cell. Gives back what was there.
    ///
    /// An id no colour has changes nothing.
    pub fn set(&mut self, cell: Cell, colour: Option<ColourId>) -> Option<ColourId> {
        if colour.is_some_and(|id| self.colour(id).is_none()) {
            return self.get(cell);
        }
        let before = match colour {
            Some(id) => self.beads.insert(cell, id),
            None => self.beads.remove(&cell),
        };
        if before != colour {
            self.revision = next_revision();
        }
        before
    }

    /// How many beads a colour has.
    pub fn count(&self, id: ColourId) -> usize {
        self.beads.values().filter(|bead| **bead == id).count()
    }

    /// Puts a colour in a place and gives back its id. A colour there changes.
    /// A removed colour there keeps its id, so its beads take the new colour.
    /// An empty place takes a new id.
    pub fn put_colour(&mut self, palette: usize, place: usize, rgb: Rgb) -> Result<ColourId> {
        let next = self.next_id();
        let slot = self.place_mut(palette, place)?;
        let colour = slot.get_or_insert_with(|| Colour::new(next, rgb));
        colour.rgb = Some(rgb);
        let id = colour.id;
        self.revision = next_revision();
        Ok(id)
    }

    /// Two places of a palette change what they hold. No bead changes.
    pub fn swap_places(&mut self, palette: usize, first: usize, second: usize) -> Result<()> {
        self.place_mut(palette, first)?;
        self.place_mut(palette, second)?;
        self.palettes[palette].places_mut().swap(first, second);
        self.revision = next_revision();
        Ok(())
    }

    /// Moves what stands in a place to the first free place of another palette,
    /// or of a new one with `None`. No bead changes. A palette left with
    /// nothing goes, and the index of those after it falls by one.
    pub fn move_colour(
        &mut self,
        palette: usize,
        place: usize,
        target: Option<usize>,
    ) -> Result<()> {
        if self.place_mut(palette, place)?.is_none() {
            return Err(PatternError::Place { palette, place });
        }
        if target == Some(palette) {
            return Ok(());
        }
        let target = match target {
            Some(target) => target,
            None => {
                self.palettes.push(Palette::new(Vec::new())?);
                self.palettes.len() - 1
            }
        };
        let full = PatternError::Full { palette: target };
        let free = self
            .palettes
            .get(target)
            .and_then(Palette::first_free)
            .ok_or(full)?;
        let colour = self.palettes[palette].places_mut()[place].take();
        self.palettes[target].places_mut()[free] = colour;
        self.drop_if_empty(palette);
        self.revision = next_revision();
        Ok(())
    }

    /// Removes the colour of a place. With beads it keeps its id and its
    /// place, with no rgb, and its beads stay. With none its place is empty,
    /// and a palette left with nothing goes.
    pub fn remove_colour(&mut self, palette: usize, place: usize) -> Result<()> {
        let id = self
            .place_mut(palette, place)?
            .as_ref()
            .map(|colour| colour.id);
        let Some(id) = id else {
            return Err(PatternError::Place { palette, place });
        };
        let has_beads = self.count(id) > 0;
        let slot = self.place_mut(palette, place)?;
        if has_beads {
            if let Some(colour) = slot.as_mut() {
                colour.rgb = None;
                colour.hidden = false;
            }
        } else {
            *slot = None;
            self.drop_if_empty(palette);
        }
        self.revision = next_revision();
        Ok(())
    }

    /// Hides a colour and its beads, or shows them. An id no colour has
    /// changes nothing.
    pub fn set_hidden(&mut self, id: ColourId, hidden: bool) {
        let found = self
            .palettes
            .iter_mut()
            .flat_map(|palette| palette.places_mut())
            .flatten()
            .find(|colour| colour.id == id);
        if let Some(colour) = found
            && colour.hidden != hidden
        {
            colour.hidden = hidden;
            self.revision = next_revision();
        }
    }

    /// Deletes the beads of a colour. The colour stays.
    pub fn delete_beads(&mut self, id: ColourId) {
        let before = self.beads.len();
        self.beads.retain(|_, bead| *bead != id);
        if self.beads.len() != before {
            self.revision = next_revision();
        }
    }

    /// Puts back palettes kept before, as an undo does. They are refused
    /// when a bead would hold an id none of their colours has.
    pub fn set_palettes(&mut self, palettes: Vec<Palette>) -> Result<()> {
        check(&palettes)?;
        let known = |id: &ColourId| {
            palettes
                .iter()
                .flat_map(Palette::places)
                .flatten()
                .any(|colour| colour.id == *id)
        };
        if let Some(lost) = self.beads.values().find(|id| !known(id)) {
            return Err(PatternError::Lost { id: lost.0 });
        }
        if palettes != self.palettes {
            self.palettes = palettes;
            self.revision = next_revision();
        }
        Ok(())
    }

    /// An id no colour has.
    fn next_id(&self) -> ColourId {
        let last = self
            .palettes
            .iter()
            .flat_map(|palette| palette.places())
            .flatten()
            .map(|colour| colour.id.0)
            .max()
            .unwrap_or(NO_BEAD);
        ColourId(last + 1)
    }

    /// A place of a palette, to change.
    fn place_mut(&mut self, palette: usize, place: usize) -> Result<&mut Option<Colour>> {
        self.palettes
            .get_mut(palette)
            .and_then(|found| found.places_mut().get_mut(place))
            .ok_or(PatternError::Place { palette, place })
    }

    /// Takes away a palette with nothing in it, unless it is the last one.
    fn drop_if_empty(&mut self, palette: usize) {
        if self.palettes.len() > 1 && self.palettes[palette].is_empty() {
            self.palettes.remove(palette);
        }
    }
}

/// Palettes a pattern can hold: one or more, and no id twice or reserved.
fn check(palettes: &[Palette]) -> Result<()> {
    if palettes.is_empty() {
        return Err(PatternError::NoPalette);
    }
    let mut seen = vec![NO_BEAD];
    for colour in palettes.iter().flat_map(Palette::places).flatten() {
        if seen.contains(&colour.id.0) {
            return Err(PatternError::DuplicateId { id: colour.id.0 });
        }
        seen.push(colour.id.0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Rgb;

    const FIRST: ColourId = ColourId(1);
    const SECOND: ColourId = ColourId(2);

    fn colour(id: ColourId) -> Colour {
        Colour::new(id, Rgb([0, 0, 0]))
    }

    fn pattern() -> Pattern {
        Pattern::new(
            Grid::Loom,
            DEFAULT_BEAD_RATIO,
            vec![colour(FIRST), colour(SECOND)],
        )
        .expect("a pattern")
    }

    const CELL: Cell = Cell { column: 0, row: 0 };

    #[test]
    fn a_bead_is_counted_by_its_colour() {
        let mut pattern = pattern();
        pattern.set(Cell { column: 0, row: 0 }, Some(SECOND));
        pattern.set(Cell { column: 2, row: 1 }, Some(SECOND));
        pattern.set(Cell { column: -1, row: 1 }, Some(FIRST));

        assert_eq!((pattern.count(FIRST), pattern.count(SECOND)), (1, 2));
        assert_eq!(pattern.set(Cell { column: 0, row: 0 }, None), Some(SECOND));
        assert_eq!((pattern.count(FIRST), pattern.count(SECOND)), (1, 1));
    }

    #[test]
    fn the_active_area_is_the_box_around_the_beads() {
        let mut pattern = pattern();
        assert_eq!(pattern.bounds(), None);

        pattern.set(Cell { column: 3, row: -2 }, Some(FIRST));
        pattern.set(Cell { column: -1, row: 4 }, Some(SECOND));
        let bounds = pattern.bounds().expect("there are beads");
        assert_eq!(
            bounds.min,
            Cell {
                column: -1,
                row: -2
            }
        );
        assert_eq!(bounds.max, Cell { column: 3, row: 4 });

        pattern.set(Cell { column: -1, row: 4 }, None);
        assert_eq!(
            pattern.bounds(),
            Some(Bounds::of(Cell { column: 3, row: -2 }))
        );
    }

    /// The work area is the sheet and the beads, never kept: a bead past
    /// the sheet grows it, and taken away gives the sheet back.
    #[test]
    fn the_work_area_is_the_sheet_and_the_beads() {
        let mut pattern = pattern();
        let sheet = Bounds {
            min: Cell { column: 0, row: 0 },
            max: Cell { column: 3, row: 2 },
        };
        pattern.set_sheet(Some(sheet));
        assert_eq!(pattern.work_area(), Some(sheet));

        let past = Cell { column: 6, row: 1 };
        pattern.set(past, Some(FIRST));
        assert_eq!(pattern.work_area().map(|area| area.width()), Some(7));

        pattern.set(past, None);
        assert_eq!(pattern.work_area(), Some(sheet));
    }

    #[test]
    fn a_change_gives_a_new_revision_and_no_change_none() {
        let mut pattern = pattern();
        let mut other = self::pattern();
        other.set_id(pattern.id());
        let first = pattern.revision();
        assert_ne!(first, other.revision());
        assert_eq!(pattern, other, "the revision is not compared");

        pattern.set(Cell { column: 0, row: 0 }, None);
        pattern.set_grid(Grid::Loom);
        assert_eq!(pattern.revision(), first);

        pattern.set(Cell { column: 0, row: 0 }, Some(FIRST));
        let second = pattern.revision();
        assert_ne!(second, first);

        pattern.set_grid(Grid::Peyote);
        assert_ne!(pattern.revision(), second);
    }

    #[test]
    fn an_id_used_twice_is_refused() {
        let error = Pattern::new(Grid::Loom, 1.0, vec![colour(FIRST), colour(FIRST)]);

        assert!(matches!(error, Err(PatternError::DuplicateId { id: 1 })));
    }

    #[test]
    fn the_number_of_no_bead_is_not_a_colour() {
        let error = Pattern::new(Grid::Loom, 1.0, vec![colour(ColourId(NO_BEAD))]);

        assert!(matches!(error, Err(PatternError::DuplicateId { .. })));
    }

    #[test]
    fn a_pattern_has_a_palette() {
        let error = Pattern::with_palettes(Grid::Loom, 1.0, Vec::new());

        assert!(matches!(error, Err(PatternError::NoPalette)));
    }

    #[test]
    fn an_id_no_colour_has_puts_no_bead() {
        let mut pattern = pattern();

        pattern.set(CELL, Some(ColourId(9)));

        assert_eq!(pattern.get(CELL), None);
    }

    #[test]
    fn a_swap_moves_colours_and_no_bead() {
        let mut pattern = pattern();
        pattern.set(CELL, Some(FIRST));

        pattern.swap_places(0, 0, 1).expect("the places are there");

        assert_eq!(
            pattern.palettes()[0].places()[1]
                .as_ref()
                .map(|colour| colour.id),
            Some(FIRST)
        );
        assert_eq!(pattern.get(CELL), Some(FIRST));
    }

    #[test]
    fn a_place_that_is_not_there_is_refused() {
        let mut pattern = pattern();

        assert!(matches!(
            pattern.swap_places(0, 0, crate::palette::MAX_COLOURS),
            Err(PatternError::Place { .. })
        ));
        assert!(matches!(
            pattern.remove_colour(1, 0),
            Err(PatternError::Place { .. })
        ));
    }

    /// A move makes a new palette, and the colour keeps its beads.
    #[test]
    fn a_move_to_a_new_palette_keeps_the_beads() {
        let mut pattern = pattern();
        pattern.set(CELL, Some(SECOND));

        pattern.move_colour(0, 1, None).expect("the colour moves");

        assert_eq!(pattern.palettes().len(), 2);
        assert!(pattern.palettes()[0].places()[1].is_none());
        assert_eq!(
            pattern.palettes()[1].places()[0]
                .as_ref()
                .map(|colour| colour.id),
            Some(SECOND)
        );
        assert_eq!(pattern.get(CELL), Some(SECOND));
        assert!(pattern.colour(SECOND).is_some());
    }

    /// A palette left with nothing goes, unless it is the last one.
    #[test]
    fn a_palette_left_with_nothing_goes() {
        let mut pattern = pattern();
        pattern.move_colour(0, 1, None).expect("the colour moves");

        pattern
            .move_colour(1, 0, Some(0))
            .expect("the colour moves");

        assert_eq!(pattern.palettes().len(), 1);
        assert_eq!(
            pattern.palettes()[0].places()[1]
                .as_ref()
                .map(|colour| colour.id),
            Some(SECOND)
        );
    }

    #[test]
    fn a_colour_does_not_go_to_a_full_palette() {
        let mut pattern = pattern();
        pattern.move_colour(0, 1, None).expect("the colour moves");
        let full: Vec<Colour> = (10..19).map(|id| colour(ColourId(id))).collect();
        pattern.palettes[0] =
            Palette::new(full.into_iter().map(Some).collect()).expect("a palette");

        assert!(matches!(
            pattern.move_colour(1, 0, Some(0)),
            Err(PatternError::Full { palette: 0 })
        ));
        assert!(pattern.colour(SECOND).is_some(), "the colour is not lost");
    }

    /// A colour with beads stays as a hatch; a new colour there colours them.
    #[test]
    fn a_removed_colour_keeps_its_beads_and_its_id() {
        let mut pattern = pattern();
        pattern.set(CELL, Some(FIRST));

        pattern.remove_colour(0, 0).expect("the colour goes");

        let removed = pattern.colour(FIRST).expect("it keeps its place");
        assert_eq!(removed.seen(), None);
        assert_eq!(pattern.get(CELL), Some(FIRST));

        let rgb = Rgb([9, 9, 9]);
        assert_eq!(pattern.put_colour(0, 0, rgb).expect("a colour"), FIRST);
        assert_eq!(pattern.colour(FIRST).and_then(Colour::seen), Some(rgb));
    }

    #[test]
    fn a_removed_colour_with_no_beads_leaves_the_place_empty() {
        let mut pattern = pattern();

        pattern.remove_colour(0, 0).expect("the colour goes");

        assert!(pattern.palettes()[0].places()[0].is_none());
        assert!(pattern.colour(FIRST).is_none());
    }

    /// Only the last colour with nothing else: the palette stays when it is
    /// the only one, and hatched beads keep a palette.
    #[test]
    fn a_palette_with_hatched_beads_stays() {
        let mut pattern = pattern();
        pattern.move_colour(0, 1, None).expect("the colour moves");
        pattern.set(CELL, Some(SECOND));

        pattern.remove_colour(1, 0).expect("the colour goes");

        assert_eq!(pattern.palettes().len(), 2);
    }

    #[test]
    fn an_empty_place_takes_a_new_id() {
        let mut pattern = pattern();

        let id = pattern
            .put_colour(0, 4, Rgb([1, 1, 1]))
            .expect("the place is there");

        assert_eq!(id, ColourId(3));
    }

    #[test]
    fn a_hidden_colour_keeps_its_rgb_and_can_have_beads() {
        let mut pattern = pattern();

        pattern.set_hidden(FIRST, true);
        pattern.set(CELL, Some(FIRST));

        let hidden = pattern.colour(FIRST).expect("it stays");
        assert!(hidden.rgb.is_some());
        assert_eq!(hidden.seen(), None);
        assert_eq!(pattern.get(CELL), Some(FIRST));

        pattern.set_hidden(FIRST, false);
        assert!(pattern.colour(FIRST).and_then(Colour::seen).is_some());
    }

    #[test]
    fn deleting_the_beads_leaves_the_colour() {
        let mut pattern = pattern();
        pattern.set(CELL, Some(FIRST));
        pattern.set(Cell { column: 1, row: 0 }, Some(SECOND));

        pattern.delete_beads(FIRST);

        assert_eq!(pattern.count(FIRST), 0);
        assert_eq!(pattern.count(SECOND), 1);
        assert!(pattern.colour(FIRST).is_some());
    }

    #[test]
    fn palettes_put_back_are_checked_against_the_beads() {
        let mut pattern = pattern();
        let kept = pattern.palettes().to_vec();
        pattern.move_colour(0, 1, None).expect("the colour moves");
        pattern.set(CELL, Some(SECOND));

        pattern
            .set_palettes(kept.clone())
            .expect("the beads have colours");
        assert_eq!(pattern.palettes(), kept.as_slice());

        let only_first = Palette::new(vec![Some(colour(FIRST))]).expect("a palette");
        assert!(matches!(
            pattern.set_palettes(vec![only_first]),
            Err(PatternError::Lost { id: 2 })
        ));
        assert_eq!(pattern.palettes(), kept.as_slice(), "nothing changed");
    }
}
