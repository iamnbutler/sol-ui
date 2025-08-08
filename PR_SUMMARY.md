# iOS Platform Support for sol-ui

## Summary

This PR adds comprehensive iOS platform support to sol-ui, enabling the framework to run on iOS devices (specifically targeting iPhone 8 with iOS 11-14). The implementation focuses on jailbroken devices for development flexibility while maintaining a clean platform abstraction that keeps the codebase maintainable.

## Key Changes

### Platform Architecture
- **New iOS platform module** (`src/platform/ios.rs`) with complete Window implementation using UIWindow/UIView
- **Unified platform abstraction** allowing seamless code sharing between macOS and iOS
- **Shared Metal renderer** between platforms for consistent rendering

### Input System
- **Touch event support** added to InputEvent enum (TouchDown, TouchUp, TouchMove, TouchCancel)
- **Automatic touch-to-mouse mapping** in InteractionSystem for backward compatibility
- **Multi-touch foundation** with touch ID tracking for future expansion

### Window Management
- iOS Window implementation with Metal layer setup
- Retina display support with proper content scaling
- Compatible API surface with macOS Window (Arc<Window>, same constructor signature)

### App Module Updates
- Platform-agnostic app initialization with conditional compilation
- Proper handling of iOS vs macOS lifecycle differences
- Removed hard dependencies on Cocoa for iOS builds

## Technical Details

### Dependencies
- Made `cocoa` macOS-specific
- Added `libc` for iOS builds
- Kept shared dependencies (metal, objc, core-foundation) platform-agnostic

### iOS Implementation Highlights
- Custom UIView subclass with CAMetalLayer backing
- Touch event handlers (touchesBegan, touchesMoved, touchesEnded, touchesCancelled)
- Event loop implementation suitable for jailbroken environments
- Objective-C class registration for custom view and delegate classes

### Testing
- Added 14 comprehensive tests covering:
  - Input event creation and handling
  - Touch event to mouse event mapping
  - Layer options and blend modes
  - Interaction system with touch events
- All tests passing on macOS (iOS tests require device/simulator)
- Marked complex iOS-specific tests with `test_todo!` for future MobileTestContext implementation

## Version Changes
- Version bumped from 0.0.1 to 0.1.0 (minor version for new feature)
- Updated project description to include iOS support

## Migration Guide
No breaking changes for existing macOS users. The platform abstraction ensures backward compatibility.

## Limitations & Future Work

### Current Limitations
- Requires jailbroken iOS device for easy deployment
- iOS example needs API updates to match current framework interfaces
- No App Store deployment configuration (intentional for this use case)

### Future Enhancements
- [ ] MobileTestContext for running tests on iOS devices/simulators
- [ ] Keyboard support on iOS
- [ ] Multi-touch gesture recognition
- [ ] iOS-specific UI adaptations (safe areas, etc.)
- [ ] Example app with full iOS UI patterns

## Testing Instructions

### macOS
```bash
cargo test
cargo run --example interactive_ui
```

### iOS (Jailbroken Device)
```bash
# Build for iOS
cargo build --target aarch64-apple-ios --release

# Deploy to device (requires SSH access)
# See plan/ios_support.md for detailed deployment instructions
```

## Files Changed

### Added
- `src/platform/ios.rs` - Complete iOS platform implementation
- `plan/ios_support.md` - Development planning and progress tracking
- `CHANGELOG.md` - Version history
- `claude.md` - Development guidelines

### Modified
- `Cargo.toml` - Platform-specific dependencies, version bump
- `src/platform.rs` - iOS platform module integration
- `src/platform/mac.rs` - Made metal_renderer public for sharing
- `src/layer.rs` - Added touch events to InputEvent enum
- `src/interaction/mod.rs` - Touch event handling and mapping
- `src/app.rs` - Platform-agnostic initialization

## Commits
- Add initial iOS platform support
- Add touch event support to InteractionSystem
- Add platform compatibility for app module
- Add comprehensive tests for iOS platform support
- Fix compiler warnings in tests
- Update plan with final iOS support status

## Review Notes
- The iOS implementation prioritizes compatibility with the existing API surface
- Touch events are automatically mapped to mouse events to maintain backward compatibility
- The implementation is designed for jailbroken devices as specified in requirements
- All platform-specific code is properly conditionally compiled
