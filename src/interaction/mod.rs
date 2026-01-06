//! Interaction system for handling mouse and keyboard events with z-order based hit testing

use crate::{
    geometry::Point,
    layer::{InputEvent, Key, Modifiers, MouseButton},
};
use glam::Vec2;
use std::collections::HashMap;

pub mod drag_drop;
pub mod element;
pub mod events;
pub mod hit_test;
pub mod registry;
pub mod shortcuts;

pub use drag_drop::{
    DragConfig, DragData, DragDropEvent, DragState, DropResult, DropZone, DropZoneRegistry,
    Draggable, DropTarget, DRAG_THRESHOLD,
};
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

    /// Stack of focus traps (for modal dialogs)
    /// Each trap contains the element IDs that form the trap boundary
    focus_trap_stack: Vec<Vec<ElementId>>,

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
            focus_trap_stack: Vec::new(),
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

    /// Create a stable element ID from a string key.
    ///
    /// This generates a deterministic ID by hashing the key, which means
    /// the same key will always produce the same ID across frames.
    /// Use this for interactive elements that need stable identity.
    ///
    /// # Example
    /// ```
    /// let id = ElementId::stable("my-button");
    /// let id2 = ElementId::stable("my-button");
    /// assert_eq!(id, id2); // Same key = same ID
    /// ```
    pub fn stable(key: impl AsRef<str>) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.as_ref().hash(&mut hasher);
        // Use high bits to avoid collision with manual IDs and auto IDs
        let hash = hasher.finish();
        // Ensure we're in a distinct range from auto() IDs
        ElementId(hash | 0x8000_0000_0000_0000)
    }

    /// Create an auto-generated element ID.
    ///
    /// **WARNING**: Auto-generated IDs are NOT stable across frames.
    /// This means click handlers may not work correctly in immediate-mode UIs.
    /// Prefer `ElementId::stable()` or `ElementId::new()` for interactive elements.
    #[deprecated(
        since = "0.0.1",
        note = "Auto IDs are not stable across frames. Use ElementId::stable(key) or ElementId::new(id) instead."
    )]
    pub fn auto() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1_000_000); // Start high to avoid conflicts
        ElementId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    #[allow(deprecated)]
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
    use crate::geometry::Rect;
    use crate::layer::InputEvent;

    fn create_test_system() -> InteractionSystem {
        InteractionSystem::new()
    }

    fn create_hit_entries(entries: &[(u64, Rect, i32)]) -> Vec<HitTestEntry> {
        entries
            .iter()
            .map(|(id, bounds, z)| HitTestEntry::new(ElementId::new(*id), *bounds, *z, 0))
            .collect()
    }

    #[test]
    fn test_interaction_system_creation() {
        let system = create_test_system();
        assert!(system.focused_element().is_none());
    }

    #[test]
    fn test_mouse_enter_leave() {
        let mut system = create_test_system();
        let button = Rect::new(10.0, 10.0, 100.0, 50.0);

        system.update_hit_test(create_hit_entries(&[(1, button, 0)]));

        // Move into button
        let events = system.handle_input(&InputEvent::MouseMove {
            position: Vec2::new(50.0, 30.0),
        });

        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::MouseEnter { element_id } if element_id.0 == 1)));

        // Verify state
        let state = system.get_state(ElementId::new(1)).unwrap();
        assert!(state.is_hovered);
        assert!(!state.is_pressed);

        // Move out of button
        let events = system.handle_input(&InputEvent::MouseMove {
            position: Vec2::new(200.0, 200.0),
        });

        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::MouseLeave { element_id } if element_id.0 == 1)));
    }

    #[test]
    fn test_mouse_click() {
        let mut system = create_test_system();
        let button = Rect::new(10.0, 10.0, 100.0, 50.0);

        system.update_hit_test(create_hit_entries(&[(1, button, 0)]));

        // Click inside button
        let down_events = system.handle_input(&InputEvent::MouseDown {
            position: Vec2::new(50.0, 30.0),
            button: MouseButton::Left,
        });

        assert!(
            down_events
                .iter()
                .any(|e| matches!(e, InteractionEvent::MouseDown { element_id, .. } if element_id.0 == 1))
        );

        // Check pressed state - use get_state with default
        if let Some(state) = system.get_state(ElementId::new(1)) {
            assert!(state.is_pressed);
        }

        // Release
        let up_events = system.handle_input(&InputEvent::MouseUp {
            position: Vec2::new(50.0, 30.0),
            button: MouseButton::Left,
        });

        // Should have both MouseUp and Click events
        assert!(
            up_events
                .iter()
                .any(|e| matches!(e, InteractionEvent::MouseUp { element_id, .. } if element_id.0 == 1))
        );
        assert!(
            up_events
                .iter()
                .any(|e| matches!(e, InteractionEvent::Click { element_id, .. } if element_id.0 == 1))
        );
    }

    #[test]
    fn test_no_click_when_released_outside() {
        let mut system = create_test_system();
        let button = Rect::new(10.0, 10.0, 100.0, 50.0);

        system.update_hit_test(create_hit_entries(&[(1, button, 0)]));

        // Press inside
        system.handle_input(&InputEvent::MouseDown {
            position: Vec2::new(50.0, 30.0),
            button: MouseButton::Left,
        });

        // Release outside
        let events = system.handle_input(&InputEvent::MouseUp {
            position: Vec2::new(200.0, 200.0),
            button: MouseButton::Left,
        });

        // Should have MouseUp but no Click
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InteractionEvent::MouseUp { element_id, .. } if element_id.0 == 1))
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InteractionEvent::Click { .. }))
        );
    }

    #[test]
    fn test_z_order_hit_testing() {
        let mut system = create_test_system();

        // Two overlapping elements - both cover the same click position
        let back = Rect::new(0.0, 0.0, 100.0, 100.0);
        let front = Rect::new(0.0, 0.0, 100.0, 100.0); // Same bounds for clear overlap

        // Front has higher z-index - entries need to be sorted by z-index (highest first)
        let mut entries = create_hit_entries(&[(1, back, 0), (2, front, 10)]);
        entries.sort_by(|a, b| b.z_index.cmp(&a.z_index));
        system.update_hit_test(entries);

        // Click in overlapping area
        let events = system.handle_input(&InputEvent::MouseDown {
            position: Vec2::new(50.0, 50.0),
            button: MouseButton::Left,
        });

        // Should hit front element (id 2) because it has higher z-index
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InteractionEvent::MouseDown { element_id, .. } if element_id.0 == 2)),
            "Expected MouseDown for element 2 (front), got events: {:?}",
            events
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InteractionEvent::MouseDown { element_id, .. } if element_id.0 == 1)),
            "Should NOT have MouseDown for element 1 (back)"
        );
    }

    #[test]
    fn test_focus_management() {
        let mut system = create_test_system();

        // Set focus to element 1
        let events = system.set_focus(Some(ElementId::new(1)));
        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::FocusIn { element_id } if element_id.0 == 1)));
        assert_eq!(system.focused_element(), Some(ElementId::new(1)));

        // Change focus to element 2
        let events = system.set_focus(Some(ElementId::new(2)));
        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::FocusOut { element_id } if element_id.0 == 1)));
        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::FocusIn { element_id } if element_id.0 == 2)));
        assert_eq!(system.focused_element(), Some(ElementId::new(2)));

        // Clear focus
        let events = system.set_focus(None);
        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::FocusOut { element_id } if element_id.0 == 2)));
        assert_eq!(system.focused_element(), None);
    }

    #[test]
    fn test_tab_focus_navigation() {
        let mut system = create_test_system();

        // Register focusable elements
        system.register_focusable(ElementId::new(1));
        system.register_focusable(ElementId::new(2));
        system.register_focusable(ElementId::new(3));

        // Tab forward
        let events = system.focus_next();
        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::FocusIn { element_id } if element_id.0 == 1)));

        system.focus_next();
        assert_eq!(system.focused_element(), Some(ElementId::new(2)));

        system.focus_next();
        assert_eq!(system.focused_element(), Some(ElementId::new(3)));

        // Wrap around
        system.focus_next();
        assert_eq!(system.focused_element(), Some(ElementId::new(1)));
    }

    #[test]
    fn test_shift_tab_focus_navigation() {
        let mut system = create_test_system();

        system.register_focusable(ElementId::new(1));
        system.register_focusable(ElementId::new(2));
        system.register_focusable(ElementId::new(3));

        // Start at element 2
        system.set_focus(Some(ElementId::new(2)));

        // Shift+Tab backward
        system.focus_previous();
        assert_eq!(system.focused_element(), Some(ElementId::new(1)));

        // Wrap around backward
        system.focus_previous();
        assert_eq!(system.focused_element(), Some(ElementId::new(3)));
    }

    #[test]
    fn test_keyboard_events_to_focused() {
        let mut system = create_test_system();

        // Set focus
        system.set_focus(Some(ElementId::new(1)));

        // Send key down
        let events = system.handle_input(&InputEvent::KeyDown {
            key: Key::A,
            modifiers: Modifiers::new(),
            character: Some('a'),
            is_repeat: false,
        });

        assert!(
            events
                .iter()
                .any(|e| matches!(e, InteractionEvent::KeyDown { element_id, key, .. }
                    if element_id.0 == 1 && *key == Key::A))
        );
    }

    #[test]
    fn test_scroll_wheel() {
        let mut system = create_test_system();
        let scrollable = Rect::new(0.0, 0.0, 200.0, 200.0);

        system.update_hit_test(create_hit_entries(&[(1, scrollable, 0)]));

        let events = system.handle_input(&InputEvent::ScrollWheel {
            position: Vec2::new(100.0, 100.0),
            delta: Vec2::new(0.0, -10.0),
        });

        assert!(
            events
                .iter()
                .any(|e| matches!(e, InteractionEvent::ScrollWheel { element_id, delta, .. }
                    if element_id.0 == 1 && delta.y == -10.0))
        );
    }

    #[test]
    fn test_mouse_leave_window() {
        let mut system = create_test_system();
        let button = Rect::new(10.0, 10.0, 100.0, 50.0);

        system.update_hit_test(create_hit_entries(&[(1, button, 0)]));

        // Move into button
        system.handle_input(&InputEvent::MouseMove {
            position: Vec2::new(50.0, 30.0),
        });

        // Mouse leaves window
        let events = system.handle_input(&InputEvent::MouseLeave);

        assert!(events
            .iter()
            .any(|e| matches!(e, InteractionEvent::MouseLeave { element_id } if element_id.0 == 1)));
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut system = create_test_system();
        let button = Rect::new(10.0, 10.0, 100.0, 50.0);

        system.update_hit_test(create_hit_entries(&[(1, button, 0)]));
        system.handle_input(&InputEvent::MouseMove {
            position: Vec2::new(50.0, 30.0),
        });
        system.set_focus(Some(ElementId::new(1)));
        system.register_focusable(ElementId::new(1));

        system.clear();

        assert!(system.focused_element().is_none());
        assert!(system.get_state(ElementId::new(1)).is_none());
    }

    #[test]
    fn test_element_id_equality() {
        let id1 = ElementId::new(42);
        let id2 = ElementId::new(42);
        let id3 = ElementId::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_element_id_from_conversions() {
        let from_u64: ElementId = 42u64.into();
        let from_usize: ElementId = 42usize.into();
        let from_i32: ElementId = 42i32.into();

        assert_eq!(from_u64, from_usize);
        assert_eq!(from_usize, from_i32);
    }
}
