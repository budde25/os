.set MAGIC, 0xE85250D6
.set VGA, 0xB8000

.section .text
.align 4
multiboot_start:
    .long MAGIC # Magic multiboot 2
    .long 0x00000000 # Arch 0
    .long multiboot_end - multiboot_start # Header length
    # Checksum
    .long 0x100000000 - (MAGIC + 0 + (multiboot_end - multiboot_start))

    # End tag
    .long 0x00000000 # Type + Flag
    .long 0x00000008 # Size
multiboot_end:

.section .bss
stack_bottom:
    .skip 16384 # 16 Kib
stack_top:

.section .text
.global _start
.type _start, @function
.code32
_start:
    movl $stack_top, %esp

    #call check_multiboot
    #hlt
    #call check_cpuid
    #call check_long_mode

    # Prints OK to the screen
    movl $0x2f4b2f4f, (VGA)

    cli
    hlt

check_multiboot:
    cmpl 0x36d76289, %eax
    jne no_multiboot
    ret
no_multiboot:
    movb error_0_str, %al
    jmp error

# https://wiki.osdev.org/Setting_Up_Long_Mode#Dectection_of_CPUID
check_cpuid:
    # Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
    # the FLAGS register. If we can flip it, CPUID is available.
 
    # Copy FLAGS in to EAX via stack
    pushfd
    popl %eax
 
    # Copy to ECX as well for comparing later on
    movl %eax, %ecx
 
    # Flip the ID bit
    xor (1 << 21), %eax
 
    # Copy EAX to FLAGS via the stack
    pushl %eax
    popfd
 
    # Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    popl %eax
 
    # Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
    # back if it was ever flipped).
    pushl %ecx
    popfd
 
    # Compare EAX and ECX. If they are equal then that means the bit wasn't
    # flipped, and CPUID isn't supported.
    cmpl %eax, %ecx
    je no_cpuid
    ret
no_cpuid:
    movb error_1_str, %al
    jmp error


check_long_mode:
    # test if extended processor info in available
    movl 0x80000000, %eax    # implicit argument for cpuid
    cpuid                  # get highest supported argument
    cmpl 0x80000001, %eax    # it needs to be at least 0x80000001
    jb no_long_mode        # if it's less, the CPU is too old for long mode

    # use extended info to test if long mode is available
    movl 0x80000001, %eax    # argument for extended processor info
    cpuid                  # returns various feature bits in ecx and edx
    testl %edx, 1 << 29  # test if the LM-bit is set in the D-register
    jz no_long_mode
    ret
no_long_mode:
    movb error_2_str, %al
    jmp error

# prints 'Err: ' and an error code stored in al regiester
error:
    movl $0x4f524f45, (VGA)
    movl $0x4f3a4f52, (VGA + 4)
    movl $0x4f204f20, (VGA + 8)
    movb %al, (VGA + 12)
    hlt

error_0_str: # no_multiboot
    .asciz "0"
error_1_str: # no_cpuid
    .asciz "1"
error_2_str: # no_long_mode
    .asciz "2"
