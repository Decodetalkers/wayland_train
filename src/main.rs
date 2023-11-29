use std::{fs::File, os::unix::prelude::AsFd};

use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_keyboard,
        wl_output::{self, WlOutput},
        wl_registry,
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, WEnum,
};

use wayland_protocols::xdg::shell::client::{
    xdg_surface,
    xdg_toplevel::XdgToplevel,
    xdg_wm_base::{self, XdgWmBase},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor},
};

#[derive(Debug)]
struct BaseState;

// so interesting, it is just need to invoke once, it just used to get the globals
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

impl Dispatch<wl_seat::WlSeat, ()> for SecondState {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for SecondState {
    fn event(
        state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                state.running = false;
            }
        }
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for SecondState {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            let surface = state.wl_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

delegate_noop!(SecondState: ignore WlCompositor); // WlCompositor is need to create a surface
delegate_noop!(SecondState: ignore WlSurface); // surface is the base needed to show buffer
delegate_noop!(SecondState: ignore WlOutput); // output is need to place layer_shell, although here
                                              // it is not used
delegate_noop!(SecondState: ignore WlShm); // shm is used to create buffer pool
delegate_noop!(SecondState: ignore XdgToplevel); // so it is the same with layer_shell, private a
                                                 // place for surface
delegate_noop!(SecondState: ignore WlShmPool); // so it is pool, created by wl_shm
delegate_noop!(SecondState: ignore WlBuffer); // buffer show the picture
delegate_noop!(SecondState: ignore ZwlrLayerShellV1); // it is simillar with xdg_toplevel, also the
                                                      // ext-session-shell

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, _) = registry_queue_init::<BaseState>(&connection).unwrap(); // We just need the
                                                                               // global, the
                                                                               // event_queue is
                                                                               // not needed, we
                                                                               // do not need
                                                                               // BaseState after
                                                                               // this anymore

    let mut state = SecondState::default();

    let mut event_queue = connection.new_event_queue::<SecondState>();
    let qh = event_queue.handle();

    let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap(); // so the first
                                                                                    // thing is to
                                                                                    // get WlCompositor
    let wl_surface = wmcompositer.create_surface(&qh, ()); // and create a surface. if two or more,
                                                           // we need to create more

    let shm = globals.bind::<WlShm, _, _>(&qh, 1..=1, ()).unwrap();
    globals.bind::<WlSeat, _, _>(&qh, 1..=1, ()).unwrap();

    let _ = connection.display().get_registry(&qh, ()); // so if you want WlOutput, you need to
                                                        // register this

    event_queue.blocking_dispatch(&mut state).unwrap(); // then make a dispatch

    println!("Hello, world!, {:?}", wl_surface);
    println!("Hello, world!, {:?}", shm);
    println!("Hello, world!, {:?}", state); // you will find you get the outputs, but if you do not
                                            // do the step before, you get empty list

    // so it is the same way, to get surface detach to protocol, first get the shell, like wmbase
    // or layer_shell or session-shell, then get `surface` from the wl_surface you get before, and
    // set it
    // finally thing to remember is to commit the surface, make the shell to init.
    let (init_w, init_h) = (320, 240);
    if true {
        // this example is ok for both xdg_surface and layer_shell
        let layer_shell = globals
            .bind::<ZwlrLayerShellV1, _, _>(&qh, 3..=4, ())
            .unwrap();
        let layer = layer_shell.get_layer_surface(
            &wl_surface,
            None,
            Layer::Top,
            "nobody".to_owned(),
            &qh,
            (),
        );
        layer.set_anchor(Anchor::Bottom | Anchor::Right | Anchor::Left | Anchor::Top);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand);
        layer.set_size(init_w, init_w);
    } else {
        let xdg_wm_base = globals.bind::<XdgWmBase, _, _>(&qh, 1..=2, ()).unwrap();
        let xdg_surface = xdg_wm_base.get_xdg_surface(&wl_surface, &qh, ());
        let toplevel = xdg_surface.get_toplevel(&qh, ());
        toplevel.set_title("EEEE".into());
    }

    wl_surface.commit();

    // so during the init Configure of the shell, a buffer, atleast a buffer is needed.
    // and if you need to reconfigure it, you need to commit the wl_surface again
    // so because this is just an example, so we just commit it once
    // like if you want to reset anchor or KeyboardInteractivity or resize, commit is needed
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
    while state.running {
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
