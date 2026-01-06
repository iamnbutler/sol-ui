//! Drag and drop system
//!
//! Provides drag and drop functionality with:
//! - Draggable elements
//! - Drop zones
//! - Drag preview/ghost
//! - Cross-element data transfer

use super::ElementId;
use crate::geometry::Rect;
use glam::Vec2;

/// Minimum distance (in pixels) mouse must move before drag starts
pub const DRAG_THRESHOLD: f32 = 5.0;

/// Data that can be transferred during drag and drop
#[derive(Debug, Clone)]
pub struct DragData {
    /// Type identifier for the drag data (e.g., "todo-item", "file", "text")
    pub data_type: String,
    /// The actual payload (boxed to support any type)
    payload: DragPayload,
}

/// Internal payload wrapper
#[derive(Debug, Clone)]
enum DragPayload {
    /// String data
    String(String),
    /// Integer index (for list reordering)
    Index(usize),
    /// Multiple indices (for multi-select)
    Indices(Vec<usize>),
    /// Custom data as JSON string
    Json(String),
}

impl DragData {
    /// Create drag data with a string payload
    pub fn string(data_type: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            payload: DragPayload::String(value.into()),
        }
    }

    /// Create drag data with an index payload (for list reordering)
    pub fn index(data_type: impl Into<String>, index: usize) -> Self {
        Self {
            data_type: data_type.into(),
            payload: DragPayload::Index(index),
        }
    }

    /// Create drag data with multiple indices
    pub fn indices(data_type: impl Into<String>, indices: Vec<usize>) -> Self {
        Self {
            data_type: data_type.into(),
            payload: DragPayload::Indices(indices),
        }
    }

    /// Create drag data with JSON payload
    pub fn json(data_type: impl Into<String>, json: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            payload: DragPayload::Json(json.into()),
        }
    }

    /// Get string payload
    pub fn as_string(&self) -> Option<&str> {
        match &self.payload {
            DragPayload::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get index payload
    pub fn as_index(&self) -> Option<usize> {
        match &self.payload {
            DragPayload::Index(i) => Some(*i),
            _ => None,
        }
    }

    /// Get indices payload
    pub fn as_indices(&self) -> Option<&[usize]> {
        match &self.payload {
            DragPayload::Indices(v) => Some(v),
            _ => None,
        }
    }

    /// Get JSON payload
    pub fn as_json(&self) -> Option<&str> {
        match &self.payload {
            DragPayload::Json(s) => Some(s),
            _ => None,
        }
    }
}

/// Current state of a drag operation
#[derive(Debug, Clone)]
pub struct DragState {
    /// Element being dragged
    pub source_element: ElementId,
    /// Position where drag started
    pub start_position: Vec2,
    /// Current mouse position
    pub current_position: Vec2,
    /// Offset from element origin to mouse position at drag start
    pub offset: Vec2,
    /// Original bounds of the dragged element
    pub source_bounds: Rect,
    /// Data being transferred
    pub data: DragData,
    /// Currently hovered drop zone (if any)
    pub hover_drop_zone: Option<ElementId>,
}

impl DragState {
    /// Get the current drag delta from start position
    pub fn delta(&self) -> Vec2 {
        self.current_position - self.start_position
    }

    /// Get the position where the drag preview should be rendered
    pub fn preview_position(&self) -> Vec2 {
        self.current_position - self.offset
    }
}

/// Drop zone configuration
#[derive(Debug, Clone)]
pub struct DropZone {
    /// Element ID of the drop zone
    pub element_id: ElementId,
    /// Bounds of the drop zone
    pub bounds: Rect,
    /// Data types this zone accepts
    pub accepted_types: Vec<String>,
    /// Whether this is an "insert" zone (for list reordering)
    pub is_insert_zone: bool,
    /// Insert index (for list reordering)
    pub insert_index: Option<usize>,
}

impl DropZone {
    /// Create a new drop zone
    pub fn new(element_id: ElementId, bounds: Rect) -> Self {
        Self {
            element_id,
            bounds,
            accepted_types: Vec::new(),
            is_insert_zone: false,
            insert_index: None,
        }
    }

    /// Add accepted data type
    pub fn accept(mut self, data_type: impl Into<String>) -> Self {
        self.accepted_types.push(data_type.into());
        self
    }

    /// Mark as insert zone for list reordering
    pub fn insert_zone(mut self, index: usize) -> Self {
        self.is_insert_zone = true;
        self.insert_index = Some(index);
        self
    }

    /// Check if this zone accepts the given data type
    pub fn accepts(&self, data_type: &str) -> bool {
        self.accepted_types.is_empty() || self.accepted_types.iter().any(|t| t == data_type)
    }

    /// Check if a point is within this drop zone
    pub fn contains(&self, point: Vec2) -> bool {
        self.bounds.contains(crate::geometry::Point::from(point))
    }
}

/// Result of a drop operation
#[derive(Debug, Clone)]
pub struct DropResult {
    /// The drop zone that received the drop
    pub drop_zone: ElementId,
    /// The data that was dropped
    pub data: DragData,
    /// Position where the drop occurred
    pub position: Vec2,
    /// Local position within the drop zone
    pub local_position: Vec2,
    /// Insert index (for list reordering)
    pub insert_index: Option<usize>,
}

/// Manages drop zones for the current frame
#[derive(Debug, Default)]
pub struct DropZoneRegistry {
    zones: Vec<DropZone>,
}

impl DropZoneRegistry {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    /// Register a drop zone for the current frame
    pub fn register(&mut self, zone: DropZone) {
        self.zones.push(zone);
    }

    /// Clear all registered zones (called at start of frame)
    pub fn clear(&mut self) {
        self.zones.clear();
    }

    /// Find the drop zone at the given position that accepts the given data type
    pub fn find_at(&self, position: Vec2, data_type: &str) -> Option<&DropZone> {
        // Iterate in reverse order (later registered = higher z-order)
        self.zones
            .iter()
            .rev()
            .find(|zone| zone.contains(position) && zone.accepts(data_type))
    }

    /// Get all zones (for debug rendering)
    pub fn zones(&self) -> &[DropZone] {
        &self.zones
    }
}

/// Drag and drop events
#[derive(Debug, Clone)]
pub enum DragDropEvent {
    /// Drag operation started
    DragStart {
        source_element: ElementId,
        position: Vec2,
        data: DragData,
    },

    /// Drag operation moved
    DragMove {
        source_element: ElementId,
        position: Vec2,
        delta: Vec2,
    },

    /// Drag entered a drop zone
    DragEnter {
        source_element: ElementId,
        drop_zone: ElementId,
        data: DragData,
    },

    /// Drag left a drop zone
    DragLeave {
        source_element: ElementId,
        drop_zone: ElementId,
    },

    /// Drag hovering over a drop zone
    DragOver {
        source_element: ElementId,
        drop_zone: ElementId,
        position: Vec2,
        local_position: Vec2,
    },

    /// Drop occurred
    Drop {
        result: DropResult,
    },

    /// Drag was cancelled (escape key, mouse left window, etc.)
    DragCancel {
        source_element: ElementId,
    },
}

/// Trait for elements that can be dragged
pub trait Draggable {
    /// Get the drag data for this element
    fn drag_data(&self) -> DragData;

    /// Called when drag starts
    fn on_drag_start(&mut self) {}

    /// Called during drag
    fn on_drag_move(&mut self, _delta: Vec2) {}

    /// Called when drag ends (either drop or cancel)
    fn on_drag_end(&mut self, _dropped: bool) {}
}

/// Trait for elements that can receive drops
pub trait DropTarget {
    /// Get the data types this target accepts
    fn accepted_types(&self) -> Vec<String>;

    /// Called when a drag enters this target
    fn on_drag_enter(&mut self, _data: &DragData) {}

    /// Called when a drag leaves this target
    fn on_drag_leave(&mut self) {}

    /// Called during drag over this target
    fn on_drag_over(&mut self, _position: Vec2, _data: &DragData) {}

    /// Called when a drop occurs on this target
    /// Returns true if the drop was handled
    fn on_drop(&mut self, data: DragData, position: Vec2) -> bool;
}

/// Configuration for draggable behavior
#[derive(Debug, Clone)]
pub struct DragConfig {
    /// Data type for drag operations
    pub data_type: String,
    /// Whether to show a drag preview
    pub show_preview: bool,
    /// Opacity of the original element during drag
    pub drag_opacity: f32,
    /// Cursor to show during drag
    pub drag_cursor: Option<String>,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            data_type: "default".into(),
            show_preview: true,
            drag_opacity: 0.5,
            drag_cursor: None,
        }
    }
}

impl DragConfig {
    pub fn new(data_type: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            ..Default::default()
        }
    }

    pub fn show_preview(mut self, show: bool) -> Self {
        self.show_preview = show;
        self
    }

    pub fn drag_opacity(mut self, opacity: f32) -> Self {
        self.drag_opacity = opacity;
        self
    }
}
