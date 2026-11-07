//! The menus of the list: of a card, of a card of the trash, the question
//! before the trash is emptied; and what each choice does.
//!
//! ```text
//! one chosen ──► rename │ duplicate │ lock │ unlock │ export to a file…
//!               │ move to the trash
//!               lock: the pattern opens only to be looked at (`App::locked`)
//!               rename: the name in a field on top, then accept, cancel
//! n chosen ──► lock n │ unlock n │ export n, one .obusiny │ move n to the trash
//!              lock while one of them is not locked, else unlock
//! a card of the trash ──► restore │ delete for good… ──► delete │ keep
//! "Empty the trash…" ──► empty │ keep
//! ```

use std::sync::{Arc, Mutex};

use obusina_pattern::PatternId;
use plate_menu::{MenuItem, Notice};
use rust_i18n::t;

use super::View;
use crate::app::{App, say};
use crate::ui::access::Named;

/// What the menu is of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Of {
    /// A pattern of the list.
    Live(PatternId),
    /// The patterns chosen, more than one (`Projects::chosen`).
    Chosen,
    /// A pattern of the trash.
    Trashed(PatternId),
    /// The whole trash.
    AllTrash,
}

/// Opens the menu of the patterns chosen: the menu of a card when one is.
pub fn open_chosen(app: &mut App) {
    let ids = app.projects.chosen_ids();
    match ids.as_slice() {
        [] => {}
        [one] => open(app, Of::Live(*one)),
        _ => open(app, Of::Chosen),
    }
}

/// Opens the menu of `of`.
pub fn open(app: &mut App, of: Of) {
    let projects = &mut app.projects;
    let (notice, items) = match of {
        Of::Chosen => {
            let ids = projects.chosen_ids();
            let count = ids.len();
            let what = super::plural(count, super::PATTERNS_OF);
            let lock = match ids.iter().all(|id| projects.locked(*id)) {
                true => MenuItem::new(
                    "unlock",
                    t!("projects.unlock_many", count = count, what = what),
                ),
                false => {
                    MenuItem::new("lock", t!("projects.lock_many", count = count, what = what))
                }
            };
            (
                t!("projects.chosen", count = count).to_string(),
                vec![
                    lock,
                    MenuItem::new(
                        "export",
                        t!("projects.export_many", count = count, what = what),
                    ),
                    MenuItem::new(
                        "trash",
                        t!("projects.to_trash_many", count = count, what = what),
                    ),
                ],
            )
        }
        Of::Live(id) => (
            projects.name_of(id),
            vec![
                MenuItem::new("rename", t!("projects.rename")),
                MenuItem::new("duplicate", t!("projects.duplicate")),
                match projects.locked(id) {
                    true => MenuItem::new("unlock", t!("projects.unlock")),
                    false => MenuItem::new("lock", t!("projects.lock")),
                },
                MenuItem::new("export", t!("projects.export")),
                MenuItem::new("trash", t!("projects.to_trash")),
            ],
        ),
        Of::Trashed(id) => (
            projects.name_of(id),
            vec![
                MenuItem::new("restore", t!("projects.restore")),
                MenuItem::new("purge", t!("projects.purge")),
            ],
        ),
        Of::AllTrash => (
            t!("projects.ask_empty", count = projects.trash.len()).to_string(),
            vec![
                MenuItem::new("empty", t!("projects.empty_yes")),
                MenuItem::new("keep", t!("projects.keep")),
            ],
        ),
    };
    show_menu(app, of, notice, items);
}

/// Opens the menu of `items` about `of`, `notice` on top.
fn show_menu(app: &mut App, of: Of, notice: String, items: Vec<MenuItem>) {
    let projects = &mut app.projects;
    if let Err(error) = projects.menu.open(items) {
        log::error!("the menu of the list does not open: {error}");
        return;
    }
    projects.menu.notice(notice);
    projects.menu_of = Some(of);
}

/// Opens the menu of the rename of `id`: the name in a field on top.
pub fn rename(app: &mut App, id: PatternId) {
    let projects = &mut app.projects;
    *projects
        .renaming
        .lock()
        .expect("the name is never poisoned") = projects.name_of(id);
    let items = vec![
        MenuItem::new("rename-accept", t!("projects.accept")),
        MenuItem::new("keep", t!("projects.cancel")),
    ];
    if projects.menu.open(items).is_err() {
        return;
    }
    projects.menu.notice(field(Arc::clone(&projects.renaming)));
    projects.menu_of = Some(Of::Live(id));
}

