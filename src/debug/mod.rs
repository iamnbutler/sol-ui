//! Debug and inspector tools for Sol development
//!
//! This module provides debugging overlays and inspection tools:
//! - Element bounds visualization
//! - Layout debug view
//! - Hit test visualization
//! - Performance metrics
//! - Entity inspector
//! - Debug console/logging

mod bounds_overlay;
mod console;
mod hit_test_viz;
mod layout_inspector;
mod metrics;
mod state;

pub use bounds_overlay::BoundsOverlay;
pub use console::{DebugConsole, LogEntry, LogLevel};
pub use hit_test_viz::HitTestVisualization;
pub use layout_inspector::LayoutInspector;
pub use metrics::{FrameMetrics, PerformanceMetrics};
pub use state::{DebugPanel, DebugState};

use crate::{
    color::{Color, ColorExt, colors},
    element::{Element, LayoutContext},
    geometry::Rect,
    layer::Key,
    render::PaintContext,
};
use glam::Vec2;
use taffy::prelude::*;

/// Debug overlay that renders all active debug visualizations
pub struct DebugOverlay {
    state: DebugState,
    bounds_overlay: BoundsOverlay,
    hit_test_viz: HitTestVisualization,
    #[allow(dead_code)]
    layout_inspector: LayoutInspector,
    metrics: PerformanceMetrics,
    console: DebugConsole,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            state: DebugState::new(),
            bounds_overlay: BoundsOverlay::new(),
            hit_test_viz: HitTestVisualization::new(),
            layout_inspector: LayoutInspector::new(),
            metrics: PerformanceMetrics::new(),
            console: DebugConsole::new(100),
        }
    }

    /// Get a reference to the debug state
    pub fn state(&self) -> &DebugState {
        &self.state
    }

    /// Get a mutable reference to the debug state
    pub fn state_mut(&mut self) -> &mut DebugState {
        &mut self.state
    }

    /// Toggle the entire debug overlay
    pub fn toggle(&mut self) {
        self.state.toggle_enabled();
    }

    /// Check if debug overlay is enabled
    pub fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    /// Handle a key press, returns true if the key was consumed
    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            // F12 toggles the debug overlay
            Key::F12 => {
                self.toggle();
                true
            }
            // Only handle other keys if debug is enabled
            _ if self.state.is_enabled() => {
                match key {
                    // F1 toggles bounds overlay
                    Key::F1 => {
                        self.state.toggle_panel(DebugPanel::Bounds);
                        true
                    }
                    // F2 toggles layout inspector
                    Key::F2 => {
                        self.state.toggle_panel(DebugPanel::Layout);
                        true
                    }
                    // F3 toggles hit test visualization
                    Key::F3 => {
                        self.state.toggle_panel(DebugPanel::HitTest);
                        true
                    }
                    // F4 toggles performance metrics
                    Key::F4 => {
                        self.state.toggle_panel(DebugPanel::Metrics);
                        true
                    }
                    // F5 toggles entity inspector
                    Key::F5 => {
                        self.state.toggle_panel(DebugPanel::Inspector);
                        true
                    }
                    // F6 toggles console
                    Key::F6 => {
                        self.state.toggle_panel(DebugPanel::Console);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Record the start of a frame
    pub fn frame_start(&mut self) {
        self.metrics.frame_start();
    }

    /// Record the end of a frame
    pub fn frame_end(&mut self) {
        self.metrics.frame_end();
    }

    /// Record layout phase timing
    pub fn record_layout_time(&mut self, duration: std::time::Duration) {
        self.metrics.record_layout_time(duration);
    }

    /// Record paint phase timing
    pub fn record_paint_time(&mut self, duration: std::time::Duration) {
        self.metrics.record_paint_time(duration);
    }

    /// Record culling statistics
    pub fn record_culling_stats(&mut self, culled: usize, rendered: usize) {
        self.metrics.record_culling_stats(culled, rendered);
    }

    /// Log a debug message
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.console.log(level, message);
    }

    /// Log an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.console.info(message);
    }

    /// Log a warning message
    pub fn warn(&mut self, message: impl Into<String>) {
        self.console.warn(message);
    }

    /// Log an error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.console.error(message);
    }

    /// Register element bounds for visualization
    pub fn register_bounds(&mut self, name: &str, bounds: Rect) {
        self.bounds_overlay.register_bounds(name, bounds);
    }

    /// Register a hit test entry for visualization
    pub fn register_hit_test(&mut self, element_id: u64, bounds: Rect, z_index: i32) {
        self.hit_test_viz.register_entry(element_id, bounds, z_index);
    }

    /// Clear frame-specific debug data
    pub fn clear_frame_data(&mut self) {
        self.bounds_overlay.clear();
        self.hit_test_viz.clear();
    }

    /// Get the console for logging
    pub fn console(&self) -> &DebugConsole {
        &self.console
    }

    /// Get mutable console for logging
    pub fn console_mut(&mut self) -> &mut DebugConsole {
        &mut self.console
    }

    /// Get performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a debug overlay element for rendering
pub fn debug_overlay(overlay: &DebugOverlay, viewport: Rect) -> impl Element + '_ {
    DebugOverlayElement {
        overlay,
        viewport,
    }
}

struct DebugOverlayElement<'a> {
    overlay: &'a DebugOverlay,
    #[allow(dead_code)]
    viewport: Rect,
}

impl<'a> Element for DebugOverlayElement<'a> {
    fn layout(&mut self, ctx: &mut LayoutContext) -> taffy::NodeId {
        // Full screen overlay
        ctx.request_layout(Style {
            position: Position::Absolute,
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        })
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !self.overlay.state.is_enabled() {
            return;
        }

        // Paint each enabled panel
        if self.overlay.state.is_panel_enabled(DebugPanel::Bounds) {
            self.overlay.bounds_overlay.paint(ctx);
        }

        if self.overlay.state.is_panel_enabled(DebugPanel::HitTest) {
            self.overlay.hit_test_viz.paint(ctx);
        }

        // Paint metrics panel in top-right corner
        if self.overlay.state.is_panel_enabled(DebugPanel::Metrics) {
            self.overlay.metrics.paint(bounds, ctx);
        }

        // Paint console at bottom
        if self.overlay.state.is_panel_enabled(DebugPanel::Console) {
            self.overlay.console.paint(bounds, ctx);
        }

        // Paint debug mode indicator
        self.paint_indicator(bounds, ctx);
    }
}

impl<'a> DebugOverlayElement<'a> {
    fn paint_indicator(&self, _bounds: Rect, ctx: &mut PaintContext) {
        // Small indicator in top-left showing debug mode is active
        let indicator_bounds = Rect::new(4.0, 4.0, 80.0, 20.0);

        // Background
        ctx.paint_solid_quad(indicator_bounds, Color::rgba(0.0, 0.0, 0.0, 0.7));

        // Text
        ctx.paint_text(crate::render::PaintText {
            position: Vec2::new(8.0, 6.0),
            text: "DEBUG".to_string(),
            style: crate::style::TextStyle {
                size: 11.0,
                color: colors::GREEN,
                line_height: 1.2,
            },
        });
    }
}
