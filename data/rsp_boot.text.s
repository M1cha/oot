.include "macro.inc"

.section .text

.balign 16

glabel D_80009320
    .incbin "baserom.z64", 0x9F20, 0xD0

glabel D_800093F0
    .incbin "baserom.z64", 0x9FF0, 0x20
