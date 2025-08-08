//! Interaction system for handling mouse events with z-order based hit testing

use crate::{
    geometry::{ScreenPoint, LocalPoint},
    layer::{InputEvent, MouseButton},
};
use std::collections::HashMap;

pub mod element;
pub mod events;
pub mod hit_test;
pub mod registry;

pub use element::{Interactable, InteractiveElement};
pub use events::{EventHandlers, InteractionEvent, InteractionState};
pub use hit_test::{HitTestBuilder, HitTestEntry, HitTestResult};
pub use registry::{ElementRegistry, get_element_state, register_element};

/// Manages interaction state across the entire UI
pub struct InteractionSystem {
    /// Current mouse position
    mouse_position: ScreenPoint,

    /// Currently hovered element ID
    hovered_element: Option<ElementId>,

    /// Currently pressed element ID and button
    pressed_element: Option<(ElementId, MouseButton)>,

    /// Element interaction states
    element_states: HashMap<ElementId, InteractionState>,

    /// Hit test results from last frame
    last_hit_test: Vec<HitTestEntry>,

    /// Whether mouse is currently over the window
    mouse_in_window: bool,
}

impl InteractionSystem {
    pub fn new() -> Self {
        Self {
            mouse_position: ScreenPoint::ZERO,
            hovered_element: None,
            pressed_element: None,
            element_states: HashMap::new(),
            last_hit_test: Vec::new(),
            mouse_in_window: false,
        }
    }

    /// Update the hit test results for the current frame
    pub fn update_hit_test(&mut self, entries: Vec<HitTestEntry>) {
        self.last_hit_test = entries;

        // Update hover state based on new hit test
        if self.mouse_in_window {
            self.update_hover_state();
        }
    }

    /// Process an input event and return interaction events
    pub fn handle_input(&mut self, event: &InputEvent) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        match event {
            InputEvent::MouseMove { position } => {
                self.mouse_position = *position;
                self.mouse_in_window = true;
                events.extend(self.handle_mouse_move(*position));
            }

            InputEvent::MouseDown { position, button } => {
                self.mouse_position = *position;
                events.extend(self.handle_mouse_down(*position, *button));
            }

            InputEvent::MouseUp { position, button } => {
                self.mouse_position = *position;
                events.extend(self.handle_mouse_up(*position, *button));
            }

            InputEvent::MouseLeave => {
                self.mouse_in_window = false;
                events.extend(self.handle_mouse_leave());
            }

            // Map touch events to mouse events for compatibility
            InputEvent::TouchDown { position, .. } => {
                self.mouse_position = *position;
                self.mouse_in_window = true;
                events.extend(self.handle_mouse_down(*position, MouseButton::Left));
            }

            InputEvent::TouchMove { position, .. } => {
                self.mouse_position = *position;
                self.mouse_in_window = true;
                events.extend(self.handle_mouse_move(*position));
            }

            InputEvent::TouchUp { position, .. } => {
                self.mouse_position = *position;
                events.extend(self.handle_mouse_up(*position, MouseButton::Left));
            }

            InputEvent::TouchCancel { .. } => {
                // Treat touch cancel like mouse leave - clear any pressed states
                self.mouse_in_window = false;
                events.extend(self.handle_mouse_leave());
                // Also clear pressed state if any
                if let Some((pressed_id, _)) = self.pressed_element {
                    if let Some(state) = self.element_states.get_mut(&pressed_id) {
                        state.is_pressed = false;
                    }
                    self.pressed_element = None;
                }
            }

            _ => {} // Other events not handled yet
        }

