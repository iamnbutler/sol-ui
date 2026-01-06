use crate::{
    color::Color,
    element::{Element, LayoutContext, PaintContext},
    geometry::{Corners, Edges, Rect},
    layout_id::LayoutId,
    render::PaintQuad,
};
use taffy::prelude::*;

/// Create a new container element
pub fn container() -> Container {
    Container::new()
}

/// Create a new horizontal layout container
pub fn row() -> Container {
    Container::new().flex_row()
}

/// Create a new vertical layout container
pub fn column() -> Container {
    Container::new().flex_col()
}

/// A container element that can have children and styling
pub struct Container {
    style: Style,
    background: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    corner_radius: f32,
    children: Vec<Box<dyn Element>>,
    child_nodes: Vec<NodeId>,
    /// Stable layout ID for caching across frames
    layout_id: Option<LayoutId>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            background: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            children: Vec::new(),
            child_nodes: Vec::new(),
            layout_id: None,
        }
    }

    /// Set a stable layout ID for caching across frames.
    ///
    /// Elements with layout IDs will have their Taffy nodes reused
    /// when style and children haven't changed, improving performance.
    ///
    /// # Example
    /// ```ignore
    /// container()
    ///     .layout_id("sidebar")
    ///     .child(button("Save"))
    /// ```
    pub fn layout_id(mut self, id: impl Into<LayoutId>) -> Self {
        self.layout_id = Some(id.into());
        self
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

    /// Add a child element
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Set display to flex
    pub fn flex(mut self) -> Self {
        self.style.display = Display::Flex;
        self
    }

    /// Set flex direction to column
    pub fn flex_col(mut self) -> Self {
        self.style.display = Display::Flex;
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    /// Set flex direction to row
    pub fn flex_row(mut self) -> Self {
        self.style.display = Display::Flex;
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    /// Set flex grow factor.
    ///
    /// Determines how much this item should grow relative to siblings
    /// when there's extra space. Default is 0 (don't grow).
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Set flex shrink factor.
    ///
    /// Determines how much this item should shrink relative to siblings
    /// when there's not enough space. Default is 1.
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    /// Set flex grow to 1 (shorthand for common case).
    pub fn grow(mut self) -> Self {
        self.style.flex_grow = 1.0;
        self
    }

    /// Set gap between flex items
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::length(gap),
            height: LengthPercentage::length(gap),
        };
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

    /// Set margin
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin = taffy::Rect {
            left: LengthPercentageAuto::length(margin),
            right: LengthPercentageAuto::length(margin),
            top: LengthPercentageAuto::length(margin),
            bottom: LengthPercentageAuto::length(margin),
        };
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

    // --- Min/Max Size Constraints ---

    /// Set minimum width
    pub fn min_width(mut self, width: f32) -> Self {
        self.style.min_size.width = Dimension::length(width);
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: f32) -> Self {
        self.style.max_size.width = Dimension::length(width);
        self
    }

    /// Set minimum height
    pub fn min_height(mut self, height: f32) -> Self {
        self.style.min_size.height = Dimension::length(height);
        self
    }

    /// Set maximum height
    pub fn max_height(mut self, height: f32) -> Self {
        self.style.max_size.height = Dimension::length(height);
        self
    }

    /// Set minimum width to 100%
    pub fn min_width_full(mut self) -> Self {
        self.style.min_size.width = Dimension::percent(1.0);
        self
    }

    /// Set minimum height to 100%
    pub fn min_height_full(mut self) -> Self {
        self.style.min_size.height = Dimension::percent(1.0);
        self
    }

    // --- Justify Content (main axis) ---

    /// Align items to the start of the main axis
    pub fn justify_start(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::Start);
        self
    }

    /// Align items to the end of the main axis
    pub fn justify_end(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::End);
        self
    }

    /// Center items on main axis
    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::Center);
        self
    }

    /// Distribute items with equal space between them
    pub fn justify_between(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::SpaceBetween);
        self
    }

    /// Distribute items with equal space around them
    pub fn justify_around(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::SpaceAround);
        self
    }

    /// Distribute items with equal space between and around them
    pub fn justify_evenly(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::SpaceEvenly);
        self
    }

    // --- Align Items (cross axis) ---

    /// Align items to the start of the cross axis
    pub fn items_start(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Start);
        self
    }

    /// Align items to the end of the cross axis
    pub fn items_end(mut self) -> Self {
        self.style.align_items = Some(AlignItems::End);
        self
    }

    /// Center items on cross axis
    pub fn items_center(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Center);
        self
    }

    /// Stretch items to fill the cross axis
    pub fn items_stretch(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Stretch);
        self
    }

    /// Align items along their baseline
    pub fn items_baseline(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Baseline);
        self
    }

    // --- Align Self (override parent's align-items for this element) ---

    /// Override alignment for this item to start
    pub fn align_self_start(mut self) -> Self {
        self.style.align_self = Some(AlignSelf::Start);
        self
    }

    /// Override alignment for this item to end
    pub fn align_self_end(mut self) -> Self {
        self.style.align_self = Some(AlignSelf::End);
        self
    }

    /// Override alignment for this item to center
    pub fn align_self_center(mut self) -> Self {
        self.style.align_self = Some(AlignSelf::Center);
        self
    }

    /// Override alignment for this item to stretch
    pub fn align_self_stretch(mut self) -> Self {
        self.style.align_self = Some(AlignSelf::Stretch);
        self
    }
}

impl Element for Container {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Layout all children first
        self.child_nodes.clear();
        for child in &mut self.children {
            let child_node = child.layout(ctx);
            self.child_nodes.push(child_node);
        }

        // Use cached layout if we have a stable ID
        if let Some(ref layout_id) = self.layout_id {
            // Generate positional IDs for children (for change detection)
            let child_ids: Vec<LayoutId> = (0..self.child_nodes.len())
                .map(|i| layout_id.child(i as u32))
                .collect();

            ctx.request_layout_cached(
                layout_id,
                self.style.clone(),
                &child_ids,
                &self.child_nodes,
            )
        } else {
            // Fallback to immediate mode (no caching)
            ctx.request_layout_with_children(self.style.clone(), &self.child_nodes)
        }
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Paint background and borders
        if self.background.is_some() || self.border_color.is_some() {
            ctx.paint_quad(PaintQuad {
                bounds,
                fill: self.background.unwrap_or(crate::color::colors::TRANSPARENT),
                corner_radii: Corners::all(self.corner_radius),
                border_widths: Edges::all(self.border_width),
                border_color: self
                    .border_color
                    .unwrap_or(crate::color::colors::TRANSPARENT),
            });
        }

        // Paint children with their computed bounds relative to this container
        for (child, &child_node) in self.children.iter_mut().zip(&self.child_nodes) {
            // Get child's layout bounds (relative to parent)
            let child_layout_bounds = ctx.layout_engine.layout_bounds(child_node);
            // Convert to absolute bounds for painting
            let child_absolute_bounds = Rect::from_pos_size(
                bounds.pos + child_layout_bounds.pos,
                child_layout_bounds.size,
            );

            child.paint(child_absolute_bounds, ctx);
        }
    }
}
