#![allow(dead_code)]
#![allow(unused_variables)]

pub mod actor;
pub mod assets;
pub mod debug;
pub mod gamestates;
pub mod graph;
pub mod rcp;
mod shrink_window;
pub mod view;

pub use shrink_window::ShrinkWindow;

use gliden64::ViReg;
use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use graph::{GameState, GameStateEnum, GameStateId};
use log::{debug, error, info};

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

pub const fn pack_rgba5551(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (((r as u32) << 8) & 0xf800)
        | (((g as u32) << 3) & 0x7c0)
        | (((b as u32) >> 2) & 0x3e)
        | ((a as u32) & 0x1)
}

/*
 * flags for G_SETGEOMETRYMODE
 * (this rendering state is maintained in RSP)
 *
 * DO NOT USE THE LOW 8 BITS OF GEOMETRYMODE:
 * The weird bit-ordering is for the micro-code: the lower byte
 * can be OR'd in with G_TRI_SHADE (11001100) to construct
 * the triangle command directly. Don't break it...
 *
 * DO NOT USE THE HIGH 8 BITS OF GEOMETRYMODE:
 * The high byte is OR'd with 0x703 to form the clip code mask.
 * If it is set to 0x04, this will cause near clipping to occur.
 * If it is zero, near clipping will not occur.
 *
 * Further explanation:
 * G_SHADE is necessary in order to see the color that you passed
 * down with the vertex. If G_SHADE isn't set, you need to set the DP
 * appropriately and use primcolor to see anything.
 *
 * G_SHADING_SMOOTH enabled means use all 3 colors of the triangle.
 * If it is not set, then do 'flat shading', where only one vertex color
 * is used (and all 3 vertices are set to that same color by the ucode)
 * See the man page for gSP1Triangle().
 *
 */
pub const ZBUFFER: u32 = 0x00000001;
/// enable Gouraud interp
pub const SHADE: u32 = 0x00000004;

/* rest of low byte reserved for setup ucode */

/// Ignored
pub const TEXTURE_ENABLE: u32 = 0x00000000;
/// flat or smooth shaded
pub const SHADING_SMOOTH: u32 = 0x00200000;
pub const CULL_FRONT: u32 = 0x00000200;
pub const CULL_BACK: u32 = 0x00000400;
/// To make code cleaner
pub const CULL_BOTH: u32 = 0x00000600;

pub const FOG: u32 = 0x00010000;
pub const LIGHTING: u32 = 0x00020000;
pub const TEXTURE_GEN: u32 = 0x00040000;
pub const TEXTURE_GEN_LINEAR: u32 = 0x00080000;
/// NOT IMPLEMENTED
pub const LOD: u32 = 0x00100000;
pub const CLIPPING: u32 = 0x00800000;

pub const fn gbl_c1(m1a: u32, m1b: u32, m2a: u32, m2b: u32) -> u32 {
    m1a << 30 | m1b << 26 | m2a << 22 | m2b << 18
}

pub const fn gbl_c2(m1a: u32, m1b: u32, m2a: u32, m2b: u32) -> u32 {
    m1a << 28 | m1b << 24 | m2a << 20 | m2b << 16
}

pub const RM_NOOP: u32 = gbl_c1(0, 0, 0, 0);
pub const RM_NOOP2: u32 = gbl_c2(0, 0, 0, 0);

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
/// unsupported
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
/// unsupported in HW 2.0
pub const MDSFT_COLORDITHER: u32 = 22;
pub const MDSFT_PIPELINE: u32 = 23;

// G_SETOTHERMODE_H gPipelineMode
pub const PM_1PRIMITIVE: u32 = 1 << MDSFT_PIPELINE;
pub const PM_NPRIMITIVE: u32 = 0 << MDSFT_PIPELINE;

// G_SETOTHERMODE_H gSetCycleType
pub const CYC_1CYCLE: u32 = 0 << MDSFT_CYCLETYPE;
pub const CYC_2CYCLE: u32 = 1 << MDSFT_CYCLETYPE;
pub const CYC_COPY: u32 = 2 << MDSFT_CYCLETYPE;
pub const CYC_FILL: u32 = 3 << MDSFT_CYCLETYPE;

// G_SETOTHERMODE_H gSetTexturePersp
pub const TP_NONE: u32 = 0 << MDSFT_TEXTPERSP;
pub const TP_PERSP: u32 = 1 << MDSFT_TEXTPERSP;

// G_SETOTHERMODE_H gSetTextureDetail
pub const TD_CLAMP: u32 = 0 << MDSFT_TEXTDETAIL;
pub const TD_SHARPEN: u32 = 1 << MDSFT_TEXTDETAIL;
pub const TD_DETAIL: u32 = 2 << MDSFT_TEXTDETAIL;

