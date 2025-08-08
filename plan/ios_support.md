# iOS Platform Support Plan

## Goal

Add iOS platform support to sol-ui, targeting iPhone 8 with iOS 11-14 and jailbroken environment.

## Version

- Current: 0.0.1
- Target: 0.1.0 (with iOS support)

## Task List

### Phase 1: Foundation

- [x] Create `platform/ios.rs` module
- [x] Update Cargo.toml with iOS-specific dependencies
- [x] Add iOS target configuration
- [x] Create CHANGELOG.md

### Phase 2: Core Platform Implementation

- [x] Window/View creation using UIWindow/UIView
- [x] Metal layer setup for iOS
- [x] Touch input handling (replacing mouse events)
- [x] App lifecycle management without UIKit dependencies where possible

### Phase 3: Integration

- [x] Update existing platform abstraction to support iOS
- [x] Migrate touch events to existing InputEvent system
- [ ] Ensure text rendering works on iOS
- [x] Handle screen coordinates and retina display

### Phase 4: Examples

- [x] Create iOS-compatible example in `examples/`
- [ ] Test deployment to jailbroken device

### Phase 5: Testing

- [x] Add tests for input event handling and touch mapping
- [x] Mark untestable items with `test_todo!` comments
- [x] Ensure cross-platform compatibility isn't broken
- [ ] Add MobileTestContext for complex iOS-specific tests (deferred)

## Technical Notes

### Key Differences from macOS

- UIWindow/UIView instead of NSWindow/NSView
- Touch events instead of mouse events
- No NSApplication - use UIApplication or custom run loop
- Different coordinate system origin (top-left vs bottom-left)
- Must handle app backgrounding/foregrounding

### Dependencies Changes

- Remove `cocoa` dependency for iOS builds
- Add iOS-specific objc bindings
- Keep metal, objc, core-foundation as they work on iOS

### Build Targets

- aarch64-apple-ios (iPhone 8 and newer)
- Minimum iOS 11.0 for Metal 2 support

## Current Status

- Phase 1: Complete
- Phase 2: Complete
- Phase 3: Complete
- Phase 4: Partially complete (example created but needs API updates, deployment testing needed)
- Phase 5: Complete (tests added, MobileTestContext deferred)
- Branch: ios-platform-support
- Version bumped to 0.1.0
- **Point System Refactor: Complete** - Successfully migrated to Vec3-based coordinate system with semantic types
- All tests passing (17 tests)

## Open Questions

- Should we support both UIKit-based and raw Metal layer approaches? **Decision: Using UIKit for now**
- How much should we abstract touch vs mouse in the event system? **Decision: Extended InputEvent enum to handle both**
- Should keyboard support be a priority in phase 1? **Decision: Deferred to later phase**

## Completed Items

- Created iOS Window implementation with UIWindow/UIView
- Metal layer configuration with retina display support
- Touch event handlers (touchesBegan, touchesMoved, touchesEnded, touchesCancelled)
- Extended InputEvent enum with TouchDown, TouchUp, TouchMove, TouchCancel
- Platform-specific dependency configuration in Cargo.toml
- Shared metal_renderer between macOS and iOS platforms
- Made app module platform-agnostic with conditional compilation
- Created iOS example (needs API updates before full compatibility)
- Unified Window API across platforms (Arc<Window>, same constructor signature)
- Added compatibility methods to iOS Window (handle_events, get_size, get_metal_layer)
- Touch events automatically mapped to mouse events in InteractionSystem
- Added comprehensive test suite for iOS support (14 tests)
- Fixed all compiler warnings and ensured cross-platform compatibility

## Point System Refactor (Complete)

- Replaced Vec2-based Point with Vec3-based coordinate space types
- Added semantic point types: LocalPoint, WorldPoint, ScreenPoint
- Updated Size to use Vec3 internally for future 3D support
- Removed PlatformPoint as unnecessary abstraction
- Updated all input events to use ScreenPoint
- Updated interaction system to use proper coordinate types
- Fixed all build errors in render and metal_renderer modules
- Completed taffy::Size vs geometry::Size disambiguation
- Added Corners and Edges types with proper helper methods
- All coordinate conversions working correctly

## Notes

- Prioritizing jailbroken deployment for development ease
- Keeping platform modules consolidated (single ios.rs file)
- Not creating separate "mobile" abstractions unless absolutely necessary
