# Sway-Draw Architecture Overview

## Core Philosophy
- **"From Scratch" Approach**: Bypass heavy UI toolkits (GTK/Qt) for a lightweight, native Wayland experience.
- **Overlay Rendering**: Act as a transparent overlay on top of existing windows rather than a traditional tile/floating window.
- **Software Rendering**: Use a CPU-based 2D vector graphics library to draw into a shared memory buffer.

## Wayland Integration (The Windowing Layer)
- **Protocol**: Raw Wayland client implementation.
- **Surface Boundary**: Use the `wlr-layer-shell` protocol. It allows the surface to be pinned as an `overlay` layer (above all standard windows and panels) and bypass normal Sway tiling rules.
- **Buffer Management**: `wl_shm` (Shared Memory) will be used to allocate memory that both the app and the Sway compositor can read/write to.
- **Multimonitor Handling**: Must listen to `wl_output` events to span correctly across single or multiple monitors (typically by spawning a separate layer-shell surface for each output).

## Input Handling
- Listen purely to standard Wayland `wl_pointer` and `wl_keyboard` events.
- **Pointer Events**: Capture standard coordinate data (X, Y) and button states (left click to draw). Ignore complex inputs (tablets/pressure) for simpler data structures.
- **Keyboard Events**: Capture specific keybinds natively (e.g., `Esc` to quit, `Ctrl+Z` to undo, numeric keys to switch colors/tools).

## Rendering Engine
- **Library**: `tiny-skia` (Rust) or Cairo (C/C++).
- **Process**:
  1. Map a chunk of memory that both the app and Sway can access (`wl_shm`).
  2. Treat that memory as a transparent RGBA pixel buffer.
  3. When a user interacts, calculate the geometry (lines, rectangles) and instruct the rendering library to rasterize those shapes into the buffer.
  4. Submit (commit) the modified buffer to Sway for integration on the screen.
- **Damage Tracking (Performance)**: Instead of redrawing the full 4K screen on every frame, the application calculates a precise `dirty_rect` combining the bounding boxes of new strokes and the active ongoing stroke. It persists committed strokes into a `completed_canvas` buffer in standard memory, and copies over only the bounds of the `dirty_rect` into the Wayland canvas to submit minimal `damage_buffer()` requests.

## Module Architecture
The codebase is structured to maximize separation of concerns and provide an excellent developer experience:
- `src/main.rs`: Execution entry point containing the Wayland connection, registry startup logic, and event loop.
- `src/state.rs`: Holds the massive `AppState` structure, manages damage rectangles alongside `completed_canvas`, handles compositor rendering (`.draw()`), and delegates all native Wayland event interactions via smithay protocol handlers.
- `src/draw.rs`: Dedicated module containing pure algorithmic drawing subroutines interfacing with `tiny-skia` (e.g., parsing path builders for `Stroke` rendering).
- `src/types.rs`: Mathematical and state primitives: coordinates (`Point`), color structures (`Stroke`), and geometry bounding tools (`Rect`).

## State Management
- **Vector-based Data Model**: Store drawings as mathematical data (e.g., coordinates, thickness, color), not raw pixel bitmaps.
- **Undo/Redo Stack**: Keep track of user actions (strokes) in an array to easily pop the last drawn element.

## Application Lifecycle
- **One-Shot Execution**: Launched via a Sway `$mod` keybind. Runs until the user resolves the annotation (e.g., presses `Escape` or copies the screen), at which point it clears the Wayland surfaces and destructs completely.
