# Test os

A test os (x86_64) written in Rust. Just for fun.

Features:  

- [x] Multiboot
- [x] VGA Buffer
- [x] Serial port
- [x] GDT
- [x] IDT
- [x] TSS

Working on:

- [ ] APIC

## Usage

Must have `grub-mkrescue`, `qemu`, `rust`, `cargo-make`, `xorriso`.  
Only tested on linux machines.  
`cargo run` in the root directory will create an iso and run it.  
`cargo test` will start run unit tests.  

## References

[OSdev Wiki](https://wiki.osdev.org/Main_Page)  
[Philipp Oppermann's blog](https://os.phil-opp.com/)  
[MIT's XV6 (x86 edition)](https://pdos.csail.mit.edu/6.828/2019/xv6.html)  
