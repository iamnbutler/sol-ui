//! Taffy-based layout system with clean builder API

use crate::color::Color;
use crate::draw::{DrawList, TextStyle};
use crate::geometry::Rect;
use crate::text_system::{TextConfig, TextSystem};
use glam::Vec2;
use parley::FontStack;
use taffy::prelude::*;
use tracing::{debug, info_span};

/// The main UI context for building element trees
pub struct UiContext {
    taffy: TaffyTree<ElementData>,
    screen_size: Vec2,
    scale_factor: f32,
    root: Option<NodeId>,
    draw_list: DrawList,
}

impl UiContext {
    /// Create a new UI context
    pub fn new(screen_size: Vec2, scale_factor: f32) -> Self {
        Self {
            taffy: TaffyTree::new(),
            screen_size,
            scale_factor,
            root: None,
            draw_list: DrawList::with_viewport(Rect::from_pos_size(Vec2::ZERO, screen_size)),
        }
    }

    /// Build the UI tree with the given root element
    pub fn build(mut self, root: Box<dyn Element>) -> Self {
        let _build_span = info_span!("ui_context_build").entered();
        let root_id = root.build(&mut self.taffy);
        self.root = Some(root_id);
        debug!("UI tree built with root node");
        self
    }

    /// Compute layout and generate draw commands
    pub fn render(mut self, text_system: &mut TextSystem) -> Result<DrawList, taffy::TaffyError> {
        let _render_span = info_span!("ui_context_render").entered();

        // Compute layout if we have a root
        if let Some(root) = self.root {
            {
                let _layout_span = info_span!("compute_layout_phase").entered();
                self.compute_layout(root, text_system)?;
            }
            {
                let _draw_span = info_span!("build_draw_commands_phase").entered();
                self.build_draw_commands(root, Vec2::ZERO, text_system)?;
            }
            debug!(
                "Generated {} draw commands",
                self.draw_list.commands().len()
            );
        }

        Ok(self.draw_list)
    }

    fn compute_layout(
        &mut self,
        root: NodeId,
        text_system: &mut TextSystem,
    ) -> Result<(), taffy::TaffyError> {
        let _compute_span = info_span!("taffy_compute_layout").entered();
        let screen_size = self.screen_size;
        let scale_factor = self.scale_factor;

        debug!("Computing layout for screen size: {:?}", screen_size);

        let result = self.taffy.compute_layout_with_measure(
            root,
            Size {
                width: AvailableSpace::Definite(screen_size.x),
                height: AvailableSpace::Definite(screen_size.y),
            },
            |known_dimensions, available_space, _node_id, node_context, _style| {
                let _measure_span = info_span!("measure_element_callback").entered();
                measure_element(
                    known_dimensions,
                    available_space,
                    node_context,
                    text_system,
                    scale_factor,
                )
            },
        );

        debug!("Layout computation complete");
        result
    }

    fn build_draw_commands(
        &mut self,
        node: NodeId,
        parent_offset: Vec2,
        text_system: &mut TextSystem,
    ) -> Result<(), taffy::TaffyError> {
        let layout = self.taffy.layout(node)?;
        let bounds = Vec2::new(layout.location.x, layout.location.y);
        let position = parent_offset + bounds;
        let size = Vec2::new(layout.size.width, layout.size.height);

        // Early exit if element is outside viewport
        let element_rect = Rect::from_pos_size(position, size);
        let viewport = Rect::from_pos_size(Vec2::ZERO, self.screen_size);
        if viewport.intersect(&element_rect).is_none() {
            debug!("Element outside viewport, skipping");
            return Ok(());
        }

        // Get element data
        if let Some(data) = self.taffy.get_node_context(node) {
            // Render background if present
            if let Some(bg) = &data.background {
                self.draw_list
                    .add_rect(Rect::from_pos_size(position, size), bg.clone());
            }

            // Render text if present
            if let Some((text, style)) = &data.text {
                self.draw_list.add_text(position, text, style.clone());
            }
        }

        // Render children
        for child in self.taffy.children(node)? {
            self.build_draw_commands(child, position, text_system)?;
        }

        Ok(())
    }
}

/// Data stored with each element
#[derive(Debug, Clone, Default)]
pub struct ElementData {
    text: Option<(String, TextStyle)>,
    background: Option<Color>,
}

/// Trait for UI elements (internal use)
pub trait Element {
    /// Build this element into the Taffy tree
    fn build(self: Box<Self>, tree: &mut TaffyTree<ElementData>) -> NodeId;
}

/// A group container element
pub struct Group {
    style: Style,
    data: ElementData,
    children: Vec<Box<dyn Element>>,
}

/// Create a new group element
pub fn group() -> Group {
    Group {
        style: Style::default(),
        data: ElementData::default(),
        children: Vec::new(),
    }
}

