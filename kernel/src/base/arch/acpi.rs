use alloc::{boxed::Box, collections::btree_map::BTreeMap, sync::Arc};
use spin::{Mutex, rwlock::RwLock};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageTableFlags, PhysFrame, frame::PhysFrameRangeInclusive, page::PageRangeInclusive},
};

use crate::{
    base::{
        info::kernel_info,
        io::mmio::allocate_persistent_mmio,
        mem::{map_page, unmap_page},
    },
    util::timer::hpet::Hpet,
};
use core::ptr::NonNull;

/// Initialize the ACPI subsystem
///
/// This includes:
/// - Parsing the ACPI tables
/// - Setting up the RSDP
pub fn init(rsdp: PhysAddr) {
    let mapper = AcpiMapper::new();
    let tables =
        unsafe { acpi::AcpiTables::from_rsdp(mapper, rsdp.as_u64() as usize) }.expect("failed to parse ACPI tables");
    let platform_info = tables.platform_info().expect("failed to parse platform info");
    match acpi::HpetInfo::new(&tables) {
        Ok(hpet) => parse_hpet_info(hpet),
        Err(e) => log::warn!("ACPI: failed to parse HPET info: {:?}", e),
    }
    parse_platform_info(&platform_info);
}

fn parse_hpet_info(hpet: acpi::HpetInfo) {
    let virt_addr = allocate_persistent_mmio(PhysAddr::new(hpet.base_address as u64), Hpet::SIZE_ALIGNED);
    let mut hpet = Hpet::new(virt_addr, hpet);
    unsafe { hpet.init() };
    kernel_info().timer.init_once(|| RwLock::new(Box::new(hpet)));
}

/// Parses the platform info and populates data structures
fn parse_platform_info(platform_info: &acpi::PlatformInfo<alloc::alloc::Global>) {
    log::debug!("ACPI: parsing platform info...");
    match platform_info.power_profile {
        acpi::PowerProfile::Unspecified => log::warn!("ACPI: unspecified power profile"),
        _ => {}
    }
    match &platform_info.interrupt_model {
        acpi::InterruptModel::Apic(apic) => {
            log::debug!("ACPI: interrupt model: APIC");
            log::debug!("{:?}", apic);
        }
        _ => panic!("ACPI: unknown/unsupported interrupt model"),
    }
    log::info!("ACPI: platform info: {:?}", platform_info);
}

/// A mapper to map ACPI frames to logical addresses
#[derive(Debug, Clone)]
pub struct AcpiMapperInner {
    mapped_frames: BTreeMap<u64, usize>,
}

impl AcpiMapperInner {
    pub const fn default() -> Self {
        Self {
            mapped_frames: BTreeMap::new(),
        }
    }

    unsafe fn map_frame(&mut self, frame: PhysFrame) {
        let start_addr = frame.start_address().as_u64();
        if let Some(count) = self.mapped_frames.get_mut(&start_addr) {
            *count += 1;
            return;
        }
        self.mapped_frames.insert(start_addr, 1);
        unsafe {
            map_page(
                frame,
                VirtAddr::new(start_addr),
                PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE,
            );
        };
    }

    unsafe fn unmap_page(&mut self, page: Page) {
        let start_addr = page.start_address().as_u64();
        let count = self.mapped_frames.get_mut(&start_addr).expect("Frame not mapped");
        *count -= 1;
        if *count == 0 {
            self.mapped_frames.remove(&start_addr);
            unsafe { unmap_page(page.start_address()) };
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcpiMapper(Arc<Mutex<AcpiMapperInner>>);

impl AcpiMapper {
    pub fn new() -> Self {
        let inner = AcpiMapperInner::default();
        Self(Arc::new(Mutex::new(inner)))
    }
}

impl acpi::AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> acpi::PhysicalMapping<Self, T> {
        let start_addr = PhysAddr::new(physical_address as u64);
        let start_frame = PhysFrame::containing_address(start_addr);
        let end_frame = PhysFrame::containing_address(start_addr + size as u64 - 1);
        let frame_range = PhysFrameRangeInclusive {
            start: start_frame,
            end: end_frame,
        };

        {
            let mut mapper = self.0.lock();
            for frame in frame_range {
                unsafe { mapper.map_frame(frame) };
            }
        }

        unsafe {
            acpi::PhysicalMapping::new(
                physical_address,
                NonNull::new_unchecked(physical_address as *mut T),
                size,
                size,
                self.clone(),
            )
        }
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        let virt_start = VirtAddr::new(region.virtual_start().as_ptr() as u64);
        let start_page = Page::containing_address(virt_start);
        let end_page = Page::containing_address(virt_start + region.region_length() as u64 - 1);
        let page_range = PageRangeInclusive {
            start: start_page,
            end: end_page,
        };
        let mut mapper = region.handler().0.lock();
        for page in page_range {
            unsafe { mapper.unmap_page(page) };
        }
    }
}
