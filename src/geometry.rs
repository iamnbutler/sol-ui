//! Geometric primitives and utilities with proper coordinate space semantics
//!
//! All points internally use Vec3 for future 3D compatibility, with z=1.0 as the default
//! for 2D operations.

use glam::Vec3;

// ================================================================================================
// MARK: - Coordinate Space Types
// ================================================================================================

/// A point in local/relative coordinate space (relative to parent element)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalPoint(Vec3);

/// A point in world/scene coordinate space (absolute position in the scene)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPoint(Vec3);

/// A point in screen/UI coordinate space (actual pixels/units on display)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint(Vec3);

// Common implementation for all point types
macro_rules! impl_point_type {
    ($type:ident) => {
        impl $type {
            /// Create a new 2D point (z defaults to 1.0)
            pub fn new(x: f32, y: f32) -> Self {
                Self(Vec3::new(x, y, 1.0))
            }

            /// Create a new 3D point
            pub fn new_3d(x: f32, y: f32, z: f32) -> Self {
                Self(Vec3::new(x, y, z))
            }

            /// Get x coordinate
            pub fn x(&self) -> f32 {
                self.0.x
            }

            /// Get y coordinate
            pub fn y(&self) -> f32 {
                self.0.y
            }

            /// Get z coordinate
            pub fn z(&self) -> f32 {
                self.0.z
            }

            /// Get as Vec3
            pub fn as_vec3(&self) -> Vec3 {
                self.0
            }

            /// Get as 2D coordinates (x, y)
            pub fn xy(&self) -> (f32, f32) {
                (self.0.x, self.0.y)
            }

            /// Zero point
            pub const ZERO: Self = Self(Vec3::ZERO);
        }

        impl From<(f32, f32)> for $type {
            fn from((x, y): (f32, f32)) -> Self {
                Self::new(x, y)
            }
        }

        impl From<Vec3> for $type {
            fn from(v: Vec3) -> Self {
                Self(v)
            }
        }
    };
}

impl_point_type!(LocalPoint);
impl_point_type!(WorldPoint);
impl_point_type!(ScreenPoint);

// ================================================================================================
// MARK: - Coordinate Space Conversions
// ================================================================================================

/// Convert from local to world coordinates
pub fn local_to_world(local: LocalPoint, parent_world: WorldPoint) -> WorldPoint {
    WorldPoint::from(local.as_vec3() + parent_world.as_vec3())
}

/// Convert from world to local coordinates
pub fn world_to_local(world: WorldPoint, parent_world: WorldPoint) -> LocalPoint {
    LocalPoint::from(world.as_vec3() - parent_world.as_vec3())
}

/// Convert from world to screen coordinates (temporary identity mapping)
/// TODO: Apply camera/viewport transformation
pub fn world_to_screen(world: WorldPoint) -> ScreenPoint {
    ScreenPoint::from(world.as_vec3())
}

/// Convert from screen to world coordinates (temporary identity mapping)
/// TODO: Apply inverse camera/viewport transformation
pub fn screen_to_world(screen: ScreenPoint) -> WorldPoint {
    WorldPoint::from(screen.as_vec3())
}

// ================================================================================================
// MARK: - Size Type
// ================================================================================================

/// A size in 2D or 3D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size(Vec3);

impl Size {
    /// Create a 2D size (depth defaults to 1.0)
    pub fn new(width: f32, height: f32) -> Self {
        Self(Vec3::new(width, height, 1.0))
    }

    /// Create a 3D size
    pub fn new_3d(width: f32, height: f32, depth: f32) -> Self {
        Self(Vec3::new(width, height, depth))
    }

    /// Get width
    pub fn width(&self) -> f32 {
        self.0.x
    }

    /// Get height
    pub fn height(&self) -> f32 {
        self.0.y
    }

    /// Get depth
    pub fn depth(&self) -> f32 {
        self.0.z
    }

    /// Get as Vec3
    pub fn as_vec3(&self) -> Vec3 {
        self.0
    }

    /// Zero size
    pub const ZERO: Self = Self(Vec3::ZERO);
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<Vec3> for Size {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

// ================================================================================================
// MARK: - Rectangle Type
// ================================================================================================

/// A rectangle defined by position and size in world space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub pos: WorldPoint,
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: WorldPoint::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Create from position and size
    pub fn from_pos_size(pos: WorldPoint, size: Size) -> Self {
        Self { pos, size }
    }

    /// Get minimum corner (top-left in screen space)
    pub fn min(&self) -> WorldPoint {
        self.pos
    }

    /// Get maximum corner (bottom-right in screen space)
    pub fn max(&self) -> WorldPoint {
        WorldPoint::from(self.pos.as_vec3() + self.size.as_vec3())
    }

    /// Check if rectangle contains a point
    pub fn contains(&self, point: WorldPoint) -> bool {
        let p = point.as_vec3();
        let min = self.pos.as_vec3();
        let max = min + self.size.as_vec3();

        p.x >= min.x && p.y >= min.y && p.x <= max.x && p.y <= max.y
    }

    /// Get intersection with another rectangle
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let min1 = self.min().as_vec3();
        let max1 = self.max().as_vec3();
        let min2 = other.min().as_vec3();
        let max2 = other.max().as_vec3();

        let min = min1.max(min2);
        let max = max1.min(max2);

        if min.x <= max.x && min.y <= max.y {
            Some(Rect::from_pos_size(
                WorldPoint::from(min),
                Size::from(max - min),
            ))
        } else {
            None
        }
    }

