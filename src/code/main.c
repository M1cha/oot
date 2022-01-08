#include "global.h"
#include "vt.h"

#include <stdint.h>
#include "gfx_pc.h"
#include "gfx_opengl.h"
#include "gfx_direct3d11.h"
#include "gfx_direct3d12.h"
#include "gfx_dxgi.h"
#include "gfx_glx.h"
#include "gfx_sdl.h"

s32 gScreenWidth = SCREEN_WIDTH;
s32 gScreenHeight = SCREEN_HEIGHT;
u32 gSystemHeapSize = 0;

PadMgr gPadMgr;
u32 gSegments[NUM_SEGMENTS];
AudioMgr gAudioMgr;
OSMesgQueue sSiIntMsgQ;
OSMesg sSiIntMsgBuf[1];

struct GfxWindowManagerAPI *wm_api;
static struct GfxRenderingAPI *rendering_api;

void Main_LogSystemHeap(void) {
    osSyncPrintf(VT_FGCOL(GREEN));
    // "System heap size% 08x (% dKB) Start address% 08x"
    osSyncPrintf("システムヒープサイズ %08x(%dKB) 開始アドレス %08x\n", gSystemHeapSize, gSystemHeapSize / 1024,
                 gSystemHeap);
    osSyncPrintf(VT_RST);
}

int main(void) {
    u32 sysHeap;
    s32 debugHeap;
    s32 debugHeapSize;
    s16* msg;

#if defined(ENABLE_DX12)
    rendering_api = &gfx_direct3d12_api;
    wm_api = &gfx_dxgi_api;
#elif defined(ENABLE_DX11)
    rendering_api = &gfx_direct3d11_api;
    wm_api = &gfx_dxgi_api;
#elif defined(ENABLE_OPENGL)
    rendering_api = &gfx_opengl_api;
    #if defined(__linux__) || defined(__BSD__)
        wm_api = &gfx_glx;
    #else
        wm_api = &gfx_sdl;
    #endif
#elif defined(ENABLE_GFX_DUMMY)
    rendering_api = &gfx_dummy_renderer_api;
    wm_api = &gfx_dummy_wm_api;
#endif

    gfx_init(wm_api, rendering_api, "Super Mario 64 PC-Port", 0);


    osSyncPrintf("mainproc 実行開始\n"); // "Start running"
    gScreenWidth = SCREEN_WIDTH;
    gScreenHeight = SCREEN_HEIGHT;
    sysHeap = (u32)gSystemHeap;
    gSystemHeapSize = 0x400000;
    SystemHeap_Init(sysHeap, gSystemHeapSize); // initializes the system heap
    debugHeapSize = 0x400;
    debugHeap = SystemArena_MallocDebug(debugHeapSize, "../main.c", 565);
    osSyncPrintf("debug_InitArena(%08x, %08x)\n", debugHeap, debugHeapSize);
    DebugArena_Init(debugHeap, debugHeapSize);
    func_800636C0();

    R_ENABLE_ARENA_DBG = 0;

    osCreateMesgQueue(&sSiIntMsgQ, sSiIntMsgBuf, 1);
    osSetEventMesg(5, &sSiIntMsgQ, 0);

    Main_LogSystemHeap();

    //AudioMgr_Init(&gAudioMgr, sAudioStack + sizeof(sAudioStack), Z_PRIORITY_AUDIOMGR, 0xA, &gSchedContext, &gIrqMgr);

    //PadMgr_Init(&gPadMgr, &sSiIntMsgQ, &gIrqMgr, 7, Z_PRIORITY_PADMGR, &sIrqMgrStack);

    AudioMgr_Unlock(&gAudioMgr);

    Graph_ThreadEntry(NULL);

    osSyncPrintf("mainproc 実行終了\n"); // "End of execution"

    return 0;
}
