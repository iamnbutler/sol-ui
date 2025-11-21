//! Layout engine using Taffy for flexbox/grid layout

use crate::geometry::Rect;
use glam::Vec2;
use std::collections::{HashMap, HashSet};
use taffy::prelude::*;
use tracing::info_span;

/// Data stored with each element in the taffy tree
#[derive(Debug, Clone, Default)]
pub struct ElementData {
    pub text: Option<(String, crate::style::TextStyle)>,
    pub background: Option<crate::color::Color>,
}

/// A layout engine that wraps Taffy and provides a simple API
pub struct TaffyLayoutEngine {
    taffy: TaffyTree<ElementData>,
    absolute_layout_bounds: HashMap<NodeId, Rect>,
    computed_layouts: HashSet<NodeId>,
}

impl TaffyLayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        let taffy = TaffyTree::new();

        TaffyLayoutEngine {
            taffy,
            absolute_layout_bounds: HashMap::default(),
            computed_layouts: HashSet::default(),
        }
    }

    /// Clear all layout data (called every frame)
    pub fn clear(&mut self) {
        self.taffy.clear();
        self.absolute_layout_bounds.clear();
        self.computed_layouts.clear();
    }

    /// Request layout for a leaf node (no children)
    pub fn request_layout(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        if children.is_empty() {
            self.taffy
                .new_leaf(style)
                .expect("Failed to create leaf node")
        } else {
            self.taffy
                .new_with_children(style, children)
                .expect("Failed to create parent node")
        }
    }

    /// Request layout with associated data
    pub fn request_layout_with_data(
        &mut self,
        style: Style,
        data: ElementData,
        children: &[NodeId],
    ) -> NodeId {
        if children.is_empty() {
            self.taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create leaf node with data")
        } else {
            let node = self
                .taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create parent node with data");

            for &child in children {
                self.taffy
                    .add_child(node, child)
                    .expect("Failed to add child to node");
            }

            node
        }
    }

    /// Compute layout for the tree
    pub fn compute_layout(
        &mut self,
        root: NodeId,
        available_space: Size<AvailableSpace>,
        text_system: &mut crate::text_system::TextSystem,
        scale_factor: f32,
    ) -> Result<(), taffy::TaffyError> {
        let _compute_span = info_span!("compute_layout").entered();

        self.taffy.compute_layout_with_measure(
            root,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_element(
                    known_dimensions,
                    available_space,
                    node_context,
                    text_system,
                    scale_factor,
                )
            },
        )
    }

    /// Get the computed layout bounds for a node
    pub fn layout_bounds(&self, id: NodeId) -> Rect {
        let layout = self.taffy.layout(id).expect("Failed to get layout");
        Rect::from_pos_size(
            Vec2::new(layout.location.x, layout.location.y),
            Vec2::new(layout.size.width, layout.size.height),
        )
    }

    /// Get the element data for a node
    pub fn get_node_context(&self, id: NodeId) -> Option<&ElementData> {
        self.taffy.get_node_context(id)
    }

    /// Get the children of a node
    pub fn children(&self, id: NodeId) -> Result<Vec<NodeId>, taffy::TaffyError> {
        self.taffy.children(id)
    }
}

