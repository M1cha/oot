#include "global.h"
#include "vt.h"

s32 gScreenWidth = SCREEN_WIDTH;
s32 gScreenHeight = SCREEN_HEIGHT;
u32 gSystemHeapSize = 0;

PadMgr gPadMgr;
u32 gSegments[NUM_SEGMENTS];
AudioMgr gAudioMgr;
OSMesgQueue sSiIntMsgQ;
OSMesg sSiIntMsgBuf[1];

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
