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
    fullscreen: bool,
) -> c_int {
    let callback = unsafe { (userdata as *mut T).as_mut().unwrap() };
    if let Err(e) = callback.set_video_mode(width as u32, height as u32, fullscreen) {
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
        if !ret {
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
}
