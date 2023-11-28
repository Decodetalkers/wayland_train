use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_registry,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy,
};
struct BaseState;

#[derive(Debug, Default)]
struct SecondState {
    outputs: Vec<wl_output::WlOutput>,
}
impl Dispatch<wl_registry::WlRegistry, ()> for SecondState {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };

        if interface == wl_output::WlOutput::interface().name {
            let output = proxy.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
            state.outputs.push(output);
        }
    }
}
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BaseState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(BaseState: ignore WlCompositor);
delegate_noop!(SecondState: ignore WlOutput);
delegate_noop!(BaseState: ignore WlSurface);

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init::<BaseState>(&connection).unwrap();

    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();

    let wl_buffer = wmcompositer.create_surface(&qh, ());

    let mut state = SecondState::default();

    let mut event_queue = connection.new_event_queue::<SecondState>();
    let qh = event_queue.handle();

    let _ = connection.display().get_registry(&qh, ());

    event_queue.roundtrip(&mut state).unwrap();

    //event_queue.roundtrip(&mut state).unwrap();
    //event_queue.roundtrip(&mut state).unwrap();
    println!("Hello, world!, {:?}", wl_buffer);
    println!("Hello, world!, {:?}", state);
}
