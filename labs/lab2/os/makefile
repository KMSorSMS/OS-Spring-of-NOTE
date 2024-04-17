TARGET := riscv64gc-unknown-none-elf
MODE   := release
OBJ_DIR    := target/$(TARGET)/$(MODE)
KERNEL_ELF := $(OBJ_DIR)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(OBJ_DIR)/asm
BOOTLOADER := ../bootloader/rustsbi-qemu.bin
KERNEL_ENTRY := 0x80200000

OBJDUMP := rust-objdump
OBJCOPY := rust-objcopy
ifeq ($(strip $(MODE)),release)
    # 如果 MODE 是 release，执行这里的代码
    MODE_ARG := --release
else
    # 如果 MODE 不是 release，执行这里的代码
    MODE_ARG := 
endif
# MODE_ARG := --$(MODE)

build: env $(KERNEL_BIN)

env:
# 如果rust的target没有安装riscv64gc-unknown-none-elf，则需要下载
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	(cargo install --list | grep "cargo-binutils"  ) || cargo install cargo-binutils
	(rustup component list | grep "rust-src (installed)") || rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Mode is now $(MODE_ARG)
	cargo build $(MODE_ARG)

clean:
	@cargo clean

run: run-inner

run-inner: build
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios $(BOOTLOADER) \
	-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY)

changeEnv:
	$(eval MODE_ARG = )

debug: changeEnv build
	tmux new-session -d \
	"qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios $(BOOTLOADER) \
	-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY) \
	-s -S" 	&& \
	tmux split-window -h \
	"gdb-multiarch -ex 'file  $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target extended-remote localhost:1234'" && \
	tmux -2 attach-session -d
gdbserver: build
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios $(BOOTLOADER) \
	-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY) \
	-s -S
gdbclient: build
	gdb-multiarch -ex 'file  $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target extended-remote localhost:1234'
disasmview: build
	rust-objdump -S target/riscv64gc-unknown-none-elf/$(MODE)/os | less

.PHONY: build run clean gdbserver gdbclient env run-inner disasmview
