//! Undo/redo system using the Command pattern
//!
//! Provides undo/redo support for user actions with:
//! - Command trait for undoable actions
//! - Undo/redo stacks with configurable depth
//! - Action grouping for compound operations
//! - Clear history API

use std::collections::VecDeque;

/// A command that can be executed, undone, and redone
pub trait Command {
    /// Execute the command (or redo it)
    fn execute(&mut self);

    /// Undo the command
    fn undo(&mut self);

    /// Human-readable description for UI display
    fn description(&self) -> &str;

    /// Whether this command can be merged with the previous one
    /// (e.g., consecutive character insertions can become a single undo)
    fn can_merge_with(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge another command into this one (called when can_merge_with returns true)
    fn merge(&mut self, _other: Box<dyn Command>) {
        // Default: no-op
    }
}

/// A group of commands that are undone/redone together
pub struct CommandGroup {
    commands: Vec<Box<dyn Command>>,
    description: String,
}

impl CommandGroup {
    /// Create a new command group
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            description: description.into(),
        }
    }

    /// Add a command to the group
    pub fn add(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Check if the group is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Command for CommandGroup {
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

    fn description(&self) -> &str {
        &self.description
    }
}

/// Manages undo and redo stacks
pub struct UndoManager {
    /// Stack of commands that can be undone
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Stack of commands that can be redone
    redo_stack: VecDeque<Box<dyn Command>>,
    /// Maximum number of undo levels
    max_undo_levels: usize,
    /// Currently recording command group (for grouping multiple commands)
    current_group: Option<CommandGroup>,
    /// Whether changes have been made since last save
    is_dirty: bool,
    /// Clean state index in undo stack (for tracking save state)
    clean_index: Option<usize>,
}

impl UndoManager {
    /// Create a new undo manager with default settings
    pub fn new() -> Self {
        Self::with_max_undo_levels(100)
    }

    /// Create a new undo manager with specified max undo levels
    pub fn with_max_undo_levels(max_levels: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_levels: max_levels,
            current_group: None,
            is_dirty: false,
            clean_index: Some(0),
        }
    }

    /// Execute a command and add it to the undo stack
    pub fn execute(&mut self, mut command: Box<dyn Command>) {
        command.execute();

        // If we're recording a group, add to the group instead
        if let Some(ref mut group) = self.current_group {
            group.add(command);
            return;
        }

        // Clear redo stack when new command is executed
        self.redo_stack.clear();

        // Try to merge with previous command
        if let Some(last) = self.undo_stack.back_mut() {
            if last.can_merge_with(command.as_ref()) {
                last.merge(command);
                self.is_dirty = true;
                self.update_clean_index_after_change();
                return;
            }
        }

        // Add to undo stack
        self.undo_stack.push_back(command);

        // Trim stack if it exceeds max levels
        while self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.pop_front();
            // Adjust clean index
            if let Some(idx) = self.clean_index {
                if idx > 0 {
                    self.clean_index = Some(idx - 1);
                } else {
                    self.clean_index = None; // Clean state was trimmed
                }
            }
        }

        self.is_dirty = true;
        self.update_clean_index_after_change();
    }

    /// Undo the last command
    pub fn undo(&mut self) -> bool {
        // If recording a group, finish it first
        self.end_group();

        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo();
            self.redo_stack.push_back(command);
            self.update_dirty_state();
            true
        } else {
            false
        }
    }

    /// Redo the last undone command
    pub fn redo(&mut self) -> bool {
        // If recording a group, finish it first
        self.end_group();

        if let Some(mut command) = self.redo_stack.pop_back() {
            command.execute();
            self.undo_stack.push_back(command);
            self.update_dirty_state();
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || self.current_group.as_ref().map_or(false, |g| !g.is_empty())
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.redo_stack.is_empty() == false
    }

    /// Get description of the command that would be undone
    pub fn undo_description(&self) -> Option<&str> {
        if let Some(ref group) = self.current_group {
            if !group.is_empty() {
                return Some(&group.description);
            }
        }
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Get description of the command that would be redone
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Start recording a group of commands
    pub fn begin_group(&mut self, description: impl Into<String>) {
        self.end_group(); // End any existing group
        self.current_group = Some(CommandGroup::new(description));
    }

    /// End the current command group and add it to the undo stack
    pub fn end_group(&mut self) {
        if let Some(group) = self.current_group.take() {
            if !group.is_empty() {
                // Clear redo stack
                self.redo_stack.clear();

                // Add group to undo stack
                self.undo_stack.push_back(Box::new(group));

                // Trim if needed
                while self.undo_stack.len() > self.max_undo_levels {
                    self.undo_stack.pop_front();
                }

                self.is_dirty = true;
                self.update_clean_index_after_change();
            }
        }
    }

    /// Clear all undo/redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
        self.clean_index = Some(0);
        self.is_dirty = false;
    }

    /// Mark the current state as "clean" (e.g., after saving)
    pub fn mark_clean(&mut self) {
        self.end_group();
        self.clean_index = Some(self.undo_stack.len());
        self.is_dirty = false;
    }

    /// Check if the document has been modified since last save
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Get the number of undo levels available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo levels available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Update dirty state based on clean index
    fn update_dirty_state(&mut self) {
        self.is_dirty = self.clean_index != Some(self.undo_stack.len());
    }

    /// Update clean index after a change
    fn update_clean_index_after_change(&mut self) {
        // If clean index was pointing beyond current position, it's now invalid
        if let Some(idx) = self.clean_index {
            if idx > self.undo_stack.len() {
                self.clean_index = None;
            }
        }
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple command that stores closures for execute and undo
pub struct ClosureCommand<E, U>
where
    E: FnMut(),
    U: FnMut(),
{
    execute_fn: E,
    undo_fn: U,
    description: String,
}

impl<E, U> ClosureCommand<E, U>
where
    E: FnMut(),
    U: FnMut(),
{
    /// Create a new closure-based command
    pub fn new(description: impl Into<String>, execute_fn: E, undo_fn: U) -> Self {
        Self {
            execute_fn,
            undo_fn,
            description: description.into(),
        }
    }
}

impl<E, U> Command for ClosureCommand<E, U>
where
    E: FnMut(),
    U: FnMut(),
{
    fn execute(&mut self) {
        (self.execute_fn)();
    }

    fn undo(&mut self) {
        (self.undo_fn)();
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Helper to create a closure-based command
pub fn command<E, U>(description: impl Into<String>, execute_fn: E, undo_fn: U) -> Box<dyn Command>
where
    E: FnMut() + 'static,
    U: FnMut() + 'static,
{
    Box::new(ClosureCommand::new(description, execute_fn, undo_fn))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_basic_undo_redo() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        // Execute command that sets value to 1
        let v = value.clone();
        manager.execute(command(
            "Set to 1",
            move || *v.borrow_mut() = 1,
            {
                let v = value.clone();
                move || *v.borrow_mut() = 0
            },
        ));

        assert_eq!(*value.borrow(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Undo
        manager.undo();
        assert_eq!(*value.borrow(), 0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo
        manager.redo();
        assert_eq!(*value.borrow(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_redo_cleared_on_new_command() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        // Execute and undo
        let v = value.clone();
        manager.execute(command(
            "Set to 1",
            move || *v.borrow_mut() = 1,
            {
                let v = value.clone();
                move || *v.borrow_mut() = 0
            },
        ));
        manager.undo();

        assert!(manager.can_redo());

        // Execute new command
        let v = value.clone();
        manager.execute(command(
            "Set to 2",
            move || *v.borrow_mut() = 2,
            {
                let v = value.clone();
                move || *v.borrow_mut() = 0
            },
        ));

        // Redo should be cleared
        assert!(!manager.can_redo());
        assert_eq!(*value.borrow(), 2);
    }

    #[test]
    fn test_command_group() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        manager.begin_group("Multiple changes");

        // Execute multiple commands
        let v = value.clone();
        manager.execute(command(
            "Add 1",
            move || *v.borrow_mut() += 1,
            {
                let v = value.clone();
                move || *v.borrow_mut() -= 1
            },
        ));

        let v = value.clone();
        manager.execute(command(
            "Add 2",
            move || *v.borrow_mut() += 2,
            {
                let v = value.clone();
                move || *v.borrow_mut() -= 2
            },
        ));

        manager.end_group();

        assert_eq!(*value.borrow(), 3);
        assert_eq!(manager.undo_count(), 1); // Group counts as one

        // Single undo should revert both
        manager.undo();
        assert_eq!(*value.borrow(), 0);
    }

    #[test]
    fn test_dirty_state() {
        let mut manager = UndoManager::new();
        let value = Rc::new(RefCell::new(0));

        assert!(!manager.is_dirty());

        let v = value.clone();
        manager.execute(command(
            "Change",
            move || *v.borrow_mut() = 1,
            {
                let v = value.clone();
                move || *v.borrow_mut() = 0
            },
        ));

        assert!(manager.is_dirty());

        manager.mark_clean();
        assert!(!manager.is_dirty());

        manager.undo();
        assert!(manager.is_dirty()); // Undoing past clean state makes it dirty

        manager.redo();
        assert!(!manager.is_dirty()); // Redoing back to clean state
    }

    #[test]
    fn test_max_undo_levels() {
        let mut manager = UndoManager::with_max_undo_levels(3);
        let value = Rc::new(RefCell::new(0));

        for i in 1..=5 {
            let v = value.clone();
            let prev = i - 1;
            manager.execute(command(
                format!("Set to {}", i),
                move || *v.borrow_mut() = i,
                {
                    let v = value.clone();
                    move || *v.borrow_mut() = prev
                },
            ));
        }

        // Should only have 3 undo levels
        assert_eq!(manager.undo_count(), 3);
        assert_eq!(*value.borrow(), 5);

        // Can only undo 3 times
        manager.undo();
        assert_eq!(*value.borrow(), 4);
        manager.undo();
        assert_eq!(*value.borrow(), 3);
        manager.undo();
        assert_eq!(*value.borrow(), 2);
        assert!(!manager.can_undo());
    }
}
