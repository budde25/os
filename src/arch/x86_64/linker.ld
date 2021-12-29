ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;

    /* Start with multiboot then .text */
    .text : ALIGN(4K)
    {
        __text_start = .;
        *(.multiboot)
        *(.text*)
        __text_end = .;
    }

    .page_table : ALIGN(4K) 
    {
        __page_table_start = .;
        *(.page_table)
        __page_table_end = .;
    }

    .page_table_2 : ALIGN(4K) 
    {
        __page_table_2_start = .;
        . += 4096; /* p3 table */
        . += 4096 * 32; /* p2 table */
        *(.page_table_2)
        __page_table_2_end = .;
    }
    
    /* Read only data */
    .rodata : ALIGN(4K)
    {
        __rodata_start = .;
        *(.rodata*)
        __rodata_end = .;
    }

    /* Read/Write data */
    .data : ALIGN(4K) 
    {
        __data_start = .;
        *(.data*)
        __data_end = .;
    }

    /* Read/Write and stack */
    .bss : ALIGN(4K)
    {
        __bss_start = .;
        *(COMMON)
        *(.bss*)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }

}
