//! SVG Icon element for rendering vector icons

use crate::{
    color::Color,
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    render::PaintQuad,
};
use std::sync::Arc;
use taffy::prelude::*;

/// Create a new icon element from SVG data
pub fn icon(svg_data: impl Into<IconSource>) -> Icon {
    Icon::new(svg_data)
}

/// Source for icon SVG data
#[derive(Clone)]
pub enum IconSource {
    /// Raw SVG string
    Svg(Arc<str>),
    /// Pre-parsed SVG tree (for efficiency when reusing)
    Tree(Arc<usvg::Tree>),
}

impl From<&str> for IconSource {
    fn from(s: &str) -> Self {
        IconSource::Svg(Arc::from(s))
    }
}

impl From<String> for IconSource {
    fn from(s: String) -> Self {
        IconSource::Svg(Arc::from(s.as_str()))
    }
}

impl From<Arc<usvg::Tree>> for IconSource {
    fn from(tree: Arc<usvg::Tree>) -> Self {
        IconSource::Tree(tree)
    }
}

/// An SVG icon element
pub struct Icon {
    /// The SVG source
    source: IconSource,
    /// Parsed SVG tree (lazily initialized)
    tree: Option<Arc<usvg::Tree>>,
    /// Icon size in logical pixels (width and height)
    size: f32,
    /// Optional color tint (replaces all colors in the SVG)
    color: Option<Color>,
    /// Cached layout node
    node_id: Option<NodeId>,
}

impl Icon {
    /// Create a new icon from SVG data
    pub fn new(source: impl Into<IconSource>) -> Self {
        let source = source.into();
        let tree = match &source {
            IconSource::Tree(t) => Some(Arc::clone(t)),
            IconSource::Svg(_) => None,
        };

        Self {
            source,
            tree,
            size: 24.0, // Default icon size
            color: None,
            node_id: None,
        }
    }

    /// Set the icon size (both width and height)
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the icon color (tints all paths in the SVG)
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Parse the SVG if not already parsed
    fn ensure_parsed(&mut self) -> Option<&Arc<usvg::Tree>> {
        if self.tree.is_none() {
            if let IconSource::Svg(svg_data) = &self.source {
                let options = usvg::Options::default();
                match usvg::Tree::from_str(svg_data, &options) {
                    Ok(tree) => {
                        self.tree = Some(Arc::new(tree));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse SVG icon: {}", e);
                        return None;
                    }
                }
            }
        }
        self.tree.as_ref()
    }

    /// Render the icon to RGBA pixel data
    fn render_to_pixels(&mut self, size: u32) -> Option<Vec<u8>> {
        let tree = self.ensure_parsed()?;

        // Create a pixmap to render into
        let mut pixmap = tiny_skia::Pixmap::new(size, size)?;

        // Calculate transform to fit icon in the target size
        let svg_size = tree.size();
        let scale_x = size as f32 / svg_size.width();
        let scale_y = size as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // Center the icon if aspect ratios don't match
        let scaled_width = svg_size.width() * scale;
        let scaled_height = svg_size.height() * scale;
        let offset_x = (size as f32 - scaled_width) / 2.0;
        let offset_y = (size as f32 - scaled_height) / 2.0;

        let transform = tiny_skia::Transform::from_translate(offset_x, offset_y)
            .post_scale(scale, scale);

        // Render the SVG
        resvg::render(tree, transform, &mut pixmap.as_mut());

        // Apply color tint if specified
        if let Some(tint) = &self.color {
            let pixels = pixmap.data_mut();
            let tint_r = (tint.color.red * 255.0) as u8;
            let tint_g = (tint.color.green * 255.0) as u8;
            let tint_b = (tint.color.blue * 255.0) as u8;

            // Replace RGB while preserving alpha
            for pixel in pixels.chunks_exact_mut(4) {
                let alpha = pixel[3];
                if alpha > 0 {
                    pixel[0] = tint_r;
                    pixel[1] = tint_g;
                    pixel[2] = tint_b;
                    // Keep original alpha
                }
            }
        }

        Some(pixmap.take())
    }
}

impl Element for Icon {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Icons have fixed size
        let style = Style {
            size: Size {
                width: Dimension::length(self.size),
                height: Dimension::length(self.size),
            },
            ..Default::default()
        };

        let node_id = ctx.request_layout(style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // For now, render a placeholder colored quad
        // TODO: Implement proper icon texture rendering
        // This requires adding icon texture support to the Metal renderer

        let render_size = (self.size * ctx.scale_factor) as u32;
        if let Some(_pixels) = self.render_to_pixels(render_size) {
            // TODO: Upload pixels to texture and render
            // For now, draw a placeholder rectangle
            let fill_color = self.color.clone().unwrap_or(Color::new(0.5, 0.5, 0.5, 1.0));

            ctx.paint_quad(PaintQuad {
                bounds,
                fill: fill_color,
                corner_radii: crate::geometry::Corners::all(2.0),
                border_widths: crate::geometry::Edges::zero(),
                border_color: crate::color::colors::TRANSPARENT,
            });
        }
    }
}

// Common icon paths (can be expanded)
pub mod icons {
    /// Checkmark icon
    pub const CHECK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>"#;

    /// X/Close icon
    pub const CLOSE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>"#;

    /// Plus/Add icon
    pub const PLUS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>"#;

    /// Minus/Remove icon
    pub const MINUS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line></svg>"#;

