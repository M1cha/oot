MAKEFLAGS += --no-builtin-rules

# Build options can either be changed by modifying the makefile, or by building with 'make SETTING=value'

# If NON_MATCHING is 1, define the NON_MATCHING C flag when building
NON_MATCHING ?= 1

ifeq ($(NON_MATCHING),1)
  CFLAGS := -DNON_MATCHING
  CPPFLAGS := -DNON_MATCHING
endif

PROJECT_DIR := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

MAKE = make
CPPFLAGS += -P

ifeq ($(OS),Windows_NT)
    DETECTED_OS=windows
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        DETECTED_OS=linux
    endif
    ifeq ($(UNAME_S),Darwin)
        DETECTED_OS=macos
        MAKE=gmake
        CPPFLAGS += -xc++
    endif
endif

CC       := $(CROSS_COMPILE)cc
AS         := $(CROSS_COMPILE)as
LD         := $(CROSS_COMPILE)ld
OBJCOPY    := $(CROSS_COMPILE)objcopy
OBJDUMP    := $(CROSS_COMPILE)objdump

INC        := -Iinclude -Isrc -Iassets -Ibuild -I. -Isrc/n64-fast3d-engine

CHECK_WARNINGS := -Wall -Wextra -Wno-format-security -Wno-unknown-pragmas -Wno-unused-parameter -Wno-unused-variable -Wno-missing-braces -Wno-int-conversion

CPP        := cpp
MKLDSCRIPT := tools/mkldscript
MKDMADATA  := tools/mkdmadata
ZAPD       := tools/ZAPD/ZAPD.out

OPTFLAGS := -Og
ASFLAGS := -march=i386 --32 -Iinclude
CFLAGS += -fno-builtin -fsigned-char -std=gnu99 -D _LANGUAGE_C -D NON_MATCHING -DENABLE_OPENGL $(INC) $(CHECK_WARNINGS) -g $(shell sdl2-config --cflags) -fstack-protector-strong -fsanitize=address -fsanitize=pointer-compare -fsanitize=pointer-subtract -fsanitize=undefined
LDFLAGS += -lm -lGL $(shell sdl2-config --libs) -lX11 -lXrandr

ifneq ($(shell getconf LONG_BIT), 32)
  # Ensure that gcc treats the code as 32-bit
  CFLAGS += -m32
endif

#### Files ####

ELF := build/zelda_ocarina_mq_dbg.elf
# description of ROM segments
SPEC := spec

