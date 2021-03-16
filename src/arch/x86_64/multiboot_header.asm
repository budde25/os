section .multiboot_header
header_start:
	dd 0xE85250D6 ; magic number for multiboot 2
	dd 0		  ; arch 0 (protected mode i386)
	dd header_end - header_start ; header length
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

	; multiboot tags go here

	; end tag
	dw 0 ; type
	dw 0 ; flag
	dd 8 ; size
header_end: