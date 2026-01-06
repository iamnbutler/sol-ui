//! Modal/dialog element for overlay content
//!
//! Provides a modal dialog with:
//! - Backdrop overlay that dims the background
//! - Centered dialog container
//! - Close on Escape key
//! - Close on backdrop click (optional)
//! - Focus trap support

use crate::{
    color::{Color, ColorExt, colors},
    element::{Element, LayoutContext},
    geometry::{Corners, Edges, Rect},
    interaction::{
        registry::register_element,
        ElementId, EventHandlers,
    },
    layer::Key,
    render::{PaintContext, PaintQuad},
};
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Create a new modal element
pub fn modal<F>(is_open: bool, on_close: F) -> Modal
where
    F: FnMut() + 'static,
{
    Modal::new(is_open, on_close)
}

/// A modal dialog overlay
pub struct Modal {
    /// Whether the modal is currently open
    is_open: bool,

    /// Callback when modal should close
    on_close: Rc<RefCell<Box<dyn FnMut()>>>,

    /// Unique ID for the backdrop (for click detection)
    backdrop_id: ElementId,

    /// Unique ID for the dialog container
    dialog_id: ElementId,

    /// Event handlers for the backdrop
    backdrop_handlers: Rc<RefCell<EventHandlers>>,

    /// Event handlers for the dialog (for key events)
    dialog_handlers: Rc<RefCell<EventHandlers>>,

    /// Backdrop color (semi-transparent overlay)
    backdrop_color: Color,

    /// Dialog background color
    dialog_background: Color,

    /// Dialog border color
    dialog_border_color: Option<Color>,

    /// Dialog border width
    dialog_border_width: f32,

    /// Dialog corner radius
    dialog_corner_radius: f32,

    /// Dialog padding
    dialog_padding: f32,

    /// Dialog min width
    dialog_min_width: f32,

    /// Dialog max width
    dialog_max_width: f32,

    /// Whether clicking the backdrop closes the modal
    close_on_backdrop_click: bool,

    /// Whether pressing Escape closes the modal
    close_on_escape: bool,

    /// Child content
    content: Option<Box<dyn Element>>,

    /// Cached layout nodes
    backdrop_node: Option<NodeId>,
    dialog_node: Option<NodeId>,
    content_node: Option<NodeId>,
}

impl Modal {
    pub fn new<F>(is_open: bool, on_close: F) -> Self
    where
        F: FnMut() + 'static,
    {
        let on_close: Rc<RefCell<Box<dyn FnMut()>>> = Rc::new(RefCell::new(Box::new(on_close)));

        Self {
            is_open,
            on_close,
            backdrop_id: ElementId::auto(),
            dialog_id: ElementId::auto(),
            backdrop_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            dialog_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            backdrop_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            dialog_background: colors::WHITE,
            dialog_border_color: Some(colors::GRAY_300),
            dialog_border_width: 1.0,
            dialog_corner_radius: 8.0,
            dialog_padding: 24.0,
            dialog_min_width: 300.0,
            dialog_max_width: 500.0,
            close_on_backdrop_click: true,
            close_on_escape: true,
            content: None,
            backdrop_node: None,
            dialog_node: None,
            content_node: None,
        }
    }

    /// Set the backdrop color
    pub fn backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = color;
        self
    }

    /// Set the dialog background color
    pub fn dialog_background(mut self, color: Color) -> Self {
        self.dialog_background = color;
        self
    }

    /// Set the dialog border
    pub fn dialog_border(mut self, color: Color, width: f32) -> Self {
        self.dialog_border_color = Some(color);
        self.dialog_border_width = width;
        self
    }

    /// Set the dialog corner radius
    pub fn dialog_corner_radius(mut self, radius: f32) -> Self {
        self.dialog_corner_radius = radius;
        self
    }

    /// Set the dialog padding
    pub fn dialog_padding(mut self, padding: f32) -> Self {
        self.dialog_padding = padding;
        self
    }

    /// Set the dialog min width
    pub fn dialog_min_width(mut self, width: f32) -> Self {
        self.dialog_min_width = width;
        self
    }

    /// Set the dialog max width
    pub fn dialog_max_width(mut self, width: f32) -> Self {
        self.dialog_max_width = width;
        self
    }

    /// Set whether clicking the backdrop closes the modal
    pub fn close_on_backdrop_click(mut self, close: bool) -> Self {
        self.close_on_backdrop_click = close;
        self
    }

    /// Set whether pressing Escape closes the modal
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Set the content of the modal
    pub fn content(mut self, content: impl Element + 'static) -> Self {
        self.content = Some(Box::new(content));
        self
    }

}

