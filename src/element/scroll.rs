//! Scrollable container element

use crate::{
    color::{Color, ColorExt},
    element::{Element, LayoutContext},
    entity::{Entity, new_entity, read_entity, update_entity},
    geometry::{Corners, Edges, Rect},
    render::{PaintContext, PaintQuad},
};
use glam::Vec2;
use taffy::{Overflow, prelude::*};

/// State for a scroll container, persisted via the Entity system
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current scroll offset (positive = scrolled down/right)
    pub offset: Vec2,
    /// Content size from last frame (for scroll limit calculation)
    pub content_size: Vec2,
    /// Viewport size from last frame
    pub viewport_size: Vec2,
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the maximum scroll offset (content size - viewport size, clamped to 0)
    pub fn max_offset(&self) -> Vec2 {
        Vec2::new(
            (self.content_size.x - self.viewport_size.x).max(0.0),
            (self.content_size.y - self.viewport_size.y).max(0.0),
        )
    }

    /// Clamp the current offset to valid bounds
    pub fn clamp_offset(&mut self) {
        let max = self.max_offset();
        self.offset = self.offset.clamp(Vec2::ZERO, max);
    }
}

/// Create a new scroll container
pub fn scroll() -> ScrollContainer {
    ScrollContainer::new()
}

/// A scrollable container element
pub struct ScrollContainer {
    style: Style,
    background: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    corner_radius: f32,
    scrollbar_color: Option<Color>,
    scrollbar_width: f32,
    show_scrollbar: bool,
    children: Vec<Box<dyn Element>>,
    child_nodes: Vec<NodeId>,
    state: Option<Entity<ScrollState>>,
}

impl ScrollContainer {
    pub fn new() -> Self {
        Self {
            style: Style {
                overflow: taffy::Point {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                },
                ..Style::default()
            },
            background: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            scrollbar_color: Some(Color::rgba(0.5, 0.5, 0.5, 0.5)),
            scrollbar_width: 8.0,
            show_scrollbar: true,
            children: Vec::new(),
            child_nodes: Vec::new(),
            state: None,
        }
    }

    /// Set the background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the border (color and width)
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    /// Set only the border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set only the border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set uniform padding (all sides)
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = taffy::Rect {
            left: LengthPercentage::length(padding),
            right: LengthPercentage::length(padding),
            top: LengthPercentage::length(padding),
            bottom: LengthPercentage::length(padding),
        };
        self
    }

    /// Set horizontal and vertical padding separately
    pub fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.style.padding = taffy::Rect {
            left: LengthPercentage::length(horizontal),
            right: LengthPercentage::length(horizontal),
            top: LengthPercentage::length(vertical),
            bottom: LengthPercentage::length(vertical),
        };
        self
    }

    /// Set horizontal padding (left and right)
    pub fn padding_h(mut self, padding: f32) -> Self {
        self.style.padding.left = LengthPercentage::length(padding);
        self.style.padding.right = LengthPercentage::length(padding);
        self
    }

    /// Set vertical padding (top and bottom)
    pub fn padding_v(mut self, padding: f32) -> Self {
        self.style.padding.top = LengthPercentage::length(padding);
        self.style.padding.bottom = LengthPercentage::length(padding);
        self
    }

    /// Set scrollbar visibility
    pub fn scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Set scrollbar color
    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.scrollbar_color = Some(color);
        self
    }

    /// Set scrollbar width
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width;
        self
    }

    /// Add a child element
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::length(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::length(height);
        self
    }

    /// Set both width and height
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.size = Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        };
        self
    }

    /// Set width to 100%
    pub fn width_full(mut self) -> Self {
        self.style.size.width = Dimension::percent(1.0);
        self
    }

    /// Set height to 100%
    pub fn height_full(mut self) -> Self {
        self.style.size.height = Dimension::percent(1.0);
        self
    }

    /// Set flex grow
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Apply scroll delta to this container (called from event handling)
    pub fn apply_scroll(&self, delta: Vec2) {
        if let Some(ref state) = self.state {
            update_entity(state, |s| {
                // Negative delta because scrolling down should increase offset
                s.offset.y -= delta.y;
                s.offset.x -= delta.x;
                s.clamp_offset();
            });
        }
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> Vec2 {
        self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.offset))
            .unwrap_or(Vec2::ZERO)
    }

    /// Get the entity handle for external scroll control
    pub fn state_entity(&self) -> Option<Entity<ScrollState>> {
        self.state.clone()
    }
}

impl Default for ScrollContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for ScrollContainer {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Initialize state entity if not already done
        if self.state.is_none() {
            self.state = Some(new_entity(ScrollState::new()));
        }

