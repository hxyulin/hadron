use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::Mutex;
use x86_64::{
    structures::paging::{frame::PhysFrameRangeInclusive, page::PageRangeInclusive, Page, PageTableFlags, PhysFrame}, PhysAddr, VirtAddr,
};

use crate::base::mem::{map_page, unmap_page, mappings};
use core::ptr::NonNull;

/// Initialize the ACPI subsystem
///
/// This includes:
/// - Parsing the ACPI tables
/// - Setting up the RSDP
pub fn init(rsdp: PhysAddr) {
    let mapper = AcpiMapper::new();
    let tables = unsafe { acpi::AcpiTables::from_rsdp(mapper, rsdp.as_u64() as usize) }.expect("Failed to parse ACPI tables");
    let platform_info = tables.platform_info().expect("Failed to parse platform info");
    parse_platform_info(&platform_info);
}

/// Parses the platform info and populates data structures
fn parse_platform_info(platform_info: &acpi::PlatformInfo<alloc::alloc::Global>) {
    match platform_info.power_profile {
        acpi::PowerProfile::Unspecified => log::warn!("ACPI: Unspecified power profile"),
        _ => {},
    }
}

/// A mapper to map ACPI frames to logical addresses
#[derive(Debug, Clone)]
pub struct AcpiMapperInner {
    mapped_frames: BTreeMap<Page, usize>,
}

impl AcpiMapperInner {
    pub const fn default() -> Self {
        Self {
            mapped_frames: BTreeMap::new(),
        }
    }

    unsafe fn map_frame(&mut self, frame: PhysFrame) {
        let page =
            unsafe { Page::from_start_address_unchecked(mappings::ACPI_TABLES + frame.start_address().as_u64()) };
        if let Some(count) = self.mapped_frames.get_mut(&page) {
            *count += 1;
            return;
        }
        self.mapped_frames.insert(page.clone(), 1);
        unsafe {
            map_page(
                frame,
                page.start_address(),
                PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE,
            )
        };
    }

    unsafe fn unmap_page(&mut self, page: Page) {
        let count = self.mapped_frames.get_mut(&page).expect("Frame not mapped");
        *count -= 1;
        if *count == 0 {
            self.mapped_frames.remove(&page);
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
                NonNull::new_unchecked((mappings::ACPI_TABLES + physical_address as u64).as_mut_ptr::<T>()),
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
