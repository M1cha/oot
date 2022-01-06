.include "macro.inc"

.section .text

.balign 16

glabel D_801120C0
    .incbin "baserom.z64", 0xB89260, 0xFB0

glabel D_80113070
    .incbin "baserom.z64", 0xB8A210, 0x18C0

glabel gJpegUCode
    .incbin "baserom.z64", 0xB8BAD0, 0xAF0
