use core::marker::PhantomData;

use spin::Mutex;

use crate::arch::{instructions::lgdt, registers::segmentation::SegmentSelector};

/// A Global Descriptor Table for the x86 and x86_64 architecture
/// Based on [OSDev Wiki](https://wiki.osdev.org/Global_Descriptor_Table)
#[repr(C, align(8))]
#[derive(Debug)]
pub struct GlobalDescriptorTable<'a, const N: usize = 64> {
    entries: [u64; N],
    _marker: PhantomData<&'a u8>,
}

impl<const N: usize> GlobalDescriptorTable<'_, N> {
    pub const fn new() -> Self {
        Self {
            entries: [0; N],
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, descriptor: Descriptor) -> SegmentSelector {
        match descriptor {
            Descriptor::User(user) => {
                // We start at 1 because we can't override the null descriptor
                for i in 1..N {
                    if self.entries[i] == 0 {
                        self.entries[i] = user.as_u64();
                        return SegmentSelector((i * size_of::<u64>()) as u16);
                    }
                }
                panic!("no free entries in GDT");
            }
            Descriptor::System() => unimplemented!(),
        }
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
    }
}

impl DescriptorFlags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
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

#[derive(Debug)]
pub enum Descriptor {
    System(),
    User(UserDescriptor),
}

impl Descriptor {
    pub const fn kernel_data() -> Self {
        Self::User(UserDescriptor::new(0xF_FFFF, 0x0000_0000, DescriptorFlags::KERNEL_DATA))
    }

    pub const fn kernel_code64() -> Self {
        Self::User(UserDescriptor::new(
            0xF_FFFF,
            0x0000_0000,
            DescriptorFlags::KERNEL_CODE64,
        ))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserDescriptor(u64);

impl UserDescriptor {
    pub const fn new(limit: u32, base: u32, flags: DescriptorFlags) -> Self {
        let mut desc = flags.bits();

        desc |= (limit & 0xFFFF) as u64;
        desc |= ((base & 0xFFFF) as u64) << 16;
        desc |= ((base & 0xFF_0000) as u64) << 32;
        desc |= ((limit & 0xF_0000) as u64) << 48;
        desc |= ((base & 0xFF00_0000) as u64) << 56;

        Self(desc)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn access_byte(&self) -> u8 {
        ((self.0 >> 40) & 0xFF) as u8
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
}

impl Selectors {
    pub const fn empty() -> Self {
        Self {
            kernel_code: SegmentSelector::NULL,
            kernel_data: SegmentSelector::NULL,
        }
    }
}

pub static GDT: Mutex<(GlobalDescriptorTable, Selectors)> =
    Mutex::new((GlobalDescriptorTable::new(), Selectors::empty()));

/// # SAFETY
/// This function should only be called once
pub unsafe fn init() {
    let mut gdt = GDT.lock();

    let kernel_code = gdt.0.push(Descriptor::kernel_code64());
    let kernel_data = gdt.0.push(Descriptor::kernel_data());

    gdt.0.load();

    unsafe {
        use crate::arch::registers::segmentation;

        segmentation::CS::set_seg(kernel_code);
        segmentation::DS::set_seg(kernel_data);
        segmentation::SS::set_seg(kernel_data);

        segmentation::ES::set_seg(SegmentSelector::NULL);
        segmentation::FS::set_seg(SegmentSelector::NULL);
        segmentation::GS::set_seg(SegmentSelector::NULL);
    }

    gdt.1 = Selectors {
        kernel_code,
        kernel_data,
    }
}
