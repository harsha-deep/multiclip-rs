mod store;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, ListBox, ListBoxRow, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;
use store::Store;

const MAX_PREVIEW_LEN: usize = 120;

fn main() {
    let app = Application::builder()
        .application_id("com.harsha.multiclip-rs")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let store = Rc::new(Store::open());

    let list_box = ListBox::new();
    list_box.add_css_class("boxed-list");

    let scrolled = ScrolledWindow::builder()
        .child(&list_box)
        .vexpand(true)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Multiclip")
        .default_width(420)
        .default_height(500)
        .child(&scrolled)
        .build();

    refresh_list(&list_box, &store);

    // Selecting an entry copies it back to the clipboard.
    {
        let store = store.clone();
        let window = window.clone();
        list_box.connect_row_activated(move |list_box, row| {
            let items = store.recent();
            let index = row.index();
            if let Some(content) = items.get(index as usize) {
                window.clipboard().set_text(content);
            }
            let _ = list_box;
        });
    }

    // Watch the system clipboard for changes and record new content.
    {
        let clipboard = window.clipboard();
        let store = store.clone();
        let list_box = list_box.clone();
        let last_seen: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        clipboard.connect_changed(move |clipboard| {
            let store = store.clone();
            let list_box = list_box.clone();
            let last_seen = last_seen.clone();

            clipboard.read_text_async(gtk4::gio::Cancellable::NONE, move |result| {
                let Ok(Some(text)) = result.map(|t| t.map(|s| s.to_string())) else {
                    return;
                };
                let text = text.trim().to_string();
                if text.is_empty() {
                    return;
                }
                if last_seen.borrow().as_deref() == Some(text.as_str()) {
                    return;
                }
                *last_seen.borrow_mut() = Some(text.clone());

                store.push(&text);
                refresh_list(&list_box, &store);
            });
        });
    }

    window.present();
}

fn refresh_list(list_box: &ListBox, store: &Store) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for content in store.recent() {
        let preview = preview_of(&content);
        let label = Label::builder()
            .label(&preview)
            .xalign(0.0)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(10)
            .margin_end(10)
            .build();
        let row = ListBoxRow::new();
        row.set_child(Some(&label));
        list_box.append(&row);
    }
}

fn preview_of(content: &str) -> String {
    let single_line = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() > MAX_PREVIEW_LEN {
        let truncated: String = single_line.chars().take(MAX_PREVIEW_LEN).collect();
        format!("{truncated}…")
    } else {
        single_line
    }
}