impl Element for Modal {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        if !self.is_open {
            // Return empty node when closed
            return ctx.request_layout(Style {
                display: Display::None,
                ..Default::default()
            });
        }

        // Layout content if present
        let content_node = if let Some(content) = &mut self.content {
            Some(content.layout(ctx))
        } else {
            None
        };
        self.content_node = content_node;

        // Dialog container style
        let dialog_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: taffy::Rect {
                left: LengthPercentage::length(self.dialog_padding),
                right: LengthPercentage::length(self.dialog_padding),
                top: LengthPercentage::length(self.dialog_padding),
                bottom: LengthPercentage::length(self.dialog_padding),
            },
            min_size: Size {
                width: Dimension::length(self.dialog_min_width),
                height: Dimension::auto(),
            },
            max_size: Size {
                width: Dimension::length(self.dialog_max_width),
                height: Dimension::auto(),
            },
            ..Default::default()
        };

        let dialog_node = if let Some(content_node) = content_node {
            ctx.request_layout_with_children(dialog_style, &[content_node])
        } else {
            ctx.request_layout(dialog_style)
        };
        self.dialog_node = Some(dialog_node);

        // Backdrop style - full screen, centered content
        let backdrop_style = Style {
            position: Position::Absolute,
            inset: taffy::Rect {
                left: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                top: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0),
            },
            display: Display::Flex,
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            ..Default::default()
        };

        let backdrop_node = ctx.request_layout_with_children(backdrop_style, &[dialog_node]);
        self.backdrop_node = Some(backdrop_node);

        backdrop_node
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !self.is_open {
            return;
        }

        // Set up backdrop click handler
        if self.close_on_backdrop_click {
            let on_close = self.on_close.clone();
            self.backdrop_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _| {
                // Only close if click was on backdrop, not dialog
                // (This is handled by z-index - dialog is on top)
                if let Ok(mut on_close) = on_close.try_borrow_mut() {
                    on_close();
                }
            }));
        }

        // Set up escape key handler
        if self.close_on_escape {
            let on_close = self.on_close.clone();
            self.dialog_handlers.borrow_mut().on_key_down =
                Some(Box::new(move |key, _, _, _| {
                    if key == Key::Escape {
                        if let Ok(mut on_close) = on_close.try_borrow_mut() {
                            on_close();
                        }
                    }
                }));
        }

        // Register backdrop for interaction
        register_element(self.backdrop_id, self.backdrop_handlers.clone());

        // Paint backdrop
        ctx.paint_solid_quad(bounds, self.backdrop_color);

        // Register backdrop for hit testing (lower z-index)
        ctx.register_hit_test(self.backdrop_id, bounds, 0);

        // Get dialog bounds
        if let Some(dialog_node) = self.dialog_node {
            let dialog_local_bounds = ctx.layout_engine.layout_bounds(dialog_node);
            let dialog_bounds = Rect::from_pos_size(
                bounds.pos + dialog_local_bounds.pos,
                dialog_local_bounds.size,
            );

            // Register dialog for interaction (for key events)
            register_element(self.dialog_id, self.dialog_handlers.clone());

            // Paint dialog background
            ctx.paint_quad(PaintQuad {
                bounds: dialog_bounds,
                fill: self.dialog_background,
                corner_radii: Corners::all(self.dialog_corner_radius),
                border_widths: Edges::all(self.dialog_border_width),
                border_color: self.dialog_border_color.unwrap_or(colors::TRANSPARENT),
            });

            // Register dialog for hit testing (higher z-index to block backdrop clicks)
            ctx.register_hit_test(self.dialog_id, dialog_bounds, 10);

            // Paint content
            if let (Some(content), Some(content_node)) = (&mut self.content, self.content_node) {
                let content_local_bounds = ctx.layout_engine.layout_bounds(content_node);
                let content_bounds = Rect::from_pos_size(
                    dialog_bounds.pos + content_local_bounds.pos,
                    content_local_bounds.size,
                );
                content.paint(content_bounds, ctx);
            }
        }
    }
}

/// A confirmation dialog with title, message, and action buttons
pub struct ConfirmDialog {
    /// Whether the dialog is open
    is_open: bool,

    /// Title text
    title: String,

    /// Message text
    message: String,