/// Measure function for elements that contain text
fn measure_element(
    _known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_data: Option<&mut ElementData>,
    text_system: &mut crate::text_system::TextSystem,
    scale_factor: f32,
) -> Size<f32> {
    if let Some(data) = node_data {
        if let Some((content, style)) = &data.text {
            let max_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                _ => None,
            };

            let text_config = crate::text_system::TextConfig {
                font_stack: parley::FontStack::from("system-ui"),
                size: style.size,
                weight: parley::FontWeight::NORMAL,
                color: style.color.clone(),
                line_height: 1.2,
            };

            let measured_size =
                text_system.measure_text(content, &text_config, max_width, scale_factor);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::TextStyle;
    use crate::color::Color;

    // Mock text system for testing
    struct MockTextSystem;
    
    impl MockTextSystem {
        fn measure_text(&mut self, text: &str, _config: &crate::text_system::TextConfig, _max_width: Option<f32>, _scale_factor: f32) -> Vec2 {
            // Simple text measurement: each character is 8 pixels wide, height is size
            let char_count = text.len();
            Vec2::new(char_count as f32 * 8.0, 16.0)
        }
    }

    #[test]
    fn test_taffy_layout_engine_creation() {
        let engine = TaffyLayoutEngine::new();
        assert_eq!(engine.computed_layouts.len(), 0);
        assert_eq!(engine.absolute_layout_bounds.len(), 0);
    }

    #[test]
    fn test_clear_layout_data() {
        let mut engine = TaffyLayoutEngine::new();
        
        // Create some nodes to populate internal state
        let style = Style::default();
        let _node1 = engine.request_layout(style.clone(), &[]);
        let _node2 = engine.request_layout(style, &[]);
        
        // Clear should remove all data
        engine.clear();
        assert_eq!(engine.computed_layouts.len(), 0);
        assert_eq!(engine.absolute_layout_bounds.len(), 0);
    }

    #[test]
    fn test_request_layout_leaf_node() {
        let mut engine = TaffyLayoutEngine::new();
        let style = Style::default();
        
        let node = engine.request_layout(style, &[]);
        assert!(node.into_inner() >= 0); // Valid node ID
    }

    #[test]
    fn test_request_layout_with_children() {
        let mut engine = TaffyLayoutEngine::new();
        let style = Style::default();
        
        // Create child nodes first
        let child1 = engine.request_layout(style.clone(), &[]);
        let child2 = engine.request_layout(style.clone(), &[]);
        
        // Create parent with children
        let parent = engine.request_layout(style, &[child1, child2]);
        assert!(parent.into_inner() >= 0); // Valid node ID
    }

    #[test]
    fn test_request_layout_with_data() {
        let mut engine = TaffyLayoutEngine::new();
        let style = Style::default();
        
        let data = ElementData {
            text: Some(("Hello".to_string(), TextStyle::default())),
            background: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        };
        
        let node = engine.request_layout_with_data(style, data, &[]);
        assert!(node.into_inner() >= 0); // Valid node ID
    }

    #[test]
    fn test_request_layout_with_data_and_children() {
        let mut engine = TaffyLayoutEngine::new();
        let style = Style::default();
        
        // Create child node
        let child = engine.request_layout(style.clone(), &[]);
        
        let data = ElementData {
            text: Some(("Hello".to_string(), TextStyle::default())),
            background: None,
        };
        
        let parent = engine.request_layout_with_data(style, data, &[child]);
        assert!(parent.into_inner() >= 0); // Valid node ID
    }

    #[test]
    fn test_element_data_default() {
        let data = ElementData::default();
        assert!(data.text.is_none());
        assert!(data.background.is_none());
    }

    #[test]
    fn test_element_data_with_text() {
        let text_style = TextStyle::default();
        let data = ElementData {
            text: Some(("Test text".to_string(), text_style.clone())),
            background: None,
        };
        
        assert!(data.text.is_some());
        if let Some((content, style)) = &data.text {
            assert_eq!(content, "Test text");
            assert_eq!(*style, text_style);
        }
    }

    #[test]
    fn test_element_data_with_background() {
        let color = Color::rgba(0.5, 0.5, 0.5, 0.8);
        let data = ElementData {
            text: None,
            background: Some(color),
        };
        
        assert!(data.background.is_some());
        assert_eq!(data.background.unwrap(), color);
    }

    #[test]
    fn test_element_data_clone() {
        let original = ElementData {
            text: Some(("Clone test".to_string(), TextStyle::default())),
            background: Some(Color::rgba(0.2, 0.4, 0.6, 1.0)),
        };
        
        let cloned = original.clone();
        assert_eq!(original.text, cloned.text);
        assert_eq!(original.background, cloned.background);
    }

    #[test]
    fn test_measure_element_no_data() {
        let mut mock_text_system = MockTextSystem;
        let result = measure_element(
            Size::NONE,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(50.0) },
            None,
            &mut mock_text_system,
            1.0,
        );
        
        assert_eq!(result, Size::ZERO);
    }

    #[test]
    fn test_measure_element_with_text() {
        let mut mock_text_system = MockTextSystem;
        let mut data = ElementData {
            text: Some(("Hello".to_string(), TextStyle::default())),
            background: None,
        };
        
        let result = measure_element(
            Size::NONE,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(50.0) },
            Some(&mut data),
            &mut mock_text_system,
            1.0,
        );
        
        // Should return measured size based on mock text system (5 chars * 8 pixels = 40, height = 16)
        assert_eq!(result.width, 40.0);
        assert_eq!(result.height, 16.0);
    }

    #[test]
    fn test_measure_element_no_text_data() {
        let mut mock_text_system = MockTextSystem;
        let mut data = ElementData {
            text: None,
            background: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        };
        
        let result = measure_element(
            Size::NONE,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(50.0) },
            Some(&mut data),
            &mut mock_text_system,
            1.0,
        );
        
        assert_eq!(result, Size::ZERO);
    }

    #[test]
    fn test_measure_element_with_max_width() {
        let mut mock_text_system = MockTextSystem;
        let mut data = ElementData {
            text: Some(("Long text content".to_string(), TextStyle::default())),
            background: None,
        };
        
        let result = measure_element(
            Size::NONE,
            Size { width: AvailableSpace::Definite(50.0), height: AvailableSpace::MaxContent },
            Some(&mut data),
            &mut mock_text_system,
            2.0, // Different scale factor
        );
        
        // Should still use mock measurement (17 chars * 8 = 136 pixels wide)
        assert_eq!(result.width, 136.0);
        assert_eq!(result.height, 16.0);
    }

    #[test]
    fn test_measure_element_min_content_width() {
        let mut mock_text_system = MockTextSystem;
        let mut data = ElementData {
            text: Some(("Test".to_string(), TextStyle::default())),
            background: None,
        };
        
        let result = measure_element(
            Size::NONE,
            Size { width: AvailableSpace::MinContent, height: AvailableSpace::MinContent },
            Some(&mut data),
            &mut mock_text_system,
            1.0,
        );
        
        // Should still return measured size (4 chars * 8 = 32)
        assert_eq!(result.width, 32.0);
        assert_eq!(result.height, 16.0);
    }
}
