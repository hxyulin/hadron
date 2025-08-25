use crate::{
    arch::{PhysAddr, VirtAddr},
    mm::paging::{PhysFrame, Size4KiB},
};

pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtAddr {
        let out: usize;
        unsafe {
            core::arch::asm!(
                "mov {}, cr2",
                out(reg) out,
                options(nostack, preserves_flags)
            );
        }
        VirtAddr::new(out)
    }
}

pub struct Cr3;

impl Cr3 {
    const ADDR_MASK: usize = 0xFFFF_FFFF_FFFF_F000;

    fn read() -> usize {
        let out: usize;
        unsafe {
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) out,
                options(nostack, preserves_flags)
            );
        }
        out
    }

    unsafe fn write_internal(val: usize) {
        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) val,
                options(nostack, preserves_flags)
            )
        }
    }

    pub fn addr() -> PhysAddr {
        PhysAddr::new(Self::read() & Self::ADDR_MASK)
    }

    pub unsafe fn write(frame: PhysFrame<Size4KiB>, flags: Cr3Flags) {
        //assert_unsafe_precondition!()
        unsafe {
            Self::write_internal(frame.start_address().as_usize() | flags.bits());
        }
    }
}

bitflags::bitflags! {
    pub struct Cr3Flags: usize {
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
    }
}
