//! Window Management Demo
//!
//! This example demonstrates window management features:
//! - Window events (resize, move, focus/blur, minimize, fullscreen)
//! - Keyboard shortcuts for window control
//! - Window state display
//!
//! Keyboard shortcuts:
//! - Cmd+F: Toggle fullscreen
//! - Cmd+M: Minimize window
//! - Escape: Exit fullscreen (if in fullscreen)

use sol_ui::{
    app::app,
    color::colors,
    element::{container, text},
    layer::{InputEvent, Key, LayerOptions},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

/// State to track window information
struct WindowState {
    width: f32,
    height: f32,
    position_x: f32,
    position_y: f32,
    is_focused: bool,
    is_minimized: bool,
    is_fullscreen: bool,
    event_log: Vec<String>,
}

impl WindowState {
    fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            position_x: 0.0,
            position_y: 0.0,
            is_focused: true,
            is_minimized: false,
            is_fullscreen: false,
            event_log: Vec::new(),
        }
    }

    fn log_event(&mut self, event: &str) {
        self.event_log.push(event.to_string());
        // Keep only last 10 events
        if self.event_log.len() > 10 {
            self.event_log.remove(0);
        }
    }
}

fn main() {
    let state = Rc::new(RefCell::new(WindowState::new()));
    let state_for_handler = state.clone();

    app()
        .title("Window Management Demo")
        .size(800.0, 600.0)
        .on_window_event(move |event, window| {
            let mut state = state_for_handler.borrow_mut();

            match event {
                InputEvent::WindowResized { size } => {
                    state.width = size.x;
                    state.height = size.y;
                    state.log_event(&format!("Resized: {}x{}", size.x as i32, size.y as i32));
                }
                InputEvent::WindowMoved { position } => {
                    state.position_x = position.x;
                    state.position_y = position.y;
                    state.log_event(&format!("Moved: ({}, {})", position.x as i32, position.y as i32));
                }
                InputEvent::WindowFocused => {
                    state.is_focused = true;
                    state.log_event("Window focused");
                }
                InputEvent::WindowBlurred => {
                    state.is_focused = false;
                    state.log_event("Window blurred");
                }
                InputEvent::WindowMinimized => {
                    state.is_minimized = true;
                    state.log_event("Window minimized");
                }
                InputEvent::WindowRestored => {
                    state.is_minimized = false;
                    state.log_event("Window restored");
                }
                InputEvent::WindowEnteredFullscreen => {
                    state.is_fullscreen = true;
                    state.log_event("Entered fullscreen");
                }
                InputEvent::WindowExitedFullscreen => {
                    state.is_fullscreen = false;
                    state.log_event("Exited fullscreen");
                }
                InputEvent::KeyDown { key, modifiers, .. } => {
                    // Cmd+F: Toggle fullscreen
                    if *key == Key::F && modifiers.cmd && !modifiers.shift && !modifiers.alt {
                        window.toggle_fullscreen();
                    }
                    // Cmd+M: Minimize
                    if *key == Key::M && modifiers.cmd && !modifiers.shift && !modifiers.alt {
                        window.minimize();
                    }
                    // Escape: Exit fullscreen
                    if *key == Key::Escape && state.is_fullscreen {
                        window.exit_fullscreen();
                    }
                }
                _ => {}
            }
        })
        .with_layers(move |layers| {
            let state_clone = state.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let state = state_clone.borrow();

                    let status_color = if state.is_focused {
                        colors::GREEN_600
                    } else {
                        colors::GRAY_500
                    };

                    let fullscreen_status = if state.is_fullscreen {
                        "Yes"
                    } else {
                        "No"
                    };

                    let minimized_status = if state.is_minimized {
                        "Yes"
                    } else {
                        "No"
                    };

                    let focus_status = if state.is_focused {
                        "Focused"
                    } else {
                        "Blurred"
                    };

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(20.0)
                            .child(
                                text(
                                    "Window Management Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 32.0,
                                        ..Default::default()
                                    },
                                )
                            )
                            .child(
                                // Window info panel
                                container()
                                    .width(400.0)
                                    .padding(20.0)
                                    .background(colors::WHITE)
                                    .corner_radius(10.0)
                                    .flex_col()
                                    .gap(10.0)
                                    .child(
                                        text(
                                            "Window State",
                                            TextStyle {
                                                color: colors::GRAY_800,
                                                size: 20.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Size: {}x{}", state.width as i32, state.height as i32),
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Position: ({}, {})", state.position_x as i32, state.position_y as i32),
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Focus: {}", focus_status),
                                            TextStyle {
                                                color: status_color,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Fullscreen: {}", fullscreen_status),
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Minimized: {}", minimized_status),
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                            )
                            .child(
                                // Keyboard shortcuts panel
                                container()
                                    .width(400.0)
                                    .padding(20.0)
                                    .background(colors::BLUE_400)
                                    .corner_radius(10.0)
                                    .flex_col()
                                    .gap(8.0)
                                    .child(
                                        text(
                                            "Keyboard Shortcuts",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            "Cmd+F: Toggle fullscreen",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 14.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            "Cmd+M: Minimize window",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 14.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            "Escape: Exit fullscreen",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 14.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                            )
                            .child(
                                // Event log panel
                                container()
                                    .width(400.0)
                                    .padding(20.0)
                                    .background(colors::GRAY_200)
                                    .corner_radius(10.0)
                                    .flex_col()
                                    .gap(4.0)
                                    .child(
                                        text(
                                            "Recent Events",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 16.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            state.event_log.iter().rev().take(5).cloned().collect::<Vec<_>>().join("\n"),
                                            TextStyle {
                                                color: colors::GRAY_600,
                                                size: 12.0,
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
