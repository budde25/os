global start
extern long_mode_start

section .text
bits 32
start:
    mov esp, stack_top ; update stack pointer register

    ; sanity check
    call check_multiboot
    call check_cpuid
    call check_long_mode
    
    ; setup paging
    call setup_page_tables
    call enable_paging

    ; load the 64-bit GDT
    lgdt [gdt64.pointer]
    
    jmp gdt64.code:long_mode_start

; check if we have a multiboot2 complient bootloder
check_multiboot:
    cmp eax, 0x36d76289 ; magic number that bootloader should put in eax if it supports mutiboot2 (https://wiki.osdev.org/Multiboot#Multiboot_2)
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0" ; error code 0, bootloader doesn't support multiboot2
    jmp error

check_cpuid: ; (https://wiki.osdev.org/Setting_Up_Long_Mode#Detection_of_CPUID)
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
    ; the FLAGS register. If we can flip it, CPUID is available.
 
    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax
 
    ; Copy to ECX as well for comparing later on
    mov ecx, eax
 
    ; Flip the ID bit
    xor eax, 1 << 21
 
    ; Copy EAX to FLAGS via the stack
    push eax
    popfd
 
    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax
 
    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
    ; back if it was ever flipped).
    push ecx
    popfd
 
    ; Compare EAX and ECX. If they are equal then that means the bit wasn't
    ; flipped, and CPUID isn't supported.
    xor eax, ecx
    jz .no_cpuid
    ret
.no_cpuid:
    mov al, "1" ; error code 1 no cpuid
    jmp error

check_long_mode: ; (https://wiki.osdev.org/Setting_Up_Long_Mode#x86_or_x86-64)
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, "2" ; error code 2 no long mode support (aka 64 bit)
    jmp error

setup_page_tables:
    ; map first P4 entry to P3 table
    mov eax, pdp
    or eax, 0b11 ; present + writable
    mov [pml4], eax

    ; map first P3 entry to P2 table
    mov eax, pd
    or eax, 0b11 ; present + writable
    mov [pdp], eax

    ; map each P2 entry to a huge 2MiB page
    mov ecx, 0         ; counter variable

.map_pd_table:
    ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page
    or eax, 0b10000011 ; present + writable + huge
    mov [pd + ecx * 8], eax ; map ecx-th entry

    inc ecx            ; increase counter
    cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
    jne .map_pd_table  ; else map the next entry

    ret

enable_paging:
    ; load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, pml4
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

; prints error to the vga buffer
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .bss
align 4096

; Page table (https://pages.cs.wisc.edu/%7Eremzi/OSTEP/vm-paging.pdf)
pml4: ; Page map level 4 table
    resb 4096
pdp: ; Page directory pointer table
    resb 4096
pd:  ; page directory table
    resb 4096
pt: ; page table
    resb 64

stack_bottom:
    resb 4096 * 4 ; four page stack
stack_top:

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64