#![allow(unexpected_cfgs, unused, dead_code)]
// todo: remove these
#![allow(deprecated)]
pub mod app;
pub mod color;
pub mod draw;
pub mod element;
pub mod elements;
pub mod geometry;
pub mod layer;
pub mod layout_engine;
pub mod paint;
pub mod platform;
pub mod text_system;

pub use app::{AppBuilder, app};
pub use platform::Window;

// Re-export commonly used types
pub use draw::{DrawCommand, DrawList, FrameStyle, TextStyle};
// pub use element_old::{ElementId, IdStack};
pub use geometry::Rect;
