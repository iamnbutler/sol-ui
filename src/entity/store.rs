//! Entity storage implementation

use super::{Entity, EntityId, subscription::SubscriptionManager};
use std::any::Any;

/// A slot in the entity store
struct EntitySlot {
    /// The stored data (type-erased)
    data: Option<Box<dyn Any>>,
    /// Current generation for this slot
    generation: u32,
    /// Reference count
    ref_count: u32,
}

impl EntitySlot {
    fn new() -> Self {
        Self {
            data: None,
            generation: 0,
            ref_count: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    fn is_valid(&self, generation: u32) -> bool {
        self.data.is_some() && self.generation == generation
    }
}

/// Storage for all entities
///
/// Owns all entity state and manages their lifecycle. Entities are stored
/// in slots that can be reused (with generation tracking to prevent stale access).
pub struct EntityStore {
    /// All entity slots
    slots: Vec<EntitySlot>,
    /// Indices of free slots for reuse
    free_list: Vec<u32>,
    /// Slots that became empty during this frame (for cleanup)
    pending_cleanup: Vec<u32>,
    /// Subscription manager for tracking observations and dirty state
    subscriptions: SubscriptionManager,
}

impl EntityStore {
    /// Create a new empty entity store
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            pending_cleanup: Vec::new(),
            subscriptions: SubscriptionManager::new(),
        }
    }

    /// Create a new entity with the given initial state
    pub fn create<T: 'static>(&mut self, value: T) -> Entity<T> {
        let (index, generation) = self.allocate_slot();

        let slot = &mut self.slots[index as usize];
        slot.data = Some(Box::new(value));
        slot.ref_count = 1;

