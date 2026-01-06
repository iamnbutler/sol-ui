//! Keyboard shortcuts system
//!
//! Provides global and contextual keyboard shortcuts with:
//! - Shortcut registration and matching
//! - Conflict detection
//! - Customizable keybindings
//! - Standard macOS shortcuts support

use crate::layer::{Key, Modifiers};
use std::collections::HashMap;

/// A keyboard shortcut defined by a key and modifier combination
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// The primary key
    pub key: Key,
    /// Required modifier keys
    pub modifiers: ShortcutModifiers,
}

/// Modifier requirements for a shortcut (more flexible than runtime Modifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShortcutModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub cmd: bool,
}

impl ShortcutModifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cmd() -> Self {
        Self { cmd: true, ..Default::default() }
    }

    pub fn cmd_shift() -> Self {
        Self { cmd: true, shift: true, ..Default::default() }
    }

    pub fn ctrl() -> Self {
        Self { ctrl: true, ..Default::default() }
    }

    pub fn alt() -> Self {
        Self { alt: true, ..Default::default() }
    }

    pub fn shift() -> Self {
        Self { shift: true, ..Default::default() }
    }

    /// Check if runtime modifiers match this shortcut's requirements
    pub fn matches(&self, modifiers: &Modifiers) -> bool {
        self.shift == modifiers.shift
            && self.ctrl == modifiers.ctrl
            && self.alt == modifiers.alt
            && self.cmd == modifiers.cmd
    }
}

impl From<Modifiers> for ShortcutModifiers {
    fn from(m: Modifiers) -> Self {
        Self {
            shift: m.shift,
            ctrl: m.ctrl,
            alt: m.alt,
            cmd: m.cmd,
        }
    }
}

impl Shortcut {
    /// Create a new shortcut
    pub fn new(key: Key, modifiers: ShortcutModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a shortcut with just the key (no modifiers)
    pub fn key(key: Key) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers::default(),
        }
    }

    /// Create a shortcut with Cmd modifier
    pub fn cmd(key: Key) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers::cmd(),
        }
    }

    /// Create a shortcut with Cmd+Shift modifiers
    pub fn cmd_shift(key: Key) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers::cmd_shift(),
        }
    }

    /// Create a shortcut with Ctrl modifier
    pub fn ctrl(key: Key) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers::ctrl(),
        }
    }

    /// Create a shortcut with Alt modifier
    pub fn alt(key: Key) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers::alt(),
        }
    }

    /// Check if this shortcut matches the given key and modifiers
    pub fn matches(&self, key: Key, modifiers: &Modifiers) -> bool {
        self.key == key && self.modifiers.matches(modifiers)
    }

    /// Get a human-readable string representation (e.g., "⌘C", "⇧⌘Z")
    pub fn display_string(&self) -> String {
        let mut s = String::new();
        if self.modifiers.ctrl {
            s.push('⌃');
        }
        if self.modifiers.alt {
            s.push('⌥');
        }
        if self.modifiers.shift {
            s.push('⇧');
        }
        if self.modifiers.cmd {
            s.push('⌘');
        }
        s.push_str(&key_display_string(&self.key));
        s
    }
}

