#![allow(unexpected_cfgs, dead_code)]
// todo: remove these
#![allow(deprecated)]
pub mod app;
pub mod color;
pub mod draw;
pub mod element;
pub mod geometry;
pub mod layer;
pub mod layout;
pub mod platform;
pub mod text_system;

pub use app::{AppBuilder, app};
pub use platform::Window;

// Re-export commonly used types
pub use draw::{DrawCommand, DrawList, FrameStyle, TextStyle};
pub use element::{ElementId, IdStack};
pub use geometry::Rect;