        events
    }

    /// Handle mouse move events
    fn handle_mouse_move(&mut self, position: ScreenPoint) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Find what's under the mouse
        let hit = self.hit_test(position);
        let new_hovered = hit.as_ref().map(|h| h.element_id);

        // Handle hover changes
        if new_hovered != self.hovered_element {
            // Mouse leave on previous element
            if let Some(prev_id) = self.hovered_element {
                if let Some(state) = self.element_states.get_mut(&prev_id) {
                    state.is_hovered = false;
                }
                events.push(InteractionEvent::MouseLeave {
                    element_id: prev_id,
                });
            }

            // Mouse enter on new element
            if let Some(new_id) = new_hovered {
                self.element_states
                    .entry(new_id)
                    .or_insert_with(InteractionState::new)
                    .is_hovered = true;

                events.push(InteractionEvent::MouseEnter { element_id: new_id });
            }

            self.hovered_element = new_hovered;
        }

        // Send move event to hovered element
        if let Some(element_id) = self.hovered_element {
            if let Some(hit_result) = hit.as_ref() {
                events.push(InteractionEvent::MouseMove {
                    element_id,
                    position,
                    local_position: hit_result.local_position,
                });
            }
        }

        events
    }

    /// Handle mouse down events
    fn handle_mouse_down(&mut self, position: ScreenPoint, button: MouseButton) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Find what's under the mouse
        if let Some(hit) = self.hit_test(position) {
            let element_id = hit.element_id;

            // Update pressed state
            self.pressed_element = Some((element_id, button));

            if let Some(state) = self.element_states.get_mut(&element_id) {
                state.is_pressed = true;
            }

            events.push(InteractionEvent::MouseDown {
                element_id,
                button,
                position,
                local_position: hit.local_position,
            });
        }

        events
    }

    /// Handle mouse up events
    fn handle_mouse_up(&mut self, position: ScreenPoint, button: MouseButton) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Check if we have a pressed element
        if let Some((pressed_id, pressed_button)) = self.pressed_element {
            if pressed_button == button {
                // Clear pressed state
                self.pressed_element = None;

                if let Some(state) = self.element_states.get_mut(&pressed_id) {
                    state.is_pressed = false;
                }

                // Find what's currently under the mouse
                let current_hit = self.hit_test(position);
                let current_element = current_hit.as_ref().map(|h| h.element_id);

                // Send mouse up to pressed element
                events.push(InteractionEvent::MouseUp {
                    element_id: pressed_id,
                    button,
                    position,
                    local_position: current_hit
                        .as_ref()
                        .filter(|h| h.element_id == pressed_id)
                        .map(|h| h.local_position)
                        .unwrap_or(LocalPoint::ZERO),
                });

                // If mouse is still over the same element, it's a click
                if current_element == Some(pressed_id) {
                    events.push(InteractionEvent::Click {
                        element_id: pressed_id,
                        button,
                        position,
                        local_position: current_hit.unwrap().local_position,
                    });
                }
            }
        }

        events
    }

    /// Handle mouse leave window
    fn handle_mouse_leave(&mut self) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Clear hover state
        if let Some(hovered_id) = self.hovered_element.take() {
            if let Some(state) = self.element_states.get_mut(&hovered_id) {
                state.is_hovered = false;
            }
            events.push(InteractionEvent::MouseLeave {
                element_id: hovered_id,
            });
        }

        // Note: We don't clear pressed state on mouse leave
        // This allows drag operations to continue outside the window

        events
    }

    /// Update hover state based on current mouse position
    fn update_hover_state(&mut self) {
        let _ = self.handle_mouse_move(self.mouse_position);
    }

    /// Perform hit testing at the given position
    fn hit_test(&self, position: ScreenPoint) -> Option<HitTestResult> {
        // Hit test entries are sorted by z-order (highest first)
        for entry in &self.last_hit_test {
            // Convert screen position to world position for bounds check
            let world_pos = crate::geometry::screen_to_world(position);
            if entry.bounds.contains(world_pos) {
                let local_position = LocalPoint::new(
                    position.x() - entry.bounds.pos.x(),
                    position.y() - entry.bounds.pos.y()
                );
                return Some(HitTestResult {
                    element_id: entry.element_id,
                    bounds: entry.bounds,
                    local_position,
                    z_index: entry.z_index,
                });
            }
        }
        None
    }

    /// Get the current interaction state for an element
    pub fn get_state(&self, element_id: ElementId) -> Option<&InteractionState> {
        self.element_states.get(&element_id)
    }

    /// Clear all interaction state
    pub fn clear(&mut self) {
        self.element_states.clear();
        self.hovered_element = None;
        self.pressed_element = None;
        self.last_hit_test.clear();
    }
}

