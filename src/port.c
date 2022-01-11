#include <functions.h>
#include <variables.h>
#include <z64audio.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

// bss
ActiveSound gActiveSounds[7][MAX_CHANNELS_PER_BANK];
u8 gSoundBankMuted[0xC];
u16 gAudioSfxSwapSource[10];
u16 gAudioSfxSwapTarget[10];
u8 gAudioSfxSwapMode[10];
u8 D_8016E348[4];
u32 sAudioSeqCmds[0x100];
unk_D_8016E750 D_8016E750[4];
AudioContext gAudioContext;
void(*D_801755D0)(void);

// undefined
Mtx D_01000000;

u32 osTvType = OS_TV_NTSC;
u32 gViConfigFeatures = OS_VI_DITHER_FILTER_ON | OS_VI_GAMMA_OFF;
f32 gViConfigXScale = 1.0;
f32 gViConfigYScale = 1.0;
u8 gViConfigAdditionalScanLines = 0;
vu8 gViConfigUseDefault = 1;
OSViMode gViConfigMode;
s8 D_80009430 = 1;
volatile OSTime gIrqMgrRetraceTime = 0;
u32 D_80009460 = 0;
u32 gDmaMgrDmaBuffSize = 0x2000;
OSPiHandle* __osPiTable;

void guMtxF2L(MtxF* mf, Mtx* m) {
    Matrix_MtxFToMtx(mf, m);
}

void guMtxL2F(MtxF* mf, Mtx* m) {
    Matrix_MtxToMtxF(mf, m);
}

void guMtxIdentF(float mf[4][4]) {
    int r, c;
    for (r = 0; r < 4; r++) {
        for (c = 0; c < 4; c++) {
            if (r == c) {
                mf[r][c] = 1.0f;
            } else {
                mf[r][c] = 0.0f;
            }
        }
    }
}

void guMtxIdent(Mtx* m) {
#ifndef GBI_FLOATS
    float mf[4][4];
    guMtxIdentF(mf);
    guMtxF2L(mf, m);
#else
    guMtxIdentF(m->m);
#endif
}

void guNormalize(f32 *x, f32 *y, f32 *z) {
    f32 tmp = 1.0f / sqrtf(*x * *x + *y * *y + *z * *z);
    *x = *x * tmp;
    *y = *y * tmp;
    *z = *z * tmp;
}

void guScaleF(float mf[4][4], float x, float y, float z) {
    guMtxIdentF(mf);
    mf[0][0] = x;
    mf[1][1] = y;
    mf[2][2] = z;
    mf[3][3] = 1.0;
}

void guScale(Mtx *m, float x, float y, float z) {
    MtxF mf;
    guScaleF(mf.mf, x, y, z);
    guMtxF2L(&mf, m);
}

void guTranslateF(float m[4][4], float x, float y, float z) {
    guMtxIdentF(m);
    m[3][0] = x;
    m[3][1] = y;
    m[3][2] = z;
}

void guTranslate(Mtx *m, float x, float y, float z) {
    MtxF mf;
    guTranslateF(mf.mf, x, y, z);
    guMtxF2L(&mf, m);
}

void osCreateViManager(OSPri pri) {
}

void osViSetMode(OSViMode *mode) {
}

void osViSetEvent(OSMesgQueue *mq, OSMesg msg, u32 retraceCount) {
}

void osViBlack(u8 active) {
}

void osViSetSpecialFeatures(u32 func) {
}

void osViSwapBuffer(void *vaddr) {
}

void osViSetXScale(f32 scale) {
}

void osViSetYScale(f32 scale) {
}

void osViExtendVStart(u32 arg0) {
}

void osCreateMesgQueue(OSMesgQueue *mq, OSMesg *msgBuf, s32 count) {
    mq->validCount = 0;
    mq->first = 0;
    mq->msgCount = count;
    mq->msg = msgBuf;
    return;
}

void osSetEventMesg(OSEvent e, OSMesgQueue *mq, OSMesg msg) {
}

s32 osJamMesg(OSMesgQueue *mq, OSMesg msg, s32 flag) {
    return 0;
}

s32 osSendMesg(OSMesgQueue *mq, OSMesg msg, s32 flag) {
    s32 index;
    if (mq->validCount >= mq->msgCount) {
        return -1;
    }
    index = (mq->first + mq->validCount) % mq->msgCount;
    mq->msg[index] = msg;
    mq->validCount++;

    return 0;
}

s32 osRecvMesg(OSMesgQueue *mq, OSMesg *msg, s32 flag) {
    if (mq->validCount == 0) {
        return -1;
    }
    if (msg != NULL) {
        *msg = *(mq->first + mq->msg);
    }
    mq->first = (mq->first + 1) % mq->msgCount;
    mq->validCount--;

    return 0;
}


OSTime osGetTime(void) {
    return 0;
}

void osWritebackDCacheAll(void) {
}

void osWritebackDCache(void *vaddr, s32 nbytes) {
}

void osInvalDCache(void *vaddr, s32 nbytes) {
}

void osInvalICache(void* vaddr, s32 nbytes) {
}

u32 osGetCount(void) {
    static u32 counter;
    return counter++;
}

void Fault_AddHungupAndCrash(const char* filename, u32 line) {
    fprintf(stderr, "HungUp %s:%d\n", filename, line);
    abort();
}

void Fault_AddClient(FaultClient* client, void* callback, void* param0, void* param1) {
}

void Fault_RemoveClient(FaultClient* client) {
}

