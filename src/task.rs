//! Non-blocking async task system for sol-ui
//!
//! This module provides a way to run background tasks without blocking the UI thread.
//! Tasks are spawned on background threads and results are delivered back to the UI
//! thread via channels.
//!
//! # Example
//!
//! ```ignore
//! use sol_ui::task::{spawn_task, with_task_runner};
//!
//! // Spawn a background task
//! spawn_task(|| {
//!     // This runs on a background thread
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//!     42 // Return a result
//! }, |result| {
//!     // This callback runs on the UI thread
//!     println!("Task completed with result: {}", result);
//! });
//! ```

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

/// Unique identifier for a spawned task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A completed task result ready for processing (sent from background thread)
pub(crate) struct CompletedTask {
    id: TaskId,
    result: Box<dyn Any + Send>,
}

// CompletedTask is Send because all its fields are Send
unsafe impl Send for CompletedTask {}

/// Type-erased callback stored on the UI thread
type Callback = Box<dyn FnOnce(Box<dyn Any + Send>)>;

/// Manages background tasks and their completion callbacks
///
/// TaskRunner is responsible for:
/// - Receiving completed task results from background threads
/// - Executing callbacks on the UI thread
/// - Tracking pending tasks
pub struct TaskRunner {
    receiver: Receiver<CompletedTask>,
    sender: Sender<CompletedTask>,
    /// Callbacks stored on the UI thread (not sent to background)
    callbacks: HashMap<TaskId, Callback>,
    pending_count: usize,
}

impl TaskRunner {
    /// Create a new TaskRunner
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            receiver,
            sender,
            callbacks: HashMap::new(),
            pending_count: 0,
        }
    }

    /// Get the sender for spawning tasks
    pub(crate) fn sender(&self) -> Sender<CompletedTask> {
        self.sender.clone()
    }

    /// Register a callback for a task (called on UI thread)
    pub(crate) fn register_callback(&mut self, id: TaskId, callback: Callback) {
        self.callbacks.insert(id, callback);
        self.pending_count += 1;
    }

    /// Process any completed tasks, running their callbacks
    ///
    /// This should be called once per frame on the UI thread.
    /// Returns the number of tasks processed.
    pub fn poll(&mut self) -> usize {
        let mut processed = 0;

        // Non-blocking receive of all completed tasks
        while let Ok(completed) = self.receiver.try_recv() {
            self.pending_count = self.pending_count.saturating_sub(1);

            // Find and run the callback for this task
            if let Some(callback) = self.callbacks.remove(&completed.id) {
                callback(completed.result);
            }

            processed += 1;
        }

        processed
    }

    /// Get the number of pending tasks
    pub fn pending_count(&self) -> usize {
        self.pending_count
    }

    /// Check if there are any pending tasks
    pub fn has_pending(&self) -> bool {
        self.pending_count > 0
    }
}

impl Default for TaskRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local access to the task runner
thread_local! {
    static TASK_RUNNER: RefCell<Option<*mut TaskRunner>> = const { RefCell::new(None) };
}

/// Set the current task runner for this thread
///
/// # Safety
/// The caller must ensure the runner remains valid for the duration it's set.
pub fn set_task_runner(runner: &mut TaskRunner) {
    TASK_RUNNER.with(|cell| {
        *cell.borrow_mut() = Some(runner as *mut TaskRunner);
    });
}

