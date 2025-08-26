use crate::{
    color::Color,
    element::{Element, LayoutContext, PaintContext},
    geometry::{Corners, Edges, Rect},
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
        }
    }

    /// Set the background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the border
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
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

    /// Set gap between flex items
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::length(gap),
            height: LengthPercentage::length(gap),
        };
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = taffy::Rect {
            left: LengthPercentage::length(padding),
            right: LengthPercentage::length(padding),
            top: LengthPercentage::length(padding),
            bottom: LengthPercentage::length(padding),
        };
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

    /// Center items on main axis
    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::Center);
        self
    }

    /// Center items on cross axis
    pub fn items_center(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Center);
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

        // Request layout with children
        ctx.request_layout_with_children(self.style.clone(), &self.child_nodes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color::colors::*,
        style::TextStyle,
    };
    use taffy::prelude::*;

    #[test]
    fn test_container_creation() {
        let container = container();
        assert_eq!(container.style.display, Display::Block);
        assert_eq!(container.children.len(), 0);
        assert_eq!(container.border_width, 0.0);
        assert_eq!(container.corner_radius, 0.0);
        assert!(container.background.is_none());
        assert!(container.border_color.is_none());
    }

    #[test]
    fn test_container_new() {
        let container = Container::new();
        assert_eq!(container.style.display, Display::Block);
        assert_eq!(container.children.len(), 0);
        assert_eq!(container.border_width, 0.0);
        assert_eq!(container.corner_radius, 0.0);
        assert!(container.background.is_none());
        assert!(container.border_color.is_none());
    }

    #[test]
    fn test_row_creation() {
        let row_container = row();
        assert_eq!(row_container.style.display, Display::Flex);
        assert_eq!(row_container.style.flex_direction, FlexDirection::Row);
    }

    #[test]
    fn test_column_creation() {
        let col_container = column();
        assert_eq!(col_container.style.display, Display::Flex);
        assert_eq!(col_container.style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_background_setting() {
        let container = Container::new().background(RED);
        assert_eq!(container.background, Some(RED));
    }

    #[test]
    fn test_border_setting() {
        let container = Container::new().border(BLUE, 2.0);
        assert_eq!(container.border_color, Some(BLUE));
        assert_eq!(container.border_width, 2.0);
    }

    #[test]
    fn test_corner_radius_setting() {
        let container = Container::new().corner_radius(8.0);
        assert_eq!(container.corner_radius, 8.0);
    }

    #[test]
    fn test_flex_setting() {
        let container = Container::new().flex();
        assert_eq!(container.style.display, Display::Flex);
    }

    #[test]
    fn test_flex_col_setting() {
        let container = Container::new().flex_col();
        assert_eq!(container.style.display, Display::Flex);
        assert_eq!(container.style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_flex_row_setting() {
        let container = Container::new().flex_row();
        assert_eq!(container.style.display, Display::Flex);
        assert_eq!(container.style.flex_direction, FlexDirection::Row);
    }

    #[test]
    fn test_gap_setting() {
        let container = Container::new().gap(10.0);
        assert_eq!(container.style.gap.width, LengthPercentage::length(10.0));
        assert_eq!(container.style.gap.height, LengthPercentage::length(10.0));
    }

    #[test]
    fn test_padding_setting() {
        let container = Container::new().padding(15.0);
        assert_eq!(container.style.padding.left, LengthPercentage::length(15.0));
        assert_eq!(container.style.padding.right, LengthPercentage::length(15.0));
        assert_eq!(container.style.padding.top, LengthPercentage::length(15.0));
        assert_eq!(container.style.padding.bottom, LengthPercentage::length(15.0));
    }

    #[test]
    fn test_margin_setting() {
        let container = Container::new().margin(20.0);
        assert_eq!(container.style.margin.left, LengthPercentageAuto::length(20.0));
        assert_eq!(container.style.margin.right, LengthPercentageAuto::length(20.0));
        assert_eq!(container.style.margin.top, LengthPercentageAuto::length(20.0));
        assert_eq!(container.style.margin.bottom, LengthPercentageAuto::length(20.0));
    }

    #[test]
    fn test_width_setting() {
        let container = Container::new().width(100.0);
        assert_eq!(container.style.size.width, Dimension::length(100.0));
    }

    #[test]
    fn test_height_setting() {
        let container = Container::new().height(200.0);
        assert_eq!(container.style.size.height, Dimension::length(200.0));
    }

    #[test]
    fn test_size_setting() {
        let container = Container::new().size(150.0, 300.0);
        assert_eq!(container.style.size.width, Dimension::length(150.0));
        assert_eq!(container.style.size.height, Dimension::length(300.0));
    }

    #[test]
    fn test_width_full() {
        let container = Container::new().width_full();
        assert_eq!(container.style.size.width, Dimension::percent(1.0));
    }

    #[test]
    fn test_height_full() {
        let container = Container::new().height_full();
        assert_eq!(container.style.size.height, Dimension::percent(1.0));
    }

    #[test]
    fn test_justify_center() {
        let container = Container::new().justify_center();
        assert_eq!(container.style.justify_content, Some(JustifyContent::Center));
    }

    #[test]
    fn test_items_center() {
        let container = Container::new().items_center();
        assert_eq!(container.style.align_items, Some(AlignItems::Center));
    }

    #[test]
    fn test_method_chaining() {
        let container = Container::new()
            .flex_col()
            .background(GREEN)
            .border(BLACK, 1.0)
            .corner_radius(5.0)
            .padding(10.0)
            .margin(5.0)
            .width(200.0)
            .height(100.0)
            .justify_center()
            .items_center();

        assert_eq!(container.style.display, Display::Flex);
        assert_eq!(container.style.flex_direction, FlexDirection::Column);
        assert_eq!(container.background, Some(GREEN));
        assert_eq!(container.border_color, Some(BLACK));
        assert_eq!(container.border_width, 1.0);
        assert_eq!(container.corner_radius, 5.0);
        assert_eq!(container.style.padding.left, LengthPercentage::length(10.0));
        assert_eq!(container.style.margin.left, LengthPercentageAuto::length(5.0));
        assert_eq!(container.style.size.width, Dimension::length(200.0));
        assert_eq!(container.style.size.height, Dimension::length(100.0));
        assert_eq!(container.style.justify_content, Some(JustifyContent::Center));
        assert_eq!(container.style.align_items, Some(AlignItems::Center));
    }

    #[test]
    fn test_container_with_child() {
        let child_container = Container::new().background(BLUE);
        let parent_container = Container::new().child(child_container);
        
        assert_eq!(parent_container.children.len(), 1);
    }

    #[test]
    fn test_container_with_multiple_children() {
        let child1 = Container::new().background(RED);
        let child2 = Container::new().background(GREEN);
        let child3 = Container::new().background(BLUE);
        
        let parent = Container::new()
            .child(child1)
            .child(child2)
            .child(child3);
        
        assert_eq!(parent.children.len(), 3);
    }

    #[test]
    fn test_complex_layout_configuration() {
        let container = row()
            .gap(20.0)
            .padding(15.0)
            .justify_center()
            .items_center()
            .background(GRAY_200)
            .border(GRAY_600, 2.0)
            .corner_radius(10.0)
            .width_full()
            .height(80.0);

        assert_eq!(container.style.display, Display::Flex);
        assert_eq!(container.style.flex_direction, FlexDirection::Row);
        assert_eq!(container.style.gap.width, LengthPercentage::length(20.0));
        assert_eq!(container.style.padding.top, LengthPercentage::length(15.0));
        assert_eq!(container.style.justify_content, Some(JustifyContent::Center));
        assert_eq!(container.style.align_items, Some(AlignItems::Center));
        assert_eq!(container.background, Some(GRAY_200));
        assert_eq!(container.border_color, Some(GRAY_600));
        assert_eq!(container.border_width, 2.0);
        assert_eq!(container.corner_radius, 10.0);
        assert_eq!(container.style.size.width, Dimension::percent(1.0));
        assert_eq!(container.style.size.height, Dimension::length(80.0));
    }

    #[test]
    fn test_nested_container_structure() {
        let inner = Container::new()
            .background(WHITE)
            .padding(5.0);
            
        let middle = Container::new()
            .background(GRAY_300)
            .padding(10.0)
            .child(inner);
            
        let outer = Container::new()
            .background(GRAY_700)
            .padding(20.0)
            .child(middle);
        
        assert_eq!(outer.children.len(), 1);
        assert_eq!(outer.background, Some(GRAY_700));
        assert_eq!(outer.style.padding.left, LengthPercentage::length(20.0));
    }

    // Test default values with different container creation methods
    #[test]
    fn test_container_creation_methods_equivalence() {
        let container1 = container();
        let container2 = Container::new();
        
        // Both should have the same default values
        assert_eq!(container1.style.display, container2.style.display);
        assert_eq!(container1.children.len(), container2.children.len());
        assert_eq!(container1.border_width, container2.border_width);
        assert_eq!(container1.corner_radius, container2.corner_radius);
        assert_eq!(container1.background, container2.background);
        assert_eq!(container1.border_color, container2.border_color);
    }

    #[test]
    fn test_row_vs_column_differences() {
        let row_container = row();
        let col_container = column();
        
        // Both should be flex containers but with different directions
        assert_eq!(row_container.style.display, Display::Flex);
        assert_eq!(col_container.style.display, Display::Flex);
        assert_eq!(row_container.style.flex_direction, FlexDirection::Row);
        assert_eq!(col_container.style.flex_direction, FlexDirection::Column);
    }
}
