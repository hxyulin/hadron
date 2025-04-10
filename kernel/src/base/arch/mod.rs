use arch_x86_64::gdt::{AccessByte, GdtFlags, GlobalDescriptorTable, GlobalDescriptorTableEntry};

lazy_static::lazy_static! {
    pub static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();

        gdt[0] = GlobalDescriptorTableEntry::null();
        gdt[1] = GlobalDescriptorTableEntry::new(
            AccessByte::PRESENT | AccessByte::READ_WRITE | AccessByte::EXECUTABLE | AccessByte::DESC_TYPE,
            GdtFlags::LONG_MODE | GdtFlags::GRANULARITY,
            0xFFFFF,
            0,
        );
        gdt[2] = GlobalDescriptorTableEntry::new(
            AccessByte::PRESENT | AccessByte::READ_WRITE | AccessByte::DESC_TYPE,
            GdtFlags::GRANULARITY | GdtFlags::DB,
            0xFFFFF,
            0,
        );

        assert_eq!(gdt[1].access().bits(), 0x9A);
        assert_eq!(gdt[1].flags().bits(), 0x0A);
        assert_eq!(gdt[2].access().bits(), 0x92);
        assert_eq!(gdt[2].flags().bits(), 0x0C);

        gdt
    };
}


pub fn init_gdt() {
    unsafe {
        GDT.load();
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [55f + rip]",
            "push {tmp}",
            "retfq",
            "55:",
            sel = in(reg) 0x08u64,
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
        core::arch::asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
        );
    };
}
