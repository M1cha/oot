use gliden64::ViReg;
use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use log::{debug, info};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
struct GlideContext {
    rawcontext: Option<glutin::RawContext<glutin::PossiblyCurrent>>,
    window: Arc<Mutex<glutin::window::Window>>,
}

impl gliden64::GfxCallback for GlideContext {
    fn swap_buffers(&mut self) {
        self.rawcontext.as_ref().unwrap().swap_buffers().unwrap()
    }

    fn toggle_fullscreen(&mut self) {
        //self.window().lock().unwrap().set_fullscreen();
    }

    fn resize_window(&mut self, width: u32, height: u32) {
        debug!("resize window to {}x{}", width, height);

        if !self.rawcontext.as_ref().unwrap().is_current() {
            panic!("not current");
        }

        self.rawcontext
            .as_ref()
            .unwrap()
            .resize(glutin::dpi::PhysicalSize { width, height });
    }

    fn set_video_mode(&mut self, width: u32, height: u32, fullscreen: bool) -> anyhow::Result<()> {
        debug!("set video mode to {}x{} fs:{}", width, height, fullscreen);

        self.window
            .lock()
            .unwrap()
            .set_inner_size(glutin::dpi::PhysicalSize { width, height });
        self.rawcontext = Some(unsafe { self.rawcontext.take().unwrap().make_current().unwrap() });
        Ok(())
    }
}

fn main() {
    env_logger::init();

    info!("main thread");

    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Zelda OOT");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let (rawcontext, window) = unsafe { windowed_context.split() };

    let window = Arc::new(Mutex::new(window));
    let rawcontext = unsafe { rawcontext.treat_as_current() };

    let glidectx = GlideContext {
        rawcontext: Some(rawcontext),
        window,
    };

    let mut gfx = gliden64::Gfx::new(glidectx).unwrap();
    gfx.set_vi_reg(ViReg::Origin, 0x401100);
    gfx.set_vi_reg(ViReg::Width, 320);
    gfx.set_vi_reg(ViReg::Timing, 0x4541e3a);
    gfx.set_vi_reg(ViReg::VerticalSync, 625);
    gfx.set_vi_reg(ViReg::HorizontalSync, 0x170c69);
    gfx.set_vi_reg(ViReg::HorizontalSyncLeap, 0xc6f0c6d);
    gfx.set_vi_reg(ViReg::HorizontalStart, 0x800300);
    gfx.set_vi_reg(ViReg::VerticalStart, 0x2f0269);
    gfx.set_vi_reg(ViReg::VerticalBurst, 0x9026b);
    gfx.set_vi_reg(ViReg::XScale, 512);
    gfx.set_vi_reg(ViReg::YScale, 1024);

    gfx.init().unwrap();
    gfx.open_rom().unwrap();

    let mut next_frame_time = std::time::Instant::now();
    el.run(move |event, _, control_flow| {
        let redraw = match event {
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } | StartCause::Init => true,
                _ => false,
            },
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    debug!("resized to {:?}", physical_size);
                    gfx.resize_video_output(physical_size.width, physical_size.height);

                    false
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => false,
            },
            Event::RedrawRequested(_) => true,
            _ => false,
        };

        if redraw {
            //gfx.process_dlist();
            gfx.update_screen();

            next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16666667);
        }

        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}
