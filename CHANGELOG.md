# Changelog

## [0.3.0] - 2026-03-17

### Added
- SVG pet image (Siberian Forest Cat with tiger stripes)
- 5 mood states: Idle, Happy, Sleepy, Excited, Curious
- Gaze tracking (eyes follow mouse)
- Head rotation based on mouse position
- Click interaction with bounce animation
- Double-click to drag window
- Random behaviors: blink, stretch, yawn
- Natural mood changes over time
- Image abstraction layer for easy replacement

### Changed
- Expression switching via CSS classes (no React re-render)
- Gaze tracking via CSS variables
- Hooks use useRef to prevent effect re-runs

### Fixed
- Timer cleanup to prevent memory leaks
- Behavior duration matches CSS animation timing

## [0.2.0] - 2026-03-17

### Added
- Settings UI with dark theme
- Configuration persistence (JSON storage)
- Test connection functionality
- API Key security handling
- .env.example for reference

### Fixed
- Test connection does not auto-save
- Config load/save race conditions

## [0.1.0] - 2026-03-17

### Added
- Initial architecture (Tauri v2 + Rust)
- Transparent window (400x400)
- AI Provider trait definition
- Frontend-backend communication
