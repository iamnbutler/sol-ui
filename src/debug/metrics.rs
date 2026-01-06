//! Performance metrics tracking and display

use crate::{
    color::{Color, ColorExt, colors},
    geometry::Rect,
    render::{PaintContext, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Metrics for a single frame
#[derive(Debug, Clone, Default)]
pub struct FrameMetrics {
    /// Total frame time
    pub frame_time: Duration,
    /// Time spent in layout phase
    pub layout_time: Duration,
    /// Time spent in paint phase
    pub paint_time: Duration,
    /// Number of elements culled
    pub culled_count: usize,
    /// Number of elements rendered
    pub rendered_count: usize,
}

impl FrameMetrics {
    /// Calculate FPS from frame time
    pub fn fps(&self) -> f32 {
        if self.frame_time.as_secs_f32() > 0.0 {
            1.0 / self.frame_time.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Get culling percentage
    pub fn culling_percentage(&self) -> f32 {
        let total = self.culled_count + self.rendered_count;
        if total > 0 {
            (self.culled_count as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Performance metrics tracker
pub struct PerformanceMetrics {
    /// History of frame metrics
    history: VecDeque<FrameMetrics>,
    /// Maximum history size
    max_history: usize,
    /// Current frame start time
    frame_start: Option<Instant>,
    /// Current frame metrics (being built)
    current_frame: FrameMetrics,
    /// Whether to show the graph
    show_graph: bool,
    /// Whether to show detailed stats
    show_details: bool,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(120),
            max_history: 120, // ~2 seconds at 60fps
            frame_start: None,
            current_frame: FrameMetrics::default(),
            show_graph: true,
            show_details: true,
        }
    }

    /// Record the start of a frame
    pub fn frame_start(&mut self) {
        self.frame_start = Some(Instant::now());
        self.current_frame = FrameMetrics::default();
    }

    /// Record the end of a frame
    pub fn frame_end(&mut self) {
        if let Some(start) = self.frame_start.take() {
            self.current_frame.frame_time = start.elapsed();

            // Store in history
            if self.history.len() >= self.max_history {
                self.history.pop_front();
            }
            self.history.push_back(self.current_frame.clone());
        }
    }

    /// Record layout phase timing
    pub fn record_layout_time(&mut self, duration: Duration) {
        self.current_frame.layout_time = duration;
    }

    /// Record paint phase timing
    pub fn record_paint_time(&mut self, duration: Duration) {
        self.current_frame.paint_time = duration;
    }

    /// Record culling statistics
    pub fn record_culling_stats(&mut self, culled: usize, rendered: usize) {
        self.current_frame.culled_count = culled;
        self.current_frame.rendered_count = rendered;
    }

    /// Get the latest frame metrics
    pub fn latest(&self) -> Option<&FrameMetrics> {
        self.history.back()
    }

    /// Get average FPS over history
    pub fn average_fps(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }

        let total_time: Duration = self.history.iter().map(|m| m.frame_time).sum();
        let count = self.history.len() as f32;

        if total_time.as_secs_f32() > 0.0 {
            count / total_time.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.history.is_empty() {
            return Duration::ZERO;
        }

        let total_time: Duration = self.history.iter().map(|m| m.frame_time).sum();
        total_time / self.history.len() as u32
    }

    /// Get min/max FPS over history
    pub fn fps_range(&self) -> (f32, f32) {
        if self.history.is_empty() {
            return (0.0, 0.0);
        }

        let min_time = self
            .history
            .iter()
            .map(|m| m.frame_time)
            .min()
            .unwrap_or(Duration::ZERO);
        let max_time = self
            .history
            .iter()
            .map(|m| m.frame_time)
            .max()
            .unwrap_or(Duration::ZERO);

        let max_fps = if min_time.as_secs_f32() > 0.0 {
            1.0 / min_time.as_secs_f32()
        } else {
            0.0
        };
        let min_fps = if max_time.as_secs_f32() > 0.0 {
            1.0 / max_time.as_secs_f32()
        } else {
            0.0
        };

        (min_fps, max_fps)
    }

    /// Toggle graph display
    pub fn toggle_graph(&mut self) {
        self.show_graph = !self.show_graph;
    }

    /// Toggle details display
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Paint the metrics panel
    pub fn paint(&self, viewport: Rect, ctx: &mut PaintContext) {
        let panel_width = 180.0;
        let panel_height = if self.show_graph { 140.0 } else { 80.0 };
        let panel_bounds = Rect::from_pos_size(
            viewport.pos + Vec2::new(viewport.size.x - panel_width - 8.0, 28.0),
            Vec2::new(panel_width, panel_height),
        );

        // Background
        ctx.paint_solid_quad(panel_bounds, Color::rgba(0.0, 0.0, 0.0, 0.8));

        // Title and FPS
        let avg_fps = self.average_fps();
        let fps_color = if avg_fps >= 55.0 {
            colors::GREEN
        } else if avg_fps >= 30.0 {
            colors::YELLOW
        } else {
            colors::RED
        };

        ctx.paint_text(PaintText {
            position: panel_bounds.pos + Vec2::new(8.0, 8.0),
            text: format!("FPS: {:.1}", avg_fps),
            style: TextStyle {
                size: 14.0,
                color: fps_color,
            },
        });

        // Detailed stats
        if self.show_details {
            let mut y = 28.0;
            let line_height = 12.0;

            if let Some(latest) = self.latest() {
                let stats = [
                    format!("Frame: {:.2}ms", latest.frame_time.as_secs_f32() * 1000.0),
                    format!("Layout: {:.2}ms", latest.layout_time.as_secs_f32() * 1000.0),
                    format!("Paint: {:.2}ms", latest.paint_time.as_secs_f32() * 1000.0),
                    format!(
                        "Culled: {}% ({}/{})",
                        latest.culling_percentage() as i32,
                        latest.culled_count,
                        latest.culled_count + latest.rendered_count
                    ),
                ];

                for stat in stats {
                    ctx.paint_text(PaintText {
                        position: panel_bounds.pos + Vec2::new(8.0, y),
                        text: stat,
                        style: TextStyle {
                            size: 10.0,
                            color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                        },
                    });
                    y += line_height;
                }
            }
        }

        // Frame time graph
        if self.show_graph && !self.history.is_empty() {
            self.paint_graph(panel_bounds, ctx);
        }
    }

    fn paint_graph(&self, panel_bounds: Rect, ctx: &mut PaintContext) {
        let graph_height = 40.0;
        let graph_y = panel_bounds.pos.y + panel_bounds.size.y - graph_height - 8.0;
        let graph_bounds = Rect::from_pos_size(
            Vec2::new(panel_bounds.pos.x + 8.0, graph_y),
            Vec2::new(panel_bounds.size.x - 16.0, graph_height),
        );

        // Graph background
        ctx.paint_solid_quad(graph_bounds, Color::rgba(0.1, 0.1, 0.1, 0.8));

        // Target frame time line (16.67ms = 60fps)
        let target_y = graph_y + graph_height * 0.5;
        ctx.paint_solid_quad(
            Rect::from_pos_size(
                Vec2::new(graph_bounds.pos.x, target_y),
                Vec2::new(graph_bounds.size.x, 1.0),
            ),
            Color::rgba(0.3, 0.3, 0.3, 0.8),
        );

        // Plot frame times
        let bar_width = graph_bounds.size.x / self.max_history as f32;
        let max_frame_time = 33.33; // 30fps cap for scaling

        for (i, metrics) in self.history.iter().enumerate() {
            let frame_ms = metrics.frame_time.as_secs_f32() * 1000.0;
            let normalized = (frame_ms / max_frame_time).min(1.0);
            let bar_height = normalized * graph_height;

            let color = if frame_ms <= 16.67 {
                colors::GREEN
            } else if frame_ms <= 33.33 {
                colors::YELLOW
            } else {
                colors::RED
            };

            let bar_x = graph_bounds.pos.x + i as f32 * bar_width;
            let bar_y = graph_bounds.pos.y + graph_height - bar_height;

            ctx.paint_solid_quad(
                Rect::from_pos_size(
                    Vec2::new(bar_x, bar_y),
                    Vec2::new(bar_width.max(1.0), bar_height),
                ),
                Color { alpha: 0.8, ..color },
            );
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}
