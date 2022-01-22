#![allow(dead_code)]
#![allow(unused_variables)]

pub mod assets;
pub mod gamestates;
pub mod graph;
pub mod rcp;
pub mod view;

use gliden64::ViReg;
use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use graph::{GameState, GameStateEnum, GameStateId};
use log::{debug, error, info};
use std::rc::Rc;
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

pub const G_TX_LOADTILE: u32 = 7;
pub const G_TX_RENDERTILE: u32 = 0;

pub const G_ON: u32 = 1;
pub const G_OFF: u32 = 0;

// Fixed point conversion factors
const FIXED2FLOATRECIP1: f32 = 0.5;
const FIXED2FLOATRECIP2: f32 = 0.25;
const FIXED2FLOATRECIP3: f32 = 0.125;
const FIXED2FLOATRECIP4: f32 = 0.0625;
const FIXED2FLOATRECIP5: f32 = 0.03125;
const FIXED2FLOATRECIP6: f32 = 0.015625;
const FIXED2FLOATRECIP7: f32 = 0.0078125;
const FIXED2FLOATRECIP8: f32 = 0.00390625;
const FIXED2FLOATRECIP9: f32 = 0.001953125;
const FIXED2FLOATRECIP10: f32 = 0.0009765625;
const FIXED2FLOATRECIP11: f32 = 0.00048828125;
const FIXED2FLOATRECIP12: f32 = 2.44140625e-04;
const FIXED2FLOATRECIP13: f32 = 1.220703125e-04;
const FIXED2FLOATRECIP14: f32 = 6.103515625e-05;
const FIXED2FLOATRECIP15: f32 = 3.0517578125e-05;
const FIXED2FLOATRECIP16: f32 = 1.52587890625e-05;

#[macro_export]
macro_rules! fixed_to_float {
    // `()` indicates that the macro takes no argument.
    ($fixed:literal, $scale:literal) => {
        match $scale {
            1 => ($fixed as f32) * FIXED2FLOATRECIP1,
            2 => ($fixed as f32) * FIXED2FLOATRECIP2,
            3 => ($fixed as f32) * FIXED2FLOATRECIP3,
            4 => ($fixed as f32) * FIXED2FLOATRECIP4,
            5 => ($fixed as f32) * FIXED2FLOATRECIP5,
            6 => ($fixed as f32) * FIXED2FLOATRECIP6,
            7 => ($fixed as f32) * FIXED2FLOATRECIP7,
            8 => ($fixed as f32) * FIXED2FLOATRECIP8,
            9 => ($fixed as f32) * FIXED2FLOATRECIP9,
            10 => ($fixed as f32) * FIXED2FLOATRECIP10,
            11 => ($fixed as f32) * FIXED2FLOATRECIP11,
            12 => ($fixed as f32) * FIXED2FLOATRECIP12,
            13 => ($fixed as f32) * FIXED2FLOATRECIP13,
            14 => ($fixed as f32) * FIXED2FLOATRECIP14,
            15 => ($fixed as f32) * FIXED2FLOATRECIP15,
            16 => ($fixed as f32) * FIXED2FLOATRECIP16,
            _ => unimplemented!(),
        }
    };
}

pub const fn GBL_c1(m1a: u32, m1b: u32, m2a: u32, m2b: u32) -> u32 {
    m1a << 30 | m1b << 26 | m2a << 22 | m2b << 18
}

pub const fn GBL_c2(m1a: u32, m1b: u32, m2a: u32, m2b: u32) -> u32 {
    m1a << 28 | m1b << 24 | m2a << 20 | m2b << 16
}

pub const RM_NOOP: u32 = GBL_c1(0, 0, 0, 0);
pub const RM_NOOP2: u32 = GBL_c2(0, 0, 0, 0);

