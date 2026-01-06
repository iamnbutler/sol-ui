//! Checkbox element with customizable styling

use crate::{
    color::{Color, colors},
    element::{Element, LayoutContext, PaintContext, text, Text},
    geometry::{Corners, Edges, Rect},
    interaction::{ElementId, Interactable, InteractiveElement},
    layer::MouseButton,
    render::PaintQuad,
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Default checkbox size in pixels
const DEFAULT_SIZE: f32 = 20.0;
/// Default gap between checkbox and label
const DEFAULT_LABEL_GAP: f32 = 8.0;

/// Create a new checkbox element
pub fn checkbox(checked: bool) -> Checkbox {
    Checkbox::new(checked)
}

/// A checkbox element with checked/unchecked states
pub struct Checkbox {
    /// Whether the checkbox is currently checked
    checked: bool,
    /// Size of the checkbox box (width and height)
    size: f32,
    /// Optional label text
    label: Option<String>,
    /// Label text style
    label_style: TextStyle,
    /// Gap between checkbox and label
    label_gap: f32,
    /// Whether the checkbox is disabled
    disabled: bool,
    /// Background color when unchecked
    unchecked_background: Color,
    /// Background color when checked
    checked_background: Color,
    /// Border color
    border_color: Color,
    /// Border width
    border_width: f32,
    /// Corner radius
    corner_radius: f32,
    /// Check mark color
    check_color: Color,
    /// On change callback
    on_change: Option<Rc<RefCell<Box<dyn FnMut(bool)>>>>,
    /// Element ID for interaction
    element_id: ElementId,
    /// Cached layout node ID
    node_id: Option<NodeId>,
    /// Child label element (created during layout)
    label_element: Option<Text>,
    /// Child label node ID
    label_node_id: Option<NodeId>,
}

impl Checkbox {
    /// Create a new checkbox with the given checked state
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            size: DEFAULT_SIZE,
            label: None,
            label_style: TextStyle {
                color: colors::BLACK,
                size: 14.0,
            },
            label_gap: DEFAULT_LABEL_GAP,
            disabled: false,
            unchecked_background: colors::WHITE,
            checked_background: colors::BLUE_500,
            border_color: colors::GRAY_400,
            border_width: 2.0,
            corner_radius: 4.0,
            check_color: colors::WHITE,
            on_change: None,
            element_id: ElementId::auto(),
            node_id: None,
            label_element: None,
            label_node_id: None,
        }
    }

    /// Set the checkbox size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set an optional label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the label text style
    pub fn label_style(mut self, style: TextStyle) -> Self {
        self.label_style = style;
        self
    }

    /// Set the gap between checkbox and label
    pub fn label_gap(mut self, gap: f32) -> Self {
        self.label_gap = gap;
        self
    }

    /// Set whether the checkbox is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the background color when unchecked
    pub fn unchecked_background(mut self, color: Color) -> Self {
        self.unchecked_background = color;
        self
    }

    /// Set the background color when checked
    pub fn checked_background(mut self, color: Color) -> Self {
        self.checked_background = color;
        self
    }

    /// Set the border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set the border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set the corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set the check mark color
    pub fn check_color(mut self, color: Color) -> Self {
        self.check_color = color;
        self
    }

    /// Set a stable element ID
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Set the on_change callback
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(bool) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Paint the check mark inside the given bounds
    fn paint_checkmark(&self, bounds: Rect, ctx: &mut PaintContext) {
        // Draw a simple checkmark using two rectangles forming an "L" shape
        // rotated to look like a checkmark
        //
        // The checkmark consists of:
        // 1. A short diagonal going down-left to bottom
        // 2. A longer diagonal going from bottom up-right
        //
        // Since we can only draw axis-aligned rectangles, we'll approximate
        // with a simplified checkmark using small rectangles

        let inset = self.size * 0.25;
        let check_bounds = Rect::from_pos_size(
            bounds.pos + Vec2::splat(inset),
            bounds.size - Vec2::splat(inset * 2.0),
        );

        let stroke_width = (self.size * 0.15).max(2.0);

        // Short stroke (bottom-left part of check): goes down and slightly right
        // Position: left side, middle-to-bottom
        let short_stroke_height = check_bounds.size.y * 0.4;
        let short_stroke = Rect::from_pos_size(
            Vec2::new(
                check_bounds.pos.x + check_bounds.size.x * 0.15,
                check_bounds.pos.y + check_bounds.size.y * 0.5,
            ),
            Vec2::new(stroke_width, short_stroke_height),
        );

        // Long stroke (bottom-right part of check): goes up and right
        // Position: from bottom-center going up to top-right
        let long_stroke_width = check_bounds.size.x * 0.6;
        let long_stroke = Rect::from_pos_size(
            Vec2::new(
                check_bounds.pos.x + check_bounds.size.x * 0.25,
                check_bounds.pos.y + check_bounds.size.y * 0.7,
            ),
            Vec2::new(long_stroke_width, stroke_width),
        );

        // Draw both strokes
        ctx.paint_quad(PaintQuad::filled(short_stroke, self.check_color));
        ctx.paint_quad(PaintQuad::filled(long_stroke, self.check_color));

        // Add a diagonal connector to make it look more like a checkmark
        // Small square at the intersection
        let connector = Rect::from_pos_size(
            Vec2::new(
                check_bounds.pos.x + check_bounds.size.x * 0.15,
                check_bounds.pos.y + check_bounds.size.y * 0.7,
            ),
            Vec2::new(stroke_width * 1.5, stroke_width),
        );
        ctx.paint_quad(PaintQuad::filled(connector, self.check_color));
    }
}

