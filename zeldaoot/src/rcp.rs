use crate::*;

static FILL_SETUP_DL: &'static [Gfx] = &[
    dp_pipe_sync(),
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
    /*
        sp_load_geometry_mode(G_ZBUFFER | G_SHADE | G_CULL_BACK | G_LIGHTING | G_SHADING_SMOOTH),
        dp_set_scissor(G_SC_NON_INTERLACE, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        dp_set_blend_color(0, 0, 0, 8),
        sp_clip_ratio(FRUSTRATIO_2),
    */
];