// Color combiner constants
pub const CCMUX_COMBINED: u32 = 0;
pub const CCMUX_TEXEL0: u32 = 1;
pub const CCMUX_TEXEL1: u32 = 2;
pub const CCMUX_PRIMITIVE: u32 = 3;
pub const CCMUX_SHADE: u32 = 4;
pub const CCMUX_ENVIRONMENT: u32 = 5;
pub const CCMUX_CENTER: u32 = 6;
pub const CCMUX_SCALE: u32 = 6;
pub const CCMUX_COMBINED_ALPHA: u32 = 7;
pub const CCMUX_TEXEL0_ALPHA: u32 = 8;
pub const CCMUX_TEXEL1_ALPHA: u32 = 9;
pub const CCMUX_PRIMITIVE_ALPHA: u32 = 10;
pub const CCMUX_SHADE_ALPHA: u32 = 11;
pub const CCMUX_ENV_ALPHA: u32 = 12;
pub const CCMUX_LOD_FRACTION: u32 = 13;
pub const CCMUX_PRIM_LOD_FRAC: u32 = 14;
pub const CCMUX_NOISE: u32 = 7;
pub const CCMUX_K4: u32 = 7;
pub const CCMUX_K5: u32 = 15;
pub const CCMUX_1: u32 = 6;
pub const CCMUX_0: u32 = 31;

// Alpha combiner constants
pub const ACMUX_COMBINED: u32 = 0;
pub const ACMUX_TEXEL0: u32 = 1;
pub const ACMUX_TEXEL1: u32 = 2;
pub const ACMUX_PRIMITIVE: u32 = 3;
pub const ACMUX_SHADE: u32 = 4;
pub const ACMUX_ENVIRONMENT: u32 = 5;
pub const ACMUX_LOD_FRACTION: u32 = 0;
pub const ACMUX_PRIM_LOD_FRAC: u32 = 6;
pub const ACMUX_1: u32 = 6;
pub const ACMUX_0: u32 = 7;

// G_SETOTHERMODE_L sft: shift count
pub const MDSFT_ALPHACOMPARE: u32 = 0;
pub const MDSFT_ZSRCSEL: u32 = 2;
pub const MDSFT_RENDERMODE: u32 = 3;
pub const MDSFT_BLENDER: u32 = 16;

// G_SETOTHERMODE_H sft: shift count
// unsupported
pub const MDSFT_BLENDMASK: u32 = 0;
pub const MDSFT_ALPHADITHER: u32 = 4;
pub const MDSFT_RGBDITHER: u32 = 6;

pub const MDSFT_COMBKEY: u32 = 8;
pub const MDSFT_TEXTCONV: u32 = 9;
pub const MDSFT_TEXTFILT: u32 = 12;
pub const MDSFT_TEXTLUT: u32 = 14;
pub const MDSFT_TEXTLOD: u32 = 16;
pub const MDSFT_TEXTDETAIL: u32 = 17;
pub const MDSFT_TEXTPERSP: u32 = 19;
pub const MDSFT_CYCLETYPE: u32 = 20;
// unsupported in HW 2.0
pub const MDSFT_COLORDITHER: u32 = 22;
pub const MDSFT_PIPELINE: u32 = 23;

// G_SETOTHERMODE_H gPipelineMode
pub const PM_1PRIMITIVE: u32 = (1 << MDSFT_PIPELINE);
pub const PM_NPRIMITIVE: u32 = (0 << MDSFT_PIPELINE);

// G_SETOTHERMODE_H gSetCycleType
pub const CYC_1CYCLE: u32 = (0 << MDSFT_CYCLETYPE);
pub const CYC_2CYCLE: u32 = (1 << MDSFT_CYCLETYPE);
pub const CYC_COPY: u32 = (2 << MDSFT_CYCLETYPE);
pub const CYC_FILL: u32 = (3 << MDSFT_CYCLETYPE);

// G_SETOTHERMODE_H gSetTexturePersp
pub const TP_NONE: u32 = (0 << MDSFT_TEXTPERSP);
pub const TP_PERSP: u32 = (1 << MDSFT_TEXTPERSP);

// G_SETOTHERMODE_H gSetTextureDetail
pub const TD_CLAMP: u32 = (0 << MDSFT_TEXTDETAIL);
pub const TD_SHARPEN: u32 = (1 << MDSFT_TEXTDETAIL);
pub const TD_DETAIL: u32 = (2 << MDSFT_TEXTDETAIL);

// G_SETOTHERMODE_H gSetTextureLOD
pub const TL_TILE: u32 = (0 << MDSFT_TEXTLOD);
pub const TL_LOD: u32 = (1 << MDSFT_TEXTLOD);

// G_SETOTHERMODE_H gSetTextureLUT
pub const TT_NONE: u32 = (0 << MDSFT_TEXTLUT);
pub const TT_RGBA16: u32 = (2 << MDSFT_TEXTLUT);
pub const TT_IA16: u32 = (3 << MDSFT_TEXTLUT);

