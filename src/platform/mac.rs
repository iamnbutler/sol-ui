mod app_delegate;
mod menu;
pub(crate) mod metal_renderer;
mod window;

pub use app_delegate::{is_app_active, setup_app_delegate};
pub use menu::create_app_menu;
pub use window::Window;
