use crate::graph::{GameState, GameStateCommon};
use crate::GraphicsContext;
use gliden64::{Mtx, Vec3};
use std::sync::Arc;

#[derive(Default)]
pub struct Viewport {
    top_y: i32,
    bottom_y: i32,
    left_x: i32,
    right_x: i32,
}

#[derive(Default)]
pub struct View {
    /// string literal "VIEW" / 0x56494557
    magic: i32,
    viewport: Viewport,
    /// vertical field of view in degrees
    fovy: f32,
    /// distance to near clipping plane
    z_near: f32,
    /// distance to far clipping plane
    z_far: f32,
    /// scale for matrix elements
    scale: f32,
    eye: Vec3,
    look_at: Vec3,
    up: Vec3,
    //Vp     vp;
    projection: Mtx,
    viewing: Mtx,
    //Mtx*   projectionPtr;
    //Mtx*   viewingPtr;
    unk_e8: Vec3,
    unk_f4: Vec3,
    unk_100: f32,
    unk_104: Vec3,
    unk_110: Vec3,
    /// used to normalize the projection matrix
    normal: u16,
    flags: i32,
    unk_124: i32,
}

#[derive(Default)]
pub struct Title {
    common: GameStateCommon,
    //staticSegment,
    view: View,
    //sramCtx,
    cover_alpha: i16,
    ult: i16,
    uls: i16,
    unk_e10: [char; 1],
    exit: bool,
    unk_e12: [char; 6],

    title_rot_y: i16,
}

impl GameState for Title {
    fn common(&self) -> &GameStateCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GameStateCommon {
        &mut self.common
    }

    fn init(&mut self) -> anyhow::Result<()> {
        // TODO: load resources

        // R_UPDATE_RATE = 1
        //Matrix_Init(&this->state);
        //View_Init(&this->view, this->state.gfxCtx);
        self.exit = false;
        //gSaveContext.fileNum = 0xFF;
        //Sram_Alloc(&this->state, &this->sramCtx);
        self.ult = 0;
        self.cover_alpha = 255;

        Ok(())
    }

    fn main(&mut self, gfx_ctx: &mut GraphicsContext) -> anyhow::Result<()> {
        crate::rcp::func_80095248(gfx_ctx, 0, 0, 0);
        self.calc();
        self.draw(gfx_ctx);

        Ok(())
    }
}

impl Title {
    /// Note: In other rom versions this function also updates unk_1D4,
    /// coverAlpha, addAlpha, visibleDuration to calculate
    /// the fade-in/fade-out + the duration of the n64 logo animation
    fn calc(&mut self) {
        //self.exit = true;
        self.cover_alpha = 0;
    }

    fn draw(&mut self, gfx_ctx: &mut GraphicsContext) {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(-4949.148, 4002.5417, 1119.0837);
        let v3 = Vec3::new(69.0, 69.0, 69.0);

        crate::actor::func_8002eabc(&v1, &v2, &v3, gfx_ctx);

        self.title_rot_y = self.title_rot_y.wrapping_add(300);
    }
}
