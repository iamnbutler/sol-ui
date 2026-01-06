//! List element with per-item actions
//!
//! A scrollable list that renders items from data with support for:
//! - Item selection (single/multi)
//! - Hover reveal action buttons
//! - Empty state display
//! - Loading state
//!
//! Future features (require drag gesture support in interaction system):
//! - Swipe-to-delete gesture
//! - Item reordering via drag

use crate::{
    color::{colors, Color, ColorExt},
    element::{Element, LayoutContext, PaintContext, text, Text},
    entity::{Entity, new_entity, read_entity, update_entity},
    geometry::{Corners, Edges, Rect},
    interaction::{ElementId, EventHandlers, registry::register_element},
    render::PaintQuad,
    style::TextStyle,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use taffy::prelude::*;

/// Selection mode for the list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// No selection allowed
    #[default]
    None,
    /// Only one item can be selected at a time
    Single,
    /// Multiple items can be selected
    Multi,
}

/// State for a list, persisted via the Entity system
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Currently selected item indices
    pub selected: HashSet<usize>,
    /// Currently hovered item index
    pub hovered: Option<usize>,
    /// Whether the list is in loading state
    pub is_loading: bool,
}

impl ListState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an item is selected
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    /// Toggle selection for an item
    pub fn toggle_selection(&mut self, index: usize, mode: SelectionMode) {
        match mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                if self.selected.contains(&index) {
                    self.selected.clear();
                } else {
                    self.selected.clear();
                    self.selected.insert(index);
                }
            }
            SelectionMode::Multi => {
                if self.selected.contains(&index) {
                    self.selected.remove(&index);
                } else {
                    self.selected.insert(index);
                }
            }
        }
    }

    /// Select a single item (clearing others in single mode)
    pub fn select(&mut self, index: usize, mode: SelectionMode) {
        match mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.selected.clear();
                self.selected.insert(index);
            }
            SelectionMode::Multi => {
                self.selected.insert(index);
            }
        }
    }

    /// Deselect an item
    pub fn deselect(&mut self, index: usize) {
        self.selected.remove(&index);
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }
}

/// Action button configuration for list items
#[derive(Clone)]
pub struct ListAction {
    /// Label for the action button
    pub label: String,
    /// Icon (optional, for future use)
    pub icon: Option<String>,
    /// Background color
    pub color: Color,
    /// Callback when action is triggered
    pub on_click: Rc<RefCell<Box<dyn FnMut(usize)>>>,
}

impl ListAction {
    pub fn new(label: impl Into<String>, color: Color, on_click: impl FnMut(usize) + 'static) -> Self {
        Self {
            label: label.into(),
            icon: None,
            color,
            on_click: Rc::new(RefCell::new(Box::new(on_click))),
        }
    }

    /// Create a delete action with red color
    pub fn delete(on_click: impl FnMut(usize) + 'static) -> Self {
        Self::new("Delete", colors::RED_500, on_click)
    }

    /// Create an edit action with blue color
    pub fn edit(on_click: impl FnMut(usize) + 'static) -> Self {
        Self::new("Edit", colors::BLUE_500, on_click)
    }
}

/// Data for a single list item
pub struct ListItemData {
    /// Primary text
    pub title: String,
    /// Secondary text (optional)
    pub subtitle: Option<String>,
    /// Whether this item is disabled
    pub disabled: bool,
}

