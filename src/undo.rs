//! Undo/Redo system using the Command pattern
//!
//! This module provides a general-purpose undo/redo system for user actions.
//!
//! ## Key Concepts
//!
//! - **Command**: A trait representing an undoable action with `execute()` and `undo()` methods
//! - **UndoManager**: Maintains undo/redo stacks and handles action grouping
//! - **Action Groups**: Multiple commands can be grouped into a single undo unit
//!
//! ## Usage
//!
//! ```ignore
//! use sol_ui::undo::{Command, UndoManager};
//!
//! // Define an undoable command
//! struct SetValueCommand {
//!     target: Rc<RefCell<i32>>,
//!     old_value: i32,
//!     new_value: i32,
//! }
//!
//! impl Command for SetValueCommand {
//!     fn execute(&mut self) {
//!         *self.target.borrow_mut() = self.new_value;
//!     }
//!     fn undo(&mut self) {
//!         *self.target.borrow_mut() = self.old_value;
//!     }
//!     fn description(&self) -> &str {
//!         "Set Value"
//!     }
//! }
//!
//! // Use with UndoManager
//! let mut manager = UndoManager::new();
//! manager.execute(Box::new(command));
//! manager.undo(); // Reverts the change
//! manager.redo(); // Re-applies the change
//! ```
//!
//! ## Keyboard Shortcuts
//!
//! The system is designed to work with Cmd+Z (undo) and Cmd+Shift+Z (redo).
//! Use the `handle_key_event()` method to integrate with keyboard input.

/// Trait for undoable commands
///
/// Each command must be able to execute itself and undo its effects.
/// Commands should capture all state needed to undo/redo the action.
pub trait Command: Send {
    /// Execute (or re-execute) the command
    fn execute(&mut self);

    /// Undo the command's effects
    fn undo(&mut self);

    /// Human-readable description of the command (for UI display)
    fn description(&self) -> &str {
        "Unknown Action"
    }

    /// Whether this command can be merged with a previous command
    ///
    /// Override to return true for commands that should merge (e.g., consecutive typing).
    /// The `merge()` method will be called if this returns true.
    fn can_merge(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge with a previous command
    ///
    /// This is called when `can_merge()` returns true.
    /// The implementation should absorb the other command's effects.
    /// Returns true if merge was successful.
    fn merge(&mut self, _other: Box<dyn Command>) -> bool {
        false
    }
}

/// A group of commands that are undone/redone together
struct CommandGroup {
    commands: Vec<Box<dyn Command>>,
    description: String,
}

impl CommandGroup {
    fn new(description: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            description: description.into(),
        }
    }

    fn add(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    fn execute(&mut self) {
        for cmd in &mut self.commands {
            cmd.execute();
        }
    }

    fn undo(&mut self) {
        // Undo in reverse order
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo();
        }
    }

    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// An entry in the undo/redo stack
enum UndoEntry {
    Single(Box<dyn Command>),
    Group(CommandGroup),
}

impl UndoEntry {
    fn execute(&mut self) {
        match self {
            UndoEntry::Single(cmd) => cmd.execute(),
            UndoEntry::Group(group) => group.execute(),
        }
    }

    fn undo(&mut self) {
        match self {
            UndoEntry::Single(cmd) => cmd.undo(),
            UndoEntry::Group(group) => group.undo(),
        }
    }

    fn description(&self) -> &str {
        match self {
            UndoEntry::Single(cmd) => cmd.description(),
            UndoEntry::Group(group) => group.description(),
        }
    }
}

/// Configuration for the UndoManager
#[derive(Debug, Clone)]
pub struct UndoConfig {
    /// Maximum number of undo levels (0 = unlimited)
    pub max_undo_levels: usize,
}

impl Default for UndoConfig {
    fn default() -> Self {
        Self {
            max_undo_levels: 100,
        }
    }
}

/// Manages undo/redo stacks for user actions
///
/// The UndoManager maintains two stacks:
/// - Undo stack: actions that can be undone
/// - Redo stack: actions that were undone and can be redone
///
/// When a new action is executed, the redo stack is cleared.
pub struct UndoManager {
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    config: UndoConfig,
    /// Currently open group (for batching multiple commands)
    current_group: Option<CommandGroup>,
    /// Nesting level for group operations
    group_depth: usize,
}

impl UndoManager {
    /// Create a new UndoManager with default configuration
    pub fn new() -> Self {
        Self::with_config(UndoConfig::default())
    }

