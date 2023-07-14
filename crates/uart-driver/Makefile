CPU := cortex-a53
TOOLCHAIN := aarch64-none-elf
CC := $(TOOLCHAIN)-gcc
LD := $(TOOLCHAIN)-ld
AS := $(TOOLCHAIN)-as
AR := $(TOOLCHAIN)-ar

BOARD_DIR := $(SEL4CP_SDK)/board/zcu102/debug

CFLAGS := -mcpu=$(CPU) -mstrict-align -nostdlib -ffreestanding -g3 -O3 -Wall \
	-Wno-unused-function -Werror -I$(BOARD_DIR)/include \
	-Ic_src -Ic_src/common -I.
LDFLAGS := -L$(BOARD_DIR)/lib
LIBS := -lsel4cp -Tsel4cp.ld

UARTPS_SRC := libuartps.c
UARTPS_OBJS = $(UARTPS_SRC:.c=.o)

DRIVER_SRC := $(wildcard c_src/*.c) $(wildcard c_src/common/*.c)
DRIVER_OBJ := $(DRIVER_SRC:.c=.o)
OBJS = $(DRIVER_OBJ) $(UARTPS_OBJS)

all: libuartps.a

clean:
	rm -rf *.o *.a
	rm -rf $(OBJS)

%.o: %.c
	$(CC) -c $(CFLAGS) $< -o $@

%.o: %.s
	$(AS) -g3 -mcpu=$(CPU) $< -o $@

libuartps.a: $(OBJS)
	$(AR) -r $@ $?
