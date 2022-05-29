#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: compile/run/debug"
    exit 1
fi

BOOTLOADER=../bootloader/rustsbi-qemu.bin
FS_IMG=target/riscv64gc-unknown-none-elf/release/kernel.bin
DOCKER_NAME=dinghao188/rcore-tutorial

case ${1} in
"compile")
    cd user
    echo 'Try to complie user lib and some application'
    cargo build --release
    cd ..
    echo "Try to complie target"
    cd kernel
    cargo build --release
    echo "Strip target"
    rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/kernel -O binary target/riscv64gc-unknown-none-elf/release/kernel.bin
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
'docker')
    echo 'start docker...'
    docker run --rm -it --mount type=bind,source=`pwd`,destination=/mnt "${DOCKER_NAME}"
    ;;
*)
   echo 'No useful arguments'
   ;;
esac

