//! Entity storage implementation

use super::observe::{ObserverCallback, ObserverRegistry, SubscriptionId};
use super::{Entity, EntityId};
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
    /// Observer registry for state change notifications
    observers: ObserverRegistry,
}

impl EntityStore {
    /// Create a new empty entity store
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            pending_cleanup: Vec::new(),
            observers: ObserverRegistry::new(),
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
    /// This will mark the entity as changed, notifying observers at frame boundaries.
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
        let result = f(value);

        // Mark this entity as changed for observer notification
        self.observers.mark_changed(id);

        Some(result)
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

    /// Clean up entities with zero references
    ///
    /// Call this at frame boundaries to actually free slots and notify observers.
    pub fn cleanup(&mut self) {
        // First, flush any pending observer notifications
        self.observers.flush();

        // Then clean up entities
        for index in self.pending_cleanup.drain(..) {
            if let Some(slot) = self.slots.get_mut(index as usize) {
                // Only clean up if still at zero refs (could have been re-referenced)
                if slot.ref_count == 0 && slot.data.is_some() {
                    // Clean up any observers for this entity
                    let entity_id = EntityId::new(index, slot.generation);
                    self.observers.unsubscribe_all(entity_id);

                    slot.data = None;
                    slot.generation = slot.generation.wrapping_add(1);
                    self.free_list.push(index);
                }
            }
        }
    }

    /// Subscribe to changes on an entity
    ///
    /// The callback will be invoked at frame boundaries whenever the entity is updated.
    /// Returns a SubscriptionId that can be used to unsubscribe.
    pub fn subscribe<T: 'static>(
        &mut self,
        entity: &Entity<T>,
        callback: ObserverCallback,
    ) -> SubscriptionId {
        self.observers.subscribe(entity.id(), callback)
    }

    /// Unsubscribe from entity changes using the subscription ID
    pub fn unsubscribe(&mut self, subscription_id: SubscriptionId) {
        self.observers.unsubscribe(subscription_id);
    }

    /// Check if UI invalidation has been requested due to state changes
    pub fn invalidation_requested(&self) -> bool {
        self.observers.invalidation_requested()
    }

    /// Clear the invalidation request flag
    pub fn clear_invalidation(&mut self) {
        self.observers.clear_invalidation();
    }

    /// Check if there are pending observer notifications
    pub fn has_pending_notifications(&self) -> bool {
        self.observers.has_pending_changes()
    }

    /// Manually flush pending observer notifications
    ///
    /// This is useful if you need to notify observers before the end of frame.
    pub fn flush_notifications(&mut self) {
        self.observers.flush();
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
}
