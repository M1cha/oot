mod sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use anyhow::anyhow;
use core::ffi::c_void;
use lazy_static::lazy_static;
use log::error;
use num_traits::cast::ToPrimitive;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
#[repr(C)]
pub struct Light {
    /// diffuse light value (rgb)
    pub col: [u8; 3],
    /// copy of diffuse light value (rgb)
    pub colc: [u8; 3],
    /// direction of light (normalized)
    pub dir: [i8; 3],
}

#[derive(Default)]
#[repr(C)]
pub struct LookAt {
    pub l: [Light; 2],
}

/// texture offsets for highlight 1/2
#[derive(Default)]
#[repr(C)]
pub struct Hilite {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

#[derive(Default)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct Mtx {
    pub m: [[f32; 4]; 4],
}

fn f_to_frac8(x: f32) -> i8 {
    (((x * 128.0).min(127.0) as i32) & 0xFF) as i8
}

impl Mtx {
    pub fn ident() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// creates the viewing matrix and sets the LookAt/Hilite structures
    pub fn look_at_hilite(
        x_eye: f32,
        y_eye: f32,
        z_eye: f32,
        x_at: f32,
        y_at: f32,
        z_at: f32,
        x_up: f32,
        y_up: f32,
        z_up: f32,
        mut xl1: f32,
        mut yl1: f32,
        mut zl1: f32,
        mut xl2: f32,
        mut yl2: f32,
        mut zl2: f32,
        hilite_width: i32,
        hilite_height: i32,
    ) -> (Self, LookAt, Hilite) {
        let hilite_width_f = hilite_width as f32;
        let hilite_height_f = hilite_height as f32;

        let mut m = Self::ident();
        let mut l = LookAt::default();
        let mut h = Hilite::default();

        let mut x_look = x_at - x_eye;
        let mut y_look = y_at - y_eye;
        let mut z_look = z_at - z_eye;
        let length = -1.0 / (x_look.powi(2) + y_look.powi(2) + z_look.powi(2)).sqrt();
        x_look *= length;
        y_look *= length;
        z_look *= length;

        let mut x_right = y_up * z_look - z_up * y_look;
        let mut y_right = z_up * x_look - x_up * z_look;
        let mut z_right = x_up * y_look - y_up * x_look;
        let length = 1.0 / (x_right.powi(2) + y_right.powi(2) + z_right.powi(2)).sqrt();
        x_right *= length;
        y_right *= length;
        z_right *= length;

        let mut x_up = y_look * z_right - z_look * y_right;
        let mut y_up = z_look * x_right - x_look * z_right;
        let mut z_up = x_look * y_right - y_look * x_right;
        let length = 1.0 / (x_up.powi(2) + y_up.powi(2) + z_up.powi(2)).sqrt();
        x_up *= length;
        y_up *= length;
        z_up *= length;

        // hilite vectors

        let length = 1.0 / (xl1.powi(2) + yl1.powi(2) + zl1.powi(2)).sqrt();
        xl1 *= length;
        yl1 *= length;
        zl1 *= length;

        let mut x_hilite = xl1 + x_look;
        let mut y_hilite = yl1 + y_look;
        let mut z_hilite = zl1 + z_look;

        let mut length = (x_hilite.powi(2) + y_hilite.powi(2) + z_hilite.powi(2)).sqrt();

        if length > 0.1 {
            length = 1.0 / length;
            x_hilite *= length;
            y_hilite *= length;
            z_hilite *= length;

            h.x1 = (hilite_width_f * 4.0
                + (x_hilite * x_right + y_hilite * y_right + z_hilite * z_right)
                    * hilite_width_f
                    * 2.0) as i32;

            h.y1 = (hilite_height_f * 4.0
                + (x_hilite * x_up + y_hilite * y_up + z_hilite * z_up) * hilite_height_f * 2.0)
                as i32;
        } else {
            h.x1 = hilite_width * 2;
            h.y1 = hilite_height * 2;
        }

        let length = 1.0 / (xl2.powi(2) + yl2.powi(2) + zl2.powi(2)).sqrt();
        xl2 *= length;
        yl2 *= length;
        zl2 *= length;

        let mut x_hilite = xl2 + x_look;
        let mut y_hilite = yl2 + y_look;
        let mut z_hilite = zl2 + z_look;
        let mut length = (x_hilite.powi(2) + y_hilite.powi(2) + z_hilite.powi(2)).sqrt();
        if length > 0.1 {
            length = 1.0 / length;
            x_hilite *= length;
            y_hilite *= length;
            z_hilite *= length;

            h.x2 = (hilite_width_f * 4.0
                + (x_hilite * x_right + y_hilite * y_right + z_hilite * z_right)
                    * hilite_width_f
                    * 2.0) as i32;

            h.y2 = (hilite_height_f * 4.0
                + (x_hilite * x_up + y_hilite * y_up + z_hilite * z_up) * hilite_height_f * 2.0)
                as i32;
        } else {
            h.x2 = hilite_width * 2;
            h.y2 = hilite_height * 2;
        }

        // reflectance vectors = Up and Right

        l.l[0].dir[0] = f_to_frac8(x_right);
        l.l[0].dir[1] = f_to_frac8(y_right);
        l.l[0].dir[2] = f_to_frac8(z_right);
        l.l[1].dir[0] = f_to_frac8(x_up);
        l.l[1].dir[1] = f_to_frac8(y_up);
        l.l[1].dir[2] = f_to_frac8(z_up);
        l.l[0].col[0] = 0x00;
        l.l[0].col[1] = 0x00;
        l.l[0].col[2] = 0x00;
        l.l[0].colc[0] = 0x00;
        l.l[0].colc[1] = 0x00;
        l.l[0].colc[2] = 0x00;
        l.l[1].col[0] = 0x00;
        l.l[1].col[1] = 0x80;
        l.l[1].col[2] = 0x00;
        l.l[1].colc[0] = 0x00;
        l.l[1].colc[1] = 0x80;
        l.l[1].colc[2] = 0x00;

        m.m[0][0] = x_right;
        m.m[1][0] = y_right;
        m.m[2][0] = z_right;
        m.m[3][0] = -(x_eye * x_right + y_eye * y_right + z_eye * z_right);

        m.m[0][1] = x_up;
        m.m[1][1] = y_up;
        m.m[2][1] = z_up;
        m.m[3][1] = -(x_eye * x_up + y_eye * y_up + z_eye * z_up);

        m.m[0][2] = x_look;
        m.m[1][2] = y_look;
        m.m[2][2] = z_look;
        m.m[3][2] = -(x_eye * x_look + y_eye * y_look + z_eye * z_look);

        m.m[0][3] = 0.0;
        m.m[1][3] = 0.0;
        m.m[2][3] = 0.0;
        m.m[3][3] = 1.0;

        (m, l, h)
    }
}