        // Layout all children first
        // Create an inner container that can grow beyond the viewport
        self.child_nodes.clear();
        for child in &mut self.children {
            let child_node = child.layout(ctx);
            self.child_nodes.push(child_node);
        }

        // Create an inner column container for children that can grow
        let inner_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            // Allow content to grow beyond container
            min_size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::auto(),
            },
            ..Style::default()
        };

        let inner_node = ctx.request_layout_with_children(inner_style, &self.child_nodes);

        // The outer container clips content
        ctx.request_layout_with_children(self.style.clone(), &[inner_node])
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Paint background and border
        if self.background.is_some() || self.border_color.is_some() {
            ctx.paint_quad(PaintQuad {
                bounds,
                fill: self.background.unwrap_or(crate::color::colors::TRANSPARENT),
                corner_radii: Corners::all(self.corner_radius),
                border_widths: Edges::all(self.border_width),
                border_color: self.border_color.unwrap_or(crate::color::colors::TRANSPARENT),
            });
        }

        // Get scroll offset from state
        let scroll_offset = self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.offset))
            .unwrap_or(Vec2::ZERO);

        // Push clip rect to confine children to this container's bounds
        ctx.draw_list.push_clip(bounds);

        // Paint children with scroll offset applied
        for (child, &child_node) in self.children.iter_mut().zip(&self.child_nodes) {
            // Get child's layout bounds (relative to parent)
            let child_layout_bounds = ctx.layout_engine.layout_bounds(child_node);

            // Apply scroll offset to child position
            let child_absolute_bounds = Rect::from_pos_size(
                bounds.pos + child_layout_bounds.pos - scroll_offset,
                child_layout_bounds.size,
            );

            child.paint(child_absolute_bounds, ctx);
        }

        // Pop clip rect
        ctx.draw_list.pop_clip();

        // Calculate content size for scroll state
        let content_height: f32 = self.child_nodes
            .iter()
            .map(|&node| {
                let child_bounds = ctx.layout_engine.layout_bounds(node);
                child_bounds.pos.y + child_bounds.size.y
            })
            .fold(0.0f32, |a, b| a.max(b));

        let content_size = Vec2::new(bounds.size.x, content_height);

        // Update state with current sizes
        if let Some(ref state) = self.state {
            update_entity(state, |s| {
                s.viewport_size = bounds.size;
                s.content_size = content_size;
                s.clamp_offset();
            });
        }

        // Paint scrollbar if enabled and content overflows
        if self.show_scrollbar && content_size.y > bounds.size.y {
            self.paint_scrollbar(bounds, content_size, scroll_offset, ctx);
        }
    }
}

impl ScrollContainer {
    fn paint_scrollbar(&self, bounds: Rect, content_size: Vec2, scroll_offset: Vec2, ctx: &mut PaintContext) {
        let scrollbar_color = self.scrollbar_color.unwrap_or(Color::rgba(0.5, 0.5, 0.5, 0.5));

        // Calculate scrollbar track position (right side of container)
        let track_x = bounds.pos.x + bounds.size.x - self.scrollbar_width - 2.0;
        let track_y = bounds.pos.y + 2.0;
        let track_height = bounds.size.y - 4.0;

        // Calculate thumb size based on viewport/content ratio
        let visible_ratio = (bounds.size.y / content_size.y).min(1.0);
        let thumb_height = (track_height * visible_ratio).max(20.0);

        // Calculate thumb position based on scroll offset
        let max_scroll = (content_size.y - bounds.size.y).max(0.0);
        let scroll_ratio = if max_scroll > 0.0 {
            scroll_offset.y / max_scroll
        } else {
            0.0
        };
        let thumb_y = track_y + (track_height - thumb_height) * scroll_ratio;

        // Paint scrollbar track (optional, subtle background)
        ctx.paint_quad(PaintQuad {
            bounds: Rect::from_pos_size(
                Vec2::new(track_x, track_y),
                Vec2::new(self.scrollbar_width, track_height),
            ),
            fill: Color::rgba(0.0, 0.0, 0.0, 0.1),
            corner_radii: Corners::all(self.scrollbar_width / 2.0),
            border_widths: Edges::zero(),
            border_color: crate::color::colors::TRANSPARENT,
        });

        // Paint scrollbar thumb
        ctx.paint_quad(PaintQuad {
            bounds: Rect::from_pos_size(
                Vec2::new(track_x, thumb_y),
                Vec2::new(self.scrollbar_width, thumb_height),
            ),
            fill: scrollbar_color,
            corner_radii: Corners::all(self.scrollbar_width / 2.0),
            border_widths: Edges::zero(),
            border_color: crate::color::colors::TRANSPARENT,
        });
    }
}
