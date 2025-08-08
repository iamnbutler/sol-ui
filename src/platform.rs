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