    /// Create a new UndoManager with custom configuration
    pub fn with_config(config: UndoConfig) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            config,
            current_group: None,
            group_depth: 0,
        }
    }

    /// Execute a command and add it to the undo stack
    ///
    /// This clears the redo stack.
    pub fn execute(&mut self, mut command: Box<dyn Command>) {
        // Execute the command
        command.execute();

        // If we're in a group, add to the group
        if let Some(group) = &mut self.current_group {
            group.add(command);
            return;
        }

        // Check if we can merge with the last command
        let can_merge = if let Some(UndoEntry::Single(last_cmd)) = self.undo_stack.last() {
            last_cmd.can_merge(command.as_ref())
        } else {
            false
        };

        if can_merge {
            // Try to merge - this consumes command
            if let Some(UndoEntry::Single(last_cmd)) = self.undo_stack.last_mut() {
                if last_cmd.merge(command) {
                    // Merge successful
                    self.redo_stack.clear();
                    return;
                }
                // Merge returned false but consumed the command - this shouldn't happen
                // with a properly implemented merge, but we can't recover the command
                self.redo_stack.clear();
                return;
            }
        }

        // No merge - add as new entry
        self.undo_stack.push(UndoEntry::Single(command));
        self.redo_stack.clear();

        // Enforce max undo levels
        self.enforce_limit();
    }

    /// Execute a command without adding to the undo stack
    ///
    /// Useful for commands that shouldn't be undoable.
    pub fn execute_without_undo(&mut self, mut command: Box<dyn Command>) {
        command.execute();
    }

    /// Undo the last action
    ///
    /// Returns true if there was an action to undo.
    pub fn undo(&mut self) -> bool {
        // Close any open group first
        if self.current_group.is_some() {
            self.end_group();
        }

        if let Some(mut entry) = self.undo_stack.pop() {
            entry.undo();
            self.redo_stack.push(entry);
            true
        } else {
            false
        }
    }

    /// Redo the last undone action
    ///
    /// Returns true if there was an action to redo.
    pub fn redo(&mut self) -> bool {
        // Close any open group first
        if self.current_group.is_some() {
            self.end_group();
        }

        if let Some(mut entry) = self.redo_stack.pop() {
            entry.execute();
            self.undo_stack.push(entry);
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || self.current_group.as_ref().map(|g| !g.is_empty()).unwrap_or(false)
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of the next undo action
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|e| e.description())
    }

    /// Get description of the next redo action
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|e| e.description())
    }

    /// Begin a group of commands that will be undone together
    ///
    /// Groups can be nested - only the outermost group is tracked.
    pub fn begin_group(&mut self, description: impl Into<String>) {
        if self.group_depth == 0 {
            self.current_group = Some(CommandGroup::new(description));
        }
        self.group_depth += 1;
    }

    /// End the current command group
    ///
    /// The grouped commands will appear as a single undo entry.
    pub fn end_group(&mut self) {
        if self.group_depth > 0 {
            self.group_depth -= 1;
        }

        if self.group_depth == 0 {
            if let Some(group) = self.current_group.take() {
                if !group.is_empty() {
                    self.undo_stack.push(UndoEntry::Group(group));
                    self.redo_stack.clear();
                    self.enforce_limit();
                }
            }
        }
    }

    /// Execute commands within a group
    ///
    /// Convenience method that wraps commands in begin_group/end_group.
    pub fn execute_grouped<F>(&mut self, description: impl Into<String>, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.begin_group(description);
        f(self);
        self.end_group();
    }

    /// Clear all undo/redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
        self.group_depth = 0;
    }

    /// Get the number of undo levels available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo levels available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Handle a keyboard event for undo/redo shortcuts
    ///
    /// Returns true if the event was handled (Cmd+Z or Cmd+Shift+Z).
    pub fn handle_key_event(
        &mut self,
        key: crate::layer::Key,
        modifiers: crate::layer::Modifiers,
    ) -> bool {
        use crate::layer::Key;

        // Check for Cmd+Z (undo) or Cmd+Shift+Z (redo)
        if key == Key::Z && modifiers.cmd && !modifiers.ctrl && !modifiers.alt {
            if modifiers.shift {
                // Cmd+Shift+Z = Redo
                return self.redo();
            } else {
                // Cmd+Z = Undo
                return self.undo();
            }
        }

        false
    }

    /// Enforce the maximum undo level limit
    fn enforce_limit(&mut self) {
        if self.config.max_undo_levels > 0 {
            while self.undo_stack.len() > self.config.max_undo_levels {
                self.undo_stack.remove(0);
            }
        }
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

// Send is safe because UndoEntry contains Send commands
unsafe impl Send for UndoManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Test command that sets a value
    struct SetValueCommand {
        target: Rc<RefCell<i32>>,
        old_value: i32,
        new_value: i32,
    }

    impl SetValueCommand {
        fn new(target: Rc<RefCell<i32>>, new_value: i32) -> Self {
            let old_value = *target.borrow();
            Self {
                target,
                old_value,
                new_value,
            }
        }
    }

    // Implement Send for test command (Rc is not Send, but this is test-only)
    unsafe impl Send for SetValueCommand {}

    impl Command for SetValueCommand {
        fn execute(&mut self) {
            *self.target.borrow_mut() = self.new_value;
        }

        fn undo(&mut self) {
            *self.target.borrow_mut() = self.old_value;
        }

        fn description(&self) -> &str {
            "Set Value"
        }
    }

    #[test]
    fn test_basic_undo_redo() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        // Execute command
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 10)));
        assert_eq!(*value.borrow(), 10);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Undo
        assert!(manager.undo());
        assert_eq!(*value.borrow(), 0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo
        assert!(manager.redo());
        assert_eq!(*value.borrow(), 10);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_multiple_commands() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.execute(Box::new(SetValueCommand::new(value.clone(), 1)));
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 2)));
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 3)));

        assert_eq!(*value.borrow(), 3);
        assert_eq!(manager.undo_count(), 3);

        manager.undo();
        assert_eq!(*value.borrow(), 2);

        manager.undo();
        assert_eq!(*value.borrow(), 1);

        manager.redo();
        assert_eq!(*value.borrow(), 2);
    }

    #[test]
    fn test_new_command_clears_redo() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.execute(Box::new(SetValueCommand::new(value.clone(), 1)));
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 2)));
        manager.undo(); // Undo to 1

        assert!(manager.can_redo());

        // New command should clear redo stack
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 5)));
        assert!(!manager.can_redo());
        assert_eq!(*value.borrow(), 5);
    }

    #[test]
    fn test_command_groups() {
        let mut manager = UndoManager::new();
        let value1 = Rc::new(RefCell::new(0));
        let value2 = Rc::new(RefCell::new(0));

        // Execute multiple commands as a group
        manager.begin_group("Set Both Values");
        manager.execute(Box::new(SetValueCommand::new(value1.clone(), 10)));
        manager.execute(Box::new(SetValueCommand::new(value2.clone(), 20)));
        manager.end_group();

        assert_eq!(*value1.borrow(), 10);
        assert_eq!(*value2.borrow(), 20);
        assert_eq!(manager.undo_count(), 1); // Only one undo entry

        // Undo should undo both commands
        manager.undo();
        assert_eq!(*value1.borrow(), 0);
        assert_eq!(*value2.borrow(), 0);

        // Redo should redo both commands
        manager.redo();
        assert_eq!(*value1.borrow(), 10);
        assert_eq!(*value2.borrow(), 20);
    }

    #[test]
    fn test_execute_grouped() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        let value_clone = value.clone();
        manager.execute_grouped("Multiple Changes", |mgr| {
            mgr.execute(Box::new(SetValueCommand::new(value_clone.clone(), 1)));
            mgr.execute(Box::new(SetValueCommand::new(value_clone.clone(), 2)));
            mgr.execute(Box::new(SetValueCommand::new(value_clone.clone(), 3)));
        });

        assert_eq!(*value.borrow(), 3);
        assert_eq!(manager.undo_count(), 1);

        manager.undo();
        assert_eq!(*value.borrow(), 0);
    }

    #[test]
    fn test_max_undo_levels() {
        let config = UndoConfig {
            max_undo_levels: 3,
        };
        let mut manager = UndoManager::with_config(config);
        let value = Rc::new(RefCell::new(0));

        for i in 1..=10 {
            manager.execute(Box::new(SetValueCommand::new(value.clone(), i)));
        }

        // Should only have 3 undo levels
        assert_eq!(manager.undo_count(), 3);

        // Undo should go back to 7 (10 - 3 commands)
        manager.undo();
        assert_eq!(*value.borrow(), 9);
        manager.undo();
        assert_eq!(*value.borrow(), 8);
        manager.undo();
        assert_eq!(*value.borrow(), 7);

        // No more undos available
        assert!(!manager.can_undo());
    }

    #[test]
    fn test_clear() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.execute(Box::new(SetValueCommand::new(value.clone(), 1)));
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 2)));
        manager.undo();

        assert!(manager.can_undo());
        assert!(manager.can_redo());

        manager.clear();

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_descriptions() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.execute(Box::new(SetValueCommand::new(value.clone(), 1)));

        assert_eq!(manager.undo_description(), Some("Set Value"));
        assert_eq!(manager.redo_description(), None);

        manager.undo();

        assert_eq!(manager.undo_description(), None);
        assert_eq!(manager.redo_description(), Some("Set Value"));
    }

    #[test]
    fn test_nested_groups() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.begin_group("Outer Group");
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 1)));

        manager.begin_group("Inner Group (ignored)");
        manager.execute(Box::new(SetValueCommand::new(value.clone(), 2)));
        manager.end_group();

        manager.execute(Box::new(SetValueCommand::new(value.clone(), 3)));
        manager.end_group();

        assert_eq!(*value.borrow(), 3);
        assert_eq!(manager.undo_count(), 1); // All in one group

        manager.undo();
        assert_eq!(*value.borrow(), 0);
    }

    #[test]
    fn test_empty_group() {
        let mut manager = UndoManager::new();

        manager.begin_group("Empty");
        manager.end_group();

        // Empty group should not create an undo entry
        assert_eq!(manager.undo_count(), 0);
    }
}
