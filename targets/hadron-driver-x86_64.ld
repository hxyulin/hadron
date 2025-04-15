/* Linker Script for Hadron Drivers (x86_64) */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(kernel_entry)

PHDRS
{
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    dyn PT_LOAD FLAGS(6);
    data PT_LOAD FLAGS(6);
    eh_frame PT_LOAD FLAGS(4);
    bss PT_LOAD FLAGS(6);
    dynamic PT_DYNAMIC FLAGS(4);
    gnu_relro PT_GNU_RELRO FLAGS(6);
    eh_frame_hdr PT_GNU_EH_FRAME FLAGS(4);
}

SECTIONS
{
    .text : {
        *(.text .text.*)
    } :text

    /* Move to the next memory page for .rodata */
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .dynsym : { *(.dynsym .dynsym.*) } :rodata
    .gnu.hash : { *(.gnu.hash .gnu.hash.*) } :rodata
    .hash : { *(.hash .hash.*) } :rodata
    .dynstr : { *(.dynstr .dynstr.*) } :rodata
    .rela.dyn : { *(.rela.dyn .rela.dyn.*) } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .dynamic : {
        *(.dynamic .dynamic.*)
    } :dyn :dynamic :gnu_relro

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .got : {
        *(.got .got.*)
        *(.got.plt .got.plt.*)
    } :dyn :gnu_relro
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        /* Place the sections that contain the Limine requests as part of the .data */
        /* output section. */
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))

        *(.data .data.*)
    } :data

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .eh_frame : {
        __EH_FRAME_START = .;
        *(.eh_frame .eh_frame.*)
        __EH_FRAME_END = .;
    } :eh_frame

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .eh_frame_hdr : {
        __EH_FRAME_HDR_START = .;
        *(.eh_frame_hdr)
        __EH_FRAME_HDR_END = .;
    } :eh_frame_hdr :eh_frame

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    /* NOTE: .bss needs to be the last thing mapped to :data, otherwise lots of */
    /* unnecessary zeros will be written to the binary. */
    /* If you need, for example, .init_array and .fini_array, those should be placed */
    /* above this. */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :bss

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    /DISCARD/ : {
        *(.note .note.*)
    }
}

