#include "global.h"
#include "message_data_static.h"

void func_8006EE50(Font* font, u16 arg1, u16 arg2) {
}

/**
 * Loads a texture from nes_font_static for the requested `character` into the character texture buffer
 * at `codePointIndex`. The value of `character` is the ASCII codepoint subtract ' '/0x20.
 */
void Font_LoadChar(Font* font, u8 character, u16 codePointIndex) {
    DmaMgr_SendRequest1(&font->charTexBuf[codePointIndex],
                        &_nes_font_staticSegmentRomStart[character * FONT_CHAR_TEX_SIZE], FONT_CHAR_TEX_SIZE,
                        "../z_kanfont.c", 93);
}

/**
 * Loads a message box icon from message_static, such as the ending triangle/square or choice arrow into the
 * icon buffer.
 * The different icons are given in the MessageBoxIcon enum.
 */
void Font_LoadMessageBoxIcon(Font* font, u16 icon) {
    DmaMgr_SendRequest1(font->iconBuf,
                        &_message_staticSegmentRomStart[4 * MESSAGE_STATIC_TEX_SIZE + icon * FONT_CHAR_TEX_SIZE],
                        FONT_CHAR_TEX_SIZE, "../z_kanfont.c", 100);
}

/**
 * Loads a full set of character textures based on their ordering in the message with text id 0xFFFC into
 * the font buffer.
 */
void Font_LoadOrderedFont(Font* font) {
    s32 len;
    s32 jj;
    s32 fontStatic;
    u32 fontBuf;
    s32 codePointIndex;
    s32 fontBufIndex;
    s32 offset;

    font->msgOffset = _message_0xFFFC_nes;
    len = 0x48;

    memcpy(font->msgBuf, font->msgOffset, len);
    osSyncPrintf("msg_data=%x,  msg_data0=%x   jj=%x\n", font->msgOffset, font->msgLength, jj = len);

    len = jj;
    for (fontBufIndex = 0, codePointIndex = 0; font->msgBuf[codePointIndex] != MESSAGE_END; codePointIndex++) {
        if (codePointIndex > len) {
            osSyncPrintf("ＥＲＲＯＲ！！  エラー！！！  error───！！！！\n");
            return;
        }

        if (font->msgBuf[codePointIndex] != MESSAGE_NEWLINE) {
            fontBuf = font->fontBuf + fontBufIndex * 8;
            fontStatic = _nes_font_staticSegmentRomStart;

            osSyncPrintf("nes_mes_buf[%d]=%d\n", codePointIndex, font->msgBuf[codePointIndex]);

            offset = (font->msgBuf[codePointIndex] - '\x20') * FONT_CHAR_TEX_SIZE;
            memcpy(fontBuf, fontStatic + offset, FONT_CHAR_TEX_SIZE);
            fontBufIndex += FONT_CHAR_TEX_SIZE / 8;
        }
    }
}
