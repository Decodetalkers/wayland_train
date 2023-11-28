use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_compositor::WlCompositor, wl_registry, wl_surface::WlSurface},
    Connection, Dispatch,
};

struct BaseState;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BaseState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(BaseState: ignore WlCompositor);
delegate_noop!(BaseState: ignore WlSurface);

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init::<BaseState>(&connection).unwrap();

    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();

    let wl_buffer = wmcompositer.create_surface(&qh, ());
    println!("Hello, world!, {:?}", wl_buffer);
}
