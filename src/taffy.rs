//! Taffy-based UI system

use crate::text_system::{TextConfig, TextSystem};
use crate::ui::{DrawList, TextStyle};
use glam::Vec2;
use parley::FontStack;
use std::collections::HashMap;
use taffy::prelude::*;

/// Node content types
#[derive(Debug, Clone)]
pub enum NodeContent {
    /// Container node (div-like)
    Container,
    /// Text leaf node
    Text { content: String, style: TextStyle },
}

/// UI layer context using Taffy for layout
pub struct UiTaffyContext {
    /// Taffy tree for layout
    taffy: TaffyTree<NodeContent>,
    /// Stack of parent nodes for building hierarchy
    node_stack: Vec<NodeId>,
    /// Current node being built
    current_node: Option<NodeId>,
    /// Screen size
    screen_size: Vec2,
    /// Scale factor for display
    scale_factor: f32,
    /// Draw commands to be rendered
    draw_list: DrawList,
    /// Map from node ID to content for rendering phase
    node_map: HashMap<NodeId, NodeContent>,
}

impl UiTaffyContext {
    /// Create a new UI context
    pub fn new(screen_size: Vec2, scale_factor: f32) -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_stack: Vec::new(),
            current_node: None,
            screen_size,
            scale_factor,
            draw_list: DrawList::new(),
            node_map: HashMap::new(),
        }
    }

    /// Begin a container (like a div)
    pub fn begin_container(&mut self, style: impl Into<Style>) {
        let node = match self
            .taffy
            .new_leaf_with_context(style.into(), NodeContent::Container)
        {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to create container node: {:?}", e);
                return;
            }
        };

        self.node_map.insert(node, NodeContent::Container);

        // If we have a current node, add this as a child
        if let Some(parent) = self.current_node {
            if let Err(e) = self.taffy.add_child(parent, node) {
                eprintln!("Failed to add child to parent: {:?}", e);
            }
        }

        // Push current node to stack and make this the current
        if let Some(current) = self.current_node {
            self.node_stack.push(current);
        }
        self.current_node = Some(node);
    }

    /// End the current container
    pub fn end_container(&mut self) {
        if let Some(_) = self.current_node {
            self.current_node = self.node_stack.pop();
        }
    }

    /// Add text to the current container
    pub fn text(&mut self, text: impl Into<String>, style: TextStyle) {
        let content = text.into();
        let node_content = NodeContent::Text {
            content: content.clone(),
            style: style.clone(),
        };

        let node = match self
            .taffy
            .new_leaf_with_context(Style::default(), node_content.clone())
        {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to create text node: {:?}", e);
                return;
            }
        };

        self.node_map.insert(node, node_content);

        // Add to current container
        if let Some(parent) = self.current_node {
            if let Err(e) = self.taffy.add_child(parent, node) {
                eprintln!("Failed to add text node to parent: {:?}", e);
            }
        }
    }

    /// Helper for vertical layout
    pub fn vertical<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_container(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        });

        let result = f(self);
        self.end_container();
        result
    }

    /// Helper for horizontal layout
    pub fn horizontal<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_container(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        });

        let result = f(self);
        self.end_container();
        result
    }

    /// Compute layout using Taffy with text measurement
    pub fn compute_layout(
        &mut self,
        text_system: &mut TextSystem,
    ) -> Result<(), taffy::TaffyError> {
        // Find root node (first node without parent)
        let root = self.find_root_node()?;

        // Compute layout with measure function
        let screen_size = self.screen_size;
        let scale_factor = self.scale_factor;

        self.taffy.compute_layout_with_measure(
            root,
            Size {
                width: AvailableSpace::Definite(screen_size.x),
                height: AvailableSpace::Definite(screen_size.y),
            },
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_node(
                    known_dimensions,
                    available_space,
                    node_context,
                    text_system,
                    scale_factor,
                )
            },
        )?;

        Ok(())
    }

    /// Measure function for text nodes
    /// Build draw commands from the laid out tree
    pub fn build_draw_list(
        &mut self,
        text_system: &mut TextSystem,
    ) -> Result<DrawList, taffy::TaffyError> {
        self.draw_list.clear();

        let root = self.find_root_node()?;
        self.build_draw_commands_recursive(root, Vec2::ZERO, text_system)?;

        let mut draw_list = DrawList::new();
        std::mem::swap(&mut draw_list, &mut self.draw_list);
        Ok(draw_list)
    }

    /// Find the root node (node without parent)
    fn find_root_node(&self) -> Result<NodeId, taffy::TaffyError> {
        // For now, assume the first node created is the root
        // In a real implementation, we'd track this explicitly
        self.node_map
            .keys()
            .find(|&&node| self.taffy.parent(node).is_none())
            .copied()
            .ok_or(taffy::TaffyError::ChildIndexOutOfBounds {
                parent: taffy::NodeId::from(0u64),
                child_index: 0,
                child_count: 0,
            })
    }

    /// Get the final draw list
    pub fn draw_list(&self) -> &DrawList {
        &self.draw_list
    }

    /// Recursively build draw commands
    fn build_draw_commands_recursive(
        &mut self,
        node: NodeId,
        parent_offset: Vec2,
        text_system: &mut TextSystem,
    ) -> Result<(), taffy::TaffyError> {
        let layout = self.taffy.layout(node)?;
        let position = parent_offset + Vec2::new(layout.location.x, layout.location.y);

        // Draw based on node content
        if let Some(content) = self.node_map.get(&node) {
            match content {
                NodeContent::Text { content, style } => {
                    // Shape and add text to draw list
                    let text_config = TextConfig {
                        font_stack: FontStack::from("system-ui"),
                        size: style.size,
                        weight: parley::FontWeight::NORMAL,
                        color: style.color.clone(),
                        line_height: 1.2,
                    };

                    let _shaped = text_system
                        .shape_text(
                            content.as_str(),
                            &text_config,
                            Some(layout.size.width),
                            self.scale_factor,
                        )
                        .map_err(|_e| taffy::TaffyError::ChildIndexOutOfBounds {
                            parent: node,
                            child_index: 0,
                            child_count: 0,
                        })?;

                    self.draw_list.add_text(position, content, style.clone());
                }
                NodeContent::Container => {
                    // Containers might have background, borders, etc.
                    // For now, just recurse to children
                }
            }
        }

        // Process children
        let children = self.taffy.children(node)?;
        for child in children {
            self.build_draw_commands_recursive(child, position, text_system)?;
        }

        Ok(())
    }
}

