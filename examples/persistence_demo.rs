//! Local data persistence demo
//!
//! Demonstrates the storage system with:
//! - Loading saved state on startup
//! - Auto-saving with debouncing
//! - Preferences persistence
//! - Manual save/load operations
//!
//! Settings are persisted to ~/Library/Application Support/PersistenceDemo/

use serde::{Deserialize, Serialize};
use sol_ui::{
    app::app,
    color::colors,
    element::{button, checkbox, column, container, row, text, CheckboxInteractable},
    layer::LayerOptions,
    storage::{AutoSaver, Storage, StorageConfig},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// App settings that will be persisted
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AppSettings {
    dark_mode: bool,
    notifications_enabled: bool,
    sound_enabled: bool,
    auto_save: bool,
    counter: i32,
}

fn main() {
    // Create storage for this demo app
    let storage = Storage::new(StorageConfig {
        app_name: "PersistenceDemo".to_string(),
        ..Default::default()
    });

    // Load existing settings or use defaults
    let initial_settings: AppSettings = storage
        .load("settings")
        .ok()
        .flatten()
        .unwrap_or_default();

    println!("Loaded settings: {:?}", initial_settings);
    if let Some(path) = storage.base_path() {
        println!("Storage location: {}", path.display());
    }

    // Shared state
    let settings = Rc::new(RefCell::new(initial_settings));
    let storage = Rc::new(storage);
    let auto_saver = Rc::new(RefCell::new(AutoSaver::new(Duration::from_secs(2))));
    let save_status = Rc::new(RefCell::new(String::from("Ready")));

    app()
        .title("Persistence Demo")
        .size(500.0, 550.0)
        .with_layers(move |layers| {
            let settings_clone = settings.clone();
            let storage_clone = storage.clone();
            let auto_saver_clone = auto_saver.clone();
            let save_status_clone = save_status.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let current = settings_clone.borrow().clone();
                    let status = save_status_clone.borrow().clone();

                    // Check if we should auto-save
                    if current.auto_save {
                        let mut saver = auto_saver_clone.borrow_mut();
                        let storage = storage_clone.clone();
                        let settings_for_save = settings_clone.clone();
                        let status_update = save_status_clone.clone();

                        let _ = saver.try_save::<_, std::io::Error>(|| {
                            let s = settings_for_save.borrow();
                            if storage.save("settings", &*s).is_ok() {
                                *status_update.borrow_mut() = "Auto-saved!".to_string();
                                println!("Auto-saved settings");
                            }
                            Ok(())
                        });
                    }

                    // Clones for callbacks
                    let dark_mode_settings = settings_clone.clone();
                    let dark_mode_saver = auto_saver_clone.clone();
                    let dark_mode_status = save_status_clone.clone();

                    let notif_settings = settings_clone.clone();
                    let notif_saver = auto_saver_clone.clone();
                    let notif_status = save_status_clone.clone();

                    let sound_settings = settings_clone.clone();
                    let sound_saver = auto_saver_clone.clone();
                    let sound_status = save_status_clone.clone();

                    let auto_save_settings = settings_clone.clone();

                    let inc_settings = settings_clone.clone();
                    let inc_saver = auto_saver_clone.clone();
                    let inc_status = save_status_clone.clone();

                    let dec_settings = settings_clone.clone();
                    let dec_saver = auto_saver_clone.clone();
                    let dec_status = save_status_clone.clone();

                    let save_settings = settings_clone.clone();
                    let save_storage = storage_clone.clone();
                    let save_status_btn = save_status_clone.clone();

                    let reset_settings = settings_clone.clone();
                    let reset_storage = storage_clone.clone();
                    let reset_status = save_status_clone.clone();

                    let text_color = if current.dark_mode { colors::WHITE } else { colors::BLACK };
                    let secondary_color = if current.dark_mode { colors::GRAY_300 } else { colors::GRAY_700 };

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(if current.dark_mode { colors::GRAY_900 } else { colors::GRAY_100 })
                            .flex_col()
                            .padding(32.0)
                            .gap(24.0)
                            // Title
                            .child(
                                text(
                                    "Persistence Demo",
                                    TextStyle {
                                        color: text_color,
                                        size: 28.0,
                                        line_height: 1.2,
                                        ..Default::default()
                                    },
                                )
                            )
                            .child(
                                text(
                                    "Settings are saved to ~/Library/Application Support/PersistenceDemo/",
                                    TextStyle {
                                        color: if current.dark_mode { colors::GRAY_400 } else { colors::GRAY_600 },
                                        size: 12.0,
                                        line_height: 1.2,
                                        ..Default::default()
                                    },
                                )
                            )
                            // Settings section
                            .child(
                                column()
                                    .gap(16.0)
                                    .child(
                                        text(
                                            "Settings",
                                            TextStyle {
                                                color: secondary_color,
                                                size: 18.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        checkbox(current.dark_mode)
                                            .label("Dark Mode")
                                            .label_style(TextStyle {
                                                color: text_color,
                                                size: 14.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            })
                                            .with_id(1)
                                            .on_change(move |new_state| {
                                                dark_mode_settings.borrow_mut().dark_mode = new_state;
                                                dark_mode_saver.borrow_mut().mark_dirty();
                                                *dark_mode_status.borrow_mut() = "Modified (will auto-save)".to_string();
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(current.notifications_enabled)
                                            .label("Enable Notifications")
                                            .label_style(TextStyle {
                                                color: text_color,
                                                size: 14.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            })
                                            .checked_background(colors::GREEN_500)
                                            .with_id(2)
                                            .on_change(move |new_state| {
                                                notif_settings.borrow_mut().notifications_enabled = new_state;
                                                notif_saver.borrow_mut().mark_dirty();
                                                *notif_status.borrow_mut() = "Modified (will auto-save)".to_string();
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(current.sound_enabled)
                                            .label("Enable Sound")
                                            .label_style(TextStyle {
                                                color: text_color,
                                                size: 14.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            })
                                            .checked_background(colors::PURPLE_500)
                                            .with_id(3)
                                            .on_change(move |new_state| {
                                                sound_settings.borrow_mut().sound_enabled = new_state;
                                                sound_saver.borrow_mut().mark_dirty();
                                                *sound_status.borrow_mut() = "Modified (will auto-save)".to_string();
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(current.auto_save)
                                            .label("Auto-save (2 second debounce)")
                                            .label_style(TextStyle {
                                                color: text_color,
                                                size: 14.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            })
                                            .checked_background(colors::BLUE_500)
                                            .with_id(4)
                                            .on_change(move |new_state| {
                                                auto_save_settings.borrow_mut().auto_save = new_state;
                                            })
                                            .interactive_checkbox()
                                    )
                            )
                            // Counter section
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Counter (persisted)",
                                            TextStyle {
                                                color: secondary_color,
                                                size: 18.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        row()
                                            .gap(16.0)
                                            .items_center()
                                            .child(
                                                button("-")
                                                    .background(colors::RED_500)
                                                    .hover_background(colors::RED_400)
                                                    .press_background(colors::RED_600)
                                                    .text_color(colors::WHITE)
                                                    .corner_radius(8.0)
                                                    .padding_xy(16.0, 8.0)
                                                    .with_id(10)
                                                    .on_click_simple(move || {
                                                        dec_settings.borrow_mut().counter -= 1;
                                                        dec_saver.borrow_mut().mark_dirty();
                                                        *dec_status.borrow_mut() = "Modified (will auto-save)".to_string();
                                                    })
                                            )
                                            .child(
                                                container()
                                                    .width(80.0)
                                                    .height(40.0)
                                                    .background(if current.dark_mode { colors::GRAY_700 } else { colors::WHITE })
                                                    .corner_radius(8.0)
                                                    .flex_col()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            current.counter.to_string(),
                                                            TextStyle {
                                                                color: text_color,
                                                                size: 20.0,
                                                                line_height: 1.2,
                                                                ..Default::default()
                                                            },
                                                        )
                                                    )
                                            )
                                            .child(
                                                button("+")
                                                    .background(colors::GREEN_500)
                                                    .hover_background(colors::GREEN_400)
                                                    .press_background(colors::GREEN_600)
                                                    .text_color(colors::WHITE)
                                                    .corner_radius(8.0)
                                                    .padding_xy(16.0, 8.0)
                                                    .with_id(11)
                                                    .on_click_simple(move || {
                                                        inc_settings.borrow_mut().counter += 1;
                                                        inc_saver.borrow_mut().mark_dirty();
                                                        *inc_status.borrow_mut() = "Modified (will auto-save)".to_string();
                                                    })
                                            )
                                    )
                            )
                            // Manual save/reset buttons
                            .child(
                                row()
                                    .gap(12.0)
                                    .child(
                                        button("Save Now")
                                            .background(colors::BLUE_500)
                                            .hover_background(colors::BLUE_400)
                                            .press_background(colors::BLUE_600)
                                            .text_color(colors::WHITE)
                                            .corner_radius(8.0)
                                            .with_id(20)
                                            .on_click_simple(move || {
                                                let s = save_settings.borrow();
                                                if save_storage.save("settings", &*s).is_ok() {
                                                    *save_status_btn.borrow_mut() = "Saved!".to_string();
                                                    println!("Manually saved settings");
                                                } else {
                                                    *save_status_btn.borrow_mut() = "Save failed!".to_string();
                                                }
                                            })
                                    )
                                    .child(
                                        button("Reset")
                                            .background(colors::RED_500)
                                            .hover_background(colors::RED_400)
                                            .press_background(colors::RED_600)
                                            .text_color(colors::WHITE)
                                            .corner_radius(8.0)
                                            .with_id(21)
                                            .on_click_simple(move || {
                                                *reset_settings.borrow_mut() = AppSettings::default();
                                                let _ = reset_storage.delete("settings");
                                                *reset_status.borrow_mut() = "Reset to defaults".to_string();
                                                println!("Reset settings to defaults");
                                            })
                                    )
                            )
                            // Status display
                            .child(
                                container()
                                    .background(if current.dark_mode { colors::GRAY_800 } else { colors::WHITE })
                                    .padding(16.0)
                                    .corner_radius(8.0)
                                    .child(
                                        text(
                                            format!("Status: {}", status),
                                            TextStyle {
                                                color: if current.dark_mode { colors::GRAY_300 } else { colors::GRAY_600 },
                                                size: 14.0,
                                                line_height: 1.2,
                                                ..Default::default()
                                            },
                                        )
                                    )
                            )
                    )
                },
            );
        })
        .run();
}
