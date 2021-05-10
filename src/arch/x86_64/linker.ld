ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;

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

    end = .;
}
