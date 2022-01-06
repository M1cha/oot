.include "macro.inc"

.section .data

.balign 16

# temporary file name, rename to something more appropriate when decompiled

glabel gMojiFontTLUTs
    .incbin "baserom.z64", 0xBA18E0, 0x80

glabel gMojiFontTex
    .incbin "baserom.z64", 0xBA1960, 0x400
