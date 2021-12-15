# set positional-arguments

ROOT := justfile_directory()
TARGET_DIR := justfile_directory() + "/target"
OUTPUT_FILE := justfile_directory() + "/target/x86_64-os/debug/os"
QEMU_ARGS := "-serial mon:stdio -smp 2"
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
    just build {{target}}
    qemu-system-x86_64 {{QEMU_ARGS}} {{QEMU_TARGET}}

run-nox:
    just build
    qemu-system-x86_64 {{QEMU_ARGS_NOX}} {{QEMU_TARGET}}

test:
    just build
    cargo test