/// The field of the new name, with room above and below its letters.
fn field(renaming: Arc<Mutex<String>>) -> Notice {
    Notice::widget(move |ui| {
        let mut name = renaming.lock().expect("the name is never poisoned");
        let line = ui.text_style_height(&egui::TextStyle::Body);
        let label = ui.label(t!("projects.new_name"));
        let field = egui::TextEdit::singleline(&mut *name)
            .margin(egui::vec2(line, line))
            .desired_width(f32::INFINITY);
        let response = ui.add(field).labelled_by(label.id);
        if !response.has_focus() && !response.lost_focus() {
            response.request_focus();
        }
        response.named(&t!("projects.new_name"));
    })
}

/// Draws the menu while it is open, and does what is chosen.
pub fn show(app: &mut App, context: &egui::Context) {
    app.projects.menu.nothing_found(t!("menu.nothing_found"));
    if app.projects.menu.is_open() {
        crate::ui::blocker::under(context, "projects-menu", &app.projects.menu);
    }
    let Some(chosen) = app.projects.menu.show(context) else {
        if !app.projects.menu.is_open() {
            app.projects.menu_of = None;
        }
        return;
    };
    let Some(of) = app.projects.menu_of.take() else {
        return;
    };
    match (chosen.id.as_str(), of) {
        ("rename", Of::Live(id)) => rename(app, id),
        ("rename-accept", Of::Live(id)) => {
            let name = app
                .projects
                .renaming
                .lock()
                .expect("the name is never poisoned")
                .trim()
                .to_owned();
            if !name.is_empty() {
                done(app, |library| library.rename(id, &name));
            }
        }
        ("duplicate", Of::Live(id)) => duplicate(app, context, id),
        ("lock", Of::Live(id)) => done(app, |library| library.set_locked(id, true)),
        ("unlock", Of::Live(id)) => done(app, |library| library.set_locked(id, false)),
        ("export", Of::Live(id)) => super::export(app, context, &[id]),
        ("trash", Of::Live(id)) => super::to_trash(app, &[id]),
        ("lock" | "unlock", Of::Chosen) => {
            let ids = app.projects.chosen_ids();
            let locked = chosen.id == "lock";
            done(app, |library| {
                ids.iter()
                    .try_for_each(|id| library.set_locked(*id, locked))
            });
        }
        ("export", Of::Chosen) => {
            let ids = app.projects.chosen_ids();
            super::export_many(app, context, &ids);
        }
        ("trash", Of::Chosen) => {
            let ids = app.projects.chosen_ids();
            super::to_trash(app, &ids);
        }
        ("restore", Of::Trashed(id)) => {
            done(app, |library| library.restore(id).map(|_| ()));
        }
        ("purge", Of::Trashed(id)) => {
            let notice = t!("projects.ask_purge", name = app.projects.name_of(id)).to_string();
            let items = vec![
                MenuItem::new("purge-yes", t!("projects.purge_yes")),
                MenuItem::new("keep", t!("projects.keep")),
            ];
            show_menu(app, of, notice, items);
        }
        ("purge-yes", Of::Trashed(id)) => {
            done(app, |library| library.purge(id));
            app.backdrop.forget(context, vec![id]);
        }
        ("empty", Of::AllTrash) => {
            let ids: Vec<PatternId> = app.projects.trash.iter().map(|t| t.pattern.id()).collect();
            done(app, |library| {
                ids.iter().try_for_each(|id| library.purge(*id))
            });
            app.backdrop.forget(context, ids);
            app.projects.view = View::List;
        }
        _ => {}
    }
}

/// A copy of a pattern, "<name> (copy)", with a copy of its picture.
fn duplicate(app: &mut App, context: &egui::Context, id: PatternId) {
    let name = format!("{} ({})", app.projects.name_of(id), t!("projects.copy"));
    let Some(library) = &app.library else {
        return;
    };
    match library.duplicate(id, &name) {
        Ok(copy) => app.backdrop.copy(context, id, copy.id()),
        Err(error) => app.projects.tell(say(&error)),
    }
    app.projects.refresh();
}

/// Does `work` on the library, tells what failed, and reads the list again.
pub fn done(
    app: &mut App,
    work: impl FnOnce(&crate::library::Library) -> crate::error::Result<()>,
) {
    if let Some(library) = &app.library
        && let Err(error) = work(library)
    {
        app.projects.tell(say(&error));
    }
    app.projects.refresh();
    if app.projects.view == View::Trash
        && app.library.as_ref().is_some_and(|l| l.trash().is_empty())
    {
        app.projects.view = View::List;
    }
}
