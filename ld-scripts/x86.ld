OUTPUT_FORMAT("elf32-i386")
ENTRY(start)

PHDRS {
    boot PT_LOAD;
    text PT_LOAD;
    data PT_LOAD;
    rodata PT_LOAD;
}

SECTIONS {

    . = 2M;

    .multiboot ALIGN(8) : {
        KEEP(*(.multiboot))
    } : boot
    
    .text ALIGN(4K) : { 
        *(.text*)
    } : text
    .rodata ALIGN(4K) : { *(.rodata*) } : rodata
    .data ALIGN(4K) : { *(.data*) } : data
    .bss ALIGN(4K) : { *(.bss*) } : data
}
