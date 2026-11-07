//! The files of a scratch directory of the disk. The browser runs the same
//! calls on the private file system of the origin; no test drives one here.

use pollster::block_on;

use crate::{Error, Files};

fn files() -> (tempfile::TempDir, Files) {
    let dir = tempfile::tempdir().expect("a scratch directory");
    let files = Files::at(dir.path().join("root"));
    (dir, files)
}

#[test]
fn a_file_written_is_read_back_and_listed() {
    let (_dir, files) = files();
    assert_eq!(block_on(files.read("a/b.json")).expect("it reads"), None);

    block_on(files.write("a/b.json", b"one")).expect("it is written");
    block_on(files.write("a/b.json", b"two")).expect("it is written again");

    assert_eq!(
        block_on(files.read("a/b.json")).expect("it reads"),
        Some(b"two".to_vec())
    );
    let listed = block_on(files.list("a")).expect("it lists");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "b.json");
    assert_eq!(
        block_on(files.modified("a/b.json")).expect("it reads"),
        Some(listed[0].modified)
    );
}

#[test]
fn a_directory_that_is_not_there_lists_nothing() {
    let (_dir, files) = files();

    assert!(block_on(files.list("none")).expect("it lists").is_empty());
    assert_eq!(block_on(files.modified("none/a")).expect("it reads"), None);
}

#[test]
fn a_removed_file_is_gone_and_a_second_removal_is_no_failure() {
    let (_dir, files) = files();
    block_on(files.write("a", b"x")).expect("it is written");

    block_on(files.remove("a")).expect("it is removed");
    block_on(files.remove("a")).expect("nothing to remove");

    assert_eq!(block_on(files.read("a")).expect("it reads"), None);
}

#[test]
fn a_rename_moves_a_file_to_another_directory() {
    let (_dir, files) = files();
    block_on(files.write("list/a", b"x")).expect("it is written");
    block_on(files.write("trash/a", b"old")).expect("it is written");

    block_on(files.rename("list/a", "trash/a")).expect("it moves");
    block_on(files.rename("list/none", "trash/none")).expect("nothing moves");

    assert_eq!(block_on(files.read("list/a")).expect("it reads"), None);
    assert_eq!(
        block_on(files.read("trash/a")).expect("it reads"),
        Some(b"x".to_vec())
    );
    assert_eq!(block_on(files.read("trash/none")).expect("it reads"), None);
}

#[test]
fn a_directory_under_the_root_is_files_of_its_own() {
    let (_dir, files) = files();
    let sub = files.sub("cache/pictures").expect("a path");

    block_on(sub.write("1.png", b"x")).expect("it is written");

    assert_eq!(
        block_on(files.read("cache/pictures/1.png")).expect("it reads"),
        Some(b"x".to_vec())
    );
}

#[test]
fn a_path_out_of_the_root_is_refused() {
    let (_dir, files) = files();

    for path in ["../a", "/a", "a//b", "a/./b", ""] {
        assert!(
            matches!(block_on(files.write(path, b"x")), Err(Error::Path { .. })),
            "{path:?}"
        );
    }
}