/// Measure function for text nodes (moved outside to avoid borrow issues)
fn measure_node(
    _known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_content: Option<&mut NodeContent>,
    text_system: &mut TextSystem,
    scale_factor: f32,
) -> Size<f32> {
    match node_content {
        Some(NodeContent::Text { content, style }) => {
            // Convert available space to max width for text wrapping
            let max_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                AvailableSpace::MinContent => None, // No wrapping
                AvailableSpace::MaxContent => None, // No wrapping
            };

            // Create text config from style
            let text_config = TextConfig {
                font_stack: FontStack::from("system-ui"),
                size: style.size,
                weight: parley::FontWeight::NORMAL,
                color: style.color.clone(),
                line_height: 1.2,
            };

            // Measure with text system
            let measured_size =
                text_system.measure_text(content, &text_config, max_width, scale_factor);

            Size {
                width: measured_size.x,
                height: measured_size.y,
            }
        }
        _ => Size::ZERO, // Container nodes have no intrinsic size
    }
}

/// Builder-style API for common patterns
impl UiTaffyContext {
    /// Create a flex column with padding
    pub fn padded_column<R>(&mut self, padding: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_container(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: Rect {
                left: LengthPercentage::length(padding),
                right: LengthPercentage::length(padding),
                top: LengthPercentage::length(padding),
                bottom: LengthPercentage::length(padding),
            },
            ..Default::default()
        });

        let result = f(self);
        self.end_container();
        result
    }

    /// Create a centered container
    pub fn centered<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_container(Style {
            display: Display::Flex,
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });

        let result = f(self);
        self.end_container();
        result
    }

    /// Add spacing between elements
    pub fn space(&mut self, height: f32) {
        self.begin_container(Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::length(height),
            },
            ..Default::default()
        });
        self.end_container();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::colors;

    #[test]
    fn test_basic_layout() {
        let mut ui = UiTaffyContext::new(Vec2::new(800.0, 600.0), 1.0);

        // Build a simple layout
        ui.vertical(|ui| {
            ui.text(
                "Hello",
                TextStyle {
                    size: 24.0,
                    color: colors::BLACK.into(),
                },
            );

            ui.space(10.0);

            ui.text(
                "World",
                TextStyle {
                    size: 16.0,
                    color: colors::BLUE.into(),
                },
            );
        });

        // Would need a text system to actually compute layout
        // This just tests that the API works
    }
}