impl ListItemData {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            disabled: false,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Create a new list element
pub fn list<T: Into<ListItemData>>(items: impl IntoIterator<Item = T>) -> List {
    List::new(items)
}

/// A list element that renders items from data
pub struct List {
    /// Item data
    items: Vec<ListItemData>,
    /// Selection mode
    selection_mode: SelectionMode,
    /// Actions available on each item (shown on hover)
    actions: Vec<ListAction>,
    /// Callback when selection changes
    on_selection_change: Option<Rc<RefCell<Box<dyn FnMut(&HashSet<usize>)>>>>,
    /// Callback when item is clicked
    on_item_click: Option<Rc<RefCell<Box<dyn FnMut(usize)>>>>,
    /// Item height
    item_height: f32,
    /// Gap between items
    gap: f32,
    /// Background color
    background: Option<Color>,
    /// Item background color
    item_background: Color,
    /// Selected item background color
    selected_background: Color,
    /// Hovered item background color
    hovered_background: Color,
    /// Title text style
    title_style: TextStyle,
    /// Subtitle text style
    subtitle_style: TextStyle,
    /// Padding inside items
    item_padding: f32,
    /// Corner radius for items
    item_corner_radius: f32,
    /// Border color for the list container
    border_color: Option<Color>,
    /// Border width for the list container
    border_width: f32,
    /// Corner radius for the list container
    corner_radius: f32,
    /// Custom empty state element
    empty_state: Option<Box<dyn Element>>,
    /// Custom loading state element
    loading_state: Option<Box<dyn Element>>,
    /// Layout style
    style: Style,
    /// Persistent state
    state: Option<Entity<ListState>>,
    /// Cached layout node ID
    node_id: Option<NodeId>,
    /// Child node IDs for items
    child_nodes: Vec<NodeId>,
    /// Rendered item elements
    item_elements: Vec<ListItemElement>,
}

impl List {
    pub fn new<T: Into<ListItemData>>(items: impl IntoIterator<Item = T>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            selection_mode: SelectionMode::None,
            actions: Vec::new(),
            on_selection_change: None,
            on_item_click: None,
            item_height: 48.0,
            gap: 1.0,
            background: None,
            item_background: colors::WHITE,
            selected_background: colors::BLUE_400.with_alpha(0.2),
            hovered_background: colors::GRAY_100,
            title_style: TextStyle {
                size: 14.0,
                color: colors::GRAY_900,
                ..Default::default()
            },
            subtitle_style: TextStyle {
                size: 12.0,
                color: colors::GRAY_500,
                ..Default::default()
            },
            item_padding: 12.0,
            item_corner_radius: 4.0,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            empty_state: None,
            loading_state: None,
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Style::default()
            },
            state: None,
            node_id: None,
            child_nodes: Vec::new(),
            item_elements: Vec::new(),
        }
    }

    /// Set the selection mode
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Enable single selection
    pub fn single_select(mut self) -> Self {
        self.selection_mode = SelectionMode::Single;
        self
    }

    /// Enable multi selection
    pub fn multi_select(mut self) -> Self {
        self.selection_mode = SelectionMode::Multi;
        self
    }

    /// Add an action to all items
    pub fn action(mut self, action: ListAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set callback for selection changes
    pub fn on_selection_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&HashSet<usize>) + 'static,
    {
        self.on_selection_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set callback for item clicks
    pub fn on_item_click<F>(mut self, handler: F) -> Self
    where
        F: FnMut(usize) + 'static,
    {
        self.on_item_click = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set item height
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Set gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set item background color
    pub fn item_background(mut self, color: Color) -> Self {
        self.item_background = color;
        self
    }

    /// Set selected item background color
    pub fn selected_background(mut self, color: Color) -> Self {
        self.selected_background = color;
        self
    }

    /// Set hovered item background color
    pub fn hovered_background(mut self, color: Color) -> Self {
        self.hovered_background = color;
        self
    }

    /// Set title text style
    pub fn title_style(mut self, style: TextStyle) -> Self {
        self.title_style = style;
        self
    }

    /// Set subtitle text style
    pub fn subtitle_style(mut self, style: TextStyle) -> Self {
        self.subtitle_style = style;
        self
    }

    /// Set item padding
    pub fn item_padding(mut self, padding: f32) -> Self {
        self.item_padding = padding;
        self
    }

    /// Set item corner radius
    pub fn item_corner_radius(mut self, radius: f32) -> Self {
        self.item_corner_radius = radius;
        self
    }

    /// Set the border (color and width)
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    /// Set only the border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set only the border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set corner radius for the list container
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set custom empty state element
    pub fn empty_state(mut self, element: impl Element + 'static) -> Self {
        self.empty_state = Some(Box::new(element));
        self
    }

    /// Set custom loading state element
    pub fn loading_state(mut self, element: impl Element + 'static) -> Self {
        self.loading_state = Some(Box::new(element));
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::length(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::length(height);
        self
    }

    /// Set width to 100%
    pub fn width_full(mut self) -> Self {
        self.style.size.width = Dimension::percent(1.0);
        self
    }

    /// Set height to 100%
    pub fn height_full(mut self) -> Self {
        self.style.size.height = Dimension::percent(1.0);
        self
    }

    /// Set flex grow
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Set the list to loading state
    pub fn set_loading(&self, loading: bool) {
        if let Some(ref state) = self.state {
            update_entity(state, |s| {
                s.is_loading = loading;
            });
        }
    }

    /// Get the current selection
    pub fn get_selection(&self) -> HashSet<usize> {
        self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.selected.clone()))
            .unwrap_or_default()
    }

    /// Get the state entity for external control
    pub fn state_entity(&self) -> Option<Entity<ListState>> {
        self.state.clone()
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new(Vec::<ListItemData>::new())
    }
}

