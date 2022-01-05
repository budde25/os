# Test os

A test os (x86_64) written in Rust. Just for fun.

Features:  

- [x] Multiboot2
- [x] VGA Buffer
- [x] Serial port
- [x] GDT
- [x] IDT
- [x] TSS
- [X] PIC (masking and disabling)
- [x] CMOS
- [x] Keyboard
- [X] LAPIC
- [X] IOAPIC
- [x] Paging (w/ memory mapped physical addresses)
- [x] ATA

WIP:
- [ ] Disk Buffer Cache
- [ ] Ext2
- [ ] Smp support

## Usage

Must have `grub-mkrescue`, `qemu`, `rust`, `cargo-make`, `xorriso`, `mtools`.  
Only tested on linux machines.  
`just run` in the root directory will create an iso and run it.  
`just test` will start run unit tests.  

## References

[OSdev Wiki](https://wiki.osdev.org/Main_Page)  
[Philipp Oppermann's blog](https://os.phil-opp.com/)  
[MIT's XV6 (x86 edition)](https://pdos.csail.mit.edu/6.828/2019/xv6.html)  