// G_SETOTHERMODE_H gSetTextureLOD
pub const TL_TILE: u32 = 0 << MDSFT_TEXTLOD;
pub const TL_LOD: u32 = 1 << MDSFT_TEXTLOD;

// G_SETOTHERMODE_H gSetTextureLUT
pub const TT_NONE: u32 = 0 << MDSFT_TEXTLUT;
pub const TT_RGBA16: u32 = 2 << MDSFT_TEXTLUT;
pub const TT_IA16: u32 = 3 << MDSFT_TEXTLUT;

// G_SETOTHERMODE_H gSetTextureFilter
pub const TF_POINT: u32 = 0 << MDSFT_TEXTFILT;
pub const TF_AVERAGE: u32 = 3 << MDSFT_TEXTFILT;
pub const TF_BILERP: u32 = 2 << MDSFT_TEXTFILT;

// G_SETOTHERMODE_H gSetTextureConvert
pub const TC_CONV: u32 = 0 << MDSFT_TEXTCONV;
pub const TC_FILTCONV: u32 = 5 << MDSFT_TEXTCONV;
pub const TC_FILT: u32 = 6 << MDSFT_TEXTCONV;

// G_SETOTHERMODE_H gSetCombineKey
pub const CK_NONE: u32 = 0 << MDSFT_COMBKEY;
pub const CK_KEY: u32 = 1 << MDSFT_COMBKEY;

// G_SETOTHERMODE_H gSetColorDither
pub const CD_MAGICSQ: u32 = 0 << MDSFT_RGBDITHER;
pub const CD_BAYER: u32 = 1 << MDSFT_RGBDITHER;
pub const CD_NOISE: u32 = 2 << MDSFT_RGBDITHER;

pub const CD_DISABLE: u32 = 3 << MDSFT_RGBDITHER;
// HW 1.0 compatibility mode
pub const CD_ENABLE: u32 = CD_NOISE;

// G_SETOTHERMODE_H gSetAlphaDither
pub const AD_PATTERN: u32 = 0 << MDSFT_ALPHADITHER;
pub const AD_NOTPATTERN: u32 = 1 << MDSFT_ALPHADITHER;
pub const AD_NOISE: u32 = 2 << MDSFT_ALPHADITHER;
pub const AD_DISABLE: u32 = 3 << MDSFT_ALPHADITHER;

// G_SETOTHERMODE_L gSetAlphaCompare
pub const AC_NONE: u32 = 0 << MDSFT_ALPHACOMPARE;
pub const AC_THRESHOLD: u32 = 1 << MDSFT_ALPHACOMPARE;
pub const AC_DITHER: u32 = 3 << MDSFT_ALPHACOMPARE;

// G_SETOTHERMODE_L gSetDepthSource
pub const ZS_PIXEL: u32 = 0 << MDSFT_ZSRCSEL;
pub const ZS_PRIM: u32 = 1 << MDSFT_ZSRCSEL;

// G_SETCONVERT: K0-5
/*pub const CV_K0:u32 =      175;
pub const CV_K1:u32 =      -43;
pub const CV_K2:u32 =      -89;
pub const CV_K3:u32 =      222;
pub const CV_K4:u32 =      114;
pub const CV_K5:u32 =      42;*/

// G_SETSCISSOR: interlace mode
pub const SC_NON_INTERLACE: u32 = 0;
pub const SC_ODD_INTERLACE: u32 = 3;
pub const SC_EVEN_INTERLACE: u32 = 2;

// flags to inhibit pushing of the display list (on branch)
pub const DL_PUSH: u32 = 0x00;
pub const DL_NOPUSH: u32 = 0x01;

pub const SCREEN_WIDTH: u16 = 320;
pub const SCREEN_HEIGHT: u16 = 240;

// G_SETIMG fmt: set image formats
pub const IM_FMT_RGBA: u32 = 0;
pub const IM_FMT_YUV: u32 = 1;
pub const IM_FMT_CI: u32 = 2;
pub const IM_FMT_IA: u32 = 3;
pub const IM_FMT_I: u32 = 4;

// G_SETIMG siz: set image pixel size
pub const IM_SIZ_4B: u32 = 0;
pub const IM_SIZ_8B: u32 = 1;
pub const IM_SIZ_16B: u32 = 2;
pub const IM_SIZ_32B: u32 = 3;
pub const IM_SIZ_DD: u32 = 5;