/// Internal element for rendering a single list item
struct ListItemElement {
    index: usize,
    title: Text,
    subtitle: Option<Text>,
    title_node: Option<NodeId>,
    subtitle_node: Option<NodeId>,
    node_id: Option<NodeId>,
    element_id: ElementId,
    handlers: Rc<RefCell<EventHandlers>>,
}

impl ListItemElement {
    fn new(
        index: usize,
        data: &ListItemData,
        title_style: TextStyle,
        subtitle_style: TextStyle,
        state: Entity<ListState>,
        selection_mode: SelectionMode,
        on_item_click: Option<Rc<RefCell<Box<dyn FnMut(usize)>>>>,
        on_selection_change: Option<Rc<RefCell<Box<dyn FnMut(&HashSet<usize>)>>>>,
    ) -> Self {
        let title = text(data.title.clone(), title_style);
        let subtitle = data.subtitle.as_ref().map(|s| text(s.clone(), subtitle_style));

        // Create handlers for this item
        let handlers = Rc::new(RefCell::new(EventHandlers::new()));

        // Set up click handler for selection
        let state_for_click = state.clone();
        let on_selection_change_for_click = on_selection_change.clone();
        let on_item_click_for_click = on_item_click.clone();
        let item_index = index;

        handlers.borrow_mut().on_click = Some(Box::new(move |_button, _pos, _local_pos| {
            // Toggle selection
            update_entity(&state_for_click, |s| {
                s.toggle_selection(item_index, selection_mode);
            });

            // Fire selection change callback
            if let Some(ref callback) = on_selection_change_for_click {
                if let Some(selected) = read_entity(&state_for_click, |s| s.selected.clone()) {
                    (callback.borrow_mut())(&selected);
                }
            }

            // Fire item click callback
            if let Some(ref callback) = on_item_click_for_click {
                (callback.borrow_mut())(item_index);
            }
        }));

        // Set up hover handlers
        let state_for_enter = state.clone();
        let item_index_enter = index;
        handlers.borrow_mut().on_mouse_enter = Some(Box::new(move || {
            update_entity(&state_for_enter, |s| {
                s.hovered = Some(item_index_enter);
            });
        }));

        let state_for_leave = state.clone();
        let item_index_leave = index;
        handlers.borrow_mut().on_mouse_leave = Some(Box::new(move || {
            update_entity(&state_for_leave, |s| {
                if s.hovered == Some(item_index_leave) {
                    s.hovered = None;
                }
            });
        }));

        Self {
            index,
            title,
            subtitle,
            title_node: None,
            subtitle_node: None,
            node_id: None,
            element_id: ElementId::auto(),
            handlers,
        }
    }
}

impl Element for List {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Initialize state entity if not already done
        if self.state.is_none() {
            self.state = Some(new_entity(ListState::new()));
        }

