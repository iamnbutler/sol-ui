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

    /// List of focusable elements in tab order
    focusable_elements: Vec<ElementId>,
}

impl ElementRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            states: HashMap::new(),
            focusable_elements: Vec::new(),
        }
    }

    /// Register an element as focusable
    pub fn register_focusable(&mut self, id: ElementId) {
        if !self.focusable_elements.contains(&id) {
            self.focusable_elements.push(id);
        }
    }

    /// Get the list of focusable elements
    pub fn focusable_elements(&self) -> &[ElementId] {
        &self.focusable_elements
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
        // ShortcutTriggered events are handled at the application level, not dispatched to elements
        let element_id = match event {
            InteractionEvent::MouseEnter { element_id }
            | InteractionEvent::MouseLeave { element_id }
            | InteractionEvent::MouseMove { element_id, .. }
            | InteractionEvent::MouseDown { element_id, .. }
            | InteractionEvent::MouseUp { element_id, .. }
            | InteractionEvent::Click { element_id, .. }
            | InteractionEvent::ScrollWheel { element_id, .. }
            | InteractionEvent::KeyDown { element_id, .. }
            | InteractionEvent::KeyUp { element_id, .. }
            | InteractionEvent::FocusIn { element_id }
            | InteractionEvent::FocusOut { element_id } => *element_id,
            InteractionEvent::ShortcutTriggered { .. } => {
                // Shortcut events aren't dispatched to specific elements
                return true;
            }
            InteractionEvent::DragDrop(_) => {
                // Drag and drop events aren't dispatched to specific elements
                return true;
            }
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
            InteractionEvent::FocusIn { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_focused = true;
                }
            }
            InteractionEvent::FocusOut { .. } => {
                if let Some(state) = self.states.get_mut(&element_id) {
                    state.is_focused = false;
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
        self.focusable_elements.clear();
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

/// Register an element as focusable with the current registry
pub fn register_focusable(id: ElementId) {
    CURRENT_REGISTRY.with(|r| {
        if let Some(registry) = r.borrow().as_ref() {
            registry.borrow_mut().register_focusable(id);
        }
    });
}