    /// Calculate visibility ratio when clipped by another rectangle
    pub fn visibility_ratio(&self, clip_rect: &Rect) -> f32 {
        if let Some(intersection) = self.intersect(clip_rect) {
            let intersection_area = intersection.size.width() * intersection.size.height();
            let total_area = self.size.width() * self.size.height();

            if total_area > 0.0 {
                intersection_area / total_area
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Expand rectangle by uniform amount
    pub fn expand(&self, amount: f32) -> Rect {
        let expansion = Vec3::new(amount, amount, 0.0);
        Rect::from_pos_size(
            WorldPoint::from(self.pos.as_vec3() - expansion),
            Size::from(self.size.as_vec3() + expansion * 2.0),
        )
    }

    /// Expand rectangle by vector amount
    pub fn expand_by(&self, expansion: Vec3) -> Rect {
        Rect::from_pos_size(
            WorldPoint::from(self.pos.as_vec3() - expansion),
            Size::from(self.size.as_vec3() + expansion * 2.0),
        )
    }

    /// Contract rectangle by uniform amount
    pub fn contract(&self, amount: f32) -> Rect {
        let contraction = Vec3::new(amount, amount, 0.0);
        let new_size = (self.size.as_vec3() - contraction * 2.0).max(Vec3::ZERO);
        Rect::from_pos_size(
            WorldPoint::from(self.pos.as_vec3() + contraction),
            Size::from(new_size),
        )
    }
}

// ================================================================================================
// MARK: - Corners Type
// ================================================================================================

/// Corner radii for rounded rectangles
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    /// Create new corners
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self { top_left, top_right, bottom_right, bottom_left }
    }

    /// Create uniform corners
    pub fn uniform(radius: f32) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    /// Zero corners (no rounding)
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Self::uniform(radius)
    }
}

impl From<[f32; 4]> for Corners {
    fn from(radii: [f32; 4]) -> Self {
        Self::new(radii[0], radii[1], radii[2], radii[3])
    }
}

// ================================================================================================
// MARK: - Edges Type
// ================================================================================================

/// Edge insets/padding/margins
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Create new edges
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Create uniform edges
    pub fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Create symmetric edges (vertical, horizontal)
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// Zero edges
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// Get total horizontal size
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical size
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// Get total size as Vec3 (for compatibility)
    pub fn size(&self) -> Vec3 {
        Vec3::new(self.horizontal(), self.vertical(), 0.0)
    }
}

impl From<f32> for Edges {
    fn from(value: f32) -> Self {
        Self::uniform(value)
    }
}

impl From<[f32; 4]> for Edges {
    fn from(values: [f32; 4]) -> Self {
        Self::new(values[0], values[1], values[2], values[3])
    }
}

impl From<(f32, f32)> for Edges {
    fn from((vertical, horizontal): (f32, f32)) -> Self {
        Self::symmetric(vertical, horizontal)
    }
}

// ================================================================================================
// MARK: - Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let local = LocalPoint::new(10.0, 20.0);
        assert_eq!(local.x(), 10.0);
        assert_eq!(local.y(), 20.0);
        assert_eq!(local.z(), 1.0); // Default z for 2D points

        let world = WorldPoint::new_3d(5.0, 10.0, 15.0);
        assert_eq!(world.z(), 15.0);
    }

    #[test]
    fn test_coordinate_conversions() {
        let parent = WorldPoint::new(100.0, 100.0);
        let local = LocalPoint::new(10.0, 20.0);

        let world = local_to_world(local, parent);
        assert_eq!(world.x(), 110.0);
        assert_eq!(world.y(), 120.0);

        let back_to_local = world_to_local(world, parent);
        assert_eq!(back_to_local.x(), 10.0);
        assert_eq!(back_to_local.y(), 20.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

        assert!(rect.contains(WorldPoint::new(50.0, 50.0)));
        assert!(rect.contains(WorldPoint::new(10.0, 10.0)));
        assert!(rect.contains(WorldPoint::new(110.0, 110.0)));
        assert!(!rect.contains(WorldPoint::new(9.0, 50.0)));
        assert!(!rect.contains(WorldPoint::new(111.0, 50.0)));
    }

    #[test]
    fn test_rect_intersect() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);

        let intersection = rect1.intersect(&rect2).unwrap();
        assert_eq!(intersection.pos.x(), 50.0);
        assert_eq!(intersection.pos.y(), 50.0);
        assert_eq!(intersection.size.width(), 50.0);
        assert_eq!(intersection.size.height(), 50.0);

        let rect3 = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert!(rect1.intersect(&rect3).is_none());
    }

    #[test]
    fn test_rect_visibility_ratio() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let clip = Rect::new(50.0, 50.0, 100.0, 100.0);

        let ratio = rect.visibility_ratio(&clip);
        assert_eq!(ratio, 0.25); // 25% visible
    }

    #[test]
    fn test_rect_expand_contract() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

        let expanded = rect.expand(10.0);
        assert_eq!(expanded.pos.x(), 0.0);
        assert_eq!(expanded.pos.y(), 0.0);
        assert_eq!(expanded.size.width(), 120.0);
        assert_eq!(expanded.size.height(), 120.0);

        let contracted = rect.contract(10.0);
        assert_eq!(contracted.pos.x(), 20.0);
        assert_eq!(contracted.pos.y(), 20.0);
        assert_eq!(contracted.size.width(), 80.0);
        assert_eq!(contracted.size.height(), 80.0);
    }

    #[test]
    fn test_edges() {
        let edges = Edges::uniform(10.0);
        assert_eq!(edges.horizontal(), 20.0);
        assert_eq!(edges.vertical(), 20.0);

        let edges2 = Edges::symmetric(5.0, 10.0);
        assert_eq!(edges2.top, 5.0);
        assert_eq!(edges2.bottom, 5.0);
        assert_eq!(edges2.left, 10.0);
        assert_eq!(edges2.right, 10.0);
    }
}
