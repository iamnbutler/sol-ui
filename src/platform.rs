//! Platform-specific code for macOS and iOS.
//!
//! This module provides platform abstractions for windowing, input, and rendering.

#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "ios")]
pub mod ios;

// Desktop platforms
#[cfg(target_os = "macos")]
pub use mac::{Window, create_app_menu};

// Mobile platforms
#[cfg(target_os = "ios")]
pub use ios::Window;

// Platform-agnostic types that might differ between desktop and mobile
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use self::platform_types::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod platform_types {
    /// Platform-specific point type for input events
    #[derive(Debug, Clone, Copy)]
    pub struct PlatformPoint {
        pub x: f32,
        pub y: f32,
    }

    impl PlatformPoint {
        pub fn new(x: f32, y: f32) -> Self {
            Self { x, y }
        }
    }
}
