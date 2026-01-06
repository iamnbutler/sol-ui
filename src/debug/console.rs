//! Debug console for logging and message display

use crate::{
    color::{Color, ColorExt, colors},
    geometry::Rect,
    render::{PaintContext, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::collections::VecDeque;
use std::time::Instant;

/// Log level for debug messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Get the color for this log level
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Debug => Color::rgba(0.6, 0.6, 0.6, 1.0),
            LogLevel::Info => Color::rgba(0.8, 0.8, 0.8, 1.0),
            LogLevel::Warn => colors::YELLOW,
            LogLevel::Error => colors::RED,
        }
    }

    /// Get the prefix for this log level
    pub fn prefix(&self) -> &'static str {
        match self {
            LogLevel::Debug => "[DBG]",
            LogLevel::Info => "[INF]",
            LogLevel::Warn => "[WRN]",
            LogLevel::Error => "[ERR]",
        }
    }
}

/// A single log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: Instant,
    pub frame_number: u64,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>, frame_number: u64) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: Instant::now(),
            frame_number,
        }
    }
}

/// Debug console for displaying log messages
pub struct DebugConsole {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    frame_counter: u64,
    show_timestamps: bool,
    show_frame_numbers: bool,
    min_level: LogLevel,
    collapsed: bool,
}

impl DebugConsole {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
            frame_counter: 0,
            show_timestamps: false,
            show_frame_numbers: true,
            min_level: LogLevel::Debug,
            collapsed: false,
        }
    }

    /// Log a message with the specified level
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        if (level as u8) < (self.min_level as u8) {
            return;
        }

        let entry = LogEntry::new(level, message, self.frame_counter);

        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Log an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Log a debug message
    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Log a warning message
    pub fn warn(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warn, message);
    }

    /// Log an error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }

    /// Increment frame counter (call each frame)
    pub fn next_frame(&mut self) {
        self.frame_counter += 1;
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the console is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Toggle timestamps display
    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps;
    }

    /// Toggle frame numbers display
    pub fn toggle_frame_numbers(&mut self) {
        self.show_frame_numbers = !self.show_frame_numbers;
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Set minimum log level
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    /// Get recent entries
    pub fn recent_entries(&self, count: usize) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().rev().take(count)
    }

    /// Paint the console
    pub fn paint(&self, viewport: Rect, ctx: &mut PaintContext) {
        let console_height = if self.collapsed { 24.0 } else { 150.0 };
        let console_bounds = Rect::from_pos_size(
            Vec2::new(viewport.pos.x, viewport.pos.y + viewport.size.y - console_height),
            Vec2::new(viewport.size.x, console_height),
        );

        // Background
        ctx.paint_solid_quad(console_bounds, Color::rgba(0.05, 0.05, 0.05, 0.9));

        // Title bar
        let title_height = 20.0;
        ctx.paint_solid_quad(
            Rect::from_pos_size(console_bounds.pos, Vec2::new(console_bounds.size.x, title_height)),
            Color::rgba(0.15, 0.15, 0.15, 1.0),
        );

        ctx.paint_text(PaintText {
            position: console_bounds.pos + Vec2::new(8.0, 4.0),
            text: format!(
                "Console ({}) {}",
                self.entries.len(),
                if self.collapsed { "[+]" } else { "[-]" }
            ),
            style: TextStyle {
                size: 11.0,
                color: colors::WHITE,
            },
        });

        if self.collapsed {
            return;
        }

        // Log entries
        let content_y = console_bounds.pos.y + title_height + 4.0;
        let line_height = 13.0;
        let max_lines = ((console_height - title_height - 8.0) / line_height) as usize;

        let entries: Vec<_> = self.entries.iter().rev().take(max_lines).collect();

        for (i, entry) in entries.iter().rev().enumerate() {
            let y = content_y + i as f32 * line_height;

            // Build the log line
            let mut line = String::new();

            if self.show_frame_numbers {
                line.push_str(&format!("#{:06} ", entry.frame_number));
            }

            line.push_str(entry.level.prefix());
            line.push(' ');
            line.push_str(&entry.message);

            // Truncate if too long
            let max_chars = (console_bounds.size.x / 6.5) as usize;
            if line.len() > max_chars {
                line.truncate(max_chars - 3);
                line.push_str("...");
            }

            ctx.paint_text(PaintText {
                position: Vec2::new(console_bounds.pos.x + 8.0, y),
                text: line,
                style: TextStyle {
                    size: 10.0,
                    color: entry.level.color(),
                },
            });
        }
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new(100)
    }
}
