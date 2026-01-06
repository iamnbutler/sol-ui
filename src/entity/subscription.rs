//! Subscription and observation system for reactive state updates
//!
//! This module provides the infrastructure for observing entity state changes
//! and automatically triggering UI updates when observed state mutates.
//!
//! ## Key Concepts
//!
//! - **Observation**: Elements can "observe" entity state during paint, registering
//!   interest in changes to that state.
//! - **Dirty tracking**: When entity state is mutated via `update_entity`, the entity
//!   is marked as "dirty" for this frame.
//! - **Automatic invalidation**: At frame boundaries, if any observed entities are dirty,
//!   the system requests a new animation frame to re-render.
//! - **Batching**: Multiple mutations within a frame are batched together, preventing
//!   excessive re-renders.
//!
//! ## Usage
//!
//! ```ignore
//! // Instead of read_entity (which doesn't track changes):
//! let value = observe(&entity, |state| state.value);
//!
//! // Mutations automatically mark the entity as dirty:
//! update_entity(&entity, |state| state.value += 1);
//!
//! // At frame end, the system detects the dirty entity was observed
//! // and automatically requests another render frame.
//! ```

use super::EntityId;
use std::collections::HashSet;

/// Tracks subscriptions and dirty state for reactive updates
#[derive(Default)]
pub struct SubscriptionManager {
    /// Entities that were observed (read with observation tracking) during this frame
    observed: HashSet<EntityId>,

    /// Entities that were mutated during this frame
    dirty: HashSet<EntityId>,

    /// Whether any observed entity was mutated (triggers re-render)
    needs_render: bool,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that an entity was observed during this frame
    pub fn observe(&mut self, id: EntityId) {
        self.observed.insert(id);
        // Check if this entity is already dirty from earlier in the frame
        if self.dirty.contains(&id) {
            self.needs_render = true;
        }
    }

    /// Mark an entity as dirty (mutated)
    pub fn mark_dirty(&mut self, id: EntityId) {
        self.dirty.insert(id);
        // Check if this entity was already observed
        if self.observed.contains(&id) {
            self.needs_render = true;
        }
    }

    /// Check if any observed entity was mutated
    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    /// Clear tracking for a new frame
    ///
    /// Returns whether a re-render was needed (for the caller to act on)
    pub fn end_frame(&mut self) -> bool {
        let result = self.needs_render;
        self.observed.clear();
        self.dirty.clear();
        self.needs_render = false;
        result
    }

    /// Get the number of observed entities (for debugging)
    pub fn observed_count(&self) -> usize {
        self.observed.len()
    }

    /// Get the number of dirty entities (for debugging)
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_then_dirty() {
        let mut mgr = SubscriptionManager::new();
        let id = EntityId::new(0, 0);

        // Observe first
        mgr.observe(id);
        assert!(!mgr.needs_render());

        // Then dirty
        mgr.mark_dirty(id);
        assert!(mgr.needs_render());
    }

    #[test]
    fn test_dirty_then_observe() {
        let mut mgr = SubscriptionManager::new();
        let id = EntityId::new(0, 0);

        // Dirty first
        mgr.mark_dirty(id);
        assert!(!mgr.needs_render());

        // Then observe
        mgr.observe(id);
        assert!(mgr.needs_render());
    }

    #[test]
    fn test_different_entities() {
        let mut mgr = SubscriptionManager::new();
        let id1 = EntityId::new(0, 0);
        let id2 = EntityId::new(1, 0);

        mgr.observe(id1);
        mgr.mark_dirty(id2);

        // Different entities, no render needed
        assert!(!mgr.needs_render());
    }

    #[test]
    fn test_end_frame_clears() {
        let mut mgr = SubscriptionManager::new();
        let id = EntityId::new(0, 0);

        mgr.observe(id);
        mgr.mark_dirty(id);
        assert!(mgr.needs_render());

        // End frame returns the needs_render value and clears
        assert!(mgr.end_frame());
        assert!(!mgr.needs_render());
        assert_eq!(mgr.observed_count(), 0);
        assert_eq!(mgr.dirty_count(), 0);
    }
}
