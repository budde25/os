.section .text
.intel_syntax noprefix
.code64

long_mode:
    # load 0 into all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax    

    call kmain
    # should hit this

    # print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword ptr [0xb8000], rax
    hlt
