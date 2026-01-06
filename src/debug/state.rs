//! Debug state management

use std::collections::HashSet;

/// Debug panels that can be enabled/disabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugPanel {
    /// Element bounds overlay
    Bounds,
    /// Layout inspector
    Layout,
    /// Hit test visualization
    HitTest,
    /// Performance metrics
    Metrics,
    /// Entity inspector
    Inspector,
    /// Debug console
    Console,
}

impl DebugPanel {
    /// Get the keyboard shortcut for this panel
    pub fn shortcut(&self) -> &'static str {
        match self {
            DebugPanel::Bounds => "F1",
            DebugPanel::Layout => "F2",
            DebugPanel::HitTest => "F3",
            DebugPanel::Metrics => "F4",
            DebugPanel::Inspector => "F5",
            DebugPanel::Console => "F6",
        }
    }

    /// Get the display name for this panel
    pub fn name(&self) -> &'static str {
        match self {
            DebugPanel::Bounds => "Bounds",
            DebugPanel::Layout => "Layout",
            DebugPanel::HitTest => "Hit Test",
            DebugPanel::Metrics => "Metrics",
            DebugPanel::Inspector => "Inspector",
            DebugPanel::Console => "Console",
        }
    }
}

/// Tracks the state of debug tools
pub struct DebugState {
    /// Whether debug mode is enabled at all
    enabled: bool,
    /// Which panels are currently active
    active_panels: HashSet<DebugPanel>,
    /// Selected element for inspection (if any)
    selected_element: Option<u64>,
    /// Mouse position for hover inspection
    mouse_position: Option<glam::Vec2>,
}

impl DebugState {
    pub fn new() -> Self {
        let mut active_panels = HashSet::new();
        // Default to showing metrics
        active_panels.insert(DebugPanel::Metrics);

        Self {
            enabled: false,
            active_panels,
            selected_element: None,
            mouse_position: None,
        }
    }

    /// Check if debug mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable debug mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debug mode
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle debug mode
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Check if a specific panel is enabled
    pub fn is_panel_enabled(&self, panel: DebugPanel) -> bool {
        self.active_panels.contains(&panel)
    }

    /// Enable a specific panel
    pub fn enable_panel(&mut self, panel: DebugPanel) {
        self.active_panels.insert(panel);
    }

    /// Disable a specific panel
    pub fn disable_panel(&mut self, panel: DebugPanel) {
        self.active_panels.remove(&panel);
    }

    /// Toggle a specific panel
    pub fn toggle_panel(&mut self, panel: DebugPanel) {
        if self.active_panels.contains(&panel) {
            self.active_panels.remove(&panel);
        } else {
            self.active_panels.insert(panel);
        }
    }

    /// Get all active panels
    pub fn active_panels(&self) -> impl Iterator<Item = &DebugPanel> {
        self.active_panels.iter()
    }

    /// Set the selected element for inspection
    pub fn select_element(&mut self, element_id: Option<u64>) {
        self.selected_element = element_id;
    }

    /// Get the selected element
    pub fn selected_element(&self) -> Option<u64> {
        self.selected_element
    }

    /// Update mouse position for hover inspection
    pub fn update_mouse_position(&mut self, position: glam::Vec2) {
        self.mouse_position = Some(position);
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> Option<glam::Vec2> {
        self.mouse_position
    }
}

impl Default for DebugState {
    fn default() -> Self {
        Self::new()
    }
}
