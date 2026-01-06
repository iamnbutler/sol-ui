//! Test utilities for sol-ui
//!
//! This module provides helpers for testing elements, layouts, and interactions
//! without requiring a full rendering context.
//!
//! # Layout Testing
//!
//! Use `TestLayoutContext` to test element layout without a text system:
//!
//! ```ignore
//! use sol_ui::testing::TestLayoutContext;
//!
//! let mut ctx = TestLayoutContext::new();
//! let node = element.layout(&mut ctx.as_layout_context());
//! ctx.compute_layout(node, 800.0, 600.0);
//! let bounds = ctx.get_bounds(node);
//! assert_eq!(bounds.size.x, 100.0);
//! ```
//!
//! # Interaction Testing
//!
//! Use `TestInteractionContext` to simulate mouse and keyboard events:
//!
//! ```ignore
//! use sol_ui::testing::TestInteractionContext;
//! use glam::Vec2;
//!
//! let mut ctx = TestInteractionContext::new();
//! ctx.mouse_move(Vec2::new(50.0, 50.0));
//! ctx.click(Vec2::new(50.0, 50.0));
//! let events = ctx.take_events();
//! ```
//!
//! # Render Output Testing
//!
//! Use `TestPaintContext` to capture draw commands:
//!
//! ```ignore
//! use sol_ui::testing::TestPaintContext;
//!
//! let mut ctx = TestPaintContext::new();
//! element.paint(bounds, &mut ctx.as_paint_context());
//! let commands = ctx.commands();
//! assert!(commands.iter().any(|c| matches!(c, DrawCommand::Rect { .. })));
//! ```

use crate::{
    geometry::Rect,
    interaction::{
        ElementId, HitTestBuilder, HitTestEntry, InteractionEvent, InteractionState,
        InteractionSystem,
    },
    layer::{InputEvent, Key, Modifiers, MouseButton},
    layout_engine::{ElementData, TaffyLayoutEngine},
    render::{DrawCommand, DrawList},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

// ============================================================================
// Layout Testing
// ============================================================================

/// A test context for layout operations that doesn't require platform-specific
/// text rendering.
pub struct TestLayoutContext {
    engine: TaffyLayoutEngine,
    /// Fixed text size for testing (width per character, height)
    text_char_size: Vec2,
}

impl TestLayoutContext {
    /// Create a new test layout context with default text sizing
    pub fn new() -> Self {
        Self {
            engine: TaffyLayoutEngine::new(),
            text_char_size: Vec2::new(8.0, 16.0), // Monospace-ish defaults
        }
    }

    /// Create with custom text character sizing
    pub fn with_text_size(char_width: f32, char_height: f32) -> Self {
        Self {
            engine: TaffyLayoutEngine::new(),
            text_char_size: Vec2::new(char_width, char_height),
        }
    }

    /// Request layout for a node (simplified version for testing)
    pub fn request_layout(&mut self, style: Style) -> NodeId {
        self.engine.request_layout(style, &[])
    }

    /// Request layout with children
    pub fn request_layout_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.engine.request_layout(style, children)
    }

    /// Compute layout for the tree
    pub fn compute_layout(&mut self, root: NodeId, width: f32, height: f32) {
        // Use a simple measure function that uses fixed text sizes
        let text_char_size = self.text_char_size;
        self.engine
            .taffy_mut()
            .compute_layout_with_measure(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
                |_known_dimensions, _available_space, _node_id, node_context, _style| {
                    if let Some(data) = node_context {
                        if let Some((text, _style)) = &data.text {
                            // Simple fixed-width text measurement for testing
                            Size {
                                width: text.len() as f32 * text_char_size.x,
                                height: text_char_size.y,
                            }
                        } else {
                            Size::ZERO
                        }
                    } else {
                        Size::ZERO
                    }
                },
            )
            .expect("Layout computation failed");
    }

    /// Get the computed bounds for a node
    pub fn get_bounds(&self, node_id: NodeId) -> Rect {
        self.engine.layout_bounds(node_id)
    }

    /// Get mutable access to the underlying taffy tree
    pub fn taffy_mut(&mut self) -> &mut TaffyTree<ElementData> {
        self.engine.taffy_mut()
    }

    /// Clear all layout data
    pub fn clear(&mut self) {
        self.engine.clear();
    }
}

