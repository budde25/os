ENTRY(_start)

SECTIONS
{
    /* Conventional starting place */
    . = 1M;

    /* Start with multiboot then .text */
    .text : ALIGN(4K)
    {
        __text_start = .;
        KEEP(*(.multiboot*))
        *(.text*)
        __mp_boot_start = .;
        *(.mp_boot)
        __mp_boot_end = .;
        __text_end = .;
    }

    .page_table : ALIGN(4K) 
    {
        __page_table_start = .;
        KEEP(*(.page_table))
        __page_table_end = .;
        __phys_page_table_start = .;
       KEEP(*(.phys_page_table))
        __phys_page_table_end = .;
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
