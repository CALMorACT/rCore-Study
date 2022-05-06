#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: compile/run/debug"
    exit 1
fi

BOOTLOADER=../bootloader/rustsbi-qemu.bin
FS_IMG=target/riscv64gc-unknown-none-elf/release/study.bin

case ${1} in
"compile")
    echo "Try to complie target"
    cargo build --release
    echo "Strip target"
    rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/study -O binary target/riscv64gc-unknown-none-elf/release/study.bin
    echo 'Over Complied'
    ;;

"run")
    echo "Start Runing in QEMU"
    qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ${BOOTLOADER} \
    -device loader,file=${FS_IMG},addr=0x80200000
    ;;
"debug")
    echo "Start Debug in QEMU"
    qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ${BOOTLOADER} \
    -device loader,file=${FS_IMG},addr=0x80200000 \
    -s -S
    ;;
esac

