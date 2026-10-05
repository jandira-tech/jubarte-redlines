//! The menu bar: every option the window offers, under the menu a Mac user
//! looks in. A choice the window carries out reaches it as a `menu` event
//! carrying the item's id (src/menu.js acts on it); a link opens in the
//! browser or the mail app from here.
//!
//! The bar is the data in [`MENUS`]; [`build`] only walks it. AppKit builds
//! menus on the main thread alone, which a test never runs on, so the tests
//! check the data.

use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// One entry of a menu.
enum Entry {
    /// Sent to the window: id, title, shortcut.
    Action(&'static str, &'static str, Option<&'static str>),
    /// Opened outside the app: id, title (the address is in [`link`]).
    Link(&'static str, &'static str),
    Submenu(&'static str, &'static [Entry]),
    Separator,
    Services,
    Hide,
    HideOthers,
    ShowAll,
    Quit,
    CloseWindow,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Fullscreen,
    Minimize,
    Zoom,
    BringAllToFront,
}

use Entry::*;

const MENUS: &[(&str, &[Entry])] = &[
    (
        "Jubarte",
        &[
            Action("about", "About Jubarte", None),
            Separator,
            Action("settings", "Settings…", Some("CmdOrCtrl+,")),
            Action("pro", "Jubarte PRO…", None),
            Action("restore", "Restore Purchase", None),
            Separator,
            Services,
            Separator,
            Hide,
            HideOthers,
            ShowAll,
            Separator,
            Quit,
        ],
    ),
    (
        "File",
        &[
            Action("choose-original", "Choose Original…", Some("CmdOrCtrl+O")),
            Action(
                "choose-modified",
                "Choose Modified…",
                Some("CmdOrCtrl+Shift+O"),
            ),
            Action(
                "choose-convert",
                "Choose Document to Convert…",
                Some("CmdOrCtrl+Alt+O"),
            ),
            Separator,
            Action("make-redline", "Make Redline", Some("CmdOrCtrl+R")),
            Action("make-pdf", "Make PDF", Some("CmdOrCtrl+Shift+R")),
            Action("export-pdf", "Export Redline as PDF", Some("CmdOrCtrl+E")),
            Action("swap", "Swap Original and Modified", None),
            Separator,
            Action("open-result", "Open Result", Some("CmdOrCtrl+Down")),
            Action("reveal", "Show in Finder", None),
            Action("save-copy", "Save a Copy…", Some("CmdOrCtrl+S")),
            Separator,
            CloseWindow,
        ],
    ),
    (
        "Edit",
        &[Undo, Redo, Separator, Cut, Copy, Paste, SelectAll],
    ),
    (
        "View",
        &[
            Action("show-redline", "Redline", Some("CmdOrCtrl+1")),
            Action("show-convert", "Convert to PDF", Some("CmdOrCtrl+2")),
            Separator,
            Action(
                "toggle-panel",
                "Show or Hide Panel",
                Some("CmdOrCtrl+Ctrl+S"),
            ),
            Separator,
            Submenu(
                "Appearance",
                &[
                    Action("appearance-system", "Match System", None),
                    Action("appearance-light", "Light", None),
                    Action("appearance-dark", "Dark", None),
                ],
            ),
            Separator,
            Fullscreen,
        ],
    ),
    ("Window", &[Minimize, Zoom, Separator, BringAllToFront]),
    (
        "Help",
        &[
            Link("website", "Jubarte Website"),
            Link("use-cases", "Use Cases"),
            Link("benchmark", "Benchmark"),
            Link("support", "Contact Support…"),
            Separator,
            Action("terms", "Terms of Use", None),
            Action("privacy", "Privacy Policy", None),
        ],
    ),
];

/// The address a Help link opens.
fn link(id: &str) -> Option<String> {
    let site = crate::facts::text("site.url");
    Some(match id {
        "website" => site,
        "use-cases" => format!("{site}/use-cases"),
        "benchmark" => format!("{site}/benchmark"),
        "support" => format!("mailto:{}", crate::facts::text("contact.support_email")),
        _ => return None,
    })
}

fn fill<'m, R: Runtime>(
    app: &'m AppHandle<R>,
    mut menu: SubmenuBuilder<'m, R, AppHandle<R>>,
    entries: &[Entry],
) -> tauri::Result<SubmenuBuilder<'m, R, AppHandle<R>>> {
    for entry in entries {
        menu = match entry {
            Action(id, title, keys) => {
                let mut item = MenuItemBuilder::with_id(*id, *title);
                if let Some(keys) = keys {
                    item = item.accelerator(*keys);
                }
                menu.item(&item.build(app)?)
            }
            Link(id, title) => menu.text(*id, *title),
            Submenu(title, inner) => {
                menu.item(&fill(app, SubmenuBuilder::new(app, *title), inner)?.build()?)
            }
            Separator => menu.separator(),
            Services => menu.services(),
            Hide => menu.hide(),
            HideOthers => menu.hide_others(),
            ShowAll => menu.show_all(),
            Quit => menu.quit(),
            CloseWindow => menu.close_window(),
            Undo => menu.undo(),
            Redo => menu.redo(),
            Cut => menu.cut(),
            Copy => menu.copy(),
            Paste => menu.paste(),
            SelectAll => menu.select_all(),
            Fullscreen => menu.fullscreen(),
            Minimize => menu.minimize(),
            Zoom => menu.maximize(),
            BringAllToFront => menu.bring_all_to_front(),
        };
    }
    Ok(menu)
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let mut bar = MenuBuilder::new(app);
    for (title, entries) in MENUS {
        let menu = fill(app, SubmenuBuilder::new(app, *title), entries)?.build()?;
        #[cfg(target_os = "macos")]
        match *title {
            "Window" => menu.set_as_windows_menu_for_nsapp()?,
            "Help" => menu.set_as_help_menu_for_nsapp()?,
            _ => {}
        }
        bar = bar.item(&menu);
    }
    bar.build()
}

