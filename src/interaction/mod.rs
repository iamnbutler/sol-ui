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
pub mod shortcuts;

pub use element::{Interactable, InteractiveElement};
pub use events::{EventHandlers, InteractionEvent, InteractionState};
pub use hit_test::{HitTestBuilder, HitTestEntry, HitTestResult};
pub use registry::{ElementRegistry, get_element_state, register_element};
pub use shortcuts::{
    Shortcut, ShortcutConflict, ShortcutId, ShortcutInfo, ShortcutMatch, ShortcutModifiers,
    ShortcutRegistry, ShortcutScope,
};

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

    /// Keyboard shortcuts registry
    shortcut_registry: ShortcutRegistry,

    /// Whether to process shortcuts before element handlers
    shortcuts_enabled: bool,
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
            shortcut_registry: ShortcutRegistry::new(),
            shortcuts_enabled: true,
        }
    }

    /// Create a new InteractionSystem with standard macOS shortcuts pre-registered
    pub fn with_standard_shortcuts() -> Self {
        let mut system = Self::new();
        shortcuts::standard::register_standard_shortcuts(&mut system.shortcut_registry);
        system
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

    /// Move focus to next focusable element (Tab key)
    pub fn focus_next(&mut self) -> Vec<InteractionEvent> {
        if self.focusable_elements.is_empty() {
            return Vec::new();
        }

        let next_index = if let Some(current) = self.focused_element {
            let current_index = self
                .focusable_elements
                .iter()
                .position(|&id| id == current)
                .unwrap_or(0);
            (current_index + 1) % self.focusable_elements.len()
        } else {
            0
        };

        let next_element = self.focusable_elements[next_index];
        self.set_focus(Some(next_element))
    }

    /// Move focus to previous focusable element (Shift+Tab key)
    pub fn focus_previous(&mut self) -> Vec<InteractionEvent> {
        if self.focusable_elements.is_empty() {
            return Vec::new();
        }

        let prev_index = if let Some(current) = self.focused_element {
            let current_index = self
                .focusable_elements
                .iter()
                .position(|&id| id == current)
                .unwrap_or(0);
            if current_index == 0 {
                self.focusable_elements.len() - 1
            } else {
                current_index - 1
            }
        } else {
            self.focusable_elements.len() - 1
        };

        let prev_element = self.focusable_elements[prev_index];
        self.set_focus(Some(prev_element))
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

        // Check for shortcuts first (only on initial key press, not repeats)
        if self.shortcuts_enabled && !is_repeat {
            if let Some(shortcut_match) =
                self.shortcut_registry.find_match(key, &modifiers, self.focused_element)
            {
                events.push(InteractionEvent::ShortcutTriggered {
                    shortcut_id: shortcut_match.id,
                    action_name: shortcut_match.action_name,
                });
                // Shortcut consumed the key event
                return events;
            }
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
    }

    /// Get current modifier state
    pub fn current_modifiers(&self) -> Modifiers {
        self.current_modifiers
    }

    // --- Shortcut methods ---

    /// Get a reference to the shortcut registry
    pub fn shortcuts(&self) -> &ShortcutRegistry {
        &self.shortcut_registry
    }

    /// Get a mutable reference to the shortcut registry
    pub fn shortcuts_mut(&mut self) -> &mut ShortcutRegistry {
        &mut self.shortcut_registry
    }

    /// Enable or disable shortcut processing
    pub fn set_shortcuts_enabled(&mut self, enabled: bool) {
        self.shortcuts_enabled = enabled;
    }

    /// Check if shortcuts are enabled
    pub fn shortcuts_enabled(&self) -> bool {
        self.shortcuts_enabled
    }

    /// Register a global shortcut
    pub fn register_shortcut(
        &mut self,
        shortcut: Shortcut,
        action_name: impl Into<String>,
    ) -> ShortcutId {
        self.shortcut_registry
            .register(shortcut, action_name, ShortcutScope::Global)
    }

    /// Register a shortcut that's only active when a specific element is focused
    pub fn register_focused_shortcut(
        &mut self,
        shortcut: Shortcut,
        action_name: impl Into<String>,
        element_id: ElementId,
    ) -> ShortcutId {
        self.shortcut_registry
            .register(shortcut, action_name, ShortcutScope::Focused(element_id))
    }

    /// Unregister a shortcut
    pub fn unregister_shortcut(&mut self, id: ShortcutId) {
        self.shortcut_registry.unregister(id);
    }

    /// Get a shortcut hint string for menus/tooltips (e.g., "⌘C" for copy)
    pub fn shortcut_hint(&self, action_name: &str) -> Option<String> {
        self.shortcut_registry.get_shortcut_hint(action_name)
    }

    /// Detect shortcut conflicts
    pub fn detect_shortcut_conflicts(&self) -> Vec<ShortcutConflict> {
        self.shortcut_registry.detect_conflicts()
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
