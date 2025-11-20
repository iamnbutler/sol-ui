//! Registry for interactive elements to enable event routing

use super::{ElementId, EventHandlers, InteractionEvent, InteractionState};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Registry for interactive elements in a UI layer
pub struct ElementRegistry {
    /// Map of element IDs to their event handlers
    handlers: HashMap<ElementId, Rc<RefCell<EventHandlers>>>,

    /// Map of element IDs to their current interaction state
    states: HashMap<ElementId, InteractionState>,
}

impl ElementRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// Register an element's event handlers
    pub fn register(&mut self, id: ElementId, handlers: Rc<RefCell<EventHandlers>>) {
        self.handlers.insert(id, handlers);
        self.states.insert(id, InteractionState::default());
    }

    /// Unregister an element
    pub fn unregister(&mut self, id: ElementId) {
        self.handlers.remove(&id);
        self.states.remove(&id);
    }

    /// Get the interaction state for an element
    pub fn get_state(&self, id: ElementId) -> Option<&InteractionState> {
        self.states.get(&id)
    }

    /// Update the interaction state for an element
    pub fn update_state(&mut self, id: ElementId, state: InteractionState) {
        if let Some(current_state) = self.states.get_mut(&id) {
            *current_state = state;
        }
    }

    /// Dispatch an event to the appropriate element
    pub fn dispatch_event(&mut self, event: &InteractionEvent) -> bool {
        let element_id = match event {
            InteractionEvent::MouseEnter { element_id }
            | InteractionEvent::MouseLeave { element_id }
            | InteractionEvent::MouseMove { element_id, .. }
            | InteractionEvent::MouseDown { element_id, .. }
            | InteractionEvent::MouseUp { element_id, .. }
            | InteractionEvent::Click { element_id, .. } => *element_id,
        };

        // Update states based on event type
        match event {
            InteractionEvent::MouseEnter { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_hovered = true;
                }
            }
            InteractionEvent::MouseLeave { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_hovered = false;
                }
            }
            InteractionEvent::MouseDown { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_pressed = true;
                }
            }
            InteractionEvent::MouseUp { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_pressed = false;
                }
            }
            _ => {}
        }

        // Dispatch to handlers
        if let Some(handlers) = self.handlers.get(&element_id) {
            handlers.borrow_mut().handle_event(event);
            true
        } else {
            false
        }
    }

    /// Clear all registrations
    pub fn clear(&mut self) {
        self.handlers.clear();
        self.states.clear();
    }

    /// Check if an element is registered
    pub fn is_registered(&self, id: ElementId) -> bool {
        self.handlers.contains_key(&id)
    }

    /// Get the number of registered elements
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for ElementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Thread-local registry for the current frame
    static CURRENT_REGISTRY: RefCell<Option<Rc<RefCell<ElementRegistry>>>> = RefCell::new(None);
}

/// Set the current element registry for this thread
pub fn set_current_registry(registry: Rc<RefCell<ElementRegistry>>) {
    CURRENT_REGISTRY.with(|r| {
        *r.borrow_mut() = Some(registry);
    });
}

/// Clear the current element registry
pub fn clear_current_registry() {
    CURRENT_REGISTRY.with(|r| {
        *r.borrow_mut() = None;
    });
}

/// Register an element with the current registry
pub fn register_element(id: ElementId, handlers: Rc<RefCell<EventHandlers>>) {
    CURRENT_REGISTRY.with(|r| {
        if let Some(registry) = r.borrow().as_ref() {
            registry.borrow_mut().register(id, handlers);
        }
    });
}