#[derive(num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum MiReg {
    InitMode,
    Version,
    Interrupt,
    InterruptMask,
}

#[derive(num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum DpReg {
    Start,
    End,
    Current,
    Status,
    ClockCounter,
    BufferBusy,
    PipeBusy,
    TMemLoadCounter,
}

#[derive(num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ViReg {
    Status,
    Origin,
    Width,
    VerticalInterrupt,
    CurrentVerticalLine,
    Timing,
    VerticalSync,
    HorizontalSync,
    HorizontalSyncLeap,
    HorizontalStart,
    VerticalStart,
    VerticalBurst,
    XScale,
    YScale,
}

const NUM_MI_REGS: usize = 4;
const NUM_DP_REGS: usize = 8;
const NUM_VI_REGS: usize = 14;
const NUM_REGS: usize = NUM_MI_REGS + NUM_DP_REGS + NUM_VI_REGS;

const MI_REG_OFFSET: usize = 0;
const DP_REG_OFFSET: usize = MI_REG_OFFSET + NUM_MI_REGS;
const VI_REG_OFFSET: usize = DP_REG_OFFSET + NUM_DP_REGS;

pub trait GfxCallback {
    fn swap_buffers(&mut self);
    fn toggle_fullscreen(&mut self);
    fn resize_window(&mut self, width: u32, height: u32);
    fn set_video_mode(&mut self, width: u32, height: u32, fullscreen: bool) -> anyhow::Result<()>;
}

pub struct Gfx {
    regs: Box<[u32; NUM_REGS]>,
    rspmem: Box<[u8; 0x2000]>,
}

extern "C" fn swap_buffers<T: GfxCallback>(userdata: *mut c_void) {
    let callback = unsafe { (userdata as *mut T).as_mut().unwrap() };
    callback.swap_buffers()
}

extern "C" fn toggle_fullscreen<T: GfxCallback>(userdata: *mut c_void) {
    let callback = unsafe { (userdata as *mut T).as_mut().unwrap() };
    callback.toggle_fullscreen()
}

extern "C" fn resize_window<T: GfxCallback>(userdata: *mut c_void, width: c_int, height: c_int) {
    let callback = unsafe { (userdata as *mut T).as_mut().unwrap() };
    callback.resize_window(width as u32, height as u32)
}

