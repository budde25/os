.code32
.section .mp_boot

start:
    mov al, error_4_str
    jmp error

error_4_str: # failed the jump to long mode
    .asciz "4"