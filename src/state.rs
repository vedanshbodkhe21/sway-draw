use std::num::NonZeroU32;

use smithay_client_toolkit::{
    compositor::CompositorHandler,
    output::{OutputHandler, OutputState},
    registry::RegistryState,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, QueueHandle,
};

use crate::draw::render_stroke;
use crate::types::{Point, Rect, Stroke};

pub struct AppState {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,

    pub exit: bool,
    pub first_configure: bool,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub keyboard_focus: bool,
    pub pointer: Option<wl_pointer::WlPointer>,

    pub active_stroke: Option<Stroke>,

    pub completed_canvas: tiny_skia::Pixmap,
    pub last_active_stroke_rect: Option<Rect>,
    pub pending_damage: Option<Rect>,
    pub needs_redraw: bool,
    pub frame_pending: bool,
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.frame_pending = false;
        if self.needs_redraw {
            self.draw(qh);
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        let height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            // Re-create the completed canvas if size changes
            self.completed_canvas = tiny_skia::Pixmap::new(self.width, self.height).unwrap();
            self.pending_damage = Some(Rect {
                x: 0,
                y: 0,
                w: self.width,
                h: self.height,
            });
        }

        if self.first_configure {
            self.first_configure = false;
            self.needs_redraw = true;
            if !self.frame_pending {
                self.draw(qh);
            }
        }
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self.seat_state.get_keyboard(qh, &seat, None).unwrap();
            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self.seat_state.get_pointer(qh, &seat).unwrap();
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for AppState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if event.keysym == Keysym::Escape {
            self.exit = true;
        }
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
    }
}

impl PointerHandler for AppState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        let mut needs_redraw = false;

        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                Enter { .. } => log::debug!("Pointer entered"),
                Leave { .. } => {
                    if let Some(stroke) = self.active_stroke.take() {
                        if let Some(bounds) = stroke.bounding_box() {
                            // Bake into completed canvas
                            render_stroke(&mut self.completed_canvas.as_mut(), &stroke);
                            self.pending_damage = match &self.pending_damage {
                                Some(d) => Some(d.union(&bounds)),
                                None => Some(bounds),
                            };
                        }
                        self.needs_redraw = true;
                    }
                }
                Motion { .. } => {
                    if let Some(stroke) = &mut self.active_stroke {
                        stroke.points.push(Point {
                            x: event.position.0 as f32,
                            y: event.position.1 as f32,
                        });
                        needs_redraw = true;
                    }
                }
                Press { button, .. } => {
                    if button == 272 {
                        let stroke = Stroke {
                            points: vec![Point {
                                x: event.position.0 as f32,
                                y: event.position.1 as f32,
                            }],
                            color: tiny_skia::Color::from_rgba8(255, 0, 0, 255),
                            thickness: 4.0,
                        };
                        self.active_stroke = Some(stroke);
                        needs_redraw = true;
                    }
                }
                Release { button, .. } => {
                    if button == 272 {
                        if let Some(stroke) = self.active_stroke.take() {
                            if let Some(bounds) = stroke.bounding_box() {
                                // Bake into completed canvas
                                render_stroke(&mut self.completed_canvas.as_mut(), &stroke);
                                self.pending_damage = match &self.pending_damage {
                                    Some(d) => Some(d.union(&bounds)),
                                    None => Some(bounds),
                                };
                            }
                            self.needs_redraw = true;
                        }
                    }
                }
                Axis { .. } => {}
            }
        }

        if needs_redraw {
            self.needs_redraw = true;
            if !self.frame_pending {
                self.draw(qh);
            }
        }
    }
}

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl AppState {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wayland_client::protocol::wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // 1. Calculate the final dirty rect for this frame
        let mut dirty_rect = self.pending_damage.take();

        // Add last frame's active stroke area so we erase it
        if let Some(r) = &self.last_active_stroke_rect {
            dirty_rect = match dirty_rect {
                Some(d) => Some(d.union(r)),
                None => Some(r.clone()),
            };
        }

        // Add current frame's active stroke
        let current_active_rect = self.active_stroke.as_ref().and_then(|s| s.bounding_box());
        if let Some(r) = &current_active_rect {
            dirty_rect = match dirty_rect {
                Some(d) => Some(d.union(r)),
                None => Some(r.clone()),
            };
        }

        self.last_active_stroke_rect = current_active_rect;

        // If nothing needs to be redrawn, we just attach buffer and commit (or we could even skip committing entirely,
        // but smithay might expect a frame callback response. Safe bet is to draw nothing and commit).
        let dirty = match dirty_rect {
            Some(r) => {
                // Constrain the dirty rect to the actual window bounds
                let screen_bound = Rect {
                    x: 0,
                    y: 0,
                    w: width,
                    h: height,
                };
                r.intersect(&screen_bound)
            }
            None => None,
        };

        if let Some(dirty) = dirty {
            // 2. Clear only the dirty part of our wayland buffer and composite the 'done' strokes
            for y in dirty.y..(dirty.y + dirty.h as i32) {
                let y = y as usize;
                let start = (y * width as usize + dirty.x as usize) * 4;
                let len = (dirty.w as usize) * 4;
                if start + len <= canvas.len() {
                    canvas[start..start + len]
                        .copy_from_slice(&self.completed_canvas.data()[start..start + len]);
                }
            }

            {
                let mut pixmap = tiny_skia::PixmapMut::from_bytes(canvas, width, height).unwrap();
                // 3. Render the active stroke on top (it inherently clips if handled correctly by skia, or it falls within dirty bounds)
                if let Some(active) = &self.active_stroke {
                    render_stroke(&mut pixmap, active);
                }
            }

            // 4. Convert RGBA to BGRA only in the dirty region
            for y in dirty.y..(dirty.y + dirty.h as i32) {
                let y = y as usize;
                let start = (y * width as usize + dirty.x as usize) * 4;
                let len = (dirty.w as usize) * 4;
                if start + len <= canvas.len() {
                    for chunk in canvas[start..start + len].chunks_exact_mut(4) {
                        chunk.swap(0, 2);
                    }
                }
            }

            self.layer
                .wl_surface()
                .damage_buffer(dirty.x, dirty.y, dirty.w as i32, dirty.h as i32);
            self.layer
                .wl_surface()
                .frame(qh, self.layer.wl_surface().clone());
            self.frame_pending = true;
            buffer
                .attach_to(self.layer.wl_surface())
                .expect("buffer attach");
            self.layer.commit();
            self.needs_redraw = false;
        } else {
            // Nothing to draw
            // We just don't commit a new buffer or a new frame callback!
            // This halts the loop until `needs_redraw` becomes true again.
        }
    }
}
