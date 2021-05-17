ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;
    .boot :
    {
        KEEP(*(.multiboot))
    }
    
    /* Start with multiboot then .text */
    .text :
    {
        *(.boot)
        *(.text)
    }
    
    /* Read only data */
    .rodata :
    {
        *(.rodata)
    }

    /* Read/Write data */
    .data :
    {
        *(.data)
    }

    /* Read/Write and stack */
    .bss :
    {
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