// G_SETOTHERMODE_H gSetTextureFilter
pub const TF_POINT: u32 = (0 << MDSFT_TEXTFILT);
pub const TF_AVERAGE: u32 = (3 << MDSFT_TEXTFILT);
pub const TF_BILERP: u32 = (2 << MDSFT_TEXTFILT);

// G_SETOTHERMODE_H gSetTextureConvert
pub const TC_CONV: u32 = (0 << MDSFT_TEXTCONV);
pub const TC_FILTCONV: u32 = (5 << MDSFT_TEXTCONV);
pub const TC_FILT: u32 = (6 << MDSFT_TEXTCONV);

// G_SETOTHERMODE_H gSetCombineKey
pub const CK_NONE: u32 = (0 << MDSFT_COMBKEY);
pub const CK_KEY: u32 = (1 << MDSFT_COMBKEY);

// G_SETOTHERMODE_H gSetColorDither
pub const CD_MAGICSQ: u32 = (0 << MDSFT_RGBDITHER);
pub const CD_BAYER: u32 = (1 << MDSFT_RGBDITHER);
pub const CD_NOISE: u32 = (2 << MDSFT_RGBDITHER);

pub const CD_DISABLE: u32 = (3 << MDSFT_RGBDITHER);
// HW 1.0 compatibility mode
pub const CD_ENABLE: u32 = CD_NOISE;

// G_SETOTHERMODE_H gSetAlphaDither
pub const AD_PATTERN: u32 = (0 << MDSFT_ALPHADITHER);
pub const AD_NOTPATTERN: u32 = (1 << MDSFT_ALPHADITHER);
pub const AD_NOISE: u32 = (2 << MDSFT_ALPHADITHER);
pub const AD_DISABLE: u32 = (3 << MDSFT_ALPHADITHER);

// G_SETOTHERMODE_L gSetAlphaCompare
pub const AC_NONE: u32 = (0 << MDSFT_ALPHACOMPARE);
pub const AC_THRESHOLD: u32 = (1 << MDSFT_ALPHACOMPARE);
pub const AC_DITHER: u32 = (3 << MDSFT_ALPHACOMPARE);

// G_SETOTHERMODE_L gSetDepthSource
pub const ZS_PIXEL: u32 = (0 << MDSFT_ZSRCSEL);
pub const ZS_PRIM: u32 = (1 << MDSFT_ZSRCSEL);

pub enum Gfx {
    SpMatrix {
        matrix: Arc<gliden64::Mtx>,
        param: u8,
    },
    SpDisplayList {
        displaylist: Arc<Vec<Gfx>>,
    },
    SpTexture {
        sc: f32,
        tc: f32,
        level: u32,
        tile: u32,
        on: u32,
    },
    DpSetCombine {
        muxs0: u32,
        muxs1: u32,
    },
    DpSetOtherMode {
        mode0: u32,
        mode1: u32,
    },
    DpPipeSync,
}

pub const fn dp_pipe_sync() -> Gfx {
    Gfx::DpPipeSync
}

pub const fn sp_texture(sc: f32, tc: f32, level: u32, tile: u32, on: u32) -> Gfx {
    Gfx::SpTexture {
        sc,
        tc,
        level,
        tile,
        on,
    }
}

pub const fn dp_set_combine(muxs0: u32, muxs1: u32) -> Gfx {
    Gfx::DpSetCombine { muxs0, muxs1 }
}

pub const fn shiftl(v: u32, s: usize, w: usize) -> u32 {
    (v & ((0x01 << w) - 1)) << s
}

pub const fn gcc_c0w0(sa_rgb0: u32, m_rgb0: u32, sa_a0: u32, m_a0: u32) -> u32 {
    shiftl(sa_rgb0, 20, 4) | shiftl(m_rgb0, 15, 5) | shiftl(sa_a0, 12, 3) | shiftl(m_a0, 9, 3)
}

pub const fn gcc_c0w1(sb_rgb0: u32, a_rgb0: u32, sb_a0: u32, a_a0: u32) -> u32 {
    shiftl(sb_rgb0, 28, 4) | shiftl(a_rgb0, 15, 3) | shiftl(sb_a0, 12, 3) | shiftl(a_a0, 9, 3)
}

pub const fn gcc_c1w0(sa_rgb1: u32, m_rgb1: u32) -> u32 {
    shiftl(sa_rgb1, 5, 4) | shiftl(m_rgb1, 0, 5)
}

