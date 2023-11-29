use std::{fs::File, os::unix::prelude::AsFd};

use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_registry,
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
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
    wl_surface: Option<WlSurface>,
    buffer: Option<WlBuffer>,
}

impl Default for SecondState {
    fn default() -> Self {
        SecondState {
            outputs: Vec::new(),
            running: true,
            wl_surface: None,
            buffer: None,
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
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            let surface = state.wl_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

delegate_noop!(SecondState: ignore WlCompositor);
delegate_noop!(SecondState: ignore WlSurface);
delegate_noop!(SecondState: ignore WlOutput);
delegate_noop!(SecondState: ignore WlShm);
delegate_noop!(SecondState: ignore XdgToplevel);
delegate_noop!(SecondState: ignore WlShmPool);
delegate_noop!(SecondState: ignore WlBuffer);

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, _) = registry_queue_init::<BaseState>(&connection).unwrap();

    let mut state = SecondState::default();

    let mut event_queue = connection.new_event_queue::<SecondState>();
    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();
    let wl_surface = wmcompositer.create_surface(&qh, ());
    let xdg_wm_base = globals.bind::<XdgWmBase, _, _>(&qh, 1..=2, ()).unwrap();
    let shm = globals.bind::<WlShm, _, _>(&qh, 1..=1, ()).unwrap();

    let _ = connection.display().get_registry(&qh, ());

    event_queue.blocking_dispatch(&mut state).unwrap();

    println!("Hello, world!, {:?}", wl_surface);
    println!("Hello, world!, {:?}", shm);
    println!("Hello, world!, {:?}", xdg_wm_base);
    println!("Hello, world!, {:?}", state);

    let xdg_surface = xdg_wm_base.get_xdg_surface(&wl_surface, &qh, ());
    let toplevel = xdg_surface.get_toplevel(&qh, ());
    toplevel.set_title("EEEE".into());
    wl_surface.commit();
    let (init_w, init_h) = (320, 240);

    let mut file = tempfile::tempfile().unwrap();
    draw(&mut file, (init_w, init_h));
    let pool = shm.create_pool(file.as_fd(), (init_w * init_h * 4) as i32, &qh, ());
    let buffer = pool.create_buffer(
        0,
        init_w as i32,
        init_h as i32,
        (init_w * 4) as i32,
        wl_shm::Format::Argb8888,
        &qh,
        (),
    );

    state.wl_surface = Some(wl_surface);
    state.buffer = Some(buffer);
    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

fn draw(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    use std::{cmp::min, io::Write};
    let mut buf = std::io::BufWriter::new(tmp);
    for y in 0..buf_y {
        for x in 0..buf_x {
            let a = 0xFF;
            let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);

            let color = (a << 24) + (r << 16) + (g << 8) + b;
            buf.write_all(&color.to_ne_bytes()).unwrap();
        }
    }
    buf.flush().unwrap();
}
