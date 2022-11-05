#!/usr/bin/python3

import argparse
import os
import sys
import subprocess
import shutil

QEMU_ARGS = [
    "-serial",
    "mon:stdio",
    "-smp",
    "1",
    "-boot",
    "order=d",
    "-drive",
    "file=fs.img,index=1,media=disk,format=raw",
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
]


class X(object):
    def __init__(self):
        parser = argparse.ArgumentParser()
        parser.add_argument("command", help="available commands")
        parser.add_argument(
            "-v",
            "--version",
            help="show version and exit",
            action="version",
            version="1.0",
        ),
        # Read the first argument (add/commit)
        args = parser.parse_args(sys.argv[1:2])
        # use dispatch pattern to invoke method with same name of the argument
        parser.add_subparsers()
        getattr(self, args.command)()

    def doctor(self):
        parser = argparse.ArgumentParser(description="doctor")
        parser.add_argument("-f", "--file-name", required=True, help="file to be added")
        # we are inside a subcommand, so ignore the first argument and read the rest
        args = parser.parse_args(sys.argv[2:])
        doctor(args)

    def build(self):
        parser = argparse.ArgumentParser(description="build")
        # we are inside a subcommand, so ignore the first argument and read the rest
        args = parser.parse_args(sys.argv[2:])
        build(args)

    def run(self):
        parser = argparse.ArgumentParser(description="run")
        parser.add_argument(
            "--nox", help="run without x window", default=False, action="store_true"
        )
        args = parser.parse_args(sys.argv[2:])
        run(args)


def main():
    X()


def doctor(args):
    print(f"Found the following in path")
    print(f"grub-mkrescue: {is_tool('grub-mkrescue')}")
    print(f"xorriso: {is_tool('xorriso')}")
    print(f"qemu: {is_tool('qemu-system-x86_64')}")


def build(args):
    subprocess.run(["cargo", "build"], check=True)
    os.makedirs(f"{target_dir()}/isofiles/boot/grub", exist_ok=True)
    shutil.copy(output_file(), f"{target_dir()}/isofiles/boot/os.bin")
    shutil.copy(
        f"{get_root()}/src/arch/x86_64/grub.cfg", f"{target_dir()}/isofiles/boot/grub"
    )
    try:
        subprocess.run(
            [
                "grub-mkrescue",
                "-o",
                f"{target_dir()}/os.iso",
                f"{target_dir()}/isofiles",
            ],
            check=True,
        )
    except:
        print("failed to run grub-mkrescue, trying grub2-mkrescue")

    subprocess.run(
        [
            "grub2-mkrescue",
            "-o",
            f"{target_dir()}/os.iso",
            f"{target_dir()}/isofiles",
        ],
        check=True,
    )

    shutil.rmtree(f"{target_dir()}/isofiles")


def run(args):
    build(args)
    command = ["qemu-system-x86_64"]
    if args.nox:
        command.append("-nographic")
    command.extend(QEMU_ARGS)
    # append the target iso

    command.append("-cdrom")
    command.append(f"{target_dir()}/os.iso")

    subprocess.run(command)


def get_root() -> str:
    return os.path.dirname(os.path.abspath(__file__))


def target_dir() -> str:
    return f"{get_root()}/target"


def output_file() -> str:
    return f"{target_dir()}/x86_64-os/debug/os"


def is_tool(name: str) -> bool:
    """Check whether `name` is on PATH and marked as executable."""

    from shutil import which

    return which(name) is not None

def create_img():
    # this will create a 100 MiB, MBR, Fat32 blank image
    try:
        subprocess.run(
            [
                "dd",
                "if=/dev/zero",
                "of=fs.img",
                "iflag=fullblock",
                "bs=1M",
                "count=10"
                "&&"
                "sync"
            ],
            check=True,
        )
    except:
        print("Failed to run dd")

    try:
        subprocess.run(
            [
                "mkfs.ext2",
                "fs.img"
            ],
            check=True,
        )
    except:
        print("Failed to run mkfs.ext2")

if __name__ == "__main__":
    main()