void FaultDrawer_Printf(const char*s, ...) {
}

void FaultDrawer_SetFontColor(u16 color) {
}

void FaultDrawer_SetCursor(s32 x, s32 y) {
}

void FaultDrawer_SetCharPad(s8 padW, s8 padH) {
}

void Fault_AddHungupAndCrashImpl(const char* arg0, const char* arg1) {
    osSyncPrintf("%s\n", arg0 != NULL ? arg0 : "(NULL)");
    osSyncPrintf("%s\n", arg1 != NULL ? arg1 : "(NULL)");
    abort();
}

void FaultDrawer_DrawText(s32 x, s32 y, const char* fmt, ...) {
}

void Fault_WaitForInput(void) {
}

OSId osGetThreadId(OSThread* thread) {
    return (OSId)(uintptr_t)thread;
}

s32 DmaMgr_SendRequest0(u32 ram, u32 vrom, u32 size) {
    abort();
    memcpy(ram, vrom, size);
    return 0;
}

s32 DmaMgr_SendRequest1(void* ram0, u32 vrom, u32 size, const char* file, s32 line) {
    abort();
    memcpy(ram0, vrom, size);
    return 0;
}

s32 DmaMgr_SendRequest2(DmaRequest* req, u32 ram, u32 vrom, u32 size, u32 unk5, OSMesgQueue* queue, OSMesg msg,
                        const char* file, s32 line) {
    abort();
    req->vromAddr = vrom;
    req->dramAddr = (void*)ram;
    req->size = size;
    req->unk_14 = 0;
    req->notifyQueue = queue;
    req->notifyMsg = msg;

    memcpy(ram, vrom, size);

    if (req->notifyQueue) {
        osSendMesg(req->notifyQueue, req->notifyMsg, OS_MESG_NOBLOCK);
    }

    return 0;
}

s32 DmaMgr_DmaRomToRam(u32 rom, u32 ram, u32 size) {
    abort();
    memcpy(ram, rom, size);
    return 0;
}

s32 osEPiStartDma(OSPiHandle *pihandle, OSIoMesg *mb, s32 direction) {
    switch(direction) {
    case OS_READ:
        memcpy(mb->dramAddr, (const void *) mb->devAddr, mb->size);
        break;

    case OS_WRITE:
        memcpy((void *) mb->devAddr, mb->dramAddr, mb->size);
        break;
    }
    osSendMesg(mb->hdr.retQueue, mb, OS_MESG_NOBLOCK);
    return 0;
}

s32 osEPiReadIo(OSPiHandle* handle, u32 devAddr, u32* data) {
    return 0;
}

s32 Jpeg_Decode(void* data, void* zbuffer, void* work, u32 workSize) {
    // TODO
    return 0;
}

void osSyncPrintf(const char* fmt, ...) {
    va_list ap;

    va_start(ap, fmt);
    vfprintf(stdout, fmt, ap);
    va_end(ap);

    fflush(stdout);
}

s32 osPfsInitPak(OSMesgQueue* queue, OSPfs* pfs, s32 channel) {
    return 0;
}

s32 osPfsDeleteFile(OSPfs* pfs, u16 companyCode, u32 gameCode, u8* gameName, u8* extName) {
    return 0;
}

s32 osPfsAllocateFile(OSPfs* pfs, u16 companyCode, u32 gameCode, u8* gameName, u8* extName, s32 fileSize, s32* fileNo) {
    return 0;
}

s32 osPfsFreeBlocks(OSPfs* pfs, s32* leftoverBytes) {
    return 0;
}

s32 osPfsFindFile(OSPfs* pfs, u16 companyCode, u32 gameCode, u8* gameName, u8* extName, s32* fileNo) {
    return 0;
}

s32 osPfsReadWriteFile(OSPfs* pfs, s32 fileNo, u8 flag, s32 offset, s32 size, u8* data) {
    return 0;
}

s32 osPfsFileState(OSPfs* pfs, s32 fileNo, OSPfsState* state) {
    return 0;
}

s32 osSetTimer(OSTimer* timer, OSTime countdown, OSTime interval, OSMesgQueue* mq, OSMesg msg) {
    return 0;
}

s32 osStopTimer(OSTimer* timer) {
    return 0;
}

s32 __osDisableInt(void) {
    return 0;
}

void __osRestoreInt(s32) {
}

OSIntMask osSetIntMask(OSIntMask mask) {
    return mask;
}

void PadMgr_UnlockSerialMesgQueue(PadMgr* padMgr, OSMesgQueue* ctrlrQ) {
}

OSMesgQueue* PadMgr_LockSerialMesgQueue(PadMgr* padMgr) {
    return NULL;
}

void PadMgr_RequestPadData(PadMgr* padMgr, Input* inputs, s32 mode) {
}

void PadMgr_RumbleSet(PadMgr* padMgr, u8* ctrlrRumbles) {
}

s32 osAiSetFrequency(u32 frequency) {
    return 0;
}

s32 osAiSetNextBuffer(void* buf, u32 size) {
    return 0;
}

u32 osAiGetLength(void) {
    return 0;
}

void AudioMgr_Unlock(AudioMgr* audioMgr) {
}

OSPiHandle* osCartRomInit(void) {
    return NULL;
}

static uint8_t fb0[0x1000000];
static uint8_t fb1[0x1000000];

u32 SysCfb_GetFbPtr(s32 idx) {
    if (idx == 0) {
        return fb0;
    }
    if (idx == 1) {
        return fb1;
    }
    return 0;
}