    /// Confirm button label
    confirm_label: String,

    /// Cancel button label
    cancel_label: String,

    /// Confirm callback
    on_confirm: Rc<RefCell<Box<dyn FnMut()>>>,

    /// Cancel callback (also called on backdrop click / escape)
    on_cancel: Rc<RefCell<Box<dyn FnMut()>>>,

    /// Confirm button color
    confirm_color: Color,

    /// Confirm button hover color
    confirm_hover_color: Color,

    /// Confirm button press color
    confirm_press_color: Color,

    /// IDs for buttons
    confirm_button_id: ElementId,
    cancel_button_id: ElementId,
    backdrop_id: ElementId,
    dialog_id: ElementId,

    /// Event handlers
    confirm_handlers: Rc<RefCell<EventHandlers>>,
    cancel_handlers: Rc<RefCell<EventHandlers>>,
    backdrop_handlers: Rc<RefCell<EventHandlers>>,
    dialog_handlers: Rc<RefCell<EventHandlers>>,

    /// Cached nodes
    backdrop_node: Option<NodeId>,
    dialog_node: Option<NodeId>,
    title_node: Option<NodeId>,
    message_node: Option<NodeId>,
    button_row_node: Option<NodeId>,
    confirm_node: Option<NodeId>,
    cancel_node: Option<NodeId>,
}

/// Create a confirmation dialog
pub fn confirm_dialog<F1, F2>(
    is_open: bool,
    title: impl Into<String>,
    message: impl Into<String>,
    on_confirm: F1,
    on_cancel: F2,
) -> ConfirmDialog
where
    F1: FnMut() + 'static,
    F2: FnMut() + 'static,
{
    ConfirmDialog::new(is_open, title, message, on_confirm, on_cancel)
}

impl ConfirmDialog {
    pub fn new<F1, F2>(
        is_open: bool,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: F1,
        on_cancel: F2,
    ) -> Self
    where
        F1: FnMut() + 'static,
        F2: FnMut() + 'static,
    {
        Self {
            is_open,
            title: title.into(),
            message: message.into(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            on_confirm: Rc::new(RefCell::new(Box::new(on_confirm))),
            on_cancel: Rc::new(RefCell::new(Box::new(on_cancel))),
            confirm_color: colors::BLUE_500,
            confirm_hover_color: colors::BLUE_400,
            confirm_press_color: colors::BLUE_600,
            confirm_button_id: ElementId::auto(),
            cancel_button_id: ElementId::auto(),
            backdrop_id: ElementId::auto(),
            dialog_id: ElementId::auto(),
            confirm_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            cancel_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            backdrop_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            dialog_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            backdrop_node: None,
            dialog_node: None,
            title_node: None,
            message_node: None,
            button_row_node: None,
            confirm_node: None,
            cancel_node: None,
        }
    }

    /// Set the confirm button label
    pub fn confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Set the cancel button label
    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Set the confirm button color
    pub fn confirm_color(mut self, color: Color) -> Self {
        self.confirm_color = color;
        self
    }

    /// Set as a destructive action (red confirm button)
    pub fn destructive(mut self) -> Self {
        self.confirm_color = colors::RED_500;
        self.confirm_hover_color = colors::RED_400;
        self.confirm_press_color = colors::RED_600;
        self
    }
}

impl Element for ConfirmDialog {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        if !self.is_open {
            return ctx.request_layout(Style {
                display: Display::None,
                ..Default::default()
            });
        }

        use crate::style::TextStyle;

        // Title text style
        let title_style = TextStyle {
            size: 18.0,
            color: colors::GRAY_900,
            ..Default::default()
        };

        // Message text style
        let message_style = TextStyle {
            size: 14.0,
            color: colors::GRAY_600,
            ..Default::default()
        };

        // Button text style
        let button_text_style = TextStyle {
            size: 14.0,
            color: colors::WHITE,
            ..Default::default()
        };

        let cancel_text_style = TextStyle {
            size: 14.0,
            color: colors::GRAY_700,
            ..Default::default()
        };

        // Title node
        let title_node = ctx.request_text_layout(
            Style {
                margin: taffy::Rect {
                    left: LengthPercentageAuto::length(0.0),
                    right: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(8.0),
                },
                ..Default::default()
            },
            &self.title,
            &title_style,
        );
        self.title_node = Some(title_node);