extern "C" fn set_video_mode<T: GfxCallback>(
    userdata: *mut c_void,
    width: c_int,
    height: c_int,
    fullscreen: sys::BOOL,
) -> c_int {
    let callback = unsafe { (userdata as *mut T).as_mut().unwrap() };
    if let Err(e) = callback.set_video_mode(width as u32, height as u32, fullscreen != 1) {
        error!("set_video_mode failed: {:?}", e);
        return -1;
    }

    0
}

extern "C" fn check_interrupts() {}

impl Gfx {
    pub fn new<T: GfxCallback>(callback: T) -> anyhow::Result<Self> {
        lazy_static! {
            static ref INITIALIZED: AtomicBool = AtomicBool::new(false);
        }

        INITIALIZED
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| anyhow!("gliden64 is already initialized"))?;

        let userdata = Box::new(callback);
        let callbacks = sys::PluginCallbacks {
            SwapBuffers: Some(swap_buffers::<T>),
            ToggleFullScreen: Some(toggle_fullscreen::<T>),
            ResizeWindow: Some(resize_window::<T>),
            SetVideoMode: Some(set_video_mode::<T>),
        };
        let ret = unsafe { sys::PluginStartup(Box::into_raw(userdata) as *mut c_void, callbacks) };
        if ret != 0 {
            return Err(anyhow!("can't start plugin"));
        }

        Ok(Self {
            regs: Box::new([0; NUM_REGS]),
            rspmem: Box::new([0; 0x2000]),
        })
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let gfxinfo = sys::GFX_INFO {
            HEADER: std::ptr::null_mut(),
            RDRAM: std::ptr::null_mut(),
            DMEM: &mut self.rspmem[0],
            IMEM: &mut self.rspmem[0x1000],
            MI_INTR_REG: self.mi_reg_mut(MiReg::Interrupt),
            DPC_START_REG: self.dp_reg_mut(DpReg::Start),
            DPC_END_REG: self.dp_reg_mut(DpReg::End),
            DPC_CURRENT_REG: self.dp_reg_mut(DpReg::Current),
            DPC_STATUS_REG: self.dp_reg_mut(DpReg::Status),
            DPC_CLOCK_REG: self.dp_reg_mut(DpReg::ClockCounter),
            DPC_BUFBUSY_REG: self.dp_reg_mut(DpReg::BufferBusy),
            DPC_PIPEBUSY_REG: self.dp_reg_mut(DpReg::PipeBusy),
            DPC_TMEM_REG: self.dp_reg_mut(DpReg::TMemLoadCounter),
            VI_STATUS_REG: self.vi_reg_mut(ViReg::Status),
            VI_ORIGIN_REG: self.vi_reg_mut(ViReg::Origin),
            VI_WIDTH_REG: self.vi_reg_mut(ViReg::Width),
            VI_INTR_REG: self.vi_reg_mut(ViReg::VerticalInterrupt),
            VI_V_CURRENT_LINE_REG: self.vi_reg_mut(ViReg::CurrentVerticalLine),
            VI_TIMING_REG: self.vi_reg_mut(ViReg::Timing),
            VI_V_SYNC_REG: self.vi_reg_mut(ViReg::VerticalSync),
            VI_H_SYNC_REG: self.vi_reg_mut(ViReg::HorizontalSync),
            VI_LEAP_REG: self.vi_reg_mut(ViReg::HorizontalSyncLeap),
            VI_H_START_REG: self.vi_reg_mut(ViReg::HorizontalStart),
            VI_V_START_REG: self.vi_reg_mut(ViReg::VerticalStart),
            VI_V_BURST_REG: self.vi_reg_mut(ViReg::VerticalBurst),
            VI_X_SCALE_REG: self.vi_reg_mut(ViReg::XScale),
            VI_Y_SCALE_REG: self.vi_reg_mut(ViReg::YScale),
            CheckInterrupts: Some(check_interrupts),
        };
        let ret = unsafe { sys::InitiateGFX(gfxinfo) };
        if ret != 1 {
            return Err(anyhow!("can't initiate gfx"));
        }

