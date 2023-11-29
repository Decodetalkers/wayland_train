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
use wayland_protocols::xdg::shell::client::xdg_wm_base::{self, XdgWmBase};
struct BaseState;

#[allow(unused)]
#[derive(Debug)]
struct SecondState {
    outputs: Vec<wl_output::WlOutput>,
    running: bool,
}

impl Default for SecondState {
    fn default() -> Self {
        SecondState {
            outputs: Vec::new(),
            running: true,
        }
    }
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
impl Dispatch<xdg_wm_base::XdgWmBase, ()> for SecondState {
    fn event(
        _state: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
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

delegate_noop!(SecondState: ignore WlCompositor);
delegate_noop!(SecondState: ignore WlSurface);
delegate_noop!(SecondState: ignore WlOutput);

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, _) = registry_queue_init::<BaseState>(&connection).unwrap();

    let mut state = SecondState::default();

    let mut event_queue = connection.new_event_queue::<SecondState>();
    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();
    let wl_buffer = wmcompositer.create_surface(&qh, ());
    let xdg_wm_base = globals.bind::<XdgWmBase, _, _>(&qh, 1..=2, ()).unwrap();

    let _ = connection.display().get_registry(&qh, ());

    event_queue.roundtrip(&mut state).unwrap();

    println!("Hello, world!, {:?}", wl_buffer);
    println!("Hello, world!, {:?}", xdg_wm_base);
    println!("Hello, world!, {:?}", state);
}
