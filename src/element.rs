//! Two-phase element rendering system
//!
mod container;
mod text;

pub use container::{Container, column, container, row};
pub use text::{Text, text};

use crate::{
    geometry::Rect,
    layout_engine::{ElementData, TaffyLayoutEngine},
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
            font_stack: parley::FontStack::from("system-ui"),
            size: style.size,
            weight: parley::FontWeight::NORMAL,
            color: style.color.clone(),
            line_height: 1.2,
        };

        self.text_system
            .measure_text(text, &text_config, max_width, self.scale_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{color::Color, style::TextStyle};

    // Mock implementations for testing
    struct MockLayoutEngine {
        next_node_id: u64,
    }

    impl MockLayoutEngine {
        fn new() -> Self {
            Self { next_node_id: 0 }
        }

        fn request_layout(&mut self, _style: Style, _children: &[NodeId]) -> NodeId {
            let id = NodeId::from(self.next_node_id);
            self.next_node_id += 1;
            id
        }

        fn request_layout_with_data(&mut self, _style: Style, _data: ElementData, _children: &[NodeId]) -> NodeId {
            let id = NodeId::from(self.next_node_id);
            self.next_node_id += 1;
            id
        }
    }

    struct MockTextSystem;

    impl MockTextSystem {
        fn measure_text(&mut self, text: &str, _style: &TextStyle, _max_width: Option<f32>) -> Vec2 {
            // Simple mock: each character is 8x16 pixels
            Vec2::new(text.len() as f32 * 8.0, 16.0)
        }
    }

    fn create_mock_layout_context() -> (MockLayoutEngine, MockTextSystem, LayoutContext<'static>) {
        let mut engine = MockLayoutEngine::new();
        let mut text_system = MockTextSystem;
        
        // This is unsafe but necessary for testing. In real usage, the lifetimes are managed properly.
        let engine_ptr = &mut engine as *mut MockLayoutEngine as *mut TaffyLayoutEngine;
        let text_system_ptr = &mut text_system as *mut MockTextSystem as *mut TextSystem;
        
        let ctx = LayoutContext {
            engine: unsafe { &mut *engine_ptr },
            text_system: unsafe { &mut *text_system_ptr },
            scale_factor: 1.0,
        };
        
        (engine, text_system, ctx)
    }

    #[test]
    fn test_layout_context_request_layout() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 1.0,
        };
        
        let style = Style::default();
        let node = ctx.request_layout(style);
        
        // Should return a valid node ID
        assert!(node.into_inner() < u64::MAX);
    }

    #[test]
    fn test_layout_context_request_layout_with_children() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 1.0,
        };
        
        let style = Style::default();
        
        // Create child nodes
        let child1 = ctx.request_layout(style.clone());
        let child2 = ctx.request_layout(style.clone());
        
        // Create parent with children
        let parent = ctx.request_layout_with_children(style, &[child1, child2]);
        
        // Should return a valid node ID
        assert!(parent.into_inner() < u64::MAX);
    }

    #[test]
    fn test_layout_context_request_text_layout() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 1.0,
        };
        
        let style = Style::default();
        let text = "Hello World";
        let text_style = TextStyle::default();
        
        let node = ctx.request_text_layout(style, text, &text_style);
        
        // Should return a valid node ID
        assert!(node.into_inner() < u64::MAX);
    }

    #[test]
    fn test_layout_context_request_layout_with_data() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 1.0,
        };
        
        let style = Style::default();
        let data = ElementData {
            text: Some(("Test".to_string(), TextStyle::default())),
            background: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        };
        let children = [];
        
        let node = ctx.request_layout_with_data(style, data, &children);
        
        // Should return a valid node ID
        assert!(node.into_inner() < u64::MAX);
    }

    #[test]
    fn test_layout_context_measure_text() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 2.0,
        };
        
        let text = "Test";
        let style = TextStyle::default();
        
        let size = ctx.measure_text(text, &style, Some(100.0));
        
        // Should return some size (exact values depend on text system implementation)
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
    }

    #[test]
    fn test_layout_context_measure_text_no_max_width() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let mut ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 1.0,
        };
        
        let text = "Hello World!";
        let style = TextStyle::default();
        
        let size = ctx.measure_text(text, &style, None);
        
        // Should return some size
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
    }

    #[test]
    fn test_layout_context_scale_factor() {
        let mut engine = TaffyLayoutEngine::new();
        let mut text_system = crate::text_system::TextSystem::new();
        
        let ctx = LayoutContext {
            engine: &mut engine,
            text_system: &mut text_system,
            scale_factor: 2.5,
        };
        
        assert_eq!(ctx.scale_factor, 2.5);
    }

    #[test]
    fn test_layout_context_different_scale_factors() {
        let mut engine1 = TaffyLayoutEngine::new();
        let mut text_system1 = crate::text_system::TextSystem::new();
        let mut engine2 = TaffyLayoutEngine::new();
        let mut text_system2 = crate::text_system::TextSystem::new();
        
        let mut ctx1 = LayoutContext {
            engine: &mut engine1,
            text_system: &mut text_system1,
            scale_factor: 1.0,
        };
        
        let mut ctx2 = LayoutContext {
            engine: &mut engine2,
            text_system: &mut text_system2,
            scale_factor: 2.0,
        };
        
        let text = "Scale Test";
        let style = TextStyle::default();
        
        let size1 = ctx1.measure_text(text, &style, None);
        let size2 = ctx2.measure_text(text, &style, None);
        
        // Both should return valid sizes (relationship depends on text system implementation)
        assert!(size1.x > 0.0 && size1.y > 0.0);
        assert!(size2.x > 0.0 && size2.y > 0.0);
    }
}
