.code16
.section .mp_boot

.global start
start:
    jmp spin
spin:
    jmp spin
