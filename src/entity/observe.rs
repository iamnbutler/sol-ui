//! State observation system for reactive UI
//!
//! This module provides a subscription-based system for observing entity state changes.
//! When entity state is mutated, registered observers are notified at frame boundaries
//! (batched updates to prevent per-change re-renders).

use super::EntityId;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Generate a new unique subscription ID
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SubscriptionId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Callback type for entity change notifications
pub type ObserverCallback = Box<dyn FnMut() + 'static>;

/// A registered observer
struct Observer {
    /// Unique ID for this subscription
    id: SubscriptionId,
    /// Callback to invoke when entity changes
    callback: ObserverCallback,
}

/// Manages subscriptions and change notifications for entities
pub struct ObserverRegistry {
    /// Map from entity ID to list of observers
    subscriptions: HashMap<EntityId, Vec<Observer>>,

    /// Set of entities that have been modified since last flush
    pending_changes: HashSet<EntityId>,

    /// Flag to request UI invalidation
    invalidation_requested: bool,
}

impl ObserverRegistry {
    /// Create a new empty observer registry
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            pending_changes: HashSet::new(),
            invalidation_requested: false,
        }
    }

    /// Subscribe to changes on an entity
    ///
    /// The callback will be invoked (at frame boundaries) whenever the entity is updated.
    /// Returns a SubscriptionId that can be used to unsubscribe.
    pub fn subscribe(&mut self, entity_id: EntityId, callback: ObserverCallback) -> SubscriptionId {
        let id = SubscriptionId::new();
        let observer = Observer { id, callback };

        self.subscriptions
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(observer);

        id
    }

    /// Unsubscribe from an entity using the subscription ID
    pub fn unsubscribe(&mut self, subscription_id: SubscriptionId) {
        // Find and remove the subscription from all entities
        for observers in self.subscriptions.values_mut() {
            observers.retain(|o| o.id != subscription_id);
        }
    }

    /// Unsubscribe all observers for a specific entity
    pub fn unsubscribe_all(&mut self, entity_id: EntityId) {
        self.subscriptions.remove(&entity_id);
    }

    /// Mark an entity as changed
    ///
    /// This is called by EntityStore when an entity is updated.
    /// The actual notification happens at frame boundaries via `flush()`.
    pub fn mark_changed(&mut self, entity_id: EntityId) {
        // Only add to pending if there are actual observers
        if let Some(observers) = self.subscriptions.get(&entity_id) {
            if !observers.is_empty() {
                self.pending_changes.insert(entity_id);
                self.invalidation_requested = true;
            }
        }
    }

    /// Check if UI invalidation has been requested
    pub fn invalidation_requested(&self) -> bool {
        self.invalidation_requested
    }

    /// Clear the invalidation request flag
    pub fn clear_invalidation(&mut self) {
        self.invalidation_requested = false;
    }

    /// Flush pending changes and notify all observers
    ///
    /// This should be called at frame boundaries. All observers for changed
    /// entities will be notified, then the pending changes are cleared.
    pub fn flush(&mut self) {
        // Take the pending changes to avoid borrow issues
        let changed_entities: Vec<_> = self.pending_changes.drain().collect();

        for entity_id in changed_entities {
            if let Some(observers) = self.subscriptions.get_mut(&entity_id) {
                for observer in observers.iter_mut() {
                    (observer.callback)();
                }
            }
        }

        self.invalidation_requested = false;
    }

    /// Get the number of subscriptions for an entity
    pub fn subscription_count(&self, entity_id: EntityId) -> usize {
        self.subscriptions
            .get(&entity_id)
            .map_or(0, |v| v.len())
    }

    /// Get the total number of pending changes
    pub fn pending_count(&self) -> usize {
        self.pending_changes.len()
    }

    /// Check if there are any pending changes
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    /// Clear all subscriptions (called during cleanup)
    pub fn clear(&mut self) {
        self.subscriptions.clear();
        self.pending_changes.clear();
        self.invalidation_requested = false;
    }
}

impl Default for ObserverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_and_notify() {
        let mut registry = ObserverRegistry::new();
        let entity_id = EntityId::new(0, 1);

        let notified = std::rc::Rc::new(std::cell::Cell::new(false));
        let notified_clone = notified.clone();

        let _sub_id = registry.subscribe(entity_id, Box::new(move || {
            notified_clone.set(true);
        }));

        // Mark as changed but don't flush yet
        registry.mark_changed(entity_id);
        assert!(!notified.get());
        assert!(registry.has_pending_changes());

        // Flush to trigger notification
        registry.flush();
        assert!(notified.get());
        assert!(!registry.has_pending_changes());
    }

    #[test]
    fn test_unsubscribe() {
        let mut registry = ObserverRegistry::new();
        let entity_id = EntityId::new(0, 1);

        let notified = std::rc::Rc::new(std::cell::Cell::new(false));
        let notified_clone = notified.clone();

        let sub_id = registry.subscribe(entity_id, Box::new(move || {
            notified_clone.set(true);
        }));

        // Unsubscribe before marking changed
        registry.unsubscribe(sub_id);

        // Mark as changed - should not add to pending since no observers
        registry.mark_changed(entity_id);
        assert!(!registry.has_pending_changes());

        registry.flush();
        assert!(!notified.get());
    }

    #[test]
    fn test_batched_changes() {
        let mut registry = ObserverRegistry::new();
        let entity_id = EntityId::new(0, 1);

        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let count_clone = count.clone();

        let _sub_id = registry.subscribe(entity_id, Box::new(move || {
            count_clone.set(count_clone.get() + 1);
        }));

        // Multiple changes before flush
        registry.mark_changed(entity_id);
        registry.mark_changed(entity_id);
        registry.mark_changed(entity_id);

        // Should only notify once (batched)
        registry.flush();
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn test_multiple_observers() {
        let mut registry = ObserverRegistry::new();
        let entity_id = EntityId::new(0, 1);

        let count1 = std::rc::Rc::new(std::cell::Cell::new(0));
        let count2 = std::rc::Rc::new(std::cell::Cell::new(0));
        let count1_clone = count1.clone();
        let count2_clone = count2.clone();

        registry.subscribe(entity_id, Box::new(move || {
            count1_clone.set(count1_clone.get() + 1);
        }));
        registry.subscribe(entity_id, Box::new(move || {
            count2_clone.set(count2_clone.get() + 1);
        }));

        registry.mark_changed(entity_id);
        registry.flush();

        assert_eq!(count1.get(), 1);
        assert_eq!(count2.get(), 1);
    }

    #[test]
    fn test_invalidation_requested() {
        let mut registry = ObserverRegistry::new();
        let entity_id = EntityId::new(0, 1);

        registry.subscribe(entity_id, Box::new(|| {}));

        assert!(!registry.invalidation_requested());

        registry.mark_changed(entity_id);
        assert!(registry.invalidation_requested());

        registry.flush();
        assert!(!registry.invalidation_requested());
    }
}