pub const fn gcc_c1w1(
    sb_rgb1: u32,
    sa_a1: u32,
    m_a1: u32,
    a_rgb1: u32,
    sb_a1: u32,
    a_a1: u32,
) -> u32 {
    shiftl(sb_rgb1, 24, 4)
        | shiftl(sa_a1, 21, 3)
        | shiftl(m_a1, 18, 3)
        | shiftl(a_rgb1, 6, 3)
        | shiftl(sb_a1, 3, 3)
        | shiftl(a_a1, 0, 3)
}

pub const fn dp_set_combine_lerp(
    a0: u32,
    b0: u32,
    c0: u32,
    d0: u32,
    a_a0: u32,
    a_b0: u32,
    a_c0: u32,
    a_d0: u32,
    a1: u32,
    b1: u32,
    c1: u32,
    d1: u32,
    a_a1: u32,
    a_b1: u32,
    a_c1: u32,
    a_d1: u32,
) -> Gfx {
    dp_set_combine(
        gcc_c0w0(a0, c0, a_a0, a_c0) | gcc_c1w0(a1, c1),
        gcc_c0w1(b0, d0, a_b0, a_d0) | gcc_c1w1(b1, a_a1, a_c1, d1, a_b1, a_d1),
    )
}

type SetCombineMode = (u32, u32, u32, u32, u32, u32, u32, u32);

pub const CC_SHADE: SetCombineMode = (
    CCMUX_0,
    CCMUX_0,
    CCMUX_0,
    CCMUX_SHADE,
    ACMUX_0,
    ACMUX_0,
    ACMUX_0,
    ACMUX_SHADE,
);

pub const fn dp_set_combine_mode(
    (a0, b0, c0, d0, a_a0, a_b0, a_c0, a_d0): SetCombineMode,
    (a1, b1, c1, d1, a_a1, a_b1, a_c1, a_d1): SetCombineMode,
) -> Gfx {
    dp_set_combine_lerp(
        a0, b0, c0, d0, a_a0, a_b0, a_c0, a_d0, a1, b1, c1, d1, a_a1, a_b1, a_c1, a_d1,
    )
}

pub const fn dp_set_other_mode(mode0: u32, mode1: u32) -> Gfx {
    Gfx::DpSetOtherMode { mode0, mode1 }
}

#[derive(Default)]
pub struct DisplayList {
    inner: Vec<Gfx>,
}

impl DisplayList {
    fn sp_matrix(&mut self, matrix: Arc<gliden64::Mtx>, param: u8) {
        self.inner.push(Gfx::SpMatrix { matrix, param });
    }

    fn sp_displaylist(&mut self, displaylist: Arc<Vec<Gfx>>) {
        self.inner.push(Gfx::SpDisplayList { displaylist });
    }

    unsafe fn render(&self, gfx: &mut gliden64::Gfx) {
        for entry in &self.inner {
            match entry {
                Gfx::SpMatrix { matrix, param } => gfx.sp_matrix(matrix.as_ref(), *param),
                Gfx::SpDisplayList { displaylist } => (),
                Gfx::DpPipeSync => (),
                _ => unimplemented!(),
            }
        }
    }
}

#[derive(Default)]
pub struct GraphicsContext {
    pub work: DisplayList,
    pub poly_opa: DisplayList,
    pub poly_xlu: DisplayList,
    pub overlay: DisplayList,
}

impl GraphicsContext {
    pub unsafe fn render(&self, gfx: &mut gliden64::Gfx) {
        self.work.render(gfx);
        self.poly_opa.render(gfx);
        self.poly_xlu.render(gfx);
        self.overlay.render(gfx);
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("main thread");

    let dp: Vec<Gfx> = vec![];

    //assets::load()?;
    //return Ok(());

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

    let mut gfx_ctx = GraphicsContext::default();

    let mut state = GameStateEnum::from_id(&GameStateId::TitleSetup);
    state.init()?;

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
            state.main(&mut gfx_ctx).unwrap();
            unsafe {
                gfx_ctx.render(&mut gfx);
            }
            //gfx.process_dlist();
            gfx.update_screen();

            next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16666667);
        }

        if !state.common().running {
            let stateid = match state.common().next_stateid {
                Some(id) => id,
                None => {
                    error!("current state stopped without returning a new state");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            };

            info!("enter state: {:?}", stateid);
            state = GameStateEnum::from_id(&stateid);
            state.init().unwrap();
        }

        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}
