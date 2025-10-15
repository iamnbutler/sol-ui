# Hello World

A lightweight immediate mode gui library for Rust, built on Metal for macOS.

`cargo add sol-ui`

### Basic Example

```rust
use std::cell::RefCell;
use std::rc::Rc;
use sol_ui::{
    app, color::colors, draw::TextStyle,
    elements::{container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
};

fn main() {
    let mood = Rc::new(RefCell::new("😐"));

    app()
        .title("Mood Picker")
        .size(600.0, 400.0)
        .with_layers(move |layers| {
            let mood_clone = mood.clone();
            layers.add_ui_layer(0, LayerOptions::default().with_input(), move || {
                let current_mood = *mood_clone.borrow();

                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(30.0)
                        .child(text(
                            "How are you feeling?",
                            TextStyle { color: colors::GRAY_400, size: 18.0 },
                        ))
                        .child(text(
                            current_mood,
                            TextStyle { color: colors::WHITE, size: 64.0 },
                        ))
                        .child(
                            row()
                                .gap(15.0)
                                .child(mood_button("😊", 1, mood_clone.clone()))
                                .child(mood_button("😎", 2, mood_clone.clone()))
                                .child(mood_button("🤔", 3, mood_clone.clone()))
                                .child(mood_button("😴", 4, mood_clone.clone()))
                        )
                )
            });
        })
        .run();
}

fn mood_button(emoji: &'static str, id: u64, mood: Rc<RefCell<&'static str>>) -> impl Interactable {
    container()
        .width(60.0)
        .height(60.0)
        .border(colors::GRAY_700, 2.0)
        .flex()
        .items_center()
        .justify_center()
        .child(text(emoji, TextStyle { color: colors::WHITE, size: 32.0 }))
        .interactive()
        .with_id(id)
        .on_click(move |button, _, _| {
            if button == MouseButton::Left {
                *mood.borrow_mut() = emoji;
            }
        })
}
```

Lots of inspiration from [GPUI](https://gpui.rs).
