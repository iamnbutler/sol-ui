//! Interaction system for handling mouse and keyboard events with z-order based hit testing

use crate::{
    geometry::Point,
    layer::{InputEvent, Key, Modifiers, MouseButton},
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

    /// Currently focused element ID (receives keyboard events)
    focused_element: Option<ElementId>,

    /// Current modifier key state
    current_modifiers: Modifiers,

    /// Element interaction states
    element_states: HashMap<ElementId, InteractionState>,

    /// Hit test results from last frame
    last_hit_test: Vec<HitTestEntry>,

    /// List of focusable elements in tab order (built during paint)
    focusable_elements: Vec<ElementId>,

    /// Whether mouse is currently over the window
    mouse_in_window: bool,

    /// Stack of focus traps (for modal dialogs)
    /// Each trap contains the element IDs that form the trap boundary
    focus_trap_stack: Vec<Vec<ElementId>>,
}

impl InteractionSystem {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
            hovered_element: None,
            pressed_element: None,
            focused_element: None,
            current_modifiers: Modifiers::new(),
            element_states: HashMap::new(),
            last_hit_test: Vec::new(),
            focusable_elements: Vec::new(),
            mouse_in_window: false,
            focus_trap_stack: Vec::new(),
        }
    }

    /// Get the currently focused element
    pub fn focused_element(&self) -> Option<ElementId> {
        self.focused_element
    }

    /// Set focus to an element, returning focus events
    pub fn set_focus(&mut self, element_id: Option<ElementId>) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        if self.focused_element == element_id {
            return events;
        }

        // Remove focus from previous element
        if let Some(prev_id) = self.focused_element {
            if let Some(state) = self.element_states.get_mut(&prev_id) {
                state.is_focused = false;
            }
            events.push(InteractionEvent::FocusOut {
                element_id: prev_id,
            });
        }

        // Set focus to new element
        if let Some(new_id) = element_id {
            self.element_states
                .entry(new_id)
                .or_insert_with(InteractionState::new)
                .is_focused = true;

            events.push(InteractionEvent::FocusIn { element_id: new_id });
        }

        self.focused_element = element_id;
        events
    }

    /// Register an element as focusable (called during paint)
    pub fn register_focusable(&mut self, element_id: ElementId) {
        if !self.focusable_elements.contains(&element_id) {
            self.focusable_elements.push(element_id);
        }
    }

    /// Clear focusable elements (called at start of paint)
    pub fn clear_focusable_elements(&mut self) {
        self.focusable_elements.clear();
    }

    /// Get the current set of navigable elements (respecting focus traps)
    fn get_navigable_elements(&self) -> &[ElementId] {
        if let Some(trap) = self.focus_trap_stack.last() {
            trap.as_slice()
        } else {
            &self.focusable_elements
        }
    }

    /// Move focus to next focusable element (Tab key)
    pub fn focus_next(&mut self) -> Vec<InteractionEvent> {
        let navigable = self.get_navigable_elements();
        if navigable.is_empty() {
            return Vec::new();
        }

        let next_index = if let Some(current) = self.focused_element {
            let current_index = navigable
                .iter()
                .position(|&id| id == current)
                .unwrap_or(0);
            (current_index + 1) % navigable.len()
        } else {
            0
        };

        let next_element = navigable[next_index];
        self.set_focus(Some(next_element))
    }

    /// Move focus to previous focusable element (Shift+Tab key)
    pub fn focus_previous(&mut self) -> Vec<InteractionEvent> {
        let navigable = self.get_navigable_elements();
        if navigable.is_empty() {
            return Vec::new();
        }

        let prev_index = if let Some(current) = self.focused_element {
            let current_index = navigable
                .iter()
                .position(|&id| id == current)
                .unwrap_or(0);
            if current_index == 0 {
                navigable.len() - 1
            } else {
                current_index - 1
            }
        } else {
            navigable.len() - 1
        };

        let prev_element = navigable[prev_index];
        self.set_focus(Some(prev_element))
    }

    /// Push a focus trap (used for modals)
    /// Elements in the trap must be from the current focusable_elements list
    /// Returns focus events if the first element in the trap gains focus
    pub fn push_focus_trap(&mut self, element_ids: Vec<ElementId>) -> Vec<InteractionEvent> {
        // Filter to only include elements that are actually focusable
        let trap: Vec<_> = element_ids
            .into_iter()
            .filter(|id| self.focusable_elements.contains(id))
            .collect();

        if trap.is_empty() {
            return Vec::new();
        }

        // Auto-focus first element in trap if nothing in trap is focused
        let should_focus_first = self.focused_element.map_or(true, |focused| {
            !trap.contains(&focused)
        });

        let first_element = trap[0];
        self.focus_trap_stack.push(trap);

        if should_focus_first {
            self.set_focus(Some(first_element))
        } else {
            Vec::new()
        }
    }

    /// Pop the current focus trap (when modal closes)
    /// Returns focus events if focus should return to a previous element
    pub fn pop_focus_trap(&mut self) -> Vec<InteractionEvent> {
        self.focus_trap_stack.pop();
        // Optionally restore focus to something in the parent scope
        // For now, keep focus where it is unless it's no longer valid
        if let Some(current) = self.focused_element {
            let navigable = self.get_navigable_elements();
            if !navigable.contains(&current) && !navigable.is_empty() {
                // Current focus is no longer in scope, move to first navigable element
                return self.set_focus(Some(navigable[0]));
            }
        }
        Vec::new()
    }

    /// Check if a focus trap is active
    pub fn has_focus_trap(&self) -> bool {
        !self.focus_trap_stack.is_empty()
    }

    /// Update the hit test results for the current frame
    pub fn update_hit_test(&mut self, entries: Vec<HitTestEntry>) {
        // Extract focusable elements in paint/tab order (lower z-index first for tab order)
        self.focusable_elements.clear();
        let mut focusables: Vec<_> = entries
            .iter()
            .filter(|e| e.focusable)
            .collect();
        // Sort by z-index ascending for tab order (paint order)
        focusables.sort_by_key(|e| e.z_index);
        for entry in focusables {
            self.focusable_elements.push(entry.element_id);
        }

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

            InputEvent::KeyDown {
                key,
                modifiers,
                character,
                is_repeat,
            } => {
                self.current_modifiers = *modifiers;
                events.extend(self.handle_key_down(*key, *modifiers, *character, *is_repeat));
            }

            InputEvent::KeyUp { key, modifiers } => {
                self.current_modifiers = *modifiers;
                events.extend(self.handle_key_up(*key, *modifiers));
            }

            InputEvent::ModifiersChanged { modifiers } => {
                self.current_modifiers = *modifiers;
            }

            InputEvent::ScrollWheel { position, delta } => {
                self.mouse_position = *position;
                events.extend(self.handle_scroll_wheel(*position, *delta));
            }

            InputEvent::ScaleFactorChanged { .. } => {
                // Scale factor changes are handled at the app level
            }
        }

        events
    }

    /// Handle key down events
    fn handle_key_down(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        character: Option<char>,
        is_repeat: bool,
    ) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Handle Tab key for focus navigation
        if key == Key::Tab && !is_repeat {
            if modifiers.shift {
                events.extend(self.focus_previous());
            } else {
                events.extend(self.focus_next());
            }
            return events;
        }

        // Route keyboard event to focused element
        if let Some(element_id) = self.focused_element {
            events.push(InteractionEvent::KeyDown {
                element_id,
                key,
                modifiers,
                character,
                is_repeat,
            });
        }

        events
    }

    /// Handle key up events
    fn handle_key_up(&mut self, key: Key, modifiers: Modifiers) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Route keyboard event to focused element
        if let Some(element_id) = self.focused_element {
            events.push(InteractionEvent::KeyUp {
                element_id,
                key,
                modifiers,
            });
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

            // Focus the clicked element if it's focusable (left click only)
            if button == MouseButton::Left && self.focusable_elements.contains(&element_id) {
                events.extend(self.set_focus(Some(element_id)));
            }
        } else {
            // Clicked on empty space - clear focus
            events.extend(self.set_focus(None));
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

    /// Handle scroll wheel events
    fn handle_scroll_wheel(&mut self, position: Vec2, delta: Vec2) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // Find what's under the mouse and send scroll event to it
        if let Some(hit) = self.hit_test(position) {
            events.push(InteractionEvent::ScrollWheel {
                element_id: hit.element_id,
                delta,
                position,
                local_position: hit.local_position,
            });
        }

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
        self.focused_element = None;
        self.last_hit_test.clear();
        self.focusable_elements.clear();
        self.focus_trap_stack.clear();
    }

    /// Get current modifier state
    pub fn current_modifiers(&self) -> Modifiers {
        self.current_modifiers
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