/// Get display string for a key
fn key_display_string(key: &Key) -> String {
    match key {
        Key::A => "A".to_string(),
        Key::B => "B".to_string(),
        Key::C => "C".to_string(),
        Key::D => "D".to_string(),
        Key::E => "E".to_string(),
        Key::F => "F".to_string(),
        Key::G => "G".to_string(),
        Key::H => "H".to_string(),
        Key::I => "I".to_string(),
        Key::J => "J".to_string(),
        Key::K => "K".to_string(),
        Key::L => "L".to_string(),
        Key::M => "M".to_string(),
        Key::N => "N".to_string(),
        Key::O => "O".to_string(),
        Key::P => "P".to_string(),
        Key::Q => "Q".to_string(),
        Key::R => "R".to_string(),
        Key::S => "S".to_string(),
        Key::T => "T".to_string(),
        Key::U => "U".to_string(),
        Key::V => "V".to_string(),
        Key::W => "W".to_string(),
        Key::X => "X".to_string(),
        Key::Y => "Y".to_string(),
        Key::Z => "Z".to_string(),
        Key::Key0 => "0".to_string(),
        Key::Key1 => "1".to_string(),
        Key::Key2 => "2".to_string(),
        Key::Key3 => "3".to_string(),
        Key::Key4 => "4".to_string(),
        Key::Key5 => "5".to_string(),
        Key::Key6 => "6".to_string(),
        Key::Key7 => "7".to_string(),
        Key::Key8 => "8".to_string(),
        Key::Key9 => "9".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::Return => "↩".to_string(),
        Key::Tab => "⇥".to_string(),
        Key::Space => "Space".to_string(),
        Key::Backspace => "⌫".to_string(),
        Key::Delete => "⌦".to_string(),
        Key::Escape => "⎋".to_string(),
        Key::Home => "↖".to_string(),
        Key::End => "↘".to_string(),
        Key::PageUp => "⇞".to_string(),
        Key::PageDown => "⇟".to_string(),
        Key::Minus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::Backslash => "\\".to_string(),
        Key::Semicolon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::Grave => "`".to_string(),
        Key::Comma => ",".to_string(),
        Key::Period => ".".to_string(),
        Key::Slash => "/".to_string(),
        _ => format!("{:?}", key),
    }
}

/// Unique identifier for a shortcut action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortcutId(pub u64);

impl ShortcutId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Scope for when a shortcut is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// Shortcut is always active (e.g., Cmd+Q)
    Global,
    /// Shortcut is only active when a specific element is focused
    Focused(super::ElementId),
    /// Shortcut is only active within a specific context (identified by string)
    Context(u64),
}

/// Information about a registered shortcut
#[derive(Debug, Clone)]
pub struct ShortcutInfo {
    /// Unique identifier
    pub id: ShortcutId,
    /// The shortcut key combination
    pub shortcut: Shortcut,
    /// Human-readable action name (e.g., "Copy", "Undo")
    pub action_name: String,
    /// Description for tooltips/help
    pub description: Option<String>,
    /// When this shortcut is active
    pub scope: ShortcutScope,
    /// Priority (higher wins in conflicts)
    pub priority: i32,
    /// Whether this shortcut is currently enabled
    pub enabled: bool,
}

/// Result of a shortcut match
#[derive(Debug, Clone)]
pub struct ShortcutMatch {
    pub id: ShortcutId,
    pub action_name: String,
}

/// A conflict between two shortcuts
#[derive(Debug, Clone)]
pub struct ShortcutConflict {
    pub shortcut: Shortcut,
    pub conflicting_ids: Vec<ShortcutId>,
    pub action_names: Vec<String>,
}

