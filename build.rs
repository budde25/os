fn main() {
    println!("cargo:rerun-if-changed=src/arch/x86_64/boot_32.s");
    println!("cargo:rerun-if-changed=src/arch/x86_64/boot_64.s");
    println!("cargo:rerun-if-changed=src/arch/x86_64/linker.ld");
}
