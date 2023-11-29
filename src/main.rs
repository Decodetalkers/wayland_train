use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_registry,
        wl_shm::WlShm,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy,
};

use wayland_protocols::xdg::shell::client::{
    xdg_surface,
    xdg_toplevel::XdgToplevel,
    xdg_wm_base::{self, XdgWmBase},
};

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

impl Dispatch<xdg_surface::XdgSurface, ()> for SecondState {
    fn event(
        _state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
        }
    }
}

delegate_noop!(SecondState: ignore WlCompositor);
delegate_noop!(SecondState: ignore WlSurface);
delegate_noop!(SecondState: ignore WlOutput);
delegate_noop!(SecondState: ignore WlShm);
delegate_noop!(SecondState: ignore XdgToplevel);

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, _) = registry_queue_init::<BaseState>(&connection).unwrap();

    let mut state = SecondState::default();

    let mut event_queue = connection.new_event_queue::<SecondState>();
    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();
    let wl_surface = wmcompositer.create_surface(&qh, ());
    let xdg_wm_base = globals.bind::<XdgWmBase, _, _>(&qh, 1..=2, ()).unwrap();
    let wl_shm = globals.bind::<WlShm, _, _>(&qh, 1..=1, ()).unwrap();

    let _ = connection.display().get_registry(&qh, ());

    event_queue.blocking_dispatch(&mut state).unwrap();

    println!("Hello, world!, {:?}", wl_surface);
    println!("Hello, world!, {:?}", wl_shm);
    println!("Hello, world!, {:?}", xdg_wm_base);
    println!("Hello, world!, {:?}", state);
    let xdg_surface = xdg_wm_base.get_xdg_surface(&wl_surface, &qh, ());
    let toplevel = xdg_surface.get_toplevel(&qh, ());
    toplevel.set_title("EEEE".into());
    wl_surface.commit();

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
