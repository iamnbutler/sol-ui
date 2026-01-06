use sol_ui::{
    app::app,
    color::colors,
    element::{container, text},
    layer::LayerOptions,
    platform::{KeyboardShortcut, Menu, MenuBar, MenuItem},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

fn main() {
    // Shared state for demonstrating menu actions
    let counter = Arc::new(AtomicUsize::new(0));
    let message = Arc::new(std::sync::Mutex::new(String::from("Select a menu item")));

    // Set up custom menu bar
    let counter_new = counter.clone();
    let message_new = message.clone();
    let counter_open = counter.clone();
    let message_open = message.clone();
    let counter_save = counter.clone();
    let message_save = message.clone();
    let message_view = message.clone();

    MenuBar::new("Menu Demo")
        .with_app_menu()
        // Custom File menu
        .menu(
            Menu::new("File")
                .item(
                    MenuItem::action("New")
                        .shortcut(KeyboardShortcut::cmd("n"))
                        .on_action(move || {
                            counter_new.fetch_add(1, Ordering::SeqCst);
                            *message_new.lock().unwrap() = "New file created!".to_string();
                            println!("New file created!");
                        })
                        .build(),
                )
                .item(
                    MenuItem::action("Open...")
                        .shortcut(KeyboardShortcut::cmd("o"))
                        .on_action(move || {
                            counter_open.fetch_add(1, Ordering::SeqCst);
                            *message_open.lock().unwrap() = "Open dialog triggered!".to_string();
                            println!("Open dialog triggered!");
                        })
                        .build(),
                )
                .separator()
                .item(
                    MenuItem::action("Save")
                        .shortcut(KeyboardShortcut::cmd("s"))
                        .on_action(move || {
                            counter_save.fetch_add(1, Ordering::SeqCst);
                            *message_save.lock().unwrap() = "File saved!".to_string();
                            println!("File saved!");
                        })
                        .build(),
                )
                .item(
                    MenuItem::action("Save As...")
                        .shortcut(KeyboardShortcut::cmd_shift("s"))
                        .build(),
                )
                .separator()
                .item(
                    MenuItem::action("Export")
                        .build(),
                )
                .item(MenuItem::submenu(
                    "Recent Files",
                    Menu::new("Recent Files")
                        .item(MenuItem::action("document1.txt").build())
                        .item(MenuItem::action("document2.txt").build())
                        .item(MenuItem::action("project.rs").build())
                        .separator()
                        .item(MenuItem::action("Clear Recent").build()),
                )),
        )
        .with_edit_menu()
        // Custom View menu
        .menu(
            Menu::new("View")
                .item(
                    MenuItem::action("Toggle Sidebar")
                        .shortcut(KeyboardShortcut::cmd("\\"))
                        .on_action(move || {
                            *message_view.lock().unwrap() = "Sidebar toggled!".to_string();
                            println!("Sidebar toggled!");
                        })
                        .build(),
                )
                .separator()
                .item(
                    MenuItem::action("Zoom In")
                        .shortcut(KeyboardShortcut::cmd("+"))
                        .build(),
                )
                .item(
                    MenuItem::action("Zoom Out")
                        .shortcut(KeyboardShortcut::cmd("-"))
                        .build(),
                )
                .item(
                    MenuItem::action("Actual Size")
                        .shortcut(KeyboardShortcut::cmd("0"))
                        .build(),
                ),
        )
        .with_window_menu()
        .with_help_menu()
        .build();

    // Display counter
    let display_counter = Rc::new(RefCell::new(0usize));

    app()
        .title("Menu Demo")
        .size(600.0, 400.0)
        .with_layers(move |layers| {
            let counter_display = display_counter.clone();

            layers.add_ui_layer(0, LayerOptions::default().with_clear(), move || {
                let _count = *counter_display.borrow();

                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(20.0)
                        .child(text(
                            "Menu Demo",
                            TextStyle {
                                color: colors::WHITE,
                                size: 28.0,
                            },
                        ))
                        .child(text(
                            "Try the menu bar above!",
                            TextStyle {
                                color: colors::GRAY_400,
                                size: 16.0,
                            },
                        ))
                        .child(text(
                            format!("Menu actions will print to console"),
                            TextStyle {
                                color: colors::GRAY_500,
                                size: 14.0,
                            },
                        ))
                        .child(text(
                            format!("Use Cmd+N, Cmd+O, Cmd+S to trigger actions"),
                            TextStyle {
                                color: colors::GRAY_500,
                                size: 14.0,
                            },
                        )),
                )
            });
        })
        .run();
}
