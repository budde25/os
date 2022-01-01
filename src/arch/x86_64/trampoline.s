.code16
.section .mp_boot

.global start
start:
    cli
    int 3