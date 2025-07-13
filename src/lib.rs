// todo: remove these
#![allow(unexpected_cfgs, deprecated)]

pub mod app;
pub mod color;
pub mod draw;
pub mod element;
pub mod geometry;
pub mod interaction;
pub mod layer;
pub mod layout_engine;
pub mod paint;
pub mod platform;
pub mod text_system;

pub use app::{AppBuilder, app};
pub use platform::Window;

pub use draw::{DrawCommand, DrawList, FrameStyle, TextStyle};
pub use geometry::Rect;
