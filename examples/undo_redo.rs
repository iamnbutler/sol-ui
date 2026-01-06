//! Undo/Redo System Demo
//!
//! Demonstrates the undo/redo system:
//! - Command pattern for undoable actions
//! - Cmd+Z to undo, Cmd+Shift+Z to redo
//! - Action grouping for batch operations
//! - Undo stack depth limiting
//!
//! This example shows a color picker where color changes can be undone/redone.

use sol_ui::{
    app::app,
    color::{colors, Color, ColorExt},
    element::{button, container, row, text, Element},
    entity::{new_entity, observe, update_entity, Entity},
    interaction::Interactable,
    layer::{Key, LayerOptions, Modifiers},
    style::TextStyle,
    undo::{Command, UndoManager},
};
use std::cell::RefCell;
use std::rc::Rc;

/// State for the color value
struct ColorState {
    color: Color,
}

/// Command to change the color
struct SetColorCommand {
    target: Entity<ColorState>,
    old_color: Color,
    new_color: Color,
    description: String,
}

impl SetColorCommand {
    fn new(target: Entity<ColorState>, new_color: Color, description: impl Into<String>) -> Self {
        let old_color = sol_ui::entity::read_entity(&target, |s| s.color).unwrap_or(colors::WHITE);
        Self {
            target,
            old_color,
            new_color,
            description: description.into(),
        }
    }
}

// Required for Command trait
unsafe impl Send for SetColorCommand {}

impl Command for SetColorCommand {
    fn execute(&mut self) {
        update_entity(&self.target, |s| s.color = self.new_color);
    }

    fn undo(&mut self) {
        update_entity(&self.target, |s| s.color = self.old_color);
    }

    fn description(&self) -> &str {
        &self.description
    }
}

