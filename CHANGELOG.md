# Changelog

## [0.5.0] - 2026-03-17

### Added
- System monitoring (active app, time, idle duration)
- Context-aware behavior rules
- Auto-greeting based on time of day
- Work mode detection (VSCode, IDE, etc.)
- Break reminders (hourly, long work)
- Configurable reminder settings
- Reminder queue with dismiss button
- Rule deduplication with cooldown
- Optional CPU and memory usage via sysinfo (`get_cpu_usage` / `get_memory_usage`)

### Changed
- Pet reacts to user's current activity
- Smart notifications based on context
- Settings panel expanded with reminder config

### Fixed
- Reminder cooldown prevents spam
- Backend polling independent of frontend
- Cross-platform support (Windows/macOS)

## [0.4.0] - 2026-03-17

### Added
- AI Chat System (OpenAI & Ollama support)
- Streaming output with typewriter effect
- Chat history persistence (chat_history.json)
- Clear history function
- Pet mood linkage with chat (Excited/Curious/Happy/Sleepy)
- AbortController for request cancellation
- Welcome message on first launch

### Changed
- Chat UI with dark theme
- Message bubbles (user/assistant)
- Auto-scroll to bottom during streaming

### Fixed
- Event listener cleanup to prevent memory leaks
- Abort previous request on new send
- Error handling without affecting mood state

### Security
- API Key not stored in chat history
- Local history file in app_data_dir

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
