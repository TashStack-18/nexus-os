/* Memory layout of the Micro-OS on a PC (x86-64), loaded by GRUB.
 *
 *   0x0010_0000  kernel image: Multiboot2 header, .text .rodata .data
 *                (boot page tables + GDT), boot stack, .bss
 *   __heap_start kernel heap (HEAP_SIZE bytes, see src/memory/mod.rs)
 *
 * The boot code identity-maps the first 1 GiB, so these addresses are both
 * physical and virtual.
 */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)
BASE_ADDRESS = 0x100000;

SECTIONS
{
    . = BASE_ADDRESS;
    __kernel_start = .;

    .text : {
        KEEP(*(.multiboot2))    /* must be within the first 32 KiB of the image */
        *(.text.boot32)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    .data : {
        *(.data.boot)
        *(.data .data.*)
    }

    . = ALIGN(4K);
    .stack (NOLOAD) : {
        *(.bss.boot_stack)      /* outside __bss_start..__bss_end: the boot code clears .bss while using it */
    }

    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    }

    . = ALIGN(4K);
    __kernel_end = .;
    __heap_start = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
    }
}
