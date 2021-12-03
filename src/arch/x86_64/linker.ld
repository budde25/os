ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;
    
    /* Start with multiboot then .text */
    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
        *(.text)
    }
    
    /* Read only data */
    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    }

    /* Read/Write data */
    .data BLOCK(4K) : ALIGN(4K) 
    {
        *(.data)
    }

    /* Read/Write and stack */
    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