/// Get the interaction state for an element from the current registry
pub fn get_element_state(id: ElementId) -> Option<InteractionState> {
    CURRENT_REGISTRY.with(|r| {
        r.borrow()
            .as_ref()
            .and_then(|registry| registry.borrow().get_state(id).cloned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::MouseButton;
    use glam::Vec2;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_element_registry_creation() {
        let registry = ElementRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        let default_registry = ElementRegistry::default();
        assert_eq!(default_registry.len(), 0);
        assert!(default_registry.is_empty());
    }

    #[test]
    fn test_element_registration() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        assert!(!registry.is_registered(id));
        
        registry.register(id, handlers.clone());
        
        assert!(registry.is_registered(id));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        
        // Check initial state is created
        let state = registry.get_state(id);
        assert!(state.is_some());
        assert!(!state.unwrap().is_hovered);
        assert!(!state.unwrap().is_pressed);
    }

    #[test]
    fn test_element_unregistration() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id, handlers);
        assert!(registry.is_registered(id));
        assert_eq!(registry.len(), 1);

        registry.unregister(id);
        assert!(!registry.is_registered(id));
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(registry.get_state(id).is_none());
    }

    #[test]
    fn test_state_updates() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id, handlers);

        // Update state
        let mut new_state = InteractionState::new();
        new_state.is_hovered = true;
        new_state.is_pressed = true;
        
        registry.update_state(id, new_state);

        let state = registry.get_state(id).unwrap();
        assert!(state.is_hovered);
        assert!(state.is_pressed);
    }

    #[test]
    fn test_state_update_nonexistent_element() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(999);

        // Should not panic when updating non-existent element
        let mut state = InteractionState::new();
        state.is_hovered = true;
        registry.update_state(id, state);

        // Still should not exist
        assert!(!registry.is_registered(id));
        assert!(registry.get_state(id).is_none());
    }

    #[test]
    fn test_event_dispatch_success() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let handlers = Rc::new(RefCell::new(
            EventHandlers::new().on_mouse_enter(move || {
                *call_count_clone.lock().unwrap() += 1;
            })
        ));

        registry.register(id, handlers);

        let event = InteractionEvent::MouseEnter { element_id: id };
        let result = registry.dispatch_event(&event);

        assert!(result);
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Check state was updated
        let state = registry.get_state(id).unwrap();
        assert!(state.is_hovered);
    }

    #[test]
    fn test_event_dispatch_nonexistent_element() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(999);

        let event = InteractionEvent::MouseEnter { element_id: id };
        let result = registry.dispatch_event(&event);

        assert!(!result);
    }

    #[test]
    fn test_event_dispatch_state_changes() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id, handlers);

        // Test mouse enter
        let enter_event = InteractionEvent::MouseEnter { element_id: id };
        registry.dispatch_event(&enter_event);
        let state = registry.get_state(id).unwrap();
        assert!(state.is_hovered);
        assert!(!state.is_pressed);

        // Test mouse down
        let down_event = InteractionEvent::MouseDown {
            element_id: id,
            button: MouseButton::Left,
            position: Vec2::ZERO,
            local_position: Vec2::ZERO,
        };
        registry.dispatch_event(&down_event);
        let state = registry.get_state(id).unwrap();
        assert!(state.is_hovered);
        assert!(state.is_pressed);

        // Test mouse up
        let up_event = InteractionEvent::MouseUp {
            element_id: id,
            button: MouseButton::Left,
            position: Vec2::ZERO,
            local_position: Vec2::ZERO,
        };
        registry.dispatch_event(&up_event);
        let state = registry.get_state(id).unwrap();
        assert!(state.is_hovered);
        assert!(!state.is_pressed);

        // Test mouse leave
        let leave_event = InteractionEvent::MouseLeave { element_id: id };
        registry.dispatch_event(&leave_event);
        let state = registry.get_state(id).unwrap();
        assert!(!state.is_hovered);
        assert!(!state.is_pressed);
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = ElementRegistry::new();
        let id1 = ElementId::from(1);
        let id2 = ElementId::from(2);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id1, handlers.clone());
        registry.register(id2, handlers.clone());

        assert_eq!(registry.len(), 2);
        assert!(registry.is_registered(id1));
        assert!(registry.is_registered(id2));

        registry.clear();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(!registry.is_registered(id1));
        assert!(!registry.is_registered(id2));
        assert!(registry.get_state(id1).is_none());
        assert!(registry.get_state(id2).is_none());
    }

    #[test]
    fn test_multiple_elements() {
        let mut registry = ElementRegistry::new();
        let id1 = ElementId::from(1);
        let id2 = ElementId::from(2);
        let id3 = ElementId::from(3);

        let handlers1 = Rc::new(RefCell::new(EventHandlers::new()));
        let handlers2 = Rc::new(RefCell::new(EventHandlers::new()));
        let handlers3 = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id1, handlers1);
        registry.register(id2, handlers2);
        registry.register(id3, handlers3);

        assert_eq!(registry.len(), 3);
        assert!(registry.is_registered(id1));
        assert!(registry.is_registered(id2));
        assert!(registry.is_registered(id3));

        // Test unregistering middle element
        registry.unregister(id2);
        assert_eq!(registry.len(), 2);
        assert!(registry.is_registered(id1));
        assert!(!registry.is_registered(id2));
        assert!(registry.is_registered(id3));
    }

    #[test]
    fn test_event_dispatch_click_no_state_change() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id, handlers);

        let click_event = InteractionEvent::Click {
            element_id: id,
            button: MouseButton::Left,
            position: Vec2::ZERO,
            local_position: Vec2::ZERO,
        };

        // Click events should not change state
        registry.dispatch_event(&click_event);
        let state = registry.get_state(id).unwrap();
        assert!(!state.is_hovered);
        assert!(!state.is_pressed);
    }

    #[test]
    fn test_event_dispatch_mouse_move_no_state_change() {
        let mut registry = ElementRegistry::new();
        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        registry.register(id, handlers);

        let move_event = InteractionEvent::MouseMove {
            element_id: id,
            position: Vec2::new(100.0, 200.0),
            local_position: Vec2::new(50.0, 100.0),
        };

        // Mouse move events should not change state
        registry.dispatch_event(&move_event);
        let state = registry.get_state(id).unwrap();
        assert!(!state.is_hovered);
        assert!(!state.is_pressed);
    }

    #[test]
    fn test_thread_local_registry_functions() {
        // Clear any existing registry
        clear_current_registry();

        // Test get_element_state with no registry
        assert!(get_element_state(ElementId::from(1)).is_none());

        // Create and set a registry
        let registry = Rc::new(RefCell::new(ElementRegistry::new()));
        let id = ElementId::from(42);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        // Register element directly
        registry.borrow_mut().register(id, handlers.clone());
        
        // Set as current
        set_current_registry(registry.clone());

        // Test register_element function
        let id2 = ElementId::from(99);
        let handlers2 = Rc::new(RefCell::new(EventHandlers::new()));
        register_element(id2, handlers2);

        // Verify both elements are registered
        assert!(registry.borrow().is_registered(id));
        assert!(registry.borrow().is_registered(id2));

        // Test get_element_state function
        let state = get_element_state(id);
        assert!(state.is_some());
        assert!(!state.unwrap().is_hovered);
        assert!(!state.unwrap().is_pressed);

        // Clear registry
        clear_current_registry();
        assert!(get_element_state(id).is_none());
    }

    #[test]
    fn test_register_element_no_current_registry() {
        clear_current_registry();

        let id = ElementId::from(1);
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        // Should not panic when no current registry
        register_element(id, handlers);
    }
}
