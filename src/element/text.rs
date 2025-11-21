use crate::{
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    render::PaintText,
    style::TextStyle,
};
use taffy::prelude::*;

/// Create a new text element
pub fn text(content: impl Into<String>, style: TextStyle) -> Text {
    Text::new(content, style)
}

/// A simple text element
pub struct Text {
    content: String,
    style: TextStyle,
    node_id: Option<NodeId>,
}

impl Text {
    pub fn new(content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
            node_id: None,
        }
    }
}

impl Element for Text {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        let node_id = ctx.request_text_layout(Style::default(), &self.content, &self.style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        ctx.paint_text(PaintText {
            position: bounds.pos,
            text: self.content.clone(),
            style: self.style.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color::colors::*,
        style::TextStyle,
    };

    #[test]
    fn test_text_creation_with_text_function() {
        let style = TextStyle::default();
        let text_element = text("Hello World", style.clone());
        
        assert_eq!(text_element.content, "Hello World");
        assert_eq!(text_element.style, style);
        assert!(text_element.node_id.is_none());
    }

    #[test]
    fn test_text_new() {
        let style = TextStyle::default();
        let text_element = Text::new("Test Content", style.clone());
        
        assert_eq!(text_element.content, "Test Content");
        assert_eq!(text_element.style, style);
        assert!(text_element.node_id.is_none());
    }

    #[test]
    fn test_text_with_string() {
        let style = TextStyle::default();
        let content = String::from("Dynamic String");
        let text_element = Text::new(content, style.clone());
        
        assert_eq!(text_element.content, "Dynamic String");
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_with_str_slice() {
        let style = TextStyle::default();
        let content: &str = "String slice";
        let text_element = Text::new(content, style.clone());
        
        assert_eq!(text_element.content, "String slice");
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_with_custom_style() {
        let style = TextStyle {
            size: 24.0,
            color: RED,
        };
        let text_element = text("Styled Text", style.clone());
        
        assert_eq!(text_element.content, "Styled Text");
        assert_eq!(text_element.style.size, 24.0);
        assert_eq!(text_element.style.color, RED);
    }

    #[test]
    fn test_text_with_empty_string() {
        let style = TextStyle::default();
        let text_element = text("", style.clone());
        
        assert_eq!(text_element.content, "");
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_with_multiline_content() {
        let style = TextStyle::default();
        let content = "Line 1\nLine 2\nLine 3";
        let text_element = text(content, style.clone());
        
        assert_eq!(text_element.content, content);
        assert!(text_element.content.contains('\n'));
    }

    #[test]
    fn test_text_with_unicode_content() {
        let style = TextStyle::default();
        let content = "Hello 🌍 World! 日本語 العربية";
        let text_element = text(content, style.clone());
        
        assert_eq!(text_element.content, content);
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_with_special_characters() {
        let style = TextStyle::default();
        let content = "Special chars: !@#$%^&*()_+-={}[]|\\:;\"'<>?,./";
        let text_element = text(content, style.clone());
        
        assert_eq!(text_element.content, content);
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_style_variations() {
        let small_text = text("Small", TextStyle { size: 12.0, color: GRAY_600 });
        let large_text = text("Large", TextStyle { size: 48.0, color: BLACK });
        
        assert_eq!(small_text.style.size, 12.0);
        assert_eq!(small_text.style.color, GRAY_600);
        assert_eq!(large_text.style.size, 48.0);
        assert_eq!(large_text.style.color, BLACK);
    }

    #[test]
    fn test_text_content_modification() {
        let style = TextStyle::default();
        let mut text_element = Text::new("Original", style);
        
        // Modify content directly
        text_element.content = "Modified".to_string();
        assert_eq!(text_element.content, "Modified");
    }

    #[test]
    fn test_text_style_modification() {
        let mut text_element = Text::new("Test", TextStyle::default());
        
        // Modify style directly
        text_element.style.size = 20.0;
        text_element.style.color = BLUE;
        
        assert_eq!(text_element.style.size, 20.0);
        assert_eq!(text_element.style.color, BLUE);
    }

    #[test]
    fn test_text_node_id_initialization() {
        let style = TextStyle::default();
        let text_element = Text::new("Test", style);
        
        // Node ID should start as None
        assert!(text_element.node_id.is_none());
    }

    #[test]
    fn test_text_long_content() {
        let style = TextStyle::default();
        let long_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
        let text_element = text(&long_content, style.clone());
        
        assert_eq!(text_element.content.len(), long_content.len());
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_numeric_content() {
        let style = TextStyle::default();
        let number_content = format!("{}", 42.7891);
        let text_element = text(&number_content, style.clone());
        
        assert_eq!(text_element.content, "42.7891");
        assert_eq!(text_element.style, style);
    }

    #[test]
    fn test_text_boolean_content() {
        let style = TextStyle::default();
        let bool_content = format!("{}", true);
        let text_element = text(&bool_content, style.clone());
        
        assert_eq!(text_element.content, "true");
    }

    #[test]
    fn test_text_function_vs_new_equivalence() {
        let style = TextStyle::default();
        let content = "Equivalence Test";
        
        let text1 = text(content, style.clone());
        let text2 = Text::new(content, style);
        
        assert_eq!(text1.content, text2.content);
        assert_eq!(text1.style, text2.style);
        assert_eq!(text1.node_id, text2.node_id);
    }

    #[test]
    fn test_text_with_different_color_variations() {
        let colors = [WHITE, BLACK, RED, GREEN, BLUE, GRAY_500, PURPLE_400];
        
        for (i, &color) in colors.iter().enumerate() {
            let style = TextStyle { size: 16.0, color };
            let text_element = text(&format!("Color test {}", i), style);
            
            assert_eq!(text_element.style.color, color);
            assert_eq!(text_element.content, format!("Color test {}", i));
        }
    }

    #[test]
    fn test_text_with_size_variations() {
        let sizes = [8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 48.0, 72.0];
        
        for &size in &sizes {
            let style = TextStyle { size, color: BLACK };
            let text_element = text(&format!("Size {}", size), style);
            
            assert_eq!(text_element.style.size, size);
            assert_eq!(text_element.content, format!("Size {}", size));
        }
    }
}