        // Message node
        let message_node = ctx.request_text_layout(
            Style {
                margin: taffy::Rect {
                    left: LengthPercentageAuto::length(0.0),
                    right: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(24.0),
                },
                ..Default::default()
            },
            &self.message,
            &message_style,
        );
        self.message_node = Some(message_node);

        // Cancel button node
        let cancel_node = ctx.request_text_layout(
            Style {
                padding: taffy::Rect {
                    left: LengthPercentage::length(16.0),
                    right: LengthPercentage::length(16.0),
                    top: LengthPercentage::length(8.0),
                    bottom: LengthPercentage::length(8.0),
                },
                margin: taffy::Rect {
                    left: LengthPercentageAuto::length(0.0),
                    right: LengthPercentageAuto::length(8.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(0.0),
                },
                ..Default::default()
            },
            &self.cancel_label,
            &cancel_text_style,
        );
        self.cancel_node = Some(cancel_node);

        // Confirm button node
        let confirm_node = ctx.request_text_layout(
            Style {
                padding: taffy::Rect {
                    left: LengthPercentage::length(16.0),
                    right: LengthPercentage::length(16.0),
                    top: LengthPercentage::length(8.0),
                    bottom: LengthPercentage::length(8.0),
                },
                ..Default::default()
            },
            &self.confirm_label,
            &button_text_style,
        );
        self.confirm_node = Some(confirm_node);

        // Button row
        let button_row_node = ctx.request_layout_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: Some(JustifyContent::FlexEnd),
                ..Default::default()
            },
            &[cancel_node, confirm_node],
        );
        self.button_row_node = Some(button_row_node);

