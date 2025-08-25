use core::arch::asm;

use crate::arch::x86_64::core::idt::PrivilegeLevel;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const NULL: Self = Self(0);

    pub const fn new(index: u16, dpl: PrivilegeLevel) -> Self {
        Self(index << 3 | dpl as u16)
    }
}

macro_rules! impl_register {
    ($name:ident, $reg:expr) => {
        impl $name {
            #[inline]
            pub fn get_seg() -> SegmentSelector {
                let segment: u16;
                unsafe {
                    asm!(
                        concat!("mov {0:x}, ", $reg),
                        out(reg) segment,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                SegmentSelector(segment)
            }

            #[inline]
            pub unsafe fn set_seg(sel: SegmentSelector) {
                unsafe {
                    asm!(
                        concat!("mov ", $reg, ", {0:x}"),
                        in(reg) sel.0,
                        options(nostack, preserves_flags)
                    );
                }
            }
        }
    }
}

pub struct CS;
pub struct DS;
pub struct SS;
pub struct ES;
pub struct FS;
pub struct GS;

pub trait Segment {}

impl CS {
    #[inline]
    pub fn get_seg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!(
               "mov {0:x}, cs",
                out(reg) segment,
                options(nomem, nostack, preserves_flags)
            );
        }
        SegmentSelector(segment)
    }

    #[inline]
    pub unsafe fn set_seg(sel: SegmentSelector) {
        unsafe {
            asm!(
                "push {sel}",
                "lea {tmp}, [55f + rip]",
                "push {tmp}",
                "retfq",
                "55:",
                sel = in(reg) u64::from(sel.0),
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }
}

impl_register!(DS, "ds");
impl_register!(SS, "ss");
impl_register!(ES, "es");
impl_register!(FS, "fs");
impl_register!(GS, "gs");
