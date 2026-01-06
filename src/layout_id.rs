//! Stable identifiers for layout node caching
//!
//! Elements with a `LayoutId` can have their Taffy nodes reused across frames,
//! avoiding the cost of rebuilding the entire layout tree every frame.

use std::fmt;

/// Stable identifier for layout node caching.
///
/// Elements with a `LayoutId` will have their Taffy `NodeId` cached and reused
/// across frames. Only nodes whose style or children change will be updated.
///
/// # Example
/// ```ignore
/// container()
///     .layout_id("sidebar")
///     .child(button("Save").layout_id("save_btn"))
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LayoutId(String);

impl LayoutId {
    /// Create a new layout ID from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Create a child layout ID.
    ///
    /// This is useful for automatically generating IDs for children
    /// based on their parent's ID and index.
    ///
    /// # Example
    /// ```ignore
    /// let parent = LayoutId::new("list");
    /// let child0 = parent.child(0);  // "list/0"
    /// let child1 = parent.child(1);  // "list/1"
    /// ```
    pub fn child(&self, index: u32) -> Self {
        Self(format!("{}/{}", self.0, index))
    }

    /// Get the string representation of this ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for LayoutId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayoutId({:?})", self.0)
    }
}

impl fmt::Display for LayoutId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for LayoutId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for LayoutId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<u64> for LayoutId {
    fn from(id: u64) -> Self {
        Self::new(id.to_string())
    }
}
