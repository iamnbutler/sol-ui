#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "macos")]
pub use mac::{Window, create_app_menu};