    /// Chevron right icon
    pub const CHEVRON_RIGHT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>"#;

    /// Chevron down icon
    pub const CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>"#;

    /// Menu/hamburger icon
    pub const MENU: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="3" y1="12" x2="21" y2="12"></line><line x1="3" y1="6" x2="21" y2="6"></line><line x1="3" y1="18" x2="21" y2="18"></line></svg>"#;

    /// Search/magnifying glass icon
    pub const SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>"#;

    /// Settings/gear icon
    pub const SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>"#;

    /// Trash/delete icon
    pub const TRASH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>"#;

    /// Edit/pencil icon
    pub const EDIT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>"#;
}

/// Create an icon-only button
pub fn icon_button(svg_data: impl Into<IconSource>) -> IconButton {
    IconButton::new(svg_data)
}

/// An icon-only clickable button
pub struct IconButton {
    /// The icon to display
    icon: Icon,
    /// Unique ID for interaction tracking
    id: crate::interaction::ElementId,
    /// Event handlers
    handlers: std::rc::Rc<std::cell::RefCell<crate::interaction::EventHandlers>>,
    /// Background color in normal state
    background: Color,
    /// Background color when hovered
    hover_background: Color,
    /// Background color when pressed
    press_background: Color,
    /// Corner radius
    corner_radius: f32,
    /// Padding around the icon
    padding: f32,
    /// Cached layout node
    node_id: Option<NodeId>,
}

impl IconButton {
    pub fn new(svg_data: impl Into<IconSource>) -> Self {
        Self {
            icon: Icon::new(svg_data),
            id: crate::interaction::ElementId::auto(),
            handlers: std::rc::Rc::new(std::cell::RefCell::new(
                crate::interaction::EventHandlers::new(),
            )),
            background: crate::color::colors::TRANSPARENT,
            hover_background: crate::color::colors::GRAY_200,
            press_background: crate::color::colors::GRAY_300,
            corner_radius: 4.0,
            padding: 8.0,
            node_id: None,
        }
    }

    /// Set the icon size
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon = self.icon.size(size);
        self
    }

    /// Set the icon color
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon = self.icon.color(color);
        self
    }

    /// Set the background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the hover background color
    pub fn hover_background(mut self, color: Color) -> Self {
        self.hover_background = color;
        self
    }

    /// Set the press background color
    pub fn press_background(mut self, color: Color) -> Self {
        self.press_background = color;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set the click handler
    /// Handler receives: (button, click_type, position, local_position, modifiers)
    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(
                crate::layer::MouseButton,
                crate::layer::ClickType,
                glam::Vec2,
                glam::Vec2,
                crate::layer::Modifiers,
            ) + 'static,
    {
        self.handlers.borrow_mut().on_click = Some(Box::new(handler));
        self
    }

    /// Set a simple click handler that doesn't need position info
    pub fn on_click_simple<F>(self, mut handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _, _, _| handler()));
        self
    }
}

impl Element for IconButton {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        let total_size = self.icon.size + self.padding * 2.0;
        let style = Style {
            size: Size {
                width: Dimension::length(total_size),
                height: Dimension::length(total_size),
            },
            ..Default::default()
        };

        let node_id = ctx.request_layout(style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        use crate::interaction::registry::{get_element_state, register_element};

        if !ctx.is_visible(&bounds) {
            return;
        }

        // Register for interaction
        register_element(self.id, self.handlers.clone());

        // Get current interaction state
        let state = get_element_state(self.id).unwrap_or_default();

        // Determine background color based on state
        let bg_color = if state.is_pressed {
            self.press_background
        } else if state.is_hovered {
            self.hover_background
        } else {
            self.background
        };

        // Paint background
        ctx.paint_quad(PaintQuad {
            bounds,
            fill: bg_color,
            corner_radii: crate::geometry::Corners::all(self.corner_radius),
            border_widths: crate::geometry::Edges::zero(),
            border_color: crate::color::colors::TRANSPARENT,
        });

        // Paint icon (centered within bounds accounting for padding)
        let icon_bounds = Rect::from_pos_size(
            bounds.pos + glam::Vec2::splat(self.padding),
            glam::Vec2::splat(self.icon.size),
        );
        self.icon.paint(icon_bounds, ctx);

        // Register for hit testing
        ctx.register_hit_test(self.id, bounds, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_creation() {
        let icon = icon(icons::CHECK);
        assert_eq!(icon.size, 24.0);
        assert!(icon.color.is_none());
    }

    #[test]
    fn test_icon_with_size() {
        let icon = icon(icons::CHECK).size(32.0);
        assert_eq!(icon.size, 32.0);
    }

    #[test]
    fn test_icon_with_color() {
        let icon = icon(icons::CHECK).color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert!(icon.color.is_some());
    }

    #[test]
    fn test_icon_source_from_str() {
        let source: IconSource = icons::CHECK.into();
        assert!(matches!(source, IconSource::Svg(_)));
    }

    #[test]
    fn test_icon_parsing() {
        let mut icon = icon(icons::CHECK);
        let tree = icon.ensure_parsed();
        assert!(tree.is_some());
    }

    #[test]
    fn test_icon_render_to_pixels() {
        let mut icon = icon(icons::CHECK);
        let pixels = icon.render_to_pixels(24);
        assert!(pixels.is_some());
        let pixels = pixels.unwrap();
        // Should be 24x24 RGBA
        assert_eq!(pixels.len(), 24 * 24 * 4);
    }
}