/// A link opens outside the app; anything else is the window's to do.
pub fn handle<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();
    if let Some(url) = link(id) {
        let _ = crate::open_path(url);
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("menu", id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk<'a>(entries: &'a [Entry], out: &mut Vec<&'a Entry>) {
        for e in entries {
            out.push(e);
            if let Submenu(_, inner) = e {
                walk(inner, out);
            }
        }
    }

    fn all() -> Vec<&'static Entry> {
        let mut out = Vec::new();
        for (_, entries) in MENUS {
            walk(entries, &mut out);
        }
        out
    }

    #[test]
    fn the_bar_reads_as_a_mac_apps_does() {
        let titles: Vec<&str> = MENUS.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            titles,
            ["Jubarte", "File", "Edit", "View", "Window", "Help"]
        );
        // About first, Quit last, in the app's own menu.
        let app = MENUS[0].1;
        assert!(matches!(app.first(), Some(Action("about", _, None))));
        assert!(matches!(app.last(), Some(Quit)));
    }

    #[test]
    fn every_id_and_every_shortcut_is_used_once() {
        let mut ids = Vec::new();
        let mut keys = Vec::new();
        for e in all() {
            match e {
                Action(id, _, k) => {
                    ids.push(*id);
                    keys.extend(*k);
                }
                Link(id, _) => ids.push(*id),
                _ => {}
            }
        }
        for list in [&mut ids, &mut keys] {
            let total = list.len();
            list.sort_unstable();
            list.dedup();
            assert_eq!(list.len(), total, "{list:?}");
        }
    }

    #[test]
    fn every_shortcut_parses_as_the_menu_will_parse_it() {
        // MenuItemBuilder::build fails on a shortcut muda cannot read, and
        // the menu bar is built at launch: one typo and the app never opens.
        for e in all() {
            if let Action(id, _, Some(keys)) = e {
                keys.parse::<muda::accelerator::Accelerator>()
                    .unwrap_or_else(|err| panic!("{id}: {keys:?}: {err}"));
            }
        }
        assert!(
            "CmdOrCtrl+Nope"
                .parse::<muda::accelerator::Accelerator>()
                .is_err()
        );
    }

    #[test]
    fn every_help_link_opens_the_site_or_the_support_address() {
        for e in all() {
            if let Link(id, _) = e {
                let url = link(id).unwrap_or_else(|| panic!("{id} has no address"));
                assert!(
                    url.starts_with("https://jubarte.pro") || url.starts_with("mailto:"),
                    "{url}"
                );
            }
            if let Action(id, _, _) = e {
                assert!(link(id).is_none(), "{id} is both an action and a link");
            }
        }
        assert_eq!(link("website").unwrap(), "https://jubarte.pro");
    }
}
