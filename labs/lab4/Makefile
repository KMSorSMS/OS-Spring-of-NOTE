DOCKER_NAME ?= rcore-tutorial-v3
.PHONY: docker build_docker

TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := ./os/target/$(TARGET)/$(MODE)/os

BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ./bootloader/$(SBI)-$(BOARD).bin
CHAPTER ?= $(shell git rev-parse --abbrev-ref HEAD | sed -E 's/ch([0-9])/\1/')
TEST ?= $(CHAPTER)
BASE ?= 0
docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt ${DOCKER_NAME} bash

build_docker: 
	docker build -t ${DOCKER_NAME} .

fmt:
	cd os ; cargo fmt;  cd ..

run:
	@make -C ./ci-user/user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@make -C ./os run
build:
	@make -C ./ci-user/user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@make -C ./os kernel
debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-kernel $(KERNEL_ELF)\
		-s -S" && \
		tmux split-window -h " RUST_GDB=/usr/bin/gdb-multiarch rust-gdb  -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'source ./.gdbinit' " && \
		tmux -2 attach-session -d
