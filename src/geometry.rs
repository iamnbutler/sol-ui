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