        let id = EntityId::new(index, generation);
        Entity::new(id)
    }

    /// Read entity state immutably
    pub fn read<T: 'static, R>(&self, entity: &Entity<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
        let id = entity.id();
        let slot = self.slots.get(id.index() as usize)?;

        if !slot.is_valid(id.generation()) {
            return None;
        }

        let data = slot.data.as_ref()?;
        let value = data.downcast_ref::<T>()?;
        Some(f(value))
    }

    /// Update entity state mutably
    ///
    /// This automatically marks the entity as dirty for the subscription system,
    /// which will trigger a re-render if the entity is being observed.
    pub fn update<T: 'static, R>(
        &mut self,
        entity: &Entity<T>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        let id = entity.id();
        let slot = self.slots.get_mut(id.index() as usize)?;

        if !slot.is_valid(id.generation()) {
            return None;
        }

        let data = slot.data.as_mut()?;
        let value = data.downcast_mut::<T>()?;

        // Mark this entity as dirty for the subscription system
        self.subscriptions.mark_dirty(id);

        Some(f(value))
    }

    /// Observe entity state (read with subscription tracking)
    ///
    /// Like `read`, but also registers this entity as observed for the current frame.
    /// If the observed entity is mutated via `update`, the system will request
    /// a re-render.
    pub fn observe<T: 'static, R>(&mut self, entity: &Entity<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
        let id = entity.id();
        let slot = self.slots.get(id.index() as usize)?;

        if !slot.is_valid(id.generation()) {
            return None;
        }

        // Register this entity as observed
        self.subscriptions.observe(id);

        let data = slot.data.as_ref()?;
        let value = data.downcast_ref::<T>()?;
        Some(f(value))
    }

    /// Increment reference count for an entity
    pub(crate) fn increment_ref(&mut self, id: EntityId) {
        if let Some(slot) = self.slots.get_mut(id.index() as usize) {
            if slot.is_valid(id.generation()) {
                slot.ref_count = slot.ref_count.saturating_add(1);
            }
        }
    }

    /// Decrement reference count for an entity
    pub(crate) fn decrement_ref(&mut self, id: EntityId) {
        if let Some(slot) = self.slots.get_mut(id.index() as usize) {
            if slot.is_valid(id.generation()) {
                slot.ref_count = slot.ref_count.saturating_sub(1);
                if slot.ref_count == 0 {
                    // Mark for cleanup rather than immediately freeing
                    self.pending_cleanup.push(id.index());
                }
            }
        }
    }

    /// Clean up entities with zero references and end the subscription frame
    ///
    /// Call this at frame boundaries to actually free slots and check for
    /// dirty observed entities.
    ///
    /// Returns `true` if any observed entity was mutated and a re-render is needed.
    pub fn cleanup(&mut self) -> bool {
        for index in self.pending_cleanup.drain(..) {
            if let Some(slot) = self.slots.get_mut(index as usize) {
                // Only clean up if still at zero refs (could have been re-referenced)
                if slot.ref_count == 0 && slot.data.is_some() {
                    slot.data = None;
                    slot.generation = slot.generation.wrapping_add(1);
                    self.free_list.push(index);
                }
            }
        }

        // End the subscription frame and return whether we need to re-render
        self.subscriptions.end_frame()
    }

    /// Check if any observed entity was mutated during this frame
    ///
    /// This can be called mid-frame to check if a re-render is already needed.
    pub fn needs_render(&self) -> bool {
        self.subscriptions.needs_render()
    }

    /// Get the number of observed entities this frame (for debugging)
    pub fn observed_count(&self) -> usize {
        self.subscriptions.observed_count()
    }

    /// Get the number of dirty entities this frame (for debugging)
    pub fn dirty_count(&self) -> usize {
        self.subscriptions.dirty_count()
    }

    /// Allocate a slot for a new entity
    fn allocate_slot(&mut self) -> (u32, u32) {
        if let Some(index) = self.free_list.pop() {
            let slot = &self.slots[index as usize];
            (index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(EntitySlot::new());
            (index, 0)
        }
    }

    /// Get the number of active entities
    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_empty()).count()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get total capacity (for debugging)
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestState {
        value: i32,
    }

    #[test]
    fn test_create_and_read() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 42 });

        let result = store.read(&entity, |s| s.value);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_update() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 0 });

        store.update(&entity, |s| s.value = 100);

        let result = store.read(&entity, |s| s.value);
        assert_eq!(result, Some(100));
    }

    #[test]
    fn test_ref_counting() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 1 });

        // Initial ref count is 1
        assert_eq!(store.len(), 1);

        // Manually increment
        store.increment_ref(entity.id());

        // Decrement once (still has 1 ref)
        store.decrement_ref(entity.id());
        store.cleanup();
        assert_eq!(store.len(), 1);

        // Decrement again (now at 0)
        store.decrement_ref(entity.id());
        store.cleanup();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_slot_reuse() {
        let mut store = EntityStore::new();

        // Create and "drop" an entity
        let entity1 = store.create(TestState { value: 1 });
        let id1 = entity1.id();
        store.decrement_ref(id1);
        store.cleanup();

        // Create another entity - should reuse the slot
        let entity2 = store.create(TestState { value: 2 });
        let id2 = entity2.id();

        // Same index but different generation
        assert_eq!(id1.index(), id2.index());
        assert_ne!(id1.generation(), id2.generation());
    }

    #[test]
    fn test_stale_access() {
        let mut store = EntityStore::new();

        let entity = store.create(TestState { value: 1 });
        let stale_id = entity.id();

        // Free the slot
        store.decrement_ref(stale_id);
        store.cleanup();

        // Create a new entity in the same slot
        let _entity2 = store.create(TestState { value: 2 });

        // Create a fake entity with the old ID (simulating stale access)
        let stale_entity: Entity<TestState> = Entity::new(stale_id);

        // Access should fail due to generation mismatch
        let result = store.read(&stale_entity, |s| s.value);
        assert_eq!(result, None);

        // Prevent the stale entity from decrementing ref count on drop
        std::mem::forget(stale_entity);
    }

    #[test]
    fn test_observe_then_update_triggers_render() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 0 });

        // Observe the entity (simulating a paint phase read)
        let value = store.observe(&entity, |s| s.value);
        assert_eq!(value, Some(0));
        assert_eq!(store.observed_count(), 1);

        // Update the entity (simulating an event handler)
        store.update(&entity, |s| s.value = 42);
        assert_eq!(store.dirty_count(), 1);

        // The store should now need a render
        assert!(store.needs_render());

        // Cleanup returns true indicating render was needed
        let needs_render = store.cleanup();
        assert!(needs_render);

        // After cleanup, tracking is reset
        assert!(!store.needs_render());
        assert_eq!(store.observed_count(), 0);
        assert_eq!(store.dirty_count(), 0);
    }

    #[test]
    fn test_update_without_observe_no_render() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 0 });

        // Just read (not observe) the entity
        let value = store.read(&entity, |s| s.value);
        assert_eq!(value, Some(0));
        assert_eq!(store.observed_count(), 0);

        // Update the entity
        store.update(&entity, |s| s.value = 42);
        assert_eq!(store.dirty_count(), 1);

        // The store should NOT need a render (entity wasn't observed)
        assert!(!store.needs_render());
        let needs_render = store.cleanup();
        assert!(!needs_render);
    }

    #[test]
    fn test_observe_different_entity_no_render() {
        let mut store = EntityStore::new();
        let entity1 = store.create(TestState { value: 1 });
        let entity2 = store.create(TestState { value: 2 });

        // Observe entity1
        store.observe(&entity1, |s| s.value);

        // Update entity2
        store.update(&entity2, |s| s.value = 99);

        // The store should NOT need a render (different entities)
        assert!(!store.needs_render());
        let needs_render = store.cleanup();
        assert!(!needs_render);
    }

    #[test]
    fn test_batched_updates_single_render() {
        let mut store = EntityStore::new();
        let entity = store.create(TestState { value: 0 });

        // Observe the entity
        store.observe(&entity, |s| s.value);

        // Multiple updates within the same frame
        store.update(&entity, |s| s.value += 1);
        store.update(&entity, |s| s.value += 1);
        store.update(&entity, |s| s.value += 1);

        // Should still need only one render
        assert!(store.needs_render());

        // Final value should be 3
        let value = store.read(&entity, |s| s.value);
        assert_eq!(value, Some(3));

        // Single cleanup handles all updates
        let needs_render = store.cleanup();
        assert!(needs_render);
    }
}