/// Clear the current task runner
pub fn clear_task_runner() {
    TASK_RUNNER.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Execute a closure with access to the current task runner
///
/// # Panics
/// Panics if called outside of the app context (when no runner is set).
pub fn with_task_runner<R>(f: impl FnOnce(&mut TaskRunner) -> R) -> R {
    TASK_RUNNER.with(|cell| {
        let ptr = cell
            .borrow()
            .expect("with_task_runner called outside app context");
        // Safety: We ensure the runner is valid while the pointer is set
        let runner = unsafe { &mut *ptr };
        f(runner)
    })
}

/// Try to execute a closure with access to the current task runner
///
/// Returns None if no runner is currently set.
pub fn try_with_task_runner<R>(f: impl FnOnce(&mut TaskRunner) -> R) -> Option<R> {
    TASK_RUNNER.with(|cell| {
        let ptr = *cell.borrow();
        ptr.map(|p| {
            // Safety: We ensure the runner is valid while the pointer is set
            let runner = unsafe { &mut *p };
            f(runner)
        })
    })
}

/// Check if a task runner is currently available
pub fn has_task_runner() -> bool {
    TASK_RUNNER.with(|cell| cell.borrow().is_some())
}

/// Spawn a background task that will execute the given closure
///
/// The task runs on a background thread. When complete, the callback
/// is invoked on the UI thread with the result.
///
/// # Arguments
/// * `task` - The closure to run on a background thread. Must return a value.
/// * `on_complete` - Callback invoked on the UI thread when the task completes.
///
/// # Returns
/// A `TaskId` that can be used to identify the task.
///
/// # Panics
/// Panics if called outside of the app context.
///
/// # Example
///
/// ```ignore
/// let task_id = spawn_task(
///     || {
///         // Expensive computation on background thread
///         compute_something()
///     },
///     |result| {
///         // Handle result on UI thread
///         println!("Got result: {:?}", result);
///     },
/// );
/// ```
pub fn spawn_task<T, F, C>(task: F, on_complete: C) -> TaskId
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
    C: FnOnce(T) + 'static,
{
    let id = TaskId::new();

    with_task_runner(|runner| {
        let sender = runner.sender();

        // Store the callback on the UI thread (type-erased)
        let callback: Callback = Box::new(move |boxed_result| {
            if let Ok(result) = boxed_result.downcast::<T>() {
                on_complete(*result);
            }
        });
        runner.register_callback(id, callback);

        // Spawn the background thread (only sends result back, not callback)
        thread::spawn(move || {
            // Run the task on background thread
            let result = task();

            // Create the completed task with just the result
            let completed = CompletedTask {
                id,
                result: Box::new(result),
            };

            // Send result back to UI thread (ignore error if receiver dropped)
            let _ = sender.send(completed);
        });
    });

    id
}

/// Spawn a background task without a completion callback
///
/// Useful for fire-and-forget operations.
///
/// # Arguments
/// * `task` - The closure to run on a background thread.
///
/// # Returns
/// A `TaskId` that can be used to identify the task.
pub fn spawn_task_detached<F>(task: F) -> TaskId
where
    F: FnOnce() + Send + 'static,
{
    spawn_task(task, |_: ()| {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    #[test]
    fn test_task_runner_creation() {
        let runner = TaskRunner::new();
        assert_eq!(runner.pending_count(), 0);
        assert!(!runner.has_pending());
    }

    #[test]
    fn test_task_runner_poll_empty() {
        let mut runner = TaskRunner::new();
        assert_eq!(runner.poll(), 0);
    }

    #[test]
    fn test_spawn_and_complete() {
        let mut runner = TaskRunner::new();
        set_task_runner(&mut runner);

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        spawn_task(
            || 42,
            move |result| {
                assert_eq!(result, 42);
                completed_clone.store(true, Ordering::SeqCst);
            },
        );

        // Wait for task to complete
        thread::sleep(Duration::from_millis(50));

        // Poll should process the completed task
        let processed = runner.poll();
        assert_eq!(processed, 1);
        assert!(completed.load(Ordering::SeqCst));

        clear_task_runner();
    }

    #[test]
    fn test_multiple_tasks() {
        let mut runner = TaskRunner::new();
        set_task_runner(&mut runner);

        let sum = Arc::new(std::sync::atomic::AtomicI32::new(0));

        for i in 0..5 {
            let sum_clone = sum.clone();
            spawn_task(
                move || i,
                move |result| {
                    sum_clone.fetch_add(result, Ordering::SeqCst);
                },
            );
        }

        // Wait for tasks to complete
        thread::sleep(Duration::from_millis(100));

        // Poll should process all tasks
        let processed = runner.poll();
        assert_eq!(processed, 5);
        assert_eq!(sum.load(Ordering::SeqCst), 0 + 1 + 2 + 3 + 4);

        clear_task_runner();
    }

    #[test]
    fn test_pending_count() {
        let mut runner = TaskRunner::new();
        set_task_runner(&mut runner);

        spawn_task(
            || {
                thread::sleep(Duration::from_millis(100));
            },
            |_| {},
        );

        assert_eq!(runner.pending_count(), 1);
        assert!(runner.has_pending());

        // Wait for completion
        thread::sleep(Duration::from_millis(150));
        runner.poll();

        assert_eq!(runner.pending_count(), 0);
        assert!(!runner.has_pending());

        clear_task_runner();
    }
}