        Ok(())
    }

    pub fn open_rom(&mut self) -> anyhow::Result<()> {
        let ret = unsafe { sys::RomOpen() };
        if ret != 1 {
            return Err(anyhow!("can't open rom"));
        }
        Ok(())
    }

    pub fn process_dlist(&mut self) {
        unsafe { sys::ProcessDList() }
    }

    pub fn update_screen(&mut self) {
        unsafe { sys::UpdateScreen() }
    }

    pub fn resize_video_output(&mut self, width: u32, height: u32) {
        unsafe { sys::ResizeVideoOutput(width as c_int, height as c_int) }
    }

    fn mi_reg_mut(&mut self, reg: MiReg) -> &mut u32 {
        &mut self.regs[MI_REG_OFFSET + reg.to_usize().unwrap()]
    }

    fn dp_reg_mut(&mut self, reg: DpReg) -> &mut u32 {
        &mut self.regs[DP_REG_OFFSET + reg.to_usize().unwrap()]
    }

    fn vi_reg_mut(&mut self, reg: ViReg) -> &mut u32 {
        &mut self.regs[VI_REG_OFFSET + reg.to_usize().unwrap()]
    }

    pub fn mi_reg(&self, reg: MiReg) -> u32 {
        self.regs[MI_REG_OFFSET + reg.to_usize().unwrap()]
    }

    pub fn dp_reg(&self, reg: DpReg) -> u32 {
        self.regs[DP_REG_OFFSET + reg.to_usize().unwrap()]
    }

    pub fn vi_reg(&self, reg: ViReg) -> u32 {
        self.regs[VI_REG_OFFSET + reg.to_usize().unwrap()]
    }

    pub fn set_mi_reg(&mut self, reg: MiReg, value: u32) {
        self.regs[MI_REG_OFFSET + reg.to_usize().unwrap()] = value;
    }

    pub fn set_dp_reg(&mut self, reg: DpReg, value: u32) {
        self.regs[DP_REG_OFFSET + reg.to_usize().unwrap()] = value;
    }

    pub fn set_vi_reg(&mut self, reg: ViReg, value: u32) {
        self.regs[VI_REG_OFFSET + reg.to_usize().unwrap()] = value;
    }

    pub fn rspmem_mut(&mut self) -> &mut [u8; 0x2000] {
        &mut self.rspmem
    }

    #[inline]
    pub fn sp_matrix(&mut self, mtx: &Mtx, param: u8) {
        unsafe { sys::gSPMatrixNative(&mtx.m as *const [f32; 4] as *mut _, param) }
    }

    #[inline]
    pub fn sp_texture(&mut self, sc: f32, tc: f32, level: u32, tile: u32, on: u32) {
        unsafe { sys::gSPTexture(sc, tc, level, tile, on) }
    }

    #[inline]
    pub fn sp_geometry_mode(&mut self, clear: u32, set: u32) {
        unsafe { sys::gSPGeometryMode(clear, set) }
    }

    #[inline]
    pub fn sp_set_geometry_mode(&mut self, mode: u32) {
        unsafe { sys::gSPSetGeometryMode(mode) }
    }

    #[inline]
    pub fn sp_clear_geometry_mode(&mut self, mode: u32) {
        unsafe { sys::gSPClearGeometryMode(mode) }
    }

    #[inline]
    pub fn sp_clip_ratio(&mut self, ratio: i16) {
        unsafe { sys::gSPClipRatioNative(ratio) }
    }

    #[inline]
    pub fn sp_set_other_mode_h(&mut self, length: u32, shift: u32, data: u32) {
        unsafe { sys::gSPSetOtherMode_H(length, shift, data) }
    }

    #[inline]
    pub fn sp_set_other_mode_l(&mut self, length: u32, shift: u32, data: u32) {
        unsafe { sys::gSPSetOtherMode_L(length, shift, data) }
    }

    #[inline]
    pub fn dp_set_combine(&mut self, muxs0: u32, muxs1: u32) {
        unsafe { sys::gDPSetCombine(muxs0, muxs1) }
    }

    #[inline]
    pub fn dp_set_other_mode(&mut self, mode0: u32, mode1: u32) {
        unsafe { sys::gDPSetOtherMode(mode0, mode1) }
    }

    #[inline]
    pub fn dp_set_scissor(&mut self, mode: u32, xh: i16, yh: i16, xl: i16, yl: i16) {
        unsafe { sys::gDPSetScissor(mode, xh, yh, xl, yl) }
    }

    #[inline]
    pub fn dp_set_blend_color(&mut self, r: u32, g: u32, b: u32, a: u32) {
        unsafe { sys::gDPSetBlendColor(r, g, b, a) }
    }

    #[inline]
    pub unsafe fn dp_set_color_image(&mut self, format: u32, size: u32, width: u32, img: &[u8]) {
        sys::gDPSetColorImageNative(format, size, width, img.as_ptr() as usize)
    }

    #[inline]
    pub unsafe fn dp_set_depth_image(&mut self, img: &[u8]) {
        sys::gDPSetDepthImageNative(img.as_ptr() as usize)
    }

    #[inline]
    pub fn dp_set_fill_color(&mut self, color: u32) {
        unsafe { sys::gDPSetFillColor(color) }
    }

    #[inline]
    pub fn dp_fill_rectangle(&mut self, ulx: i32, uly: i32, lrx: i32, lry: i32) {
        unsafe { sys::gDPFillRectangle(ulx, uly, lrx, lry) }
    }
}
