//! Geometric primitives and utilities

use glam::Vec2;

/// A point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Vec2> for Point {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Point> for Vec2 {
    fn from(p: Point) -> Self {
        Vec2::new(p.x, p.y)
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub fn min(&self) -> Vec2 {
        self.pos
    }

    pub fn max(&self) -> Vec2 {
        self.pos + self.size
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.pos.x
            && point.y >= self.pos.y
            && point.x <= self.pos.x + self.size.x
            && point.y <= self.pos.y + self.size.y
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let min = self.min().max(other.min());
        let max = self.max().min(other.max());

        if min.x < max.x && min.y < max.y {
            Some(Rect::from_pos_size(min, max - min))
        } else {
            None
        }
    }

    /// Check if this rectangle is fully contained within another rectangle
    pub fn is_contained_in(&self, other: &Rect) -> bool {
        self.pos.x >= other.pos.x
            && self.pos.y >= other.pos.y
            && self.pos.x + self.size.x <= other.pos.x + other.size.x
            && self.pos.y + self.size.y <= other.pos.y + other.size.y
    }

    /// Check if this rectangle is partially visible within another rectangle
    pub fn is_partially_visible_in(&self, other: &Rect) -> bool {
        self.intersect(other).is_some()
    }

    /// Get the area of this rectangle
    pub fn area(&self) -> f32 {
        self.size.x * self.size.y
    }

    /// Get the visibility ratio when clipped by another rectangle (0.0 to 1.0)
    pub fn visibility_ratio_in(&self, other: &Rect) -> f32 {
        if let Some(intersection) = self.intersect(other) {
            let intersection_area = intersection.area();
            let total_area = self.area();
            if total_area > 0.0 {
                intersection_area / total_area
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Expand the rectangle by the given amount in all directions
    pub fn expand(&self, amount: f32) -> Rect {
        Rect::from_pos_size(
            self.pos - Vec2::splat(amount),
            self.size + Vec2::splat(amount * 2.0),
        )
    }

    /// Expand the rectangle by the given vector amount
    pub fn expand_by(&self, expansion: Vec2) -> Rect {
        Rect::from_pos_size(self.pos - expansion, self.size + expansion * 2.0)
    }

    /// Contract the rectangle by the given amount in all directions
    pub fn contract(&self, amount: f32) -> Rect {
        let new_size = (self.size - Vec2::splat(amount * 2.0)).max(Vec2::ZERO);
        Rect::from_pos_size(self.pos + Vec2::splat(amount), new_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

        assert!(rect.contains(Point::new(50.0, 50.0)));
        assert!(rect.contains(Point::new(10.0, 10.0)));
        assert!(rect.contains(Point::new(110.0, 110.0)));
        assert!(!rect.contains(Point::new(9.0, 50.0)));
        assert!(!rect.contains(Point::new(111.0, 50.0)));
    }

    #[test]
    fn test_rect_intersect() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);

        let intersection = rect1.intersect(&rect2).unwrap();
        assert_eq!(intersection.pos, Vec2::new(50.0, 50.0));
        assert_eq!(intersection.size, Vec2::new(50.0, 50.0));

        let rect3 = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert!(rect1.intersect(&rect3).is_none());
    }

    #[test]
    fn test_rect_visibility_ratio() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let viewport = Rect::new(50.0, 0.0, 100.0, 100.0);

        assert_eq!(rect.visibility_ratio_in(&viewport), 0.5);
        assert_eq!(rect.visibility_ratio_in(&rect), 1.0);

        let offscreen = Rect::new(200.0, 0.0, 100.0, 100.0);
        assert_eq!(rect.visibility_ratio_in(&offscreen), 0.0);
    }

    #[test]
    fn test_rect_expand_contract() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

        let expanded = rect.expand(10.0);
        assert_eq!(expanded.pos, Vec2::new(0.0, 0.0));
        assert_eq!(expanded.size, Vec2::new(120.0, 120.0));

        let contracted = rect.contract(10.0);
        assert_eq!(contracted.pos, Vec2::new(20.0, 20.0));
        assert_eq!(contracted.size, Vec2::new(80.0, 80.0));
    }

    #[test]
    fn test_point_creation() {
        let point = Point::new(15.0, 25.0);
        assert_eq!(point.x, 15.0);
        assert_eq!(point.y, 25.0);
    }

    #[test]
    fn test_point_from_vec2() {
        let vec = Vec2::new(30.0, 40.0);
        let point = Point::from(vec);
        assert_eq!(point.x, 30.0);
        assert_eq!(point.y, 40.0);
    }

    #[test]
    fn test_vec2_from_point() {
        let point = Point::new(50.0, 60.0);
        let vec = Vec2::from(point);
        assert_eq!(vec.x, 50.0);
        assert_eq!(vec.y, 60.0);
    }

    #[test]
    fn test_rect_from_pos_size() {
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(30.0, 40.0);
        let rect = Rect::from_pos_size(pos, size);
        
        assert_eq!(rect.pos, pos);
        assert_eq!(rect.size, size);
    }

    #[test]
    fn test_rect_min_max() {
        let rect = Rect::new(10.0, 15.0, 20.0, 25.0);
        
        assert_eq!(rect.min(), Vec2::new(10.0, 15.0));
        assert_eq!(rect.max(), Vec2::new(30.0, 40.0));
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0.0, 0.0, 10.0, 20.0);
        assert_eq!(rect.area(), 200.0);
        
        let zero_rect = Rect::new(0.0, 0.0, 0.0, 10.0);
        assert_eq!(zero_rect.area(), 0.0);
    }

    #[test]
    fn test_rect_is_contained_in() {
        let outer = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inner = Rect::new(10.0, 10.0, 50.0, 50.0);
        let overlapping = Rect::new(50.0, 50.0, 100.0, 100.0);
        let outside = Rect::new(150.0, 150.0, 20.0, 20.0);

        assert!(inner.is_contained_in(&outer));
        assert!(!overlapping.is_contained_in(&outer));
        assert!(!outside.is_contained_in(&outer));
        
        // A rect should be contained in itself
        assert!(outer.is_contained_in(&outer));
    }

    #[test]
    fn test_rect_is_partially_visible_in() {
        let viewport = Rect::new(0.0, 0.0, 100.0, 100.0);
        let fully_visible = Rect::new(10.0, 10.0, 50.0, 50.0);
        let partially_visible = Rect::new(80.0, 80.0, 50.0, 50.0);
        let not_visible = Rect::new(150.0, 150.0, 20.0, 20.0);

        assert!(fully_visible.is_partially_visible_in(&viewport));
        assert!(partially_visible.is_partially_visible_in(&viewport));
        assert!(!not_visible.is_partially_visible_in(&viewport));
    }

    #[test]
    fn test_rect_visibility_ratio_edge_cases() {
        let viewport = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        // Fully contained rect
        let inner = Rect::new(25.0, 25.0, 50.0, 50.0);
        assert_eq!(inner.visibility_ratio_in(&viewport), 1.0);
        
        // Not visible rect
        let outside = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert_eq!(outside.visibility_ratio_in(&viewport), 0.0);
        
        // Zero-area rect
        let zero_area = Rect::new(50.0, 50.0, 0.0, 0.0);
        assert_eq!(zero_area.visibility_ratio_in(&viewport), 0.0);
    }

    #[test]
    fn test_rect_expand_by() {
        let rect = Rect::new(10.0, 15.0, 20.0, 30.0);
        let expansion = Vec2::new(5.0, 8.0);
        
        let expanded = rect.expand_by(expansion);
        
        // Position should move by -expansion
        assert_eq!(expanded.pos, Vec2::new(5.0, 7.0));
        // Size should increase by 2 * expansion
        assert_eq!(expanded.size, Vec2::new(30.0, 46.0));
    }

    #[test]
    fn test_rect_contract_edge_cases() {
        let rect = Rect::new(10.0, 10.0, 20.0, 30.0);
        
        // Contract by small amount
        let contracted_small = rect.contract(2.0);
        assert_eq!(contracted_small.pos, Vec2::new(12.0, 12.0));
        assert_eq!(contracted_small.size, Vec2::new(16.0, 26.0));
        
        // Contract by amount larger than half the dimensions
        let contracted_large = rect.contract(15.0);
        assert_eq!(contracted_large.pos, Vec2::new(25.0, 25.0));
        assert_eq!(contracted_large.size, Vec2::ZERO); // Should clamp to zero
    }

    #[test]
    fn test_corners_all() {
        let corners = Corners::all(5.0);
        assert_eq!(corners.top_left, 5.0);
        assert_eq!(corners.top_right, 5.0);
        assert_eq!(corners.bottom_right, 5.0);
        assert_eq!(corners.bottom_left, 5.0);
    }

    #[test]
    fn test_corners_zero() {
        let corners = Corners::zero();
        assert_eq!(corners.top_left, 0.0);
        assert_eq!(corners.top_right, 0.0);
        assert_eq!(corners.bottom_right, 0.0);
        assert_eq!(corners.bottom_left, 0.0);
    }

    #[test]
    fn test_corners_default() {
        let corners = Corners::default();
        assert_eq!(corners, Corners::zero());
    }

    #[test]
    fn test_edges_all() {
        let edges = Edges::all(10.0);
        assert_eq!(edges.top, 10.0);
        assert_eq!(edges.right, 10.0);
        assert_eq!(edges.bottom, 10.0);
        assert_eq!(edges.left, 10.0);
    }

    #[test]
    fn test_edges_zero() {
        let edges = Edges::zero();
        assert_eq!(edges.top, 0.0);
        assert_eq!(edges.right, 0.0);
        assert_eq!(edges.bottom, 0.0);
        assert_eq!(edges.left, 0.0);
    }

    #[test]
    fn test_edges_xy() {
        let edges = Edges::xy(5.0, 8.0);
        assert_eq!(edges.top, 8.0);
        assert_eq!(edges.right, 5.0);
        assert_eq!(edges.bottom, 8.0);
        assert_eq!(edges.left, 5.0);
    }

    #[test]
    fn test_edges_default() {
        let edges = Edges::default();
        assert_eq!(edges, Edges::zero());
    }

    #[test]
    fn test_edges_horizontal() {
        let edges = Edges {
            top: 10.0,
            right: 5.0,
            bottom: 15.0,
            left: 8.0,
        };
        assert_eq!(edges.horizontal(), 13.0); // 5.0 + 8.0
    }

    #[test]
    fn test_edges_vertical() {
        let edges = Edges {
            top: 10.0,
            right: 5.0,
            bottom: 15.0,
            left: 8.0,
        };
        assert_eq!(edges.vertical(), 25.0); // 10.0 + 15.0
    }

    #[test]
    fn test_edges_size() {
        let edges = Edges {
            top: 2.0,
            right: 3.0,
            bottom: 4.0,
            left: 5.0,
        };
        let size = edges.size();
        assert_eq!(size.x, 8.0); // 3.0 + 5.0
        assert_eq!(size.y, 6.0); // 2.0 + 4.0
    }

    #[test]
    fn test_rect_contains_boundary_cases() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        
        // Test exact boundaries
        assert!(rect.contains(Point::new(10.0, 20.0))); // top-left corner
        assert!(rect.contains(Point::new(40.0, 60.0))); // bottom-right corner
        assert!(rect.contains(Point::new(10.0, 60.0))); // bottom-left corner
        assert!(rect.contains(Point::new(40.0, 20.0))); // top-right corner
        
        // Test just outside boundaries
        assert!(!rect.contains(Point::new(9.9, 30.0)));  // just left
        assert!(!rect.contains(Point::new(40.1, 30.0))); // just right
        assert!(!rect.contains(Point::new(25.0, 19.9))); // just above
        assert!(!rect.contains(Point::new(25.0, 60.1))); // just below
    }

    #[test]
    fn test_rect_intersect_edge_cases() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        
        // Touching but not overlapping (should be None)
        let touching_right = Rect::new(10.0, 0.0, 10.0, 10.0);
        assert!(rect.intersect(&touching_right).is_none());
        
        let touching_bottom = Rect::new(0.0, 10.0, 10.0, 10.0);
        assert!(rect.intersect(&touching_bottom).is_none());
        
        // Identical rectangles
        let identical = Rect::new(0.0, 0.0, 10.0, 10.0);
        let intersection = rect.intersect(&identical).unwrap();
        assert_eq!(intersection, rect);
    }

    #[test]
    fn test_point_equality() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(1.0, 2.0);
        let p3 = Point::new(1.0, 3.0);
        
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_rect_equality() {
        let r1 = Rect::new(1.0, 2.0, 3.0, 4.0);
        let r2 = Rect::new(1.0, 2.0, 3.0, 4.0);
        let r3 = Rect::new(1.0, 2.0, 3.0, 5.0);
        
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_corners_equality() {
        let c1 = Corners::all(5.0);
        let c2 = Corners::all(5.0);
        let c3 = Corners::all(6.0);
        
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_edges_equality() {
        let e1 = Edges::xy(3.0, 4.0);
        let e2 = Edges::xy(3.0, 4.0);
        let e3 = Edges::xy(3.0, 5.0);
        
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }
}

/// Corner radii for rounded rectangles
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    /// Create corners with all the same radius
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Create corners with no radius (square corners)
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Edge values (for borders, padding, margins, etc.)
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Create edges with all the same value
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create edges with no value
    pub fn zero() -> Self {
        Self::default()
    }

    /// Create edges with horizontal and vertical values
    pub fn xy(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Get the total horizontal size (left + right)
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get the total vertical size (top + bottom)
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// Get the total size as a Vec2
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.horizontal(), self.vertical())
    }
}
