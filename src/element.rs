//! Two-phase element rendering system
//!
mod button;
mod checkbox;
mod container;
mod dropdown;
mod icon;
mod list;
mod modal;
mod scroll;
mod text;
mod text_input;
mod toast;
mod tooltip;

pub use button::{Button, button};
pub use checkbox::{Checkbox, CheckboxInteractable, InteractiveCheckbox, checkbox, interactive_checkbox};
pub use container::{Container, column, container, row};
pub use dropdown::{Dropdown, DropdownOption, DropdownState, dropdown};
pub use icon::{Icon, IconButton, IconSource, icon, icon_button, icons};
pub use list::{List, ListAction, ListItemData, ListState, SelectionMode, list};
pub use modal::{Modal, modal};
pub use scroll::{ScrollContainer, ScrollState, scroll};
pub use text::{Text, text};
pub use toast::{Toast, ToastPosition, ToastSeverity, toast};
pub use tooltip::{Tooltip, TooltipPosition, tooltip};
pub use text_input::{
    InteractiveTextInput, TextInput, TextInputInteractable, TextInputState, text_input,
};

use crate::{
    geometry::Rect,
    layout_engine::{ElementData, TaffyLayoutEngine},
    layout_id::LayoutId,
    render::PaintContext,
    style::TextStyle,
    text_system::TextSystem,
};
use glam::Vec2;
use taffy::prelude::*;

/// Elements participate in a two-phase rendering process
pub trait Element {
    /// Phase 1: Declare layout requirements and return a layout id
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId;

    /// Phase 2: Paint using the computed bounds
    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext);
}

/// Context for the layout phase
pub struct LayoutContext<'a> {
    pub(crate) engine: &'a mut TaffyLayoutEngine,
    pub(crate) text_system: &'a mut TextSystem,
    pub(crate) scale_factor: f32,
}

impl<'a> LayoutContext<'a> {
    /// Request layout for a leaf node (no children)
    pub fn request_layout(&mut self, style: Style) -> NodeId {
        self.engine.request_layout(style, &[])
    }

    /// Request layout with children
    pub fn request_layout_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.engine.request_layout(style, children)
    }

    /// Request layout for a text element that needs measuring
    pub fn request_text_layout(
        &mut self,
        style: Style,
        text: &str,
        text_style: &TextStyle,
    ) -> NodeId {
        // Store text data for measurement
        let data = ElementData {
            text: Some((text.to_string(), text_style.clone())),
            background: None,
        };
        self.engine.request_layout_with_data(style, data, &[])
    }

    /// Request layout with custom data
    pub fn request_layout_with_data(
        &mut self,
        style: Style,
        data: ElementData,
        children: &[NodeId],
    ) -> NodeId {
        self.engine.request_layout_with_data(style, data, children)
    }

    /// Measure text (for use during layout)
    pub fn measure_text(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Vec2 {
        let text_config = crate::text_system::TextConfig {
            font_stack: parley::FontStack::from(style.font_family),
            size: style.size,
            weight: style.weight,
            color: style.color.clone(),
            line_height: style.line_height,
        };

        self.text_system
            .measure_text(text, &text_config, max_width, self.scale_factor)
    }

    // --- Cached/Retained Mode Methods ---

    /// Request layout with a stable ID for caching across frames.
    ///
    /// Elements with stable LayoutIds will have their Taffy nodes reused
    /// when style and children haven't changed.
    pub fn request_layout_cached(
        &mut self,
        layout_id: &LayoutId,
        style: Style,
        child_ids: &[LayoutId],
        child_nodes: &[NodeId],
    ) -> NodeId {
        self.engine.request_layout_cached(
            layout_id,
            style,
            ElementData::default(),
            child_ids,
            child_nodes,
        )
    }

    /// Request layout for a text element with a stable ID for caching.
    pub fn request_text_layout_cached(
        &mut self,
        layout_id: &LayoutId,
        style: Style,
        text: &str,
        text_style: &TextStyle,
    ) -> NodeId {
        let data = ElementData {
            text: Some((text.to_string(), text_style.clone())),
            background: None,
        };
        self.engine
            .request_layout_cached(layout_id, style, data, &[], &[])
    }

    /// Request layout with custom data and a stable ID for caching.
    pub fn request_layout_with_data_cached(
        &mut self,
        layout_id: &LayoutId,
        style: Style,
        data: ElementData,
        child_ids: &[LayoutId],
        child_nodes: &[NodeId],
    ) -> NodeId {
        self.engine
            .request_layout_cached(layout_id, style, data, child_ids, child_nodes)
    }
}
