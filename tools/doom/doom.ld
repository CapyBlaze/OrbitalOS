ENTRY(_start)

SECTIONS {
    . = 0x04000000;

    .text.start : { KEEP(*(.text._start)) }
    .text       : { *(.text .text.*) }
    .rodata     : { *(.rodata .rodata.*) }
    .data       : { *(.data .data.*) }

    __bss_start = .;
    .bss        : { *(.bss .bss.*) *(COMMON) }
    __bss_end = .;
}