# Sway-Draw

A lightweight, native Wayland screen annotation and drawing utility designed for the [Sway](https://swaywm.org/) compositor.

## Overview

Sway-Draw acts as a transparent overlay on top of your existing windows, allowing you to quickly draw and annotate directly on your screen. 
It bypasses heavy UI toolkits like GTK or Qt, instead using pure Wayland protocols (`wlr-layer-shell`) and software rendering (`tiny-skia`) for a fast, minimal, and responsive experience.

## Features

- **Native Wayland**: Uses `smithay-client-toolkit` for direct Wayland integration.
- **Lightweight Rendering**: Software rendering via `tiny-skia` into shared memory buffers (`wl_shm`).
- **Performance Optimized**: Implements partial screen damage tracking. Instead of redrawing the entire 4K screen on every frame, it only calculates and updates the precise bounding boxes of your strokes.

## Prerequisites

- A Wayland compositor that supports `wlr-layer-shell` (e.g., Sway, Hyprland).
- Rust toolchain (`cargo`).

## Building and Running

Clone the repository and run:

```bash
cargo build --release
./target/release/sway-draw
```

Or run directly with Cargo:

```bash
cargo run --release
```

## Usage

- Launch the application (you may want to bind this to a key in your Sway config).
- Click and drag the left mouse button to draw.
- Press `Esc` to exit and clear the annotations.

## Architecture

For more details on the internal design, rendering engine, and module structure, please see [Architecture.md](./Architecture.md).