SRC_DIRS := $(shell find src -type d)
ASM_DIRS := $(shell find asm -type d -not -path "asm/non_matchings*") $(shell find data -type d)
ASSET_BIN_DIRS := $(shell find assets/* -type d -not -path "assets/xml*" -not -path "assets/text")
ASSET_FILES_XML := $(foreach dir,$(ASSET_BIN_DIRS),$(wildcard $(dir)/*.xml))
ASSET_FILES_BIN := $(foreach dir,$(ASSET_BIN_DIRS),$(wildcard $(dir)/*.bin))
ASSET_FILES_OUT := $(foreach f,$(ASSET_FILES_XML:.xml=.c),$f) \
				   $(foreach f,$(ASSET_FILES_BIN:.bin=.bin.inc.c),build/$f) \
				   $(foreach f,$(wildcard assets/text/*.c),build/$(f:.c=.o))

# source files
O_FILES       := $(shell cat $(SPEC) | grep '^.*".*\.o"$$' | awk '{ gsub(/"/, "", $$2); print $$2 }' | sort -u)

# Automatic dependency files
# (Only asm_processor dependencies are handled for now)
DEP_FILES := $(O_FILES:.o=.asmproc.d)

TEXTURE_FILES_PNG := $(foreach dir,$(ASSET_BIN_DIRS),$(wildcard $(dir)/*.png))
TEXTURE_FILES_JPG := $(foreach dir,$(ASSET_BIN_DIRS),$(wildcard $(dir)/*.jpg))
TEXTURE_FILES_OUT := $(foreach f,$(TEXTURE_FILES_PNG:.png=.inc.c),build/$f) \
					 $(foreach f,$(TEXTURE_FILES_JPG:.jpg=.jpg.inc.c),build/$f) \

# create build directories
$(shell mkdir -p build/baserom build/assets/text $(foreach dir,$(SRC_DIRS) $(ASM_DIRS) $(ASSET_BIN_DIRS),build/$(dir)))

build/src/code/fault.o: OPTFLAGS := -O2 -g3
build/src/code/fault_drawer.o: OPTFLAGS := -O2 -g3
build/src/code/ucode_disas.o: OPTFLAGS := -O2 -g3
build/src/code/code_801068B0.o: OPTFLAGS := -g
build/src/code/code_80106860.o: OPTFLAGS := -g
build/src/code/code_801067F0.o: OPTFLAGS := -g

build/src/libultra/libc/absf.o: OPTFLAGS := -O2 -g3
build/src/libultra/libc/sqrt.o: OPTFLAGS := -O2 -g3
build/src/libultra/libc/ll.o: OPTFLAGS := -O1
build/src/libultra/libc/llcvt.o: OPTFLAGS := -O1

build/src/libultra/os/%.o: OPTFLAGS := -O1
build/src/libultra/io/%.o: OPTFLAGS := -O2
build/src/libultra/libc/%.o: OPTFLAGS := -O2
build/src/libultra/rmon/%.o: OPTFLAGS := -O2
build/src/libultra/gu/%.o: OPTFLAGS := -O2

#### Main Targets ###

all: $(ELF)

$(ELF): $(TEXTURE_FILES_OUT) $(ASSET_FILES_OUT) $(O_FILES) build/ldscript.txt build/undefined_syms.txt
	@$(CC) $(CFLAGS) -Wl,-T build/ldscript.txt -Wl,--no-check-sections -Wl,--accept-unknown-input-arch -Wl,-Map,build/z64.map -o $@ $(LDFLAGS)

build/$(SPEC): $(SPEC)
	$(CPP) $(CPPFLAGS) $< > $@

build/ldscript.txt: build/$(SPEC)
	$(MKLDSCRIPT) $< $@

build/undefined_syms.txt: undefined_syms.txt
	$(CPP) $(CPPFLAGS) $< > build/undefined_syms.txt

clean:
	$(RM) -r $(ELF) build

assetclean:
	$(RM) -r $(ASSET_BIN_DIRS)
	$(RM) -r assets/text/*.h
	$(RM) -r build/assets
	$(RM) -r .extracted-assets.json

distclean: clean assetclean
	$(RM) -r baserom/
	$(MAKE) -C tools distclean

setup:
	$(MAKE) -C tools
	python3 fixbaserom.py
	python3 extract_baserom.py
	python3 extract_assets.py

resources: $(ASSET_FILES_OUT)

.PHONY: all clean setup test distclean assetclean

#### Various Recipes ####

build/baserom/%.o: baserom/%
	$(OBJCOPY) -I binary -O elf32-big $< $@

build/asm/%.o: asm/%.s
	$(AS) $(ASFLAGS) $< -o $@

build/data/%.o: data/%.s
	iconv --from UTF-8 --to EUC-JP $< | $(AS) $(ASFLAGS) -o $@

build/assets/text/%.enc.h: assets/text/%.h assets/text/charmap.txt
	python3 tools/msgenc.py assets/text/charmap.txt $< $@

build/assets/text/fra_message_data_static.o: build/assets/text/message_data.enc.h
build/assets/text/ger_message_data_static.o: build/assets/text/message_data.enc.h
build/assets/text/nes_message_data_static.o: build/assets/text/message_data.enc.h
build/assets/text/staff_message_data_static.o: build/assets/text/message_data_staff.enc.h

build/assets/%.o: assets/%.c
	$(CC) -c $(CFLAGS) $(OPTFLAGS) -o $@ $<

build/dmadata_table_spec.h: build/$(SPEC)
	$(MKDMADATA) $< $@

build/src/boot/z_std_dma.o: build/dmadata_table_spec.h
build/src/dmadata/dmadata.o: build/dmadata_table_spec.h

build/src/overlays/%.o: src/overlays/%.c
	$(CC) -c $(CFLAGS) $(OPTFLAGS) -o $@ $<
#	$(ZAPD) bovl -eh -i $@ -cfg $< --outputpath $(@D)/$(notdir $(@D))_reloc.s
#	-test -f $(@D)/$(notdir $(@D))_reloc.s && $(AS) $(ASFLAGS) $(@D)/$(notdir $(@D))_reloc.s -o $(@D)/$(notdir $(@D))_reloc.o

build/src/%.o: src/%.c
	$(CC) -c $(CFLAGS) $(OPTFLAGS) -o $@ $<

build/src/libultra/libc/ll.o: src/libultra/libc/ll.c
	$(CC) -c $(CFLAGS) $(OPTFLAGS) -o $@ $<

build/src/libultra/libc/llcvt.o: src/libultra/libc/llcvt.c
	$(CC) -c $(CFLAGS) $(OPTFLAGS) -o $@ $<

build/%.inc.c: %.png
	$(ZAPD) btex -eh -tt $(subst .,,$(suffix $*)) -i $< -o $@

build/assets/%.bin.inc.c: assets/%.bin
	$(ZAPD) bblb -eh -i $< -o $@

build/assets/%.jpg.inc.c: assets/%.jpg
	$(ZAPD) bren -eh -i $< -o $@

-include $(DEP_FILES)
