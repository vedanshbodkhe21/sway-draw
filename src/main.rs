use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
    shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell},
    shm::{slot::SlotPool, Shm},
};
use wayland_client::{globals::registry_queue_init, Connection};

mod draw;
mod state;
mod types;
use smithay_client_toolkit::shell::WaylandSurface;
use state::AppState;
use types::Rect;

delegate_compositor!(AppState);
delegate_output!(AppState);
delegate_shm!(AppState);
delegate_seat!(AppState);
delegate_keyboard!(AppState);
delegate_pointer!(AppState);
delegate_layer!(AppState);
delegate_registry!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting sway-draw");

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("sway-draw"), None);

    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_exclusive_zone(-1); // Don't move other windows

    // Commit to get the configure event
    layer.commit();

    let pool = SlotPool::new(1920 * 1080 * 4, &shm).expect("Failed to create pool");

    let mut app_state = AppState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: 1920,
        height: 1080,
        layer,
        keyboard: None,
        keyboard_focus: false,
        pointer: None,

        active_stroke: None,
        completed_canvas: tiny_skia::Pixmap::new(1920, 1080).unwrap(),
        last_active_stroke_rect: None,
        pending_damage: Some(Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        }),
        needs_redraw: true,
        frame_pending: false,
    };

    loop {
        event_queue.blocking_dispatch(&mut app_state).unwrap();
        if app_state.exit {
            log::info!("Exiting");
            break;
        }
    }
    Ok(())
}
