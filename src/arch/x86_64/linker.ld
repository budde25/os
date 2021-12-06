ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;
    
    /* Start with multiboot then .text */
    .text : ALIGN(4K)
    {
        *(.multiboot)
        *(.text*)
    }
    
    /* Read only data */
    .rodata : ALIGN(4K)
    {
        *(.rodata*)
    }

    /* Read/Write data */
    .data : ALIGN(4K) 
    {
        *(.data*)
    }

    /* Read/Write and stack */
    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }

}
