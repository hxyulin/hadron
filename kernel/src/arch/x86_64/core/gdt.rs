use core::marker::PhantomData;

use spin::Mutex;

use crate::{
    arch::{
        VirtAddr,
        instructions::lgdt,
        registers::segmentation::SegmentSelector,
        x86_64::core::{idt::PrivilegeLevel, tss::TaskStateSegment},
    },
    util::bits::BitHelper,
};

/// A Global Descriptor Table for the x86 and x86_64 architecture
/// Based on [OSDev Wiki](https://wiki.osdev.org/Global_Descriptor_Table)
#[repr(C, align(8))]
#[derive(Debug)]
pub struct GlobalDescriptorTable<'a, const N: usize = 64> {
    entries: [u64; N],
    len: usize,
    _marker: PhantomData<&'a u8>,
}

impl<const N: usize> GlobalDescriptorTable<'_, N> {
    pub const fn new() -> Self {
        Self {
            entries: [0; N],
            len: 1,
            _marker: PhantomData,
        }
    }

    pub const fn size(&self) -> usize {
        N
    }

    #[inline]
    pub const fn push(&mut self, value: u64) -> usize {
        let index = self.len;
        self.entries[index] = value;
        self.len += 1;
        index
    }

    pub fn append(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::User(value) => {
                assert!(self.len < self.size(), "GDT full");
                self.push(value)
            }
            Descriptor::System(low, high) => {
                assert!(self.len < self.size() - 1, "GDT full");
                let index = self.push(low);
                self.push(high);
                index
            }
        };
        SegmentSelector::new(index as u16, entry.dpl())
    }
}

impl<const N: usize> GlobalDescriptorTable<'static, N> {
    pub fn load(&self) {
        let gdtr = GDTDescriptor {
            size: (N * size_of::<u64>() - 1) as u16,
            offset: self.entries.as_ptr() as usize,
        };
        unsafe { lgdt(&gdtr) };
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DescriptorFlags: u64 {
        const ACCESSED = 1 << 40;
        const WRITABLE = 1 << 41;
        const CONFORMING = 1 << 42;
        const EXECUTABLE = 1 << 43;
        const USER_SEGMENT = 1 << 44;
        const DPL_RING_0 = 0 << 45;
        const DPL_RING_1 = 1 << 45;
        const DPL_RING_2 = 2 << 45;
        const DPL_RING_3 = 3 << 45;
        const PRESENT = 1 << 47;

        const RESERVED = 1 << 52;
        const LONG_MODE = 1 << 53;
        const DEFAULT_SIZE = 1 << 54;
        const GRANULARITY = 1 << 55;

        /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
        const LIMIT_0_15        = 0xFFFF;
        /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
        const LIMIT_16_19       = 0xF << 48;
        /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_0_23         = 0xFF_FFFF << 16;
        /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_24_31        = 0xFF << 56;
    }
}

impl DescriptorFlags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    pub const KERNEL_DATA: Self = Self::from_bits_truncate(Self::COMMON.bits() | Self::DEFAULT_SIZE.bits());

    pub const KERNEL_CODE32: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::DEFAULT_SIZE.bits());

    pub const KERNEL_CODE64: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::LONG_MODE.bits());

    pub const USER_DATA: Self = Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());

    pub const USER_CODE32: Self = Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::DPL_RING_3.bits());

    pub const USER_CODE64: Self = Self::from_bits_truncate(Self::KERNEL_CODE64.bits() | Self::DPL_RING_3.bits());
}

#[derive(Debug, Clone, Copy)]
pub enum Descriptor {
    System(u64, u64),
    User(u64),
}

impl Descriptor {
    pub const fn kernel_data() -> Self {
        Self::User(DescriptorFlags::KERNEL_DATA.bits())
    }

    pub const fn kernel_code64() -> Self {
        Self::User(DescriptorFlags::KERNEL_CODE64.bits())
    }

    pub const fn user_data() -> Self {
        Self::User(DescriptorFlags::USER_DATA.bits())
    }

    pub const fn user_code64() -> Self {
        Self::User(DescriptorFlags::USER_CODE64.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Self {
        let ptr = tss as *const _ as u64;
        let mut low = DescriptorFlags::PRESENT.bits();

        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(16..40, ptr.get_bits(0..24) as u64);
        low.set_bits(56..64, ptr.get_bits(24..32) as u64);

        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        Self::System(low, ptr.get_bits(32..64))
    }

    fn low_part(&self) -> u64 {
        match self {
            Descriptor::User(v) => *v,
            Descriptor::System(v, _) => *v,
        }
    }

    #[inline]
    pub fn dpl(&self) -> PrivilegeLevel {
        let dpl = (self.low_part() & DescriptorFlags::DPL_RING_3.bits()) >> 45;
        PrivilegeLevel::from_u16(dpl as u16)
    }
}

#[repr(C, packed)]
pub struct GDTDescriptor {
    size: u16,
    offset: usize,
}

static_assertions::const_assert_eq!(size_of::<GDTDescriptor>(), size_of::<usize>() + size_of::<u16>());

#[derive(Debug)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

impl Selectors {
    pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

    pub const fn empty() -> Self {
        Self {
            kernel_code: SegmentSelector::NULL,
            kernel_data: SegmentSelector::NULL,
            user_code: SegmentSelector::NULL,
            user_data: SegmentSelector::NULL,
            tss_selector: SegmentSelector::NULL,
        }
    }
}

pub static GDT: Mutex<(GlobalDescriptorTable, Selectors)> =
    Mutex::new((GlobalDescriptorTable::new(), Selectors::empty()));

lazy_static::lazy_static! {
    pub static ref TSS_SEGMENT: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[Selectors::DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 8;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            #[allow(static_mut_refs)]
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE
        };
        tss
    };
}

/// # SAFETY
/// This function should only be called once
pub unsafe fn init() {
    let mut gdt = GDT.try_lock().expect("lock contention!");

    let kernel_code = gdt.0.append(Descriptor::kernel_code64());
    let kernel_data = gdt.0.append(Descriptor::kernel_data());
    let user_code = gdt.0.append(Descriptor::user_code64());
    let user_data = gdt.0.append(Descriptor::user_data());
    let tss_selector = gdt.0.append(Descriptor::tss_segment(&TSS_SEGMENT));

    gdt.0.load();
    unsafe {
        use crate::arch::registers::segmentation;

        segmentation::CS::set_seg(kernel_code);
        segmentation::DS::set_seg(kernel_data);
        segmentation::SS::set_seg(kernel_data);

        segmentation::ES::set_seg(SegmentSelector::NULL);
        segmentation::FS::set_seg(SegmentSelector::NULL);
        segmentation::GS::set_seg(SegmentSelector::NULL);

        crate::arch::instructions::load_tss(tss_selector);
    }

    gdt.1 = Selectors {
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss_selector,
    }
}
