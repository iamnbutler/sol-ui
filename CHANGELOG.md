# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-08

### Added
- iOS platform support targeting iPhone 8 with iOS 11-14
- Touch input events (TouchDown, TouchUp, TouchMove, TouchCancel)
- iOS-specific platform module with UIWindow/UIView integration
- Platform-specific dependency configuration in Cargo.toml
- Support for jailbroken iOS deployment
- Platform abstraction for desktop vs mobile differences

### Changed
- Updated project description to include iOS support
- Reorganized platform modules to support multiple targets
- Extended InputEvent enum to handle both mouse and touch events
- Made cocoa dependency macOS-specific
- Updated edition from 2024 to 2021 for broader compatibility

### Fixed
- Platform module conditional compilation for iOS targets

## [0.0.1] - Initial Release

### Added
- Initial macOS support via Metal
- Basic windowing system
- Layer-based rendering architecture
- Text rendering system
- Layout engine using Taffy
- Interactive UI elements
- Example applications