/// Registry for managing keyboard shortcuts
pub struct ShortcutRegistry {
    /// All registered shortcuts
    shortcuts: HashMap<ShortcutId, ShortcutInfo>,
    /// Next ID to assign
    next_id: u64,
    /// Current active context (for context-scoped shortcuts)
    active_context: Option<u64>,
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
            next_id: 1,
            active_context: None,
        }
    }

    /// Register a new shortcut, returns its ID
    pub fn register(
        &mut self,
        shortcut: Shortcut,
        action_name: impl Into<String>,
        scope: ShortcutScope,
    ) -> ShortcutId {
        let id = ShortcutId(self.next_id);
        self.next_id += 1;

        let info = ShortcutInfo {
            id,
            shortcut,
            action_name: action_name.into(),
            description: None,
            scope,
            priority: 0,
            enabled: true,
        };

        self.shortcuts.insert(id, info);
        id
    }

    /// Register a shortcut with full configuration
    pub fn register_full(&mut self, mut info: ShortcutInfo) -> ShortcutId {
        let id = ShortcutId(self.next_id);
        self.next_id += 1;
        info.id = id;
        self.shortcuts.insert(id, info);
        id
    }

    /// Unregister a shortcut
    pub fn unregister(&mut self, id: ShortcutId) {
        self.shortcuts.remove(&id);
    }

    /// Enable or disable a shortcut
    pub fn set_enabled(&mut self, id: ShortcutId, enabled: bool) {
        if let Some(info) = self.shortcuts.get_mut(&id) {
            info.enabled = enabled;
        }
    }

    /// Set the active context for context-scoped shortcuts
    pub fn set_active_context(&mut self, context: Option<u64>) {
        self.active_context = context;
    }

    /// Find matching shortcuts for a key event
    pub fn find_matches(
        &self,
        key: Key,
        modifiers: &Modifiers,
        focused_element: Option<super::ElementId>,
    ) -> Vec<ShortcutMatch> {
        let mut matches: Vec<(i32, ShortcutMatch)> = Vec::new();

        for info in self.shortcuts.values() {
            if !info.enabled {
                continue;
            }

            if !info.shortcut.matches(key, modifiers) {
                continue;
            }

            // Check scope
            let scope_matches = match info.scope {
                ShortcutScope::Global => true,
                ShortcutScope::Focused(element_id) => focused_element == Some(element_id),
                ShortcutScope::Context(ctx) => self.active_context == Some(ctx),
            };

            if scope_matches {
                matches.push((
                    info.priority,
                    ShortcutMatch {
                        id: info.id,
                        action_name: info.action_name.clone(),
                    },
                ));
            }
        }

        // Sort by priority (highest first) and return
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches.into_iter().map(|(_, m)| m).collect()
    }

    /// Get the highest-priority match (if any)
    pub fn find_match(
        &self,
        key: Key,
        modifiers: &Modifiers,
        focused_element: Option<super::ElementId>,
    ) -> Option<ShortcutMatch> {
        self.find_matches(key, modifiers, focused_element).into_iter().next()
    }

    /// Get shortcut info by ID
    pub fn get(&self, id: ShortcutId) -> Option<&ShortcutInfo> {
        self.shortcuts.get(&id)
    }

    /// Get mutable shortcut info by ID
    pub fn get_mut(&mut self, id: ShortcutId) -> Option<&mut ShortcutInfo> {
        self.shortcuts.get_mut(&id)
    }

    /// Get all registered shortcuts
    pub fn all(&self) -> impl Iterator<Item = &ShortcutInfo> {
        self.shortcuts.values()
    }

    /// Get display string for a shortcut by its action name
    pub fn get_shortcut_hint(&self, action_name: &str) -> Option<String> {
        self.shortcuts
            .values()
            .find(|info| info.action_name == action_name && info.enabled)
            .map(|info| info.shortcut.display_string())
    }

    /// Detect conflicts between shortcuts
    pub fn detect_conflicts(&self) -> Vec<ShortcutConflict> {
        let mut conflicts: HashMap<Shortcut, Vec<(ShortcutId, String)>> = HashMap::new();

        for info in self.shortcuts.values() {
            if !info.enabled {
                continue;
            }

            // Only check global shortcuts for conflicts (scoped shortcuts can overlap)
            if !matches!(info.scope, ShortcutScope::Global) {
                continue;
            }

            conflicts
                .entry(info.shortcut.clone())
                .or_default()
                .push((info.id, info.action_name.clone()));
        }

        conflicts
            .into_iter()
            .filter(|(_, ids)| ids.len() > 1)
            .map(|(shortcut, ids)| ShortcutConflict {
                shortcut,
                conflicting_ids: ids.iter().map(|(id, _)| *id).collect(),
                action_names: ids.into_iter().map(|(_, name)| name).collect(),
            })
            .collect()
    }

    /// Update a shortcut's key combination (for customization)
    pub fn rebind(&mut self, id: ShortcutId, new_shortcut: Shortcut) -> bool {
        if let Some(info) = self.shortcuts.get_mut(&id) {
            info.shortcut = new_shortcut;
            true
        } else {
            false
        }
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard macOS shortcuts
pub mod standard {
    use super::*;

    /// Standard action identifiers
    pub mod actions {
        pub const QUIT: &str = "quit";
        pub const CLOSE_WINDOW: &str = "close_window";
        pub const MINIMIZE: &str = "minimize";
        pub const COPY: &str = "copy";
        pub const CUT: &str = "cut";
        pub const PASTE: &str = "paste";
        pub const UNDO: &str = "undo";
        pub const REDO: &str = "redo";
        pub const SELECT_ALL: &str = "select_all";
        pub const FIND: &str = "find";
        pub const FIND_NEXT: &str = "find_next";
        pub const FIND_PREVIOUS: &str = "find_previous";
        pub const SAVE: &str = "save";
        pub const SAVE_AS: &str = "save_as";
        pub const OPEN: &str = "open";
        pub const NEW: &str = "new";
        pub const PRINT: &str = "print";
        pub const PREFERENCES: &str = "preferences";
        pub const ZOOM_IN: &str = "zoom_in";
        pub const ZOOM_OUT: &str = "zoom_out";
        pub const ZOOM_RESET: &str = "zoom_reset";
    }

    /// Register standard macOS shortcuts
    pub fn register_standard_shortcuts(registry: &mut ShortcutRegistry) {
        use actions::*;

        // Application shortcuts
        registry.register(Shortcut::cmd(Key::Q), QUIT, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::W), CLOSE_WINDOW, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::M), MINIMIZE, ShortcutScope::Global);

        // Edit shortcuts
        registry.register(Shortcut::cmd(Key::C), COPY, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::X), CUT, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::V), PASTE, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::Z), UNDO, ShortcutScope::Global);
        registry.register(Shortcut::cmd_shift(Key::Z), REDO, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::A), SELECT_ALL, ShortcutScope::Global);

        // Find shortcuts
        registry.register(Shortcut::cmd(Key::F), FIND, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::G), FIND_NEXT, ShortcutScope::Global);
        registry.register(Shortcut::cmd_shift(Key::G), FIND_PREVIOUS, ShortcutScope::Global);

        // File shortcuts
        registry.register(Shortcut::cmd(Key::S), SAVE, ShortcutScope::Global);
        registry.register(Shortcut::cmd_shift(Key::S), SAVE_AS, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::O), OPEN, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::N), NEW, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::P), PRINT, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::Comma), PREFERENCES, ShortcutScope::Global);

        // View shortcuts
        registry.register(Shortcut::cmd(Key::Equal), ZOOM_IN, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::Minus), ZOOM_OUT, ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::Key0), ZOOM_RESET, ShortcutScope::Global);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_matching() {
        let shortcut = Shortcut::cmd(Key::C);

        let modifiers = Modifiers { cmd: true, ..Default::default() };
        assert!(shortcut.matches(Key::C, &modifiers));

        let no_mods = Modifiers::default();
        assert!(!shortcut.matches(Key::C, &no_mods));

        let wrong_key = Modifiers { cmd: true, ..Default::default() };
        assert!(!shortcut.matches(Key::V, &wrong_key));
    }

    #[test]
    fn test_shortcut_display() {
        assert_eq!(Shortcut::cmd(Key::C).display_string(), "⌘C");
        assert_eq!(Shortcut::cmd_shift(Key::Z).display_string(), "⇧⌘Z");
        assert_eq!(Shortcut::key(Key::Escape).display_string(), "⎋");
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = ShortcutRegistry::new();

        let id = registry.register(Shortcut::cmd(Key::C), "copy", ShortcutScope::Global);

        let modifiers = Modifiers { cmd: true, ..Default::default() };
        let result = registry.find_match(Key::C, &modifiers, None);

        assert!(result.is_some());
        assert_eq!(result.unwrap().action_name, "copy");
    }

    #[test]
    fn test_conflict_detection() {
        let mut registry = ShortcutRegistry::new();

        registry.register(Shortcut::cmd(Key::C), "copy", ShortcutScope::Global);
        registry.register(Shortcut::cmd(Key::C), "duplicate", ShortcutScope::Global);

        let conflicts = registry.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].action_names.len(), 2);
    }
}
