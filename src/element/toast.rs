//! Toast notification element

use crate::{
    color::{colors, Color, ColorExt},
    element::{Element, LayoutContext},
    geometry::{Corners, Edges, Rect},
    interaction::{registry::register_element, ElementId, EventHandlers},
    render::{PaintContext, PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Severity level for toast notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastSeverity {
    /// Get the background color for this severity
    fn background_color(&self) -> Color {
        match self {
            ToastSeverity::Info => colors::BLUE_500,
            ToastSeverity::Success => colors::GREEN_500,
            ToastSeverity::Warning => Color::rgba(0.9, 0.7, 0.0, 1.0), // Yellow/amber
            ToastSeverity::Error => colors::RED_500,
        }
    }

    /// Get the icon character for this severity
    fn icon(&self) -> &'static str {
        match self {
            ToastSeverity::Info => "i",
            ToastSeverity::Success => "✓",
            ToastSeverity::Warning => "!",
            ToastSeverity::Error => "✕",
        }
    }
}

/// Position for toast notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    #[default]
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Create a new toast element
pub fn toast(message: impl Into<String>) -> Toast {
    Toast::new(message)
}

/// A toast notification element
pub struct Toast {
    /// Toast message
    message: String,
    /// Severity level
    severity: ToastSeverity,
    /// Position on screen
    position: ToastPosition,
    /// Whether the toast is visible
    visible: bool,
    /// Corner radius
    corner_radius: f32,
    /// Padding
    padding: f32,
    /// Min width
    min_width: f32,
    /// Dismiss callback
    on_dismiss: Option<Rc<RefCell<Box<dyn FnMut()>>>>,
    /// Element ID for dismiss button
    dismiss_id: ElementId,
    /// Dismiss handlers
    dismiss_handlers: Rc<RefCell<EventHandlers>>,
    /// Margin from screen edge
    margin: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: ToastSeverity::Info,
            position: ToastPosition::TopRight,
            visible: true,
            corner_radius: 6.0,
            padding: 12.0,
            min_width: 200.0,
            on_dismiss: None,
            dismiss_id: ElementId::auto(),
            dismiss_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            margin: 16.0,
        }
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set severity level
    pub fn severity(mut self, severity: ToastSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set info severity
    pub fn info(self) -> Self {
        self.severity(ToastSeverity::Info)
    }

    /// Set success severity
    pub fn success(self) -> Self {
        self.severity(ToastSeverity::Success)
    }

    /// Set warning severity
    pub fn warning(self) -> Self {
        self.severity(ToastSeverity::Warning)
    }

    /// Set error severity
    pub fn error(self) -> Self {
        self.severity(ToastSeverity::Error)
    }

    /// Set position
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Set dismiss callback
    pub fn on_dismiss<F>(mut self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_dismiss = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self::new("")
    }
}

impl Element for Toast {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Toast takes no space in layout - it's positioned absolutely
        ctx.request_layout(Style::default())
    }

    fn paint(&mut self, _bounds: Rect, ctx: &mut PaintContext) {
        if !self.visible {
            return;
        }

        // Get viewport for positioning
        let viewport = ctx.draw_list.viewport().unwrap_or(Rect::from_pos_size(
            Vec2::ZERO,
            Vec2::new(800.0, 600.0),
        ));

        // Measure text to determine toast size
        let text_style = TextStyle {
            size: 14.0,
            color: colors::WHITE,
            ..Default::default()
        };
        let text_size = ctx.text_system.measure_text(
            &self.message,
            &crate::text_system::TextConfig {
                font_stack: parley::FontStack::from("system-ui"),
                size: text_style.size,
                weight: parley::FontWeight::NORMAL,
                color: text_style.color.clone(),
                line_height: 1.2,
            },
            Some(300.0), // Max width for text
            ctx.scale_factor,
        );

        // Calculate toast dimensions
        let icon_space = 24.0;
        let dismiss_space = 24.0;
        let toast_width = (text_size.x + icon_space + dismiss_space + self.padding * 2.0)
            .max(self.min_width);
        let toast_height = text_size.y.max(20.0) + self.padding * 2.0;

        // Calculate position based on ToastPosition
        let toast_pos = match self.position {
            ToastPosition::TopLeft => Vec2::new(
                viewport.pos.x + self.margin,
                viewport.pos.y + self.margin,
            ),
            ToastPosition::TopCenter => Vec2::new(
                viewport.pos.x + (viewport.size.x - toast_width) / 2.0,
                viewport.pos.y + self.margin,
            ),
            ToastPosition::TopRight => Vec2::new(
                viewport.pos.x + viewport.size.x - toast_width - self.margin,
                viewport.pos.y + self.margin,
            ),
            ToastPosition::BottomLeft => Vec2::new(
                viewport.pos.x + self.margin,
                viewport.pos.y + viewport.size.y - toast_height - self.margin,
            ),
            ToastPosition::BottomCenter => Vec2::new(
                viewport.pos.x + (viewport.size.x - toast_width) / 2.0,
                viewport.pos.y + viewport.size.y - toast_height - self.margin,
            ),
            ToastPosition::BottomRight => Vec2::new(
                viewport.pos.x + viewport.size.x - toast_width - self.margin,
                viewport.pos.y + viewport.size.y - toast_height - self.margin,
            ),
        };

        let toast_bounds = Rect::from_pos_size(toast_pos, Vec2::new(toast_width, toast_height));

        // Paint toast background
        ctx.paint_quad(PaintQuad {
            bounds: toast_bounds,
            fill: self.severity.background_color(),
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::zero(),
            border_color: colors::TRANSPARENT,
        });

        // Paint icon
        let icon_pos = Vec2::new(
            toast_pos.x + self.padding,
            toast_pos.y + (toast_height - 16.0) / 2.0,
        );
        ctx.paint_text(PaintText {
            position: icon_pos,
            text: self.severity.icon().to_string(),
            style: TextStyle {
                size: 16.0,
                color: colors::WHITE,
                ..Default::default()
            },
            measured_size: None,
        });

        // Paint message
        let text_pos = Vec2::new(
            toast_pos.x + self.padding + icon_space,
            toast_pos.y + (toast_height - text_size.y) / 2.0,
        );
        ctx.paint_text(PaintText {
            position: text_pos,
            text: self.message.clone(),
            style: text_style,
            measured_size: Some(text_size),
        });

        // Paint dismiss button (×)
        let dismiss_pos = Vec2::new(
            toast_pos.x + toast_width - self.padding - 16.0,
            toast_pos.y + (toast_height - 16.0) / 2.0,
        );
        let dismiss_bounds = Rect::from_pos_size(dismiss_pos, Vec2::new(16.0, 16.0));

        // Setup dismiss handler
        if let Some(ref on_dismiss) = self.on_dismiss {
            let handler = on_dismiss.clone();
            self.dismiss_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _, _, _| {
                (handler.borrow_mut())();
            }));
            register_element(self.dismiss_id, self.dismiss_handlers.clone());
            ctx.register_hit_test(self.dismiss_id, dismiss_bounds, 1100);
        }

        ctx.paint_text(PaintText {
            position: dismiss_pos,
            text: "×".to_string(),
            style: TextStyle {
                size: 16.0,
                color: colors::WHITE,
                ..Default::default()
            },
            measured_size: None,
        });
    }
}
