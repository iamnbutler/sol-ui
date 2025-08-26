//! Interaction system for handling mouse events with z-order based hit testing

use crate::{
    geometry::Point,
    layer::{InputEvent, MouseButton},
};
use glam::Vec2;
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
    mouse_position: Vec2,

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
            mouse_position: Vec2::ZERO,
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

            _ => {} // Other events not handled yet
        }

        events
    }

    /// Handle mouse move events
    fn handle_mouse_move(&mut self, position: Vec2) -> Vec<InteractionEvent> {
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
    fn handle_mouse_down(&mut self, position: Vec2, button: MouseButton) -> Vec<InteractionEvent> {
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
    fn handle_mouse_up(&mut self, position: Vec2, button: MouseButton) -> Vec<InteractionEvent> {
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
                        .unwrap_or(position),
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
    fn hit_test(&self, position: Vec2) -> Option<HitTestResult> {
        // Hit test entries are sorted by z-order (highest first)
        for entry in &self.last_hit_test {
            if entry.bounds.contains(Point::from(position)) {
                let local_position = position - entry.bounds.pos;
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
    use crate::layer::{InputEvent, MouseButton};
    use glam::Vec2;

    #[test]
    fn test_element_id_creation() {
        let id1 = ElementId::new(42);
        let id2 = ElementId::from(42u64);
        let id3 = ElementId::from(42usize);
        let id4 = ElementId::from(42i32);

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id3, id4);
        assert_eq!(id1.0, 42);
    }

    #[test]
    fn test_element_id_auto() {
        let id1 = ElementId::auto();
        let id2 = ElementId::auto();
        let id3 = ElementId::default();

        // Auto-generated IDs should be different
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        
        // Auto IDs should start high to avoid conflicts
        assert!(id1.0 >= 1_000_000);
        assert!(id2.0 >= 1_000_000);
        assert!(id3.0 >= 1_000_000);
    }

    #[test]
    fn test_element_id_hash_and_eq() {
        use std::collections::HashMap;

        let id1 = ElementId::from(100);
        let id2 = ElementId::from(100);
        let id3 = ElementId::from(200);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        // Test HashMap usage
        let mut map = HashMap::new();
        map.insert(id1, "first");
        map.insert(id3, "second");
        
        assert_eq!(map.get(&id2), Some(&"first"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_interaction_system_creation() {
        let system = InteractionSystem::new();
        
        assert_eq!(system.mouse_position, Vec2::ZERO);
        assert_eq!(system.hovered_element, None);
        assert_eq!(system.pressed_element, None);
        assert_eq!(system.element_states.len(), 0);
        assert_eq!(system.last_hit_test.len(), 0);
        assert!(!system.mouse_in_window);
    }

    #[test]
    fn test_interaction_system_hit_test_update() {
        let mut system = InteractionSystem::new();
        let entries = vec![
            HitTestEntry {
                element_id: ElementId::from(1),
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            },
            HitTestEntry {
                element_id: ElementId::from(2),
                bounds: crate::geometry::Rect::new(50.0, 50.0, 100.0, 100.0),
                z_index: 2,
                layer: 0,
            }
        ];

        system.update_hit_test(entries.clone());
        assert_eq!(system.last_hit_test, entries);
    }

    #[test]
    fn test_interaction_system_mouse_move_input() {
        let mut system = InteractionSystem::new();
        let position = Vec2::new(10.0, 20.0);

        let event = InputEvent::MouseMove { position };
        let events = system.handle_input(&event);

        assert_eq!(system.mouse_position, position);
        assert!(system.mouse_in_window);
        // With no hit test entries, no interaction events should be generated
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_interaction_system_mouse_move_with_hit_test() {
        let mut system = InteractionSystem::new();
        let element_id = ElementId::from(1);
        let position = Vec2::new(50.0, 50.0);

        // Set up hit test entry
        let entries = vec![
            HitTestEntry {
                element_id,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        // Move mouse over the element
        let event = InputEvent::MouseMove { position };
        let events = system.handle_input(&event);

        assert_eq!(events.len(), 2); // MouseEnter + MouseMove
        
        match &events[0] {
            InteractionEvent::MouseEnter { element_id: id } => assert_eq!(*id, element_id),
            _ => panic!("Expected MouseEnter event"),
        }

        match &events[1] {
            InteractionEvent::MouseMove { element_id: id, position: pos, local_position } => {
                assert_eq!(*id, element_id);
                assert_eq!(*pos, position);
                assert_eq!(*local_position, position);
            }
            _ => panic!("Expected MouseMove event"),
        }

        assert_eq!(system.hovered_element, Some(element_id));
    }

    #[test]
    fn test_interaction_system_mouse_down_up() {
        let mut system = InteractionSystem::new();
        let element_id = ElementId::from(1);
        let position = Vec2::new(50.0, 50.0);
        let button = MouseButton::Left;

        // Set up hit test entry
        let entries = vec![
            HitTestEntry {
                element_id,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        // Mouse down
        let down_event = InputEvent::MouseDown { position, button };
        let events = system.handle_input(&down_event);

        assert_eq!(events.len(), 1);
        match &events[0] {
            InteractionEvent::MouseDown { element_id: id, button: btn, .. } => {
                assert_eq!(*id, element_id);
                assert_eq!(*btn, button);
            }
            _ => panic!("Expected MouseDown event"),
        }

        assert_eq!(system.pressed_element, Some((element_id, button)));

        // Mouse up
        let up_event = InputEvent::MouseUp { position, button };
        let events = system.handle_input(&up_event);

        assert_eq!(events.len(), 2); // MouseUp + Click
        
        match &events[0] {
            InteractionEvent::MouseUp { element_id: id, button: btn, .. } => {
                assert_eq!(*id, element_id);
                assert_eq!(*btn, button);
            }
            _ => panic!("Expected MouseUp event"),
        }

        match &events[1] {
            InteractionEvent::Click { element_id: id, button: btn, .. } => {
                assert_eq!(*id, element_id);
                assert_eq!(*btn, button);
            }
            _ => panic!("Expected Click event"),
        }

        assert_eq!(system.pressed_element, None);
    }

    #[test]
    fn test_interaction_system_mouse_down_up_different_elements() {
        let mut system = InteractionSystem::new();
        let element_id1 = ElementId::from(1);
        let element_id2 = ElementId::from(2);
        let position1 = Vec2::new(25.0, 25.0);
        let position2 = Vec2::new(75.0, 75.0);
        let button = MouseButton::Left;

        // Set up hit test entries
        let entries = vec![
            HitTestEntry {
                element_id: element_id1,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 50.0, 50.0),
                z_index: 1,
                layer: 0,
            },
            HitTestEntry {
                element_id: element_id2,
                bounds: crate::geometry::Rect::new(50.0, 50.0, 50.0, 50.0),
                z_index: 2,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        // Mouse down on first element
        let down_event = InputEvent::MouseDown { position: position1, button };
        let events = system.handle_input(&down_event);
        assert_eq!(events.len(), 1);
        assert_eq!(system.pressed_element, Some((element_id1, button)));

        // Mouse up on second element
        let up_event = InputEvent::MouseUp { position: position2, button };
        let events = system.handle_input(&up_event);

        // Should get MouseUp but no Click (different elements)
        assert_eq!(events.len(), 1);
        match &events[0] {
            InteractionEvent::MouseUp { element_id: id, .. } => {
                assert_eq!(*id, element_id1); // MouseUp goes to pressed element
            }
            _ => panic!("Expected MouseUp event"),
        }

        assert_eq!(system.pressed_element, None);
    }

    #[test]
    fn test_interaction_system_mouse_leave_window() {
        let mut system = InteractionSystem::new();
        let element_id = ElementId::from(1);
        let position = Vec2::new(50.0, 50.0);

        // Set up element and hover it
        let entries = vec![
            HitTestEntry {
                element_id,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        // Hover element
        system.handle_input(&InputEvent::MouseMove { position });
        assert_eq!(system.hovered_element, Some(element_id));
        assert!(system.mouse_in_window);

        // Mouse leave window
        let leave_event = InputEvent::MouseLeave;
        let events = system.handle_input(&leave_event);

        assert_eq!(events.len(), 1);
        match &events[0] {
            InteractionEvent::MouseLeave { element_id: id } => {
                assert_eq!(*id, element_id);
            }
            _ => panic!("Expected MouseLeave event"),
        }

        assert_eq!(system.hovered_element, None);
        assert!(!system.mouse_in_window);
    }

    #[test]
    fn test_interaction_system_get_state() {
        let mut system = InteractionSystem::new();
        let element_id = ElementId::from(1);

        // Initially no state
        assert!(system.get_state(element_id).is_none());

        // Create state by interacting
        let entries = vec![
            HitTestEntry {
                element_id,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        let position = Vec2::new(50.0, 50.0);
        system.handle_input(&InputEvent::MouseMove { position });

        // Now should have state
        let state = system.get_state(element_id);
        assert!(state.is_some());
        assert!(state.unwrap().is_hovered);
        assert!(!state.unwrap().is_pressed);
    }

    #[test]
    fn test_interaction_system_clear() {
        let mut system = InteractionSystem::new();
        let element_id = ElementId::from(1);

        // Set up some state
        let entries = vec![
            HitTestEntry {
                element_id,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 100.0, 100.0),
                z_index: 1,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        let position = Vec2::new(50.0, 50.0);
        system.handle_input(&InputEvent::MouseMove { position });
        system.handle_input(&InputEvent::MouseDown { position, button: MouseButton::Left });

        // Verify state exists
        assert_eq!(system.hovered_element, Some(element_id));
        assert_eq!(system.pressed_element, Some((element_id, MouseButton::Left)));
        assert!(!system.element_states.is_empty());
        assert!(!system.last_hit_test.is_empty());

        // Clear
        system.clear();

        // Verify everything is cleared
        assert_eq!(system.hovered_element, None);
        assert_eq!(system.pressed_element, None);
        assert!(system.element_states.is_empty());
        assert!(system.last_hit_test.is_empty());
    }

    #[test]
    fn test_interaction_system_other_input_events() {
        let mut system = InteractionSystem::new();

        // Test that other events don't cause panics or unexpected behavior
        let other_events = vec![
            InputEvent::KeyDown { key_code: 65 },
            InputEvent::KeyUp { key_code: 65 },
            InputEvent::WindowResize { width: 800, height: 600 },
        ];

        for event in other_events {
            let events = system.handle_input(&event);
            assert_eq!(events.len(), 0); // Should not generate interaction events
        }
    }

    #[test]
    fn test_interaction_system_hover_state_transitions() {
        let mut system = InteractionSystem::new();
        let element_id1 = ElementId::from(1);
        let element_id2 = ElementId::from(2);

        let entries = vec![
            HitTestEntry {
                element_id: element_id1,
                bounds: crate::geometry::Rect::new(0.0, 0.0, 50.0, 50.0),
                z_index: 1,
                layer: 0,
            },
            HitTestEntry {
                element_id: element_id2,
                bounds: crate::geometry::Rect::new(50.0, 0.0, 50.0, 50.0),
                z_index: 2,
                layer: 0,
            }
        ];
        system.update_hit_test(entries);

        // Hover first element
        let events = system.handle_input(&InputEvent::MouseMove { position: Vec2::new(25.0, 25.0) });
        assert_eq!(events.len(), 2); // MouseEnter + MouseMove
        assert_eq!(system.hovered_element, Some(element_id1));

        // Move to second element
        let events = system.handle_input(&InputEvent::MouseMove { position: Vec2::new(75.0, 25.0) });
        assert_eq!(events.len(), 3); // MouseLeave + MouseEnter + MouseMove
        
        match &events[0] {
            InteractionEvent::MouseLeave { element_id } => assert_eq!(*element_id, element_id1),
            _ => panic!("Expected MouseLeave for first element"),
        }

        match &events[1] {
            InteractionEvent::MouseEnter { element_id } => assert_eq!(*element_id, element_id2),
            _ => panic!("Expected MouseEnter for second element"),
        }

        assert_eq!(system.hovered_element, Some(element_id2));

        // Verify state changes
        let state1 = system.get_state(element_id1).unwrap();
        let state2 = system.get_state(element_id2).unwrap();
        assert!(!state1.is_hovered);
        assert!(state2.is_hovered);
    }
}