fn main() {
    app()
        .title("Undo/Redo Demo")
        .size(600.0, 500.0)
        .with_layers(|layers| {
            // Shared undo manager
            let undo_manager = Rc::new(RefCell::new(UndoManager::new()));

            // Color state entity (will be initialized on first render)
            let color_entity: Rc<RefCell<Option<Entity<ColorState>>>> = Rc::new(RefCell::new(None));

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    // Initialize color entity on first render
                    let entity = {
                        let mut e = color_entity.borrow_mut();
                        if e.is_none() {
                            *e = Some(new_entity(ColorState {
                                color: colors::BLUE_500,
                            }));
                        }
                        e.clone().unwrap()
                    };

                    // Observe the color (subscribes to changes)
                    let current_color = observe(&entity, |s| s.color).unwrap_or(colors::WHITE);

                    // Get undo/redo status
                    let can_undo = undo_manager.borrow().can_undo();
                    let can_redo = undo_manager.borrow().can_redo();
                    let undo_desc = undo_manager
                        .borrow()
                        .undo_description()
                        .map(|s| s.to_string());
                    let redo_desc = undo_manager
                        .borrow()
                        .redo_description()
                        .map(|s| s.to_string());
                    let undo_count = undo_manager.borrow().undo_count();

                    // Clone for button handlers
                    let entity_red = entity.clone();
                    let entity_green = entity.clone();
                    let entity_blue = entity.clone();
                    let entity_purple = entity.clone();
                    let entity_random = entity.clone();
                    let undo_mgr_red = undo_manager.clone();
                    let undo_mgr_green = undo_manager.clone();
                    let undo_mgr_blue = undo_manager.clone();
                    let undo_mgr_purple = undo_manager.clone();
                    let undo_mgr_random = undo_manager.clone();
                    let undo_mgr_undo = undo_manager.clone();
                    let undo_mgr_redo = undo_manager.clone();
                    let undo_mgr_key = undo_manager.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(24.0)
                            // Title
                            .child(text(
                                "Undo/Redo Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 28.0,
                                },
                            ))
                            // Instructions
                            .child(text(
                                "Cmd+Z to undo, Cmd+Shift+Z to redo",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 14.0,
                                },
                            ))
                            // Color preview box
                            .child(
                                container()
                                    .width(200.0)
                                    .height(100.0)
                                    .background(current_color)
                                    .corner_radius(12.0)
                                    .border(colors::GRAY_300, 2.0),
                            )
                            // Color name (show RGB values)
                            .child(text(
                                format!(
                                    "R: {:.0}  G: {:.0}  B: {:.0}",
                                    current_color.red * 255.0,
                                    current_color.green * 255.0,
                                    current_color.blue * 255.0
                                ),
                                TextStyle {
                                    color: colors::GRAY_700,
                                    size: 14.0,
                                },
                            ))
                            // Color buttons
                            .child(
                                row()
                                    .gap(12.0)
                                    .child(
                                        button("Red")
                                            .padding(12.0, 8.0)
                                            .corner_radius(6.0)
                                            .backgrounds(
                                                colors::RED_500,
                                                colors::RED_400,
                                                colors::RED_600,
                                            )
                                            .on_click_simple(move || {
                                                let cmd = SetColorCommand::new(
                                                    entity_red.clone(),
                                                    colors::RED_500,
                                                    "Set Red",
                                                );
                                                undo_mgr_red.borrow_mut().execute(Box::new(cmd));
                                            }),
                                    )
                                    .child(
                                        button("Green")
                                            .padding(12.0, 8.0)
                                            .corner_radius(6.0)
                                            .backgrounds(
                                                colors::GREEN_500,
                                                colors::GREEN_400,
                                                colors::GREEN_600,
                                            )
                                            .on_click_simple(move || {
                                                let cmd = SetColorCommand::new(
                                                    entity_green.clone(),
                                                    colors::GREEN_500,
                                                    "Set Green",
                                                );
                                                undo_mgr_green.borrow_mut().execute(Box::new(cmd));
                                            }),
                                    )
                                    .child(
                                        button("Blue")
                                            .padding(12.0, 8.0)
                                            .corner_radius(6.0)
                                            .backgrounds(
                                                colors::BLUE_500,
                                                colors::BLUE_400,
                                                colors::BLUE_600,
                                            )
                                            .on_click_simple(move || {
                                                let cmd = SetColorCommand::new(
                                                    entity_blue.clone(),
                                                    colors::BLUE_500,
                                                    "Set Blue",
                                                );
                                                undo_mgr_blue.borrow_mut().execute(Box::new(cmd));
                                            }),
                                    )
                                    .child(
                                        button("Purple")
                                            .padding(12.0, 8.0)
                                            .corner_radius(6.0)
                                            .backgrounds(
                                                colors::PURPLE_500,
                                                colors::PURPLE_400,
                                                colors::PURPLE_600,
                                            )
                                            .on_click_simple(move || {
                                                let cmd = SetColorCommand::new(
                                                    entity_purple.clone(),
                                                    colors::PURPLE_500,
                                                    "Set Purple",
                                                );
                                                undo_mgr_purple.borrow_mut().execute(Box::new(cmd));
                                            }),
                                    )
                                    .child(
                                        button("Random")
                                            .padding(12.0, 8.0)
                                            .corner_radius(6.0)
                                            .backgrounds(
                                                colors::GRAY_500,
                                                colors::GRAY_400,
                                                colors::GRAY_600,
                                            )
                                            .on_click_simple(move || {
                                                // Generate random color using time as seed
                                                let seed = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_millis();
                                                let r = ((seed % 256) as f32) / 255.0;
                                                let g = (((seed / 256) % 256) as f32) / 255.0;
                                                let b = (((seed / 65536) % 256) as f32) / 255.0;
                                                let random_color = Color::rgb(r, g, b);
                                                let cmd = SetColorCommand::new(
                                                    entity_random.clone(),
                                                    random_color,
                                                    "Set Random Color",
                                                );
                                                undo_mgr_random.borrow_mut().execute(Box::new(cmd));
                                            }),
                                    ),
                            )
                            // Undo/Redo buttons
                            .child(
                                row()
                                    .gap(12.0)
                                    .child(
                                        button(if can_undo {
                                            format!("Undo ({})", undo_desc.as_deref().unwrap_or(""))
                                        } else {
                                            "Undo".to_string()
                                        })
                                        .padding(12.0, 8.0)
                                        .corner_radius(6.0)
                                        .backgrounds(
                                            if can_undo {
                                                colors::BLUE_500
                                            } else {
                                                colors::GRAY_400
                                            },
                                            colors::BLUE_400,
                                            colors::BLUE_600,
                                        )
                                        .on_click_simple(move || {
                                            undo_mgr_undo.borrow_mut().undo();
                                        }),
                                    )
                                    .child(
                                        button(if can_redo {
                                            format!("Redo ({})", redo_desc.as_deref().unwrap_or(""))
                                        } else {
                                            "Redo".to_string()
                                        })
                                        .padding(12.0, 8.0)
                                        .corner_radius(6.0)
                                        .backgrounds(
                                            if can_redo {
                                                colors::GREEN_500
                                            } else {
                                                colors::GRAY_400
                                            },
                                            colors::GREEN_400,
                                            colors::GREEN_600,
                                        )
                                        .on_click_simple(move || {
                                            undo_mgr_redo.borrow_mut().redo();
                                        }),
                                    ),
                            )
                            // Undo stack info
                            .child(text(
                                format!("Undo stack depth: {}", undo_count),
                                TextStyle {
                                    color: colors::GRAY_500,
                                    size: 12.0,
                                },
                            ))
                            // Keyboard handler for Cmd+Z / Cmd+Shift+Z
                            .interactive()
                            .with_id(100)
                            .focusable()
                            .on_key_down(move |key: Key, modifiers: Modifiers, _char, _repeat| {
                                undo_mgr_key.borrow_mut().handle_key_event(key, modifiers);
                            }),
                    ) as Box<dyn Element>
                },
            );
        })
        .run();
}
