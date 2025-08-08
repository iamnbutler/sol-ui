# iOS Platform Support Plan

## Goal
Add iOS platform support to sol-ui, targeting iPhone 8 with iOS 11-14 and jailbroken environment.

## Version
- Current: 0.0.1
- Target: 0.1.0 (with iOS support)

## Task List

### Phase 1: Foundation
- [ ] Create `platform/ios.rs` module
- [ ] Update Cargo.toml with iOS-specific dependencies
- [ ] Add iOS target configuration
- [ ] Create CHANGELOG.md

### Phase 2: Core Platform Implementation
- [ ] Window/View creation using UIWindow/UIView
- [ ] Metal layer setup for iOS
- [ ] Touch input handling (replacing mouse events)
- [ ] App lifecycle management without UIKit dependencies where possible

### Phase 3: Integration
- [ ] Update existing platform abstraction to support iOS
- [ ] Migrate touch events to existing InputEvent system
- [ ] Ensure text rendering works on iOS
- [ ] Handle screen coordinates and retina display

### Phase 4: Examples
- [ ] Create iOS-compatible example in `examples/`
- [ ] Test deployment to jailbroken device

### Phase 5: Testing
- [ ] Add MobileTestContext for complex iOS-specific tests
- [ ] Mark untestable items with `test_todo!` comments
- [ ] Ensure cross-platform compatibility isn't broken

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
- Starting implementation
- Branch: ios-platform-support

## Open Questions
- Should we support both UIKit-based and raw Metal layer approaches?
- How much should we abstract touch vs mouse in the event system?
- Should keyboard support be a priority in phase 1?

## Notes
- Prioritizing jailbroken deployment for development ease
- Keeping platform modules consolidated (single ios.rs file)
- Not creating separate "mobile" abstractions unless absolutely necessary