pub enum Gfx {
    SpMatrix {
        matrix: Arc<gliden64::Mtx>,
        param: u8,
    },
    SpDisplayList {
        displaylist: Arc<Vec<Gfx>>,
    },
    SpDisplayListStatic {
        displaylist: &'static [Gfx],
    },
    SpTexture {
        sc: f32,
        tc: f32,
        level: u32,
        tile: u32,
        on: u32,
    },
    SpGeometryMode {
        clear: u32,
        set: u32,
    },
    SpSetGeometryMode {
        mode: u32,
    },
    SpClearGeometryMode {
        mode: u32,
    },
    SpSetOtherModeH {
        length: u32,
        shift: u32,
        data: u32,
    },
    SpSetOtherModeL {
        length: u32,
        shift: u32,
        data: u32,
    },
    SpClipRatio {
        ratio: i16,
    },
    DpSetCombine {
        muxs0: u32,
        muxs1: u32,
    },
    DpSetOtherMode {
        mode0: u32,
        mode1: u32,
    },
    DpSetScissor {
        mode: u32,
        xh: i16,
        yh: i16,
        xl: i16,
        yl: i16,
    },
    DpSetBlendColor {
        r: u32,
        g: u32,
        b: u32,
        a: u32,
    },
    DpSetColorImage {
        format: u32,
        size: u32,
        width: u32,
        img: Arc<Vec<u8>>,
    },
    DpSetDepthImage {
        img: Arc<Vec<u8>>,
    },
    DpSetFillColor {
        color: u32,
    },
    DpFillRectangle {
        ulx: i32,
        uly: i32,
        lrx: i32,
        lry: i32,
    },
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

pub const fn sp_geometry_mode(clear: u32, set: u32) -> Gfx {
    Gfx::SpGeometryMode { clear, set }
}

pub const fn sp_set_geometry_mode(mode: u32) -> Gfx {
    Gfx::SpSetGeometryMode { mode }
}

pub const fn sp_clear_geometry_mode(mode: u32) -> Gfx {
    Gfx::SpClearGeometryMode { mode }
}

pub const fn sp_load_geometry_mode(set: u32) -> Gfx {
    sp_geometry_mode(u32::MAX, set)
}

pub const fn sp_set_other_mode_h(length: u32, shift: u32, data: u32) -> Gfx {
    Gfx::SpSetOtherModeH {
        length,
        shift,
        data,
    }
}

pub const fn sp_set_other_mode_l(length: u32, shift: u32, data: u32) -> Gfx {
    Gfx::SpSetOtherModeL {
        length,
        shift,
        data,
    }
}

pub const fn dp_set_blend_color(r: u32, g: u32, b: u32, a: u32) -> Gfx {
    Gfx::DpSetBlendColor { r, g, b, a }
}

pub const fn dp_set_fill_color(color: u32) -> Gfx {
    Gfx::DpSetFillColor { color }
}

pub const fn dp_set_scissor(mode: u32, xh: i16, yh: i16, xl: i16, yl: i16) -> Gfx {
    Gfx::DpSetScissor {
        mode,
        xh,
        yh,
        xl,
        yl,
    }
}

pub const fn dp_fill_rectangle(ulx: i32, uly: i32, lrx: i32, lry: i32) -> Gfx {
    Gfx::DpFillRectangle { ulx, uly, lrx, lry }
}

pub const fn sp_clip_ratio(ratio: i16) -> Gfx {
    Gfx::SpClipRatio { ratio }
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

unsafe fn render_gfxarr(arr: &[Gfx], gfx: &mut gliden64::Gfx) {
    for entry in arr {
        match entry {
            Gfx::SpMatrix { matrix, param } => gfx.sp_matrix(matrix.as_ref(), *param),
            Gfx::SpDisplayList { displaylist } => render_gfxarr(displaylist, gfx),
            Gfx::SpDisplayListStatic { displaylist } => render_gfxarr(displaylist, gfx),
            Gfx::SpTexture {
                sc,
                tc,
                level,
                tile,
                on,
            } => gfx.sp_texture(*sc, *tc, *level, *tile, *on),
            Gfx::SpGeometryMode { clear, set } => gfx.sp_geometry_mode(*clear, *set),
            Gfx::SpSetGeometryMode { mode } => gfx.sp_set_geometry_mode(*mode),
            Gfx::SpClearGeometryMode { mode } => gfx.sp_clear_geometry_mode(*mode),
            Gfx::SpClipRatio { ratio } => gfx.sp_clip_ratio(*ratio),
            Gfx::SpSetOtherModeH {
                length,
                shift,
                data,
            } => gfx.sp_set_other_mode_h(*length, *shift, *data),
            Gfx::SpSetOtherModeL {
                length,
                shift,
                data,
            } => gfx.sp_set_other_mode_l(*length, *shift, *data),

            Gfx::DpSetCombine { muxs0, muxs1 } => gfx.dp_set_combine(*muxs0, *muxs1),
            Gfx::DpSetOtherMode { mode0, mode1 } => gfx.dp_set_other_mode(*mode0, *mode1),
            Gfx::DpSetScissor {
                mode,
                xh,
                yh,
                xl,
                yl,
            } => gfx.dp_set_scissor(*mode, *xh, *yh, *xl, *yl),
            Gfx::DpSetBlendColor { r, g, b, a } => gfx.dp_set_blend_color(*r, *g, *b, *a),
            Gfx::DpSetColorImage {
                format,
                size,
                width,
                img,
            } => gfx.dp_set_color_image(*format, *size, *width, img.as_ref()),
            Gfx::DpSetDepthImage { img } => gfx.dp_set_depth_image(img.as_ref()),
            Gfx::DpSetFillColor { color } => gfx.dp_set_fill_color(*color),
            Gfx::DpFillRectangle { ulx, uly, lrx, lry } => {
                gfx.dp_fill_rectangle(*ulx, *uly, *lrx, *lry)
            }
        }
    }
}

#[derive(Default)]
pub struct DisplayList {
    inner: Vec<Gfx>,
}

impl DisplayList {
    pub fn sp_matrix(&mut self, matrix: Arc<gliden64::Mtx>, param: u8) {
        self.inner.push(Gfx::SpMatrix { matrix, param });
    }

    pub fn sp_displaylist(&mut self, displaylist: Arc<Vec<Gfx>>) {
        self.inner.push(Gfx::SpDisplayList { displaylist });
    }

    pub fn sp_displaylist_static(&mut self, displaylist: &'static [Gfx]) {
        self.inner.push(Gfx::SpDisplayListStatic { displaylist });
    }

    pub fn sp_set_other_mode_h(&mut self, length: u32, shift: u32, data: u32) {
        self.inner.push(sp_set_other_mode_h(length, shift, data))
    }

    pub fn sp_set_other_mode_l(&mut self, length: u32, shift: u32, data: u32) {
        self.inner.push(sp_set_other_mode_l(length, shift, data))
    }

    pub fn dp_set_scissor(&mut self, mode: u32, xh: i16, yh: i16, xl: i16, yl: i16) {
        self.inner.push(dp_set_scissor(mode, xh, yh, xl, yl));
    }

    pub fn dp_set_color_image(&mut self, format: u32, size: u32, width: u32, img: Arc<Vec<u8>>) {
        self.inner.push(Gfx::DpSetColorImage {
            format,
            size,
            width,
            img,
        });
    }

    pub fn dp_set_depth_image(&mut self, img: Arc<Vec<u8>>) {
        self.inner.push(Gfx::DpSetDepthImage { img });
    }

    pub fn dp_set_cycle_type(&mut self, cycle_type: u32) {
        self.inner
            .push(sp_set_other_mode_h(MDSFT_CYCLETYPE, 2, cycle_type));
    }

    pub fn dp_set_render_mode(&mut self, c0: u32, c1: u32) {
        self.inner
            .push(sp_set_other_mode_l(MDSFT_RENDERMODE, 29, c0 | c1))
    }

    pub fn dp_set_fill_color(&mut self, color: u32) {
        self.inner.push(dp_set_fill_color(color));
    }

    pub fn dp_fill_rectangle(&mut self, ulx: i32, uly: i32, lrx: i32, lry: i32) {
        self.inner.push(dp_fill_rectangle(ulx, uly, lrx, lry));
    }

    unsafe fn render(&self, gfx: &mut gliden64::Gfx) {
        render_gfxarr(&self.inner, gfx)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[derive(Default)]
pub struct GraphicsContext {
    pub work: DisplayList,
    pub poly_opa: DisplayList,
    pub poly_xlu: DisplayList,
    pub overlay: DisplayList,

    pub cur_framebuffer: Arc<Vec<u8>>,
    pub zbuffer: Arc<Vec<u8>>,

    pub gameinfo: debug::GameInfo,
    pub shrink_window: ShrinkWindow,
}

impl GraphicsContext {
    pub unsafe fn render(&self, gfx: &mut gliden64::Gfx) {
        self.work.render(gfx);
        self.poly_opa.render(gfx);
        self.poly_xlu.render(gfx);
        self.overlay.render(gfx);
    }

    pub unsafe fn clear(&mut self) {
        self.work.clear();
        self.poly_opa.clear();
        self.poly_xlu.clear();
        self.overlay.clear();
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("main thread");

    assets::load("textures/nintendo_rogo_static.xml")?;
    return Ok(());

    let dp: Vec<Gfx> = vec![];

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
            unsafe { gfx_ctx.clear() }

            state.main(&mut gfx_ctx).unwrap();

            unsafe {
                gfx_ctx.render(&mut gfx);
            }
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
