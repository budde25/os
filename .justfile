# set positional-arguments

ROOT := justfile_directory()
TARGET_DIR := justfile_directory() + "/target"
OUTPUT_FILE := justfile_directory() + "/target/x86_64-os/debug/os"
QEMU_ARGS := "-serial mon:stdio -smp 2 -drive file=fs.img,index=1,media=disk,format=raw -device isa-debug-exit,iobase=0xf4,iosize=0x04"
QEMU_ARGS_NOX := QEMU_ARGS + " -nographic"
QEMU_TARGET := "-cdrom " + TARGET_DIR + "/os.iso"

is-multiboot2:
    grub-file --is-x86-multiboot2 {{OUTPUT_FILE}}

build target=OUTPUT_FILE:
    cargo build
    @mkdir -p {{TARGET_DIR}}/isofiles/boot/grub
    @cp {{target}} {{TARGET_DIR}}/isofiles/boot/os.bin
    @cp {{ROOT}}/src/arch/x86_64/grub.cfg {{TARGET_DIR}}/isofiles/boot/grub
    @grub-mkrescue -o {{ROOT}}/target/os.iso {{TARGET_DIR}}/isofiles 2> /dev/null
    @rm -r {{TARGET_DIR}}/isofiles

run:
    just build
    qemu-system-x86_64 {{QEMU_ARGS}} {{QEMU_TARGET}}

cargo-run target:
    #!/usr/bin/env sh
    just build {{target}}
    qemu-system-x86_64 {{QEMU_ARGS}} {{QEMU_TARGET}}
    status=$?
    # return status if test
    if [ "$TEST" == "true" ]; then
        [ $status -eq 33 ] && echo "tests PASSED!" || echo "tests FAILED!"
    fi

run-nox:
    just build
    qemu-system-x86_64 {{QEMU_ARGS_NOX}} {{QEMU_TARGET}}

test $TEST="true":
    just build
    cargo test --lib

create-img:
    # this will create a 100 MiB, MBR, Fat32 blank image
    dd if=/dev/zero of=fs.img iflag=fullblock bs=1M count=100 && sync