/// Unique identifier for interactive elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub u64);

impl ElementId {
    /// Create a new element ID with a specific value
    pub fn new(id: u64) -> Self {
        ElementId(id)
    }

    /// Create an auto-generated element ID (should be avoided for stable IDs)
    pub fn auto() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1_000_000); // Start high to avoid conflicts
        ElementId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::auto()
    }
}

impl From<u64> for ElementId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<usize> for ElementId {
    fn from(id: usize) -> Self {
        Self::new(id as u64)
    }
}

impl From<i32> for ElementId {
    fn from(id: i32) -> Self {
        Self::new(id as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::InputEvent;
    use crate::geometry::ScreenPoint;

    #[test]
    fn test_touch_down_maps_to_mouse_down() {
        let mut system = InteractionSystem::new();

        // Simulate touch down
        let touch_event = InputEvent::TouchDown {
            position: ScreenPoint::new(100.0, 200.0),
            id: 1,
        };

        let _events = system.handle_input(&touch_event);

        // Since there's no element at this position, we shouldn't get interaction events
        // but the system should update its internal state
        assert_eq!(system.mouse_position, ScreenPoint::new(100.0, 200.0));
        assert!(system.mouse_in_window);
    }

    #[test]
    fn test_touch_move_maps_to_mouse_move() {
        let mut system = InteractionSystem::new();

        // Simulate touch move
        let touch_event = InputEvent::TouchMove {
            position: ScreenPoint::new(150.0, 250.0),
            id: 1,
        };

        let _events = system.handle_input(&touch_event);

        // Check internal state updated
        assert_eq!(system.mouse_position, ScreenPoint::new(150.0, 250.0));
        assert!(system.mouse_in_window);
    }

    #[test]
    fn test_touch_up_maps_to_mouse_up() {
        let mut system = InteractionSystem::new();

        // Simulate touch up
        let touch_event = InputEvent::TouchUp {
            position: ScreenPoint::new(200.0, 300.0),
            id: 1,
        };

        let _events = system.handle_input(&touch_event);

        // Check internal state updated
        assert_eq!(system.mouse_position, ScreenPoint::new(200.0, 300.0));
    }

    #[test]
    fn test_touch_cancel_clears_state() {
        let mut system = InteractionSystem::new();

        // Set up some state first
        system.mouse_in_window = true;
        system.mouse_position = ScreenPoint::new(100.0, 100.0);

        // Simulate touch cancel
        let touch_event = InputEvent::TouchCancel { id: 1 };

        let _events = system.handle_input(&touch_event);

        // Check that mouse_in_window is cleared
        assert!(!system.mouse_in_window);
    }

    #[test]
    fn test_element_id_creation() {
        let id1 = ElementId::new(42);
        let id2 = ElementId::from(42usize);
        let id3 = ElementId::from(42i32);

        assert_eq!(id1.0, 42);
        assert_eq!(id2.0, 42);
        assert_eq!(id3.0, 42);
    }

    #[test]
    fn test_interaction_state_default() {
        let state = InteractionState::default();
        assert!(!state.is_hovered);
        assert!(!state.is_pressed);
    }

    // test_todo!("Test touch events with actual elements using MobileTestContext")
    // test_todo!("Test multi-touch handling with MobileTestContext")
    // test_todo!("Test touch event to InteractionEvent conversion with hit testing")
}