impl Default for TestLayoutContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Interaction Testing
// ============================================================================

/// A test context for simulating user interactions
pub struct TestInteractionContext {
    system: InteractionSystem,
    hit_test_entries: Vec<HitTestEntry>,
    collected_events: Vec<InteractionEvent>,
}

impl TestInteractionContext {
    /// Create a new test interaction context
    pub fn new() -> Self {
        Self {
            system: InteractionSystem::new(),
            hit_test_entries: Vec::new(),
            collected_events: Vec::new(),
        }
    }

    /// Register an element for hit testing
    pub fn register_element(&mut self, id: ElementId, bounds: Rect, z_index: i32) {
        self.hit_test_entries
            .push(HitTestEntry::new(id, bounds, z_index, 0));
        // Sort by z-index (highest first)
        self.hit_test_entries
            .sort_by(|a, b| b.z_index.cmp(&a.z_index));
    }

    /// Update the system's hit test entries
    fn sync_hit_test(&mut self) {
        self.system.update_hit_test(self.hit_test_entries.clone());
    }

    /// Simulate a mouse move
    pub fn mouse_move(&mut self, position: Vec2) -> Vec<InteractionEvent> {
        self.sync_hit_test();
        let events = self.system.handle_input(&InputEvent::MouseMove { position });
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate a mouse button press
    pub fn mouse_down(&mut self, position: Vec2, button: MouseButton) -> Vec<InteractionEvent> {
        self.mouse_down_with_count(position, button, 1)
    }

    /// Simulate a mouse button press with a specific click count (for double/triple click)
    pub fn mouse_down_with_count(
        &mut self,
        position: Vec2,
        button: MouseButton,
        click_count: u32,
    ) -> Vec<InteractionEvent> {
        self.sync_hit_test();
        let events = self.system.handle_input(&InputEvent::MouseDown {
            position,
            button,
            click_count,
        });
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate a mouse button release
    pub fn mouse_up(&mut self, position: Vec2, button: MouseButton) -> Vec<InteractionEvent> {
        self.sync_hit_test();
        let events = self
            .system
            .handle_input(&InputEvent::MouseUp { position, button });
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate a complete click (mouse down + up at same position)
    pub fn click(&mut self, position: Vec2) -> Vec<InteractionEvent> {
        let mut events = Vec::new();
        events.extend(self.mouse_down(position, MouseButton::Left));
        events.extend(self.mouse_up(position, MouseButton::Left));
        events
    }

    /// Simulate a right click
    pub fn right_click(&mut self, position: Vec2) -> Vec<InteractionEvent> {
        let mut events = Vec::new();
        events.extend(self.mouse_down(position, MouseButton::Right));
        events.extend(self.mouse_up(position, MouseButton::Right));
        events
    }

    /// Simulate mouse leaving the window
    pub fn mouse_leave(&mut self) -> Vec<InteractionEvent> {
        let events = self.system.handle_input(&InputEvent::MouseLeave);
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate scroll wheel
    pub fn scroll(&mut self, position: Vec2, delta: Vec2) -> Vec<InteractionEvent> {
        self.sync_hit_test();
        let events = self
            .system
            .handle_input(&InputEvent::ScrollWheel { position, delta });
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate a key press
    pub fn key_down(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        character: Option<char>,
    ) -> Vec<InteractionEvent> {
        let events = self.system.handle_input(&InputEvent::KeyDown {
            key,
            modifiers,
            character,
            is_repeat: false,
        });
        self.collected_events.extend(events.clone());
        events
    }

    /// Simulate a key release
    pub fn key_up(&mut self, key: Key, modifiers: Modifiers) -> Vec<InteractionEvent> {
        let events = self
            .system
            .handle_input(&InputEvent::KeyUp { key, modifiers });
        self.collected_events.extend(events.clone());
        events
    }

    /// Set focus to an element
    pub fn set_focus(&mut self, element_id: Option<ElementId>) -> Vec<InteractionEvent> {
        let events = self.system.set_focus(element_id);
        self.collected_events.extend(events.clone());
        events
    }

    /// Get the interaction state for an element
    pub fn get_state(&self, element_id: ElementId) -> Option<InteractionState> {
        self.system.get_state(element_id).cloned()
    }

    /// Get the currently focused element
    pub fn focused_element(&self) -> Option<ElementId> {
        self.system.focused_element()
    }

    /// Take all collected events (clears the internal buffer)
    pub fn take_events(&mut self) -> Vec<InteractionEvent> {
        std::mem::take(&mut self.collected_events)
    }

    /// Get all collected events without clearing
    pub fn events(&self) -> &[InteractionEvent] {
        &self.collected_events
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.system.clear();
        self.hit_test_entries.clear();
        self.collected_events.clear();
    }
}

impl Default for TestInteractionContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Paint Testing
// ============================================================================

/// A test context for capturing paint output
pub struct TestPaintContext {
    draw_list: DrawList,
    layout_engine: TaffyLayoutEngine,
    hit_test_builder: Rc<RefCell<HitTestBuilder>>,
    scale_factor: f32,
}

impl TestPaintContext {
    /// Create a new test paint context
    pub fn new() -> Self {
        Self {
            draw_list: DrawList::new(),
            layout_engine: TaffyLayoutEngine::new(),
            hit_test_builder: Rc::new(RefCell::new(HitTestBuilder::default_for_testing())),
            scale_factor: 1.0,
        }
    }

    /// Create with a viewport for culling tests
    pub fn with_viewport(viewport: Rect) -> Self {
        Self {
            draw_list: DrawList::with_viewport(viewport),
            layout_engine: TaffyLayoutEngine::new(),
            hit_test_builder: Rc::new(RefCell::new(HitTestBuilder::default_for_testing())),
            scale_factor: 1.0,
        }
    }

    /// Create with a specific scale factor
    pub fn with_scale_factor(scale_factor: f32) -> Self {
        Self {
            draw_list: DrawList::new(),
            layout_engine: TaffyLayoutEngine::new(),
            hit_test_builder: Rc::new(RefCell::new(HitTestBuilder::default_for_testing())),
            scale_factor,
        }
    }

    /// Get the captured draw commands
    pub fn commands(&self) -> &[DrawCommand] {
        self.draw_list.commands()
    }

    /// Get hit test entries that were registered
    pub fn hit_test_entries(&self) -> Vec<HitTestEntry> {
        self.hit_test_builder.borrow().entries().to_vec()
    }

    /// Check if any rect was drawn
    pub fn has_rects(&self) -> bool {
        self.commands()
            .iter()
            .any(|c| matches!(c, DrawCommand::Rect { .. }))
    }

    /// Check if any text was drawn
    pub fn has_text(&self) -> bool {
        self.commands()
            .iter()
            .any(|c| matches!(c, DrawCommand::Text { .. }))
    }

    /// Get all rect commands
    pub fn rects(&self) -> Vec<(&Rect, &crate::color::Color)> {
        self.commands()
            .iter()
            .filter_map(|c| match c {
                DrawCommand::Rect { rect, color } => Some((rect, color)),
                _ => None,
            })
            .collect()
    }

    /// Get all text commands
    pub fn texts(&self) -> Vec<(&Vec2, &str, &TextStyle)> {
        self.commands()
            .iter()
            .filter_map(|c| match c {
                DrawCommand::Text {
                    position,
                    text,
                    style,
                } => Some((position, text.as_str(), style)),
                _ => None,
            })
            .collect()
    }

    /// Clear all captured data
    pub fn clear(&mut self) {
        self.draw_list.clear();
        self.hit_test_builder.borrow_mut().clear();
    }

    /// Get the culling statistics
    pub fn culling_stats(&self) -> &crate::render::CullingStats {
        self.draw_list.culling_stats()
    }
}

impl Default for TestPaintContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that two rectangles are approximately equal (within epsilon)
pub fn assert_rect_approx_eq(actual: &Rect, expected: &Rect, epsilon: f32) {
    assert!(
        (actual.pos.x - expected.pos.x).abs() < epsilon,
        "Rect x position mismatch: {} vs {}",
        actual.pos.x,
        expected.pos.x
    );
    assert!(
        (actual.pos.y - expected.pos.y).abs() < epsilon,
        "Rect y position mismatch: {} vs {}",
        actual.pos.y,
        expected.pos.y
    );
    assert!(
        (actual.size.x - expected.size.x).abs() < epsilon,
        "Rect width mismatch: {} vs {}",
        actual.size.x,
        expected.size.x
    );
    assert!(
        (actual.size.y - expected.size.y).abs() < epsilon,
        "Rect height mismatch: {} vs {}",
        actual.size.y,
        expected.size.y
    );
}

/// Assert that a bounds rectangle has expected dimensions
pub fn assert_bounds(bounds: &Rect, x: f32, y: f32, width: f32, height: f32) {
    assert_rect_approx_eq(bounds, &Rect::new(x, y, width, height), 0.1);
}

/// Check if an event list contains a specific event type for an element
pub fn has_event_for_element<F>(events: &[InteractionEvent], element_id: ElementId, pred: F) -> bool
where
    F: Fn(&InteractionEvent) -> bool,
{
    events.iter().any(|e| {
        let matches_element = match e {
            InteractionEvent::MouseEnter { element_id: id }
            | InteractionEvent::MouseLeave { element_id: id }
            | InteractionEvent::MouseMove { element_id: id, .. }
            | InteractionEvent::MouseDown { element_id: id, .. }
            | InteractionEvent::MouseUp { element_id: id, .. }
            | InteractionEvent::Click { element_id: id, .. }
            | InteractionEvent::DoubleClick { element_id: id, .. }
            | InteractionEvent::TripleClick { element_id: id, .. }
            | InteractionEvent::RightClick { element_id: id, .. }
            | InteractionEvent::ScrollWheel { element_id: id, .. }
            | InteractionEvent::KeyDown { element_id: id, .. }
            | InteractionEvent::KeyUp { element_id: id, .. }
            | InteractionEvent::FocusIn { element_id: id }
            | InteractionEvent::FocusOut { element_id: id } => *id == element_id,
            // ShortcutTriggered is a global event, not associated with a specific element
            InteractionEvent::ShortcutTriggered { .. } => false,
            // DragDrop is a global event, not associated with a specific element
            InteractionEvent::DragDrop(_) => false,
        };
        matches_element && pred(e)
    })
}

/// Check if events contain a click for an element
pub fn has_click_event(events: &[InteractionEvent], element_id: ElementId) -> bool {
    has_event_for_element(events, element_id, |e| {
        matches!(e, InteractionEvent::Click { .. })
    })
}

/// Check if events contain a hover enter for an element
pub fn has_hover_enter_event(events: &[InteractionEvent], element_id: ElementId) -> bool {
    has_event_for_element(events, element_id, |e| {
        matches!(e, InteractionEvent::MouseEnter { .. })
    })
}

/// Check if events contain a hover leave for an element
pub fn has_hover_leave_event(events: &[InteractionEvent], element_id: ElementId) -> bool {
    has_event_for_element(events, element_id, |e| {
        matches!(e, InteractionEvent::MouseLeave { .. })
    })
}

// ============================================================================
// Tests for the testing utilities themselves
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_context_basic() {
        let mut ctx = TestLayoutContext::new();

        let style = Style {
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::length(50.0),
            },
            ..Default::default()
        };

        let node = ctx.request_layout(style);
        ctx.compute_layout(node, 800.0, 600.0);

        let bounds = ctx.get_bounds(node);
        assert_eq!(bounds.size.x, 100.0);
        assert_eq!(bounds.size.y, 50.0);
    }

    #[test]
    fn test_layout_context_with_children() {
        let mut ctx = TestLayoutContext::new();

        let child_style = Style {
            size: Size {
                width: Dimension::length(50.0),
                height: Dimension::length(30.0),
            },
            ..Default::default()
        };

        let parent_style = Style {
            padding: taffy::Rect::length(10.0),
            ..Default::default()
        };

        let child = ctx.request_layout(child_style);
        let parent = ctx.request_layout_with_children(parent_style, &[child]);

        ctx.compute_layout(parent, 800.0, 600.0);

        let parent_bounds = ctx.get_bounds(parent);
        let child_bounds = ctx.get_bounds(child);

        // Parent should be child size + padding
        assert_eq!(parent_bounds.size.x, 70.0); // 50 + 10 + 10
        assert_eq!(parent_bounds.size.y, 50.0); // 30 + 10 + 10

        // Child should be offset by padding
        assert_eq!(child_bounds.pos.x, 10.0);
        assert_eq!(child_bounds.pos.y, 10.0);
    }

    #[test]
    fn test_interaction_context_click() {
        let mut ctx = TestInteractionContext::new();

        let element_id = ElementId::new(1);
        let bounds = Rect::new(0.0, 0.0, 100.0, 50.0);
        ctx.register_element(element_id, bounds, 0);

        let events = ctx.click(Vec2::new(50.0, 25.0));

        assert!(has_click_event(&events, element_id));
    }

    #[test]
    fn test_interaction_context_hover() {
        let mut ctx = TestInteractionContext::new();

        let element_id = ElementId::new(1);
        let bounds = Rect::new(0.0, 0.0, 100.0, 50.0);
        ctx.register_element(element_id, bounds, 0);

        // Move into element
        let events = ctx.mouse_move(Vec2::new(50.0, 25.0));
        assert!(has_hover_enter_event(&events, element_id));

        // Verify hover state
        let state = ctx.get_state(element_id).unwrap();
        assert!(state.is_hovered);

        // Move out of element
        let events = ctx.mouse_move(Vec2::new(200.0, 200.0));
        assert!(has_hover_leave_event(&events, element_id));
    }

    #[test]
    fn test_interaction_context_z_order() {
        let mut ctx = TestInteractionContext::new();

        let back_id = ElementId::new(1);
        let front_id = ElementId::new(2);

        // Both elements cover the same area, but front has higher z-index
        let bounds = Rect::new(0.0, 0.0, 100.0, 50.0);
        ctx.register_element(back_id, bounds, 0);
        ctx.register_element(front_id, bounds, 1);

        // Click should hit the front element
        let events = ctx.click(Vec2::new(50.0, 25.0));

        assert!(has_click_event(&events, front_id));
        assert!(!has_click_event(&events, back_id));
    }

    #[test]
    fn test_paint_context_captures_rects() {
        let ctx = TestPaintContext::new();

        // The TestPaintContext doesn't have a way to directly paint yet,
        // but it demonstrates the API
        assert!(!ctx.has_rects());
        assert!(!ctx.has_text());
    }

    #[test]
    fn test_assert_rect_approx_eq() {
        let rect1 = Rect::new(10.0, 20.0, 100.0, 50.0);
        let rect2 = Rect::new(10.05, 20.05, 100.05, 50.05);

        // Should pass with reasonable epsilon
        assert_rect_approx_eq(&rect1, &rect2, 0.1);
    }

    #[test]
    #[should_panic(expected = "mismatch")]
    fn test_assert_rect_approx_eq_fails() {
        let rect1 = Rect::new(10.0, 20.0, 100.0, 50.0);
        let rect2 = Rect::new(15.0, 20.0, 100.0, 50.0);

        assert_rect_approx_eq(&rect1, &rect2, 0.1);
    }
}
