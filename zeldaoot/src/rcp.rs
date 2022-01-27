use crate::*;

static FILL_SETUP_DL: &'static [Gfx] = &[
    sp_texture(
        fixed_to_float!(0xFFFF, 16),
        fixed_to_float!(0xFFFF, 16),
        0,
        G_TX_RENDERTILE,
        G_OFF,
    ),
    dp_set_combine_mode(CC_SHADE, CC_SHADE),
    dp_set_other_mode(
        AD_DISABLE
            | CD_MAGICSQ
            | CK_NONE
            | TC_FILT
            | TF_BILERP
            | TT_NONE
            | TL_TILE
            | TD_CLAMP
            | TP_PERSP
            | CYC_FILL
            | PM_NPRIMITIVE,
        AC_NONE | ZS_PIXEL | RM_NOOP | RM_NOOP2,
    ),
    sp_load_geometry_mode(ZBUFFER | SHADE | CULL_BACK | LIGHTING | SHADING_SMOOTH),
    dp_set_scissor(
        SC_NON_INTERLACE,
        0,
        0,
        SCREEN_WIDTH as i16,
        SCREEN_HEIGHT as i16,
    ),
    dp_set_blend_color(0, 0, 0, 8),
    sp_clip_ratio(-2),
];

pub fn func_80095248(gfx_ctx: &mut GraphicsContext, mut r: u8, mut g: u8, mut b: u8) {
    let screen_width: i16 = SCREEN_WIDTH.try_into().unwrap();
    let screen_height: i16 = SCREEN_HEIGHT.try_into().unwrap();

    gfx_ctx.poly_opa.sp_displaylist_static(FILL_SETUP_DL);
    gfx_ctx.poly_xlu.sp_displaylist_static(FILL_SETUP_DL);
    gfx_ctx.overlay.sp_displaylist_static(FILL_SETUP_DL);

    gfx_ctx
        .poly_opa
        .dp_set_scissor(SC_NON_INTERLACE, 0, 0, screen_width, screen_height);
    gfx_ctx
        .poly_xlu
        .dp_set_scissor(SC_NON_INTERLACE, 0, 0, screen_width, screen_height);
    gfx_ctx
        .overlay
        .dp_set_scissor(SC_NON_INTERLACE, 0, 0, screen_width, screen_height);

    gfx_ctx.poly_opa.dp_set_color_image(
        IM_FMT_RGBA,
        IM_SIZ_16B,
        SCREEN_WIDTH.into(),
        gfx_ctx.cur_framebuffer.clone(),
    );
    gfx_ctx.poly_xlu.dp_set_color_image(
        IM_FMT_RGBA,
        IM_SIZ_16B,
        SCREEN_WIDTH.into(),
        gfx_ctx.cur_framebuffer.clone(),
    );
    gfx_ctx.overlay.dp_set_color_image(
        IM_FMT_RGBA,
        IM_SIZ_16B,
        SCREEN_WIDTH.into(),
        gfx_ctx.cur_framebuffer.clone(),
    );

    gfx_ctx.poly_opa.dp_set_depth_image(gfx_ctx.zbuffer.clone());
    gfx_ctx.poly_xlu.dp_set_depth_image(gfx_ctx.zbuffer.clone());
    gfx_ctx.overlay.dp_set_depth_image(gfx_ctx.zbuffer.clone());

    if (gfx_ctx.gameinfo.pause_menu_mode() < 2) && (gfx_ctx.gameinfo.trnsn_unk_state < 2) {
        let mut ret = gfx_ctx.shrink_window.get_current_val();

        if gfx_ctx.gameinfo.hreg(80) == 16 {
            if gfx_ctx.gameinfo.hreg(95) != 16 {
                gfx_ctx.gameinfo.set_hreg(81, 3);
                gfx_ctx.gameinfo.set_hreg(82, 3);
                gfx_ctx.gameinfo.set_hreg(83, 0);
                gfx_ctx.gameinfo.set_hreg(84, 0);
                gfx_ctx.gameinfo.set_hreg(85, 0);
                gfx_ctx.gameinfo.set_hreg(86, 0);
                gfx_ctx.gameinfo.set_hreg(87, 0);
                gfx_ctx.gameinfo.set_hreg(88, 0);
                gfx_ctx.gameinfo.set_hreg(89, 0);
                gfx_ctx.gameinfo.set_hreg(90, 0);
                gfx_ctx.gameinfo.set_hreg(91, 0);
                gfx_ctx.gameinfo.set_hreg(92, 0);
                gfx_ctx.gameinfo.set_hreg(93, 0);
                gfx_ctx.gameinfo.set_hreg(94, 0);
                gfx_ctx.gameinfo.set_hreg(95, 16);
            }

            if (gfx_ctx.gameinfo.hreg(81) & 1) != 0 {
                gfx_ctx.gameinfo.set_hreg(83, ret.try_into().unwrap());
            }

            if (gfx_ctx.gameinfo.hreg(81) & 2) != 0 {
                gfx_ctx.gameinfo.set_hreg(84, r.into());
                gfx_ctx.gameinfo.set_hreg(85, g.into());
                gfx_ctx.gameinfo.set_hreg(86, b.into());
            }

            if (gfx_ctx.gameinfo.hreg(82) & 1) != 0 {
                ret = gfx_ctx.gameinfo.hreg(83).into();
            }

            if (gfx_ctx.gameinfo.hreg(82) & 2) != 0 {
                r = gfx_ctx.gameinfo.hreg(84).try_into().unwrap();
                g = gfx_ctx.gameinfo.hreg(85).try_into().unwrap();
                b = gfx_ctx.gameinfo.hreg(86).try_into().unwrap();
            }
        }

        let retu16: u16 = ret.try_into().unwrap();

        gfx_ctx.poly_opa.dp_set_color_image(
            IM_FMT_RGBA,
            IM_SIZ_16B,
            SCREEN_WIDTH.into(),
            gfx_ctx.zbuffer.clone(),
        );
        gfx_ctx.poly_opa.dp_set_cycle_type(CYC_FILL);
        gfx_ctx.poly_opa.dp_set_render_mode(RM_NOOP, RM_NOOP2);
        gfx_ctx.poly_opa.dp_set_fill_color(
            (pack_rgba5551(255, 255, 240, 0) << 16) | pack_rgba5551(255, 255, 240, 0),
        );
        gfx_ctx.poly_opa.dp_fill_rectangle(
            0,
            ret,
            (SCREEN_WIDTH - 1).into(),
            (SCREEN_HEIGHT - retu16 - 1).into(),
        );

        gfx_ctx.poly_opa.dp_set_color_image(
            IM_FMT_RGBA,
            IM_SIZ_16B,
            SCREEN_WIDTH.into(),
            gfx_ctx.cur_framebuffer.clone(),
        );
        gfx_ctx.poly_opa.dp_set_cycle_type(CYC_FILL);
        gfx_ctx.poly_opa.dp_set_render_mode(RM_NOOP, RM_NOOP2);
        gfx_ctx
            .poly_opa
            .dp_set_fill_color((pack_rgba5551(r, g, b, 1) << 16) | pack_rgba5551(r, g, b, 1));
        gfx_ctx.poly_opa.dp_fill_rectangle(
            0,
            ret,
            (SCREEN_WIDTH - 1).into(),
            (SCREEN_HEIGHT - retu16 - 1).into(),
        );

        if ret > 0 {
            gfx_ctx.overlay.dp_set_cycle_type(CYC_FILL);
            gfx_ctx.overlay.dp_set_render_mode(RM_NOOP, RM_NOOP2);
            gfx_ctx
                .overlay
                .dp_set_fill_color((pack_rgba5551(r, g, b, 1) << 16) | pack_rgba5551(r, g, b, 1));
            gfx_ctx
                .overlay
                .dp_fill_rectangle(0, 0, (SCREEN_WIDTH - 1).into(), ret - 1);
            gfx_ctx.overlay.dp_fill_rectangle(
                0,
                (SCREEN_HEIGHT - retu16).into(),
                (SCREEN_WIDTH - 1).into(),
                (SCREEN_HEIGHT - 1).into(),
            );
        }
    }
}