        // Check if we're in loading state
        let is_loading = self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.is_loading))
            .unwrap_or(false);

        // If loading, just render loading state
        if is_loading {
            if let Some(ref mut loading) = self.loading_state {
                let child_node = loading.layout(ctx);
                self.child_nodes = vec![child_node];
                let node_id = ctx.request_layout_with_children(self.style.clone(), &self.child_nodes);
                self.node_id = Some(node_id);
                return node_id;
            }
        }

        // If empty, render empty state
        if self.items.is_empty() {
            if let Some(ref mut empty) = self.empty_state {
                let child_node = empty.layout(ctx);
                self.child_nodes = vec![child_node];
                let node_id = ctx.request_layout_with_children(self.style.clone(), &self.child_nodes);
                self.node_id = Some(node_id);
                return node_id;
            }
            // No custom empty state, just return empty container
            let node_id = ctx.request_layout(self.style.clone());
            self.node_id = Some(node_id);
            return node_id;
        }

        // Create item elements
        self.item_elements.clear();
        self.child_nodes.clear();

        // Get state entity for handlers (must exist after init above)
        let state = self.state.clone().unwrap();

        for (index, item_data) in self.items.iter().enumerate() {
            let mut item_element = ListItemElement::new(
                index,
                item_data,
                self.title_style.clone(),
                self.subtitle_style.clone(),
                state.clone(),
                self.selection_mode,
                self.on_item_click.clone(),
                self.on_selection_change.clone(),
            );

            // Layout title
            let title_node = item_element.title.layout(ctx);
            item_element.title_node = Some(title_node);

            // Layout subtitle if present
            let subtitle_node = if let Some(ref mut subtitle) = item_element.subtitle {
                let node = subtitle.layout(ctx);
                item_element.subtitle_node = Some(node);
                Some(node)
            } else {
                None
            };

            // Create item container style
            let item_style = Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: Some(JustifyContent::Center),
                padding: taffy::Rect {
                    left: LengthPercentage::length(self.item_padding),
                    right: LengthPercentage::length(self.item_padding),
                    top: LengthPercentage::length(self.item_padding / 2.0),
                    bottom: LengthPercentage::length(self.item_padding / 2.0),
                },
                min_size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::length(self.item_height),
                },
                gap: Size {
                    width: LengthPercentage::length(0.0),
                    height: LengthPercentage::length(2.0),
                },
                ..Style::default()
            };

            // Create item node with children
            let children: Vec<NodeId> = if let Some(sn) = subtitle_node {
                vec![title_node, sn]
            } else {
                vec![title_node]
            };

            let item_node = ctx.request_layout_with_children(item_style, &children);
            item_element.node_id = Some(item_node);

            self.child_nodes.push(item_node);
            self.item_elements.push(item_element);
        }

        // Update container style with gap
        let mut container_style = self.style.clone();
        container_style.gap = Size {
            width: LengthPercentage::length(0.0),
            height: LengthPercentage::length(self.gap),
        };

        let node_id = ctx.request_layout_with_children(container_style, &self.child_nodes);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Paint background and border
        if self.background.is_some() || self.border_color.is_some() {
            ctx.paint_quad(PaintQuad {
                bounds,
                fill: self.background.unwrap_or(colors::TRANSPARENT),
                corner_radii: Corners::all(self.corner_radius),
                border_widths: Edges::all(self.border_width),
                border_color: self.border_color.unwrap_or(colors::TRANSPARENT),
            });
        }

        // Check if we're in loading state
        let is_loading = self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.is_loading))
            .unwrap_or(false);

        // Paint loading state
        if is_loading {
            if let Some(ref mut loading) = self.loading_state {
                if let Some(&child_node) = self.child_nodes.first() {
                    let child_bounds = ctx.layout_engine.layout_bounds(child_node);
                    let absolute_bounds = Rect::from_pos_size(
                        bounds.pos + child_bounds.pos,
                        child_bounds.size,
                    );
                    loading.paint(absolute_bounds, ctx);
                }
            }
            return;
        }

        // Paint empty state
        if self.items.is_empty() {
            if let Some(ref mut empty) = self.empty_state {
                if let Some(&child_node) = self.child_nodes.first() {
                    let child_bounds = ctx.layout_engine.layout_bounds(child_node);
                    let absolute_bounds = Rect::from_pos_size(
                        bounds.pos + child_bounds.pos,
                        child_bounds.size,
                    );
                    empty.paint(absolute_bounds, ctx);
                }
            }
            return;
        }

        // Get current state
        let (selected, hovered) = self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| (state.selected.clone(), state.hovered)))
            .unwrap_or_default();

        // Paint items
        for (item_element, &item_node) in self.item_elements.iter_mut().zip(&self.child_nodes) {
            let item_bounds = ctx.layout_engine.layout_bounds(item_node);
            let absolute_bounds = Rect::from_pos_size(
                bounds.pos + item_bounds.pos,
                item_bounds.size,
            );

            if !ctx.is_visible(&absolute_bounds) {
                continue;
            }

            let index = item_element.index;
            let is_selected = selected.contains(&index);
            let is_hovered = hovered == Some(index);

            // Determine background color
            let bg_color = if is_selected {
                self.selected_background
            } else if is_hovered {
                self.hovered_background
            } else {
                self.item_background
            };

            // Paint item background
            ctx.paint_quad(PaintQuad {
                bounds: absolute_bounds,
                fill: bg_color,
                corner_radii: Corners::all(self.item_corner_radius),
                border_widths: Edges::zero(),
                border_color: colors::TRANSPARENT,
            });

            // Paint title
            if let Some(title_node) = item_element.title_node {
                let title_bounds = ctx.layout_engine.layout_bounds(title_node);
                let title_absolute = Rect::from_pos_size(
                    absolute_bounds.pos + title_bounds.pos,
                    title_bounds.size,
                );
                item_element.title.paint(title_absolute, ctx);
            }

            // Paint subtitle
            if let (Some(subtitle), Some(subtitle_node)) =
                (&mut item_element.subtitle, item_element.subtitle_node)
            {
                let subtitle_bounds = ctx.layout_engine.layout_bounds(subtitle_node);
                let subtitle_absolute = Rect::from_pos_size(
                    absolute_bounds.pos + subtitle_bounds.pos,
                    subtitle_bounds.size,
                );
                subtitle.paint(subtitle_absolute, ctx);
            }

            // Paint action buttons on hover
            if is_hovered && !self.actions.is_empty() {
                let action_button_height = 28.0;
                let action_button_padding = 8.0;
                let action_gap = 4.0;

                // Calculate total width of all action buttons
                let mut total_actions_width = 0.0;
                for action in &self.actions {
                    let approx_text_width = action.label.len() as f32 * 7.0;
                    total_actions_width += approx_text_width + action_button_padding * 2.0 + action_gap;
                }
                total_actions_width -= action_gap; // Remove last gap

                // Position actions on right side of item
                let actions_start_x = absolute_bounds.pos.x + absolute_bounds.size.x - total_actions_width - self.item_padding;
                let actions_y = absolute_bounds.pos.y + (absolute_bounds.size.y - action_button_height) / 2.0;

                let mut current_x = actions_start_x;
                for (action_idx, action) in self.actions.iter().enumerate() {
                    let approx_text_width = action.label.len() as f32 * 7.0;
                    let button_width = approx_text_width + action_button_padding * 2.0;

                    let button_bounds = Rect::from_pos_size(
                        glam::Vec2::new(current_x, actions_y),
                        glam::Vec2::new(button_width, action_button_height),
                    );

                    // Paint button background
                    ctx.paint_quad(PaintQuad {
                        bounds: button_bounds,
                        fill: action.color,
                        corner_radii: Corners::all(4.0),
                        border_widths: Edges::zero(),
                        border_color: colors::TRANSPARENT,
                    });

                    // Paint button label
                    let text_style = TextStyle {
                        size: 12.0,
                        color: colors::WHITE,
                        ..Default::default()
                    };
                    ctx.paint_text(crate::render::PaintText {
                        position: glam::Vec2::new(
                            button_bounds.pos.x + action_button_padding,
                            button_bounds.pos.y + (action_button_height - text_style.size) / 2.0,
                        ),
                        text: action.label.clone(),
                        style: text_style,
                        measured_size: None,
                    });

                    // Create unique element ID for this action button
                    let action_id = ElementId::new(
                        item_element.element_id.0.wrapping_add((action_idx + 1000) as u64)
                    );

                    // Create handler for action button
                    let action_handlers = Rc::new(RefCell::new(EventHandlers::new()));
                    let on_action = action.on_click.clone();
                    let item_idx = index;
                    action_handlers.borrow_mut().on_click = Some(Box::new(move |_btn, _pos, _local| {
                        (on_action.borrow_mut())(item_idx);
                    }));

                    // Register action button for interaction (higher z-index to be on top)
                    register_element(action_id, action_handlers);
                    ctx.register_hit_test(action_id, button_bounds, 1);

                    current_x += button_width + action_gap;
                }
            }

            // Register element for interaction and hit testing
            register_element(item_element.element_id, item_element.handlers.clone());
            ctx.register_hit_test(item_element.element_id, absolute_bounds, 0);
        }
    }
}

// Implement From for common types to make list creation easier
impl From<String> for ListItemData {
    fn from(title: String) -> Self {
        ListItemData::new(title)
    }
}

impl From<&str> for ListItemData {
    fn from(title: &str) -> Self {
        ListItemData::new(title)
    }
}

impl From<(String, String)> for ListItemData {
    fn from((title, subtitle): (String, String)) -> Self {
        ListItemData::new(title).subtitle(subtitle)
    }
}

impl From<(&str, &str)> for ListItemData {
    fn from((title, subtitle): (&str, &str)) -> Self {
        ListItemData::new(title).subtitle(subtitle)
    }
}
