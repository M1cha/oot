use crate::*;
use gliden64::{Hilite, Light, LookAt, Mtx, Vec3};

pub fn func_8002e830(
    object: &Vec3,
    eye: &Vec3,
    light_dir: &Vec3, /*, gfx_ctx: &GraphicsContext*/
    gfx: &mut DisplayList,
) -> Arc<Hilite> {
    let corrected_eye_x = if (eye.x == object.x) && (eye.z == object.z) {
        eye.x + 0.001
    } else {
        eye.x
    };

    /*if gfx_ctx.gameinfo.hreg(80) == 6 {
        log::debug!("eye=[{}({}) {} {}] object=[{} {} {}] light_direction=[{} {} {}]", corrected_eye_x,
                     eye.x, eye.y, eye.z, object.x, object.y, object.z, light_dir.x, light_dir.y, light_dir.z);
    }*/

    crate::view::func_800abe74(corrected_eye_x, eye.y, eye.z);
    let (m, look_at, hilite) = Mtx::look_at_hilite(
        corrected_eye_x,
        eye.y,
        eye.z,
        object.x,
        object.y,
        object.z,
        0.0,
        1.0,
        0.0,
        light_dir.x,
        light_dir.y,
        light_dir.z,
        light_dir.x,
        light_dir.y,
        light_dir.z,
        0x10,
        0x10,
    );
    let hilite = Arc::new(hilite);

    let bytes = unsafe {
        std::slice::from_raw_parts(
            (&look_at) as *const _ as *const u8,
            std::mem::size_of::<LookAt>(),
        )
    };

    /*
    println!("data = {{");
    for (i, byte) in bytes.iter().enumerate() {
        if i != 0 && (i % 4) == 0 {
            println!();
        }

        print!("{:#04X}, ", byte);
    }
    println!("}};");*/

    //gSPLookAt(gfx++, lookAt);
    //gDPSetHilite1Tile(gfx++, 1, *hilite, 0x10, 0x10);

    hilite
}

pub fn func_8002eabc(
    object: &Vec3,
    eye: &Vec3,
    light_dir: &Vec3,
    gfx_ctx: &mut GraphicsContext,
) -> Arc<Hilite> {
    func_8002e830(
        object,
        eye,
        light_dir, /*, gfx_ctx*/
        &mut gfx_ctx.poly_opa,
    )
}
