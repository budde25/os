.code32

.set MAGIC, 0xE85250D6
.set VGA, 0xB8000

.section .multiboot
.align 4
multiboot_start:
    .long MAGIC # Magic multiboot 2
    .long 0x00000000 # Arch 0
    .long multiboot_end - multiboot_start # Header length
    # Checksum
    .long 0x100000000 - (MAGIC + 0 + (multiboot_end - multiboot_start))

    # End tag
    .word 0x0000 # Type
    .word 0x0000 # Flag
    .long 0x00000008 # Size
multiboot_end:

.section .page_table, "aw"
.align 4096
p4_table:
    .skip 4096
p3_table:
    .skip 4096
p2_table:
    .skip 4096

.section .bss
.align 4096
stack_bottom:
    .skip 16384 # 16 Kib
stack_top:

.section .text
.global _start
.type _start, @function
_start:
    mov esp, offset stack_top # stack grows from downwards
  
    call check_multiboot
    call check_cpuid
    call check_long_mode

    call setup_page_tables
    call enable_paging

    call load_gdt64
    
    # jump to long mode
    push 0x8
    mov eax, offset long_mode
    push eax
    retf

    # unreachable
    hlt

# https://nongnu.askapache.com/grub/phcoder/multiboot.pdf
# since we booted from grub multiboot we are probably in multiboot but good to check_multiboot
# non multiboot in the future
check_multiboot:
    cmp eax, 0x36d76289
    jne no_multiboot
    ret
no_multiboot:
    mov al, error_0_str
    jmp error

# https://wiki.osdev.org/Setting_Up_Long_Mode#Dectection_of_CPUID
check_cpuid:
    # Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
    # the FLAGS register. If we can flip it, CPUID is available.
 
    # Copy FLAGS in to EAX via stack
    pushfd
    pop eax
 
    # Copy to ECX as well for comparing later on
    mov ecx, eax
 
    # Flip the ID bit
    xor eax, (1 << 21)
 
    # Copy EAX to FLAGS via the stack
    push eax
    popfd
 
    # Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax
 
    # Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
    # back if it was ever flipped).
    push ecx
    popfd
 
    # Compare EAX and ECX. If they are equal then that means the bit wasn't
    # flipped, and CPUID isn't supported.
    cmp eax, ecx
    je no_cpuid
    ret
no_cpuid:
    mov al, error_1_str
    jmp error

# https://wiki.osdev.org/Setting_Up_Long_Mode#x86_or_x86-64
check_long_mode:
    # test if extended processor info in available
    mov eax, 0x80000000    # implicit argument for cpuid
    cpuid                  # get highest supported argument
    cmp eax, 0x80000001    # it needs to be at least 0x80000001
    jb no_long_mode        # if it's less, the CPU is too old for long mode

    # use extended info to test if long mode is available
    mov eax, 0x80000001    # argument for extended processor info
    cpuid                  # returns various feature bits in ecx and edx
    test edx, (1 << 29)    # test if the LM-bit is set in the D-register
    jz no_long_mode        # If it's not set, there is no long mode
    ret
no_long_mode:
    mov al, error_2_str
    jmp error

# https://os.phil-opp.com/entering-longmode/
setup_page_tables:
    # map first P4 entry to P3 table 
    mov eax, offset p3_table
    or eax, 0b11 # present + writable
    mov dword ptr [p4_table], eax

    # map first P3 entry to P2 table
    mov eax, offset p2_table
    or eax, 0b11 # present + writable
    mov dword ptr [p3_table], eax

    mov ecx, 0 # counter

map_p2_table:
    # map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000  # 2MiB
    mul ecx            # start address of ecx-th page
    or eax, 0b10000011 # present + writable + huge
    mov dword ptr [p2_table + ecx * 8], eax # map ecx-th entry

    inc ecx            # increase counter
    cmp ecx, 512       # if counter == 512, the whole P2 table is mapped
    jne map_p2_table   # else map the next entry

    ret

# https://os.phil-opp.com/entering-longmode/
enable_paging:
    # load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    # enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    # set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    # enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

load_gdt64:
    lgdt [gdt_64_pointer]
    ret

# prints 'Err: ' and an error code stored in al regiester
error:
    mov dword ptr [VGA], 0x4f524f45
    mov dword ptr [VGA + 4], 0x4f3a4f52
    mov dword ptr [VGA + 8], 0x4f204f20
    mov byte ptr [VGA + 10], al
    hlt

error_0_str: # no_multiboot
    .asciz "0"
error_1_str: # no_cpuid
    .asciz "1"
error_2_str: # no_long_mode
    .asciz "2"
error_3_str: # failed the jump to long mode
    .asciz "3"

.section .rodata
gdt_64:
    .quad 0x0000000000000000          # Null Descriptor - should be present.
gdt_code:
    .quad 0x00209A0000000000          # 64-bit code descriptor (exec/read).
gdt_data:
    .quad 0x0000920000000000          # 64-bit data descriptor (read/write).
.align 4
    .word 0                              # Padding to make the "address of the GDT" field aligned on a 4-byte boundary

gdt_64_pointer:
    .word gdt_64_pointer - gdt_64 - 1    # 16-bit Size (Limit) of GDT.
    .long gdt_64
