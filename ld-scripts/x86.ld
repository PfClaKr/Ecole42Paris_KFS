OUTPUT_FORMAT("elf32-i386")
ENTRY(start)

SECTIONS
{
    . = 1M;
    .text : { *(.multiboot_header) *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss : { *(.bss*) }
}
