.section .boot, "awx"
.intel_syntax noprefix
.code16

second_stage_start_str: .asciz "Booting (second stage)..."

stage_2:

    mov si, offset second_stage_start_str
    call real_mode_println

set_target_operating_mode:
    # Some BIOSs assume the processor will only operate in Legacy Mode. We change the Target
    # Operating Mode to "Long Mode Target Only", so the firmware expects each CPU to enter Long Mode
    # once and then stay in it. This allows the firmware to enable mode-specifc optimizations.
    # We save the flags, because CF is set if the callback is not supported (in which case, this is
    # a NOP)
    pushf
    mov ax, 0xec00
    mov bl, 0x2
    int 0x15
    popf

video_mode_config:
    call config_video_mode_80x25

enter_protected_mode_again:
    cli
    lgdt [gdt32info]
    mov eax, cr0
    or al, 1    # set protected mode bit
    mov cr0, eax

    push 0x8
    mov eax, offset stage_3
    push eax
    retf

spin32:
    jmp spin32

config_video_mode_320x200:
    mov ah, 0
    mov al, 0x13 # 320x200 256 color graphics
    int 0x10
    ret

config_video_mode_80x25:
    mov ah, 0
    mov al, 0x03 # 80x25 16 color text
    int 0x10
    ret