impl Group {
    /// Set the background color
    pub fn bg(mut self, color: impl Into<Color>) -> Self {
        self.data.background = Some(color.into());
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

    /// Add padding
    pub fn p(mut self, padding: f32) -> Self {
        self.style.padding = taffy::Rect {
            left: LengthPercentage::length(padding),
            right: LengthPercentage::length(padding),
            top: LengthPercentage::length(padding),
            bottom: LengthPercentage::length(padding),
        };
        self
    }

    /// Add margin
    pub fn m(mut self, margin: f32) -> Self {
        self.style.margin = taffy::Rect {
            left: LengthPercentageAuto::length(margin),
            right: LengthPercentageAuto::length(margin),
            top: LengthPercentageAuto::length(margin),
            bottom: LengthPercentageAuto::length(margin),
        };
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

    /// Set width
    pub fn w(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::length(width);
        self
    }

    /// Set height
    pub fn h(mut self, height: f32) -> Self {
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
    pub fn w_full(mut self) -> Self {
        self.style.size.width = Dimension::percent(1.0);
        self
    }

    /// Set height to 100%
    pub fn h_full(mut self) -> Self {
        self.style.size.height = Dimension::percent(1.0);
        self
    }

    /// Set both width and height to 100%
    pub fn size_full(mut self) -> Self {
        self.style.size = Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        };
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

    /// Add a child element
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = impl Element + 'static>) -> Self {
        for child in children {
            self.children.push(Box::new(child));
        }
        self
    }
}

impl Element for Group {
    fn build(self: Box<Self>, tree: &mut TaffyTree<ElementData>) -> NodeId {
        let children_count = self.children.len();
        let _build_span = info_span!("build_group", children_count = children_count).entered();
        let node = tree
            .new_leaf_with_context(self.style, self.data)
            .expect("Failed to create group node");

        // Build and add children
        for (i, child) in self.children.into_iter().enumerate() {
            let _child_span = info_span!("build_child", index = i).entered();
            let child_node = child.build(tree);
            tree.add_child(node, child_node)
                .expect("Failed to add child");
        }

        debug!("Built group with {} children", children_count);
        node
    }
}

/// A text element
pub struct Text {
    content: String,
    style: TextStyle,
}

/// Create a text element
pub fn text(content: impl Into<String>, style: TextStyle) -> Text {
    Text {
        content: content.into(),
        style,
    }
}

impl Element for Text {
    fn build(self: Box<Self>, tree: &mut TaffyTree<ElementData>) -> NodeId {
        let _build_span = info_span!("build_text", content_len = self.content.len()).entered();
        let data = ElementData {
            text: Some((self.content.clone(), self.style)),
            background: None,
        };

        debug!(
            "Building text node: '{}'",
            if self.content.len() > 20 {
                format!("{}...", &self.content[..20])
            } else {
                self.content.clone()
            }
        );

        tree.new_leaf_with_context(Style::default(), data)
            .expect("Failed to create text node")
    }
}

/// Measure function for elements
fn measure_element(
    _known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_data: Option<&mut ElementData>,
    text_system: &mut TextSystem,
    scale_factor: f32,
) -> Size<f32> {
    let _measure_span = info_span!("measure_element").entered();
    if let Some(data) = node_data {
        if let Some((content, style)) = &data.text {
            let _text_measure_span =
                info_span!("measure_text_element", content_len = content.len()).entered();
            let max_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                _ => None,
            };

            let text_config = TextConfig {
                font_stack: FontStack::from("system-ui"),
                size: style.size,
                weight: parley::FontWeight::NORMAL,
                color: style.color.clone(),
                line_height: 1.2,
            };

            let measured_size =
                text_system.measure_text(content, &text_config, max_width, scale_factor);

            debug!(
                "Measured text '{}' -> {}x{}",
                if content.len() > 20 {
                    format!("{}...", &content[..20])
                } else {
                    content.clone()
                },
                measured_size.x,
                measured_size.y
            );

            Size {
                width: measured_size.x,
                height: measured_size.y,
            }
        } else {
            Size::ZERO
        }
    } else {
        Size::ZERO
    }
}

/// Builder for complex layouts
pub struct Column {
    group: Group,
}

/// Create a column layout
pub fn col() -> Column {
    Column {
        group: group().flex_col(),
    }
}

impl Column {
    /// Add gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.group = self.group.gap(gap);
        self
    }

    /// Add padding
    pub fn p(mut self, padding: f32) -> Self {
        self.group = self.group.p(padding);
        self
    }

    /// Add a child
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.group = self.group.child(child);
        self
    }
}

impl Element for Column {
    fn build(self: Box<Self>, tree: &mut TaffyTree<ElementData>) -> NodeId {
        let _build_span = info_span!("build_column").entered();
        Box::new(self.group).build(tree)
    }
}

/// Builder for row layouts
pub struct Row {
    group: Group,
}

/// Create a row layout
pub fn row() -> Row {
    Row {
        group: group().flex_row(),
    }
}

impl Row {
    /// Add gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.group = self.group.gap(gap);
        self
    }

    /// Add padding
    pub fn p(mut self, padding: f32) -> Self {
        self.group = self.group.p(padding);
        self
    }

    /// Add a child
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.group = self.group.child(child);
        self
    }
}

impl Element for Row {
    fn build(self: Box<Self>, tree: &mut TaffyTree<ElementData>) -> NodeId {
        let _build_span = info_span!("build_row").entered();
        Box::new(self.group).build(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::colors;

    #[test]
    fn test_builder_api() {
        let ui = group()
            .flex_col()
            .p(20.0)
            .gap(10.0)
            .bg(colors::WHITE)
            .child(text(
                "Hello World",
                TextStyle {
                    size: 24.0,
                    color: colors::BLACK.into(),
                },
            ))
            .child(
                row()
                    .gap(5.0)
                    .child(text(
                        "Item 1",
                        TextStyle {
                            size: 16.0,
                            color: colors::BLACK.into(),
                        },
                    ))
                    .child(text(
                        "Item 2",
                        TextStyle {
                            size: 16.0,
                            color: colors::BLACK.into(),
                        },
                    )),
            );

        // This compiles and creates a clean tree structure
        let _ = ui;
    }
}
