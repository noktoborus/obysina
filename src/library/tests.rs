//! Tests of the list, the trash and the import, on a scratch directory of
//! the disk; each test is one future, waited for.

use futures::executor::block_on;

use super::*;

/// A library in a scratch directory, kept as long as the directory is.
fn library() -> (tempfile::TempDir, Library) {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let library = Library::new(Files::at(directory.path()));
    (directory, library)
}

fn named(name: &str) -> Pattern {
    let mut pattern = crate::app::new_pattern();
    pattern.set_name(name);
    pattern
}

async fn names(library: &Library) -> Vec<String> {
    let list = library.list().await.expect("it lists");
    list.iter()
        .map(|entry| entry.pattern.name().to_owned())
        .collect()
}

#[test]
fn a_saved_pattern_opens_with_its_history() {
    let (_directory, library) = library();
    block_on(async {
        let mut pattern = named("one");
        let mut history = History::default();
        let cell = obusina_pattern::Cell { column: 1, row: 1 };
        history.paint(&mut pattern, &[cell], Some(obusina_pattern::ColourId(1)));

        library.save(&pattern, &history).await.expect("it saves");
        let (mut again, mut history) = library.open(pattern.id()).await.expect("it opens");

        assert_eq!(again, pattern);
        history.undo(&mut again);
        assert_eq!(again.get(cell), None);
    });
}

/// A rename keeps the history: it is written again for the new text.
#[test]
fn a_renamed_pattern_keeps_its_history() {
    let (_directory, library) = library();
    block_on(async {
        let mut pattern = named("one");
        let mut history = History::default();
        let cell = obusina_pattern::Cell { column: 1, row: 1 };
        history.paint(&mut pattern, &[cell], Some(obusina_pattern::ColourId(1)));
        library.save(&pattern, &history).await.expect("it saves");

        library
            .rename(pattern.id(), "two")
            .await
            .expect("it renames");

        let (again, history) = library.open(pattern.id()).await.expect("it opens");
        assert_eq!(again.name(), "two");
        assert!(history.can_undo());
    });
}

#[test]
fn the_trash_holds_a_pattern_until_it_is_restored() {
    let (_directory, library) = library();
    block_on(async {
        let pattern = named("one");
        library.add(&pattern).await.expect("it adds");

        library
            .to_trash(&[pattern.id()], SystemTime::now())
            .await
            .expect("it goes");
        assert!(library.list().await.expect("it lists").is_empty());
        assert_eq!(library.trash().await.expect("it lists").len(), 1);

        assert_eq!(
            library.restore(pattern.id()).await.expect("it comes back"),
            pattern.id()
        );
        assert_eq!(names(&library).await, ["one"]);
        assert!(library.trash().await.expect("it lists").is_empty());
    });
}

#[test]
fn a_pattern_goes_from_the_trash_after_its_days() {
    let (_directory, library) = library();
    block_on(async {
        let (old, young) = (named("old"), named("young"));
        library.add(&old).await.expect("it adds");
        library.add(&young).await.expect("it adds");
        let now = SystemTime::now();
        let long_ago = now - Duration::from_secs((TRASH_DAYS + 1) * DAY);
        library
            .to_trash(&[old.id()], long_ago)
            .await
            .expect("it goes");
        library.to_trash(&[young.id()], now).await.expect("it goes");

        assert_eq!(library.purge_old(now).await, vec![old.id()]);

        let left = library.trash().await.expect("it lists");
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].days_left(now), TRASH_DAYS);
    });
}

#[test]
fn a_duplicate_has_a_new_identifier_and_its_name() {
    let (_directory, library) = library();
    block_on(async {
        let pattern = named("one");
        library.add(&pattern).await.expect("it adds");

        let copy = library
            .duplicate(pattern.id(), "one (copy)")
            .await
            .expect("it copies");

        assert_ne!(copy.id(), pattern.id());
        assert!(copy.same_work(&pattern));
        assert_eq!(library.list().await.expect("it lists").len(), 2);
    });
}

/// An import keeps a new identifier, passes over the same work under a
/// known one even when renamed, and copies another work under a known one.
#[test]
fn an_import_keeps_passes_over_or_copies() {
    let (_directory, library) = library();
    block_on(async {
        let same = named("same");
        let changed = named("changed");
        library.add(&same).await.expect("it adds");
        library.add(&changed).await.expect("it adds");
        let mut renamed = same.clone();
        renamed.set_name("renamed elsewhere");
        let mut other = changed.clone();
        other.set_paper(obusina_pattern::Rgb([0, 0, 0]));
        let fresh = named("fresh");

        let report = library
            .import(vec![renamed, other, fresh.clone()])
            .await
            .expect("it imports");

        assert_eq!(report.new, vec![fresh.id()]);
        assert_eq!(report.copies.len(), 1);
        assert_ne!(report.copies[0], changed.id());
        assert_eq!(report.present, vec!["same".to_owned()]);
        assert_eq!(library.list().await.expect("it lists").len(), 4);
    });
}

/// A file that cannot be read is passed over; the rest of the list shows.
#[test]
fn a_damaged_file_is_passed_over() {
    let (directory, library) = library();
    block_on(async {
        library.add(&named("one")).await.expect("it adds");
        let damaged = Files::at(directory.path());
        let path = pattern_path(PATTERNS, PatternId::new());
        damaged.write(&path, b"{").await.expect("it writes");

        assert_eq!(names(&library).await, ["one"]);
    });
}

/// The one pattern the version before kept on android comes into the list
/// once, and its file is renamed.
#[test]
fn the_pattern_of_the_version_before_is_taken_in_once() {
    let (directory, library) = library();
    block_on(async {
        let files = Files::at(directory.path());
        let text = named("before").to_json().expect("it writes");
        files
            .write(OLD_PATTERN, text.as_bytes())
            .await
            .expect("it writes");

        library.take_in_old().await.expect("it is taken in");
        library.take_in_old().await.expect("nothing to take");

        assert_eq!(names(&library).await, ["before"]);
        assert_eq!(files.read(OLD_PATTERN).await.expect("it reads"), None);
    });
}