impl Element for Checkbox {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Create label element if we have label text
        let label_node = if let Some(label_text) = &self.label {
            let mut label = text(label_text.clone()).style(self.label_style.clone());
            let node = label.layout(ctx);
            self.label_element = Some(label);
            self.label_node_id = Some(node);
            Some(node)
        } else {
            self.label_element = None;
            self.label_node_id = None;
            None
        };

        // Calculate total width (checkbox + gap + label if present)
        let checkbox_style = Style {
            size: Size {
                width: Dimension::length(self.size),
                height: Dimension::length(self.size),
            },
            ..Default::default()
        };

        if label_node.is_some() {
            // Container style with row layout
            let container_style = Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Center),
                gap: Size {
                    width: LengthPercentage::length(self.label_gap),
                    height: LengthPercentage::length(0.0),
                },
                ..Default::default()
            };

            // Create checkbox box node
            let checkbox_node = ctx.request_layout(checkbox_style);

            // Create container with checkbox and label as children
            let children = if let Some(ln) = label_node {
                vec![checkbox_node, ln]
            } else {
                vec![checkbox_node]
            };

            let node_id = ctx.request_layout_with_children(container_style, &children);
            self.node_id = Some(node_id);
            node_id
        } else {
            // Just the checkbox box
            let node_id = ctx.request_layout(checkbox_style);
            self.node_id = Some(node_id);
            node_id
        }
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Calculate checkbox box bounds
        let checkbox_bounds = Rect::from_pos_size(
            bounds.pos,
            Vec2::new(self.size, self.size),
        );

        // Determine colors based on state
        let (bg_color, border_color) = if self.disabled {
            (colors::GRAY_200, colors::GRAY_300)
        } else if self.checked {
            (self.checked_background, self.checked_background)
        } else {
            (self.unchecked_background, self.border_color)
        };

        // Paint the checkbox box
        ctx.paint_quad(PaintQuad {
            bounds: checkbox_bounds,
            fill: bg_color,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color,
        });

        // Paint check mark if checked
        if self.checked && !self.disabled {
            self.paint_checkmark(checkbox_bounds, ctx);
        } else if self.checked && self.disabled {
            // Dimmed checkmark for disabled checked state
            let original_check_color = self.check_color;
            self.check_color = colors::GRAY_400;
            self.paint_checkmark(checkbox_bounds, ctx);
            self.check_color = original_check_color;
        }

        // Paint label if present
        if let (Some(label_element), Some(label_node_id)) =
            (&mut self.label_element, self.label_node_id)
        {
            let label_layout_bounds = ctx.layout_engine.layout_bounds(label_node_id);
            let label_bounds = Rect::from_pos_size(
                bounds.pos + Vec2::new(self.size + self.label_gap, label_layout_bounds.pos.y),
                label_layout_bounds.size,
            );
            label_element.paint(label_bounds, ctx);
        }
    }
}

/// A checkbox wrapped with interaction capabilities
pub struct InteractiveCheckbox {
    inner: InteractiveElement<Checkbox>,
}

impl InteractiveCheckbox {
    /// Create a new interactive checkbox
    pub fn new(checkbox: Checkbox) -> Self {
        let element_id = checkbox.element_id;
        let disabled = checkbox.disabled;
        let on_change = checkbox.on_change.clone();
        let checked = checkbox.checked;

        let mut interactive = checkbox.interactive().with_id(element_id);

        if !disabled {
            interactive = interactive
                .hover_overlay(colors::BLACK.with_alpha(0.05))
                .press_overlay(colors::BLACK.with_alpha(0.1))
                .on_click(move |button, _, _| {
                    if button == MouseButton::Left {
                        if let Some(handler) = &on_change {
                            // Toggle the checked state
                            let new_state = !checked;
                            (handler.borrow_mut())(new_state);
                        }
                    }
                });
        } else {
            interactive = interactive.enabled(false);
        }

        Self { inner: interactive }
    }
}

impl Element for InteractiveCheckbox {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        self.inner.layout(ctx)
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        self.inner.paint(bounds, ctx);
    }
}

/// Create an interactive checkbox (convenience function)
pub fn interactive_checkbox(checked: bool) -> Checkbox {
    checkbox(checked)
}

/// Extension trait to make Checkbox interactive
pub trait CheckboxInteractable {
    /// Make this checkbox interactive with click handling
    fn interactive_checkbox(self) -> InteractiveCheckbox;
}

impl CheckboxInteractable for Checkbox {
    fn interactive_checkbox(self) -> InteractiveCheckbox {
        InteractiveCheckbox::new(self)
    }
}

// Import ColorExt for with_alpha
use crate::color::ColorExt;