        // Dialog content container
        let dialog_node = ctx.request_layout_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: taffy::Rect {
                    left: LengthPercentage::length(24.0),
                    right: LengthPercentage::length(24.0),
                    top: LengthPercentage::length(24.0),
                    bottom: LengthPercentage::length(24.0),
                },
                min_size: Size {
                    width: Dimension::length(300.0),
                    height: Dimension::auto(),
                },
                max_size: Size {
                    width: Dimension::length(400.0),
                    height: Dimension::auto(),
                },
                ..Default::default()
            },
            &[title_node, message_node, button_row_node],
        );
        self.dialog_node = Some(dialog_node);

        // Backdrop - full screen centered
        let backdrop_node = ctx.request_layout_with_children(
            Style {
                position: Position::Absolute,
                inset: taffy::Rect {
                    left: LengthPercentageAuto::length(0.0),
                    right: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(0.0),
                },
                display: Display::Flex,
                justify_content: Some(JustifyContent::Center),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            &[dialog_node],
        );
        self.backdrop_node = Some(backdrop_node);

        backdrop_node
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        use crate::interaction::registry::get_element_state;
        use crate::render::PaintText;
        use crate::style::TextStyle;

        if !self.is_open {
            return;
        }

        // Set up backdrop click handler
        {
            let on_cancel = self.on_cancel.clone();
            self.backdrop_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _| {
                if let Ok(mut cb) = on_cancel.try_borrow_mut() {
                    cb();
                }
            }));
        }

        // Set up escape key handler
        {
            let on_cancel = self.on_cancel.clone();
            self.dialog_handlers.borrow_mut().on_key_down =
                Some(Box::new(move |key, _, _, _| {
                    if key == Key::Escape {
                        if let Ok(mut cb) = on_cancel.try_borrow_mut() {
                            cb();
                        }
                    }
                }));
        }

        // Set up confirm button handler
        {
            let on_confirm = self.on_confirm.clone();
            self.confirm_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _| {
                if let Ok(mut cb) = on_confirm.try_borrow_mut() {
                    cb();
                }
            }));
        }

        // Set up cancel button handler
        {
            let on_cancel = self.on_cancel.clone();
            self.cancel_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _| {
                if let Ok(mut cb) = on_cancel.try_borrow_mut() {
                    cb();
                }
            }));
        }

        // Register backdrop
        register_element(self.backdrop_id, self.backdrop_handlers.clone());

        // Paint backdrop
        ctx.paint_solid_quad(bounds, Color::rgba(0.0, 0.0, 0.0, 0.5));
        ctx.register_hit_test(self.backdrop_id, bounds, 0);

        // Get dialog bounds
        if let Some(dialog_node) = self.dialog_node {
            let dialog_local = ctx.layout_engine.layout_bounds(dialog_node);
            let dialog_bounds = Rect::from_pos_size(
                bounds.pos + dialog_local.pos,
                dialog_local.size,
            );

            // Register dialog for key events
            register_element(self.dialog_id, self.dialog_handlers.clone());

            // Paint dialog background
            ctx.paint_quad(PaintQuad {
                bounds: dialog_bounds,
                fill: colors::WHITE,
                corner_radii: Corners::all(8.0),
                border_widths: Edges::all(1.0),
                border_color: colors::GRAY_300,
            });

            // Block backdrop clicks
            ctx.register_hit_test(self.dialog_id, dialog_bounds, 10);

            // Paint title
            if let Some(title_node) = self.title_node {
                let title_local = ctx.layout_engine.layout_bounds(title_node);
                let title_pos = dialog_bounds.pos + title_local.pos;
                ctx.paint_text(PaintText {
                    position: title_pos,
                    text: self.title.clone(),
                    style: TextStyle {
                        size: 18.0,
                        color: colors::GRAY_900,
                        ..Default::default()
                    },
                    measured_size: None,
                });
            }

            // Paint message
            if let Some(message_node) = self.message_node {
                let message_local = ctx.layout_engine.layout_bounds(message_node);
                let message_pos = dialog_bounds.pos + message_local.pos;
                ctx.paint_text(PaintText {
                    position: message_pos,
                    text: self.message.clone(),
                    style: TextStyle {
                        size: 14.0,
                        color: colors::GRAY_600,
                        ..Default::default()
                    },
                    measured_size: None,
                });
            }

            // Paint cancel button
            if let Some(cancel_node) = self.cancel_node {
                let cancel_local = ctx.layout_engine.layout_bounds(cancel_node);
                // Need to get button row position first
                if let Some(button_row_node) = self.button_row_node {
                    let row_local = ctx.layout_engine.layout_bounds(button_row_node);
                    let cancel_bounds = Rect::from_pos_size(
                        dialog_bounds.pos + row_local.pos + cancel_local.pos,
                        cancel_local.size,
                    );

                    register_element(self.cancel_button_id, self.cancel_handlers.clone());
                    let state = get_element_state(self.cancel_button_id).unwrap_or_default();

                    let bg = if state.is_pressed {
                        colors::GRAY_200
                    } else if state.is_hovered {
                        colors::GRAY_100
                    } else {
                        colors::WHITE
                    };

                    ctx.paint_quad(PaintQuad {
                        bounds: cancel_bounds,
                        fill: bg,
                        corner_radii: Corners::all(4.0),
                        border_widths: Edges::all(1.0),
                        border_color: colors::GRAY_300,
                    });

                    // Center text in button
                    ctx.paint_text(PaintText {
                        position: cancel_bounds.pos + glam::Vec2::new(16.0, 8.0),
                        text: self.cancel_label.clone(),
                        style: TextStyle {
                            size: 14.0,
                            color: colors::GRAY_700,
                            ..Default::default()
                        },
                        measured_size: None,
                    });

                    ctx.register_hit_test(self.cancel_button_id, cancel_bounds, 20);
                }
            }

            // Paint confirm button
            if let Some(confirm_node) = self.confirm_node {
                if let Some(button_row_node) = self.button_row_node {
                    let row_local = ctx.layout_engine.layout_bounds(button_row_node);
                    let confirm_local = ctx.layout_engine.layout_bounds(confirm_node);
                    let confirm_bounds = Rect::from_pos_size(
                        dialog_bounds.pos + row_local.pos + confirm_local.pos,
                        confirm_local.size,
                    );

                    register_element(self.confirm_button_id, self.confirm_handlers.clone());
                    let state = get_element_state(self.confirm_button_id).unwrap_or_default();

                    let bg = if state.is_pressed {
                        self.confirm_press_color
                    } else if state.is_hovered {
                        self.confirm_hover_color
                    } else {
                        self.confirm_color
                    };

                    ctx.paint_quad(PaintQuad {
                        bounds: confirm_bounds,
                        fill: bg,
                        corner_radii: Corners::all(4.0),
                        border_widths: Edges::zero(),
                        border_color: colors::TRANSPARENT,
                    });

                    ctx.paint_text(PaintText {
                        position: confirm_bounds.pos + glam::Vec2::new(16.0, 8.0),
                        text: self.confirm_label.clone(),
                        style: TextStyle {
                            size: 14.0,
                            color: colors::WHITE,
                            ..Default::default()
                        },
                        measured_size: None,
                    });

                    ctx.register_hit_test(self.confirm_button_id, confirm_bounds, 20);
                }
            }
        }
    }
}
