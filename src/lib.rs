// todo: remove these
#![allow(unexpected_cfgs, deprecated)]

pub mod app;
pub mod color;
pub mod element;
pub mod entity;
pub mod geometry;
pub mod interaction;
pub mod layer;
pub mod layout_engine;
pub mod platform;
pub mod render;
pub mod style;
pub mod task;
pub mod text_system;
pub mod undo;

/// Test utilities for layout, interaction, and render testing
#[cfg(any(test, feature = "testing"))]
pub mod testing;
