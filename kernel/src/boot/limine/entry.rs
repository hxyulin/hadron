use core::fmt::Arguments;

use alloc::boxed::Box;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, PageSize, PageTableFlags, PhysFrame, Size4KiB},
};

use super::requests;
use crate::{
    ALLOCATOR, KernelParams,
    base::mem::{
        FRAME_ALLOCATOR, PAGE_TABLE,
        frame_allocator::KernelFrameAllocator,
        mappings,
        memory_map::{MemoryMap, MemoryRegionTag},
        page_table::KernelPageTable,
        sync::RacyCell,
    },
    boot::{
        arch::{
            memory_map::{FrameBasedAllocator, MemoryMapEntry, MemoryRegionType},
            x86_64::{frame_allocator::BasicFrameAllocator, page_table::BootstrapPageTable},
        },
        info::boot_info_mut,
    },
    util::{
        logging::{
            LOGGER, WRITER,
            framebuffer::{Framebuffer, FramebufferInfo, FramebufferWriter, PixelFormat},
            serial::SerialWriter,
        },
        machine_state::MachineState,
    },
};
static SERIAL: RacyCell<Option<SerialWriter>> = RacyCell::new(None);
static FRAMEBUFFER: RacyCell<Option<FramebufferWriter>> = RacyCell::new(None);

fn write_str(s: &str) -> core::fmt::Result {
    use core::fmt::Write;
    if let Some(serial) = SERIAL.get_mut().as_mut() {
        serial.write_str(s)?;
    }

    if let Some(framebuffer) = FRAMEBUFFER.get_mut().as_mut() {
        framebuffer.write_str(s)?;
    }

    Ok(())
}

fn write_fmt(args: Arguments) -> core::fmt::Result {
    use core::fmt::Write;
    if let Some(serial) = SERIAL.get_mut().as_mut() {
        serial.write_fmt(args)?;
    }
    if let Some(framebuffer) = FRAMEBUFFER.get_mut().as_mut() {
        framebuffer.write_fmt(args)?;
    }
    Ok(())
}

pub fn limine_entry() -> ! {
    // TODO: We need to parse the framebuffer here as well
    init_core();
    populate_boot_info();
    allocate_pages();
}

pub fn limine_print_panic(info: &core::panic::PanicInfo) {
    // SAFETY: If we panic, we are in an unsafe context anyways,
    // we try to notify the user about the panic
    let machine_state = MachineState::here();

    let writer = &crate::util::logging::WRITER;
    _ = writer.write_fmt(format_args!("BOOT KERNEL PANIC: {}\n", info.message()));
    if let Some(location) = info.location() {
        _ = writer.write_fmt(format_args!(
            "    at {}:{}:{}\n",
            location.file(),
            location.line(),
            location.column()
        ));
    }

    if ALLOCATOR.generic_size() != 0 {
        let mut unwinder = crate::util::backtrace::create_unwinder(&machine_state);
        while let Ok(Some(frame)) = unwinder.next() {
            _ = writer.write_fmt(format_args!("    at {:#X}\n", frame.pc));
        }
    } else {
        _ = writer.write_str("PANIC: No allocator available for backtrace\n");
    }

    _ = writer.write_fmt(format_args!("{}", machine_state));
}

/// Initializes the core of the kernel.
///
/// This includes:
/// - Initializing the serial port
/// - Initializing the GDT
/// - Initializing the IDT
fn init_core() {
    x86_64::instructions::interrupts::disable();

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    WRITER.add_fallback(write_str, write_fmt);

    // We initialize the seiral port so it is available for printing
    SERIAL.get_mut().replace({
        let mut serial = SerialWriter::new(0x3F8);
        serial.init();
        serial
    });

    if !requests::BASE_REVISION.is_supported() {
        panic!(
            "Limine Base Revision {} is not supported\n",
            requests::BASE_REVISION.revision()
        );
    }

    // We want the framebuffer as early as possible
    let framebuffers = requests::FRAMEBUFFER.get_response().unwrap().framebuffers();
    log::debug!("BOOT: found {} framebuffers", framebuffers.len());

    let fb = framebuffers.first().unwrap();
    let fb_info = FramebufferInfo {
        width: fb.width() as u32,
        height: fb.height() as u32,
        pixel_format: PixelFormat::RGB,
        stride: fb.pitch() as u32,
        bpp: fb.bpp() as u32 / 8,
    };
    let fb =
        unsafe { core::slice::from_raw_parts_mut(fb.address() as *mut u8, fb.pitch() as usize * fb.height() as usize) };
    FRAMEBUFFER
        .get_mut()
        .replace(FramebufferWriter::new(Framebuffer::new(fb_info, fb)));

    let response = requests::BOOTLOADER_INFO.get_response().unwrap();
    log::debug!("BOOT: booted from {} {}", response.name(), response.version());

    log::debug!("BOOT: initializing GDT...");
    crate::base::arch::x86_64::gdt::init();
    log::debug!("BOOT: initializing IDT...");
    crate::base::arch::x86_64::idt::init();
}

/// Populates the boot info.
/// This includes:
/// - Reading the HHDM offset
/// - Reading the kernel start physical address
/// - Reading the kernel start virtual address
/// - Reading the RSDP address
/// - Reading the memory map
fn populate_boot_info() {
    let boot_info = unsafe { boot_info_mut() };
    let hhdm = requests::HHDM.get_response().unwrap();
    boot_info.hhdm_offset = hhdm.offset;
    log::debug!("BOOT: hhdm_offset: {:#x}", boot_info.hhdm_offset);
    let kernel_addr = requests::EXECUTABLE_ADDRESS.get_response().unwrap();
    boot_info.kernel_start_phys = PhysAddr::new(kernel_addr.physical_address);
    boot_info.kernel_start_virt = VirtAddr::new(kernel_addr.virtual_address);
    log::debug!("BOOT: kernel loaded at {:#x}", boot_info.kernel_start_virt);
    log::debug!("BOOT: hhdm offset: {:#x}", boot_info.hhdm_offset);

    // TODO: parse modules

    log::debug!("BOOT: parsing memory map...");
    boot_info
        .memory_map
        .parse_from_limine(requests::MEMORY_MAP.get_response().unwrap(), boot_info.hhdm_offset);
    log::debug!("BOOT: total memory available: {:#?}", boot_info.memory_map.total_size());
    let rsdp = requests::RSDP.get_response().unwrap();
    boot_info.rsdp_addr = PhysAddr::new(rsdp.address);
}

/// Calculates the number of pages needed for the page table
/// when mapping the given number of pages
fn calculate_pages_needed(pages: usize) -> usize {
    let pds = pages / (4096 * 4096) + 1;
    let pts = (pages % (4096 * 4096)).div_ceil(4096);
    // 1 for the PDPT table
    pds + pts + 1
}

/// Allocates the pages for the kernel.
/// This creates the frame allocator, page table, and allocates pages
fn allocate_pages() -> ! {
    let boot_info = unsafe { boot_info_mut() };
    let (mm_start, mm_len) = boot_info.memory_map.mapped_range();
    let mut frame_allocator = BasicFrameAllocator::new(&mut boot_info.memory_map);
    log::debug!("BOOT: calculating pages to allocate for page table");
    let kernel_size = get_kernel_size();
    let mut pages_to_allocate = 0;
    pages_to_allocate += calculate_pages_needed(kernel_size.0 / Size4KiB::SIZE as usize);
    pages_to_allocate += calculate_pages_needed(kernel_size.1 / Size4KiB::SIZE as usize);
    let stack_frames = requests::KERNEL_STACK_SIZE / Size4KiB::SIZE as usize;
    pages_to_allocate += calculate_pages_needed(stack_frames);
    const HEAP_SIZE: u64 = 512 * 1024;
    let heap_frames = HEAP_SIZE / Size4KiB::SIZE;
    pages_to_allocate += calculate_pages_needed(heap_frames as usize);
    let mmap_frames = mm_len.div_ceil(Size4KiB::SIZE);
    pages_to_allocate += calculate_pages_needed(mmap_frames as usize);
    if let Some(framebuffer) = FRAMEBUFFER.get().as_ref() {
        pages_to_allocate += calculate_pages_needed(framebuffer.fb_size().div_ceil(Size4KiB::SIZE as usize));
    }

    // FIXME: In the future we can split this between many regions, since they don't need to be contiguous
    let frame = frame_allocator.allocate_mapped_contiguous(pages_to_allocate).unwrap();
    let allocator = unsafe {
        FrameBasedAllocator::new(
            VirtAddr::new(frame.start_address().as_u64() + boot_info.hhdm_offset),
            pages_to_allocate * Size4KiB::SIZE as usize,
        )
    };
    let mut page_table = BootstrapPageTable::new(boot_info.hhdm_offset, &mut frame_allocator, &allocator);

    let start_phys = boot_info.kernel_start_phys;
    assert!(
        (boot_info.kernel_start_virt + kernel_size.0 as u64 + kernel_size.1 as u64) < mappings::KERNEL_TEXT_END,
        "Kernel is too large"
    );

    // Map text section with execute permissions
    for i in 0..kernel_size.0 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            boot_info.kernel_start_virt + offset,
            PhysFrame::from_start_address(start_phys + offset).unwrap(),
            PageTableFlags::PRESENT,
            &mut frame_allocator,
        );
    }

    // Map data section with writable permissions but no execute permissions
    for i in 0..kernel_size.1 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE + kernel_size.0 as u64;
        page_table.map(
            boot_info.kernel_start_virt + offset,
            PhysFrame::from_start_address(start_phys + offset).unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            &mut frame_allocator,
        );
    }

    let stack_virt = mappings::KERNEL_STACK_START + mappings::KERNEL_STACK_SIZE - requests::KERNEL_STACK_SIZE as u64;
    for i in 0..stack_frames {
        let offset = i as u64 * Size4KiB::SIZE;
        page_table.map(
            stack_virt + offset,
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }

    for i in 0..heap_frames {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            mappings::KERNEL_HEAP + offset,
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }
    boot_info.heap = (mappings::KERNEL_HEAP, HEAP_SIZE);

    // Allocate memory map
    for i in 0..mmap_frames {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            mm_start + offset,
            PhysFrame::from_start_address(PhysAddr::new(mm_start.as_u64() - boot_info.hhdm_offset)).unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        )
    }

    // Map framebuffer
    if let Some(framebuffer) = FRAMEBUFFER.get().as_ref() {
        let fb_virt = VirtAddr::new(framebuffer.fb_addr() as u64);
        let fb_phys = PhysAddr::new(fb_virt.as_u64() - boot_info.hhdm_offset);
        let fb_pages = framebuffer.fb_size().div_ceil(Size4KiB::SIZE as usize);

        for i in 0..fb_pages {
            let offset = i as u64 * Size4KiB::SIZE;
            page_table.map(
                mappings::FRAMEBUFFER + offset,
                PhysFrame::from_start_address(fb_phys + offset).unwrap(),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::NO_CACHE,
                &mut frame_allocator,
            );
        }
    }

    let page_table_ptr = page_table.as_phys_addr().as_u64();

    allocator.deinit(&mut frame_allocator, boot_info.hhdm_offset);

    // Setup the page tables, switch to the new stack, and push a null pointer to the stack
    unsafe {
        if let Some(framebuffer) = FRAMEBUFFER.get_mut().as_mut() {
            framebuffer.set_fb_addr(mappings::FRAMEBUFFER.as_u64() as usize);
        }
        core::arch::asm!(
            "mov cr3, {ptr}",
            "mov rsp, {stack}",
            "push 0",
            "jmp {entry}",
            ptr = in(reg) page_table_ptr,
            stack = in(reg) mappings::KERNEL_STACK_END.as_u64(),
            entry = sym limine_stage_2,
            options(noreturn, preserves_flags)
        )
    }
}

fn limine_stage_2() -> ! {
    init_heap();
    init_logging();

    log::debug!("BOOT: calling constructors...");
    crate::boot::ctor::init();

    let boot_info = unsafe { boot_info_mut() };
    log::debug!("BOOT: constructing page tables...");
    let mut page_table = KernelPageTable::new();
    log::debug!("BOOT: constructing memory map...");
    let mut memory_map = MemoryMap::from_bootstrap(&mut boot_info.memory_map, &mut page_table);

    // We can't run the drop code, since it is `null`
    core::mem::forget(PAGE_TABLE.replace(page_table));
    // We can free the memory map and unmap it
    let mapped_range = boot_info.memory_map.mapped_range();
    memory_map.push_entry(MemoryMapEntry {
        base: PhysAddr::new((mapped_range.0 - boot_info.hhdm_offset).as_u64()),
        length: mapped_range.1,
        memory_type: MemoryRegionType::Usable,
    });
    // TODO: Can we somehow make it type safe so that the old memory map is not used?
    log::debug!("BOOT: constructing frame allocator...");
    let mut frame_allocator = KernelFrameAllocator::new(memory_map);
    log::debug!("BOOT: freeing bootloader memory...");
    frame_allocator.free_special_region(MemoryRegionTag::BootloaderReclaimable);

    let rsdp = boot_info.rsdp_addr;
    let total_pages = frame_allocator.total_pages();
    log::debug!("BOOT: pages available: {}", total_pages);
    // We can't run the drop code, since it is `null`
    core::mem::forget(FRAME_ALLOCATOR.replace(frame_allocator));

    log::debug!("Jumping to kernel main");
    jump_to_kernel_main(KernelParams { rsdp });
}

fn init_heap() {
    let boot_info = unsafe { boot_info_mut() };
    log::debug!("BOOT: setting up initial kernel heap...");
    unsafe { crate::ALLOCATOR.init_generic(mappings::KERNEL_HEAP.as_mut_ptr(), boot_info.heap.1 as usize) };
}

fn init_logging() {
    WRITER.remove_fallback();
    let serial = SERIAL.get_mut().take().unwrap();
    WRITER.add_output(Box::new(serial));
    let fb = FRAMEBUFFER.get_mut().take().unwrap();
    WRITER.add_output(Box::new(fb));
}

fn jump_to_kernel_main(params: KernelParams) -> ! {
    // It is not considered not 'boot' anymore, since we are jumping to the kernel afterwards
    crate::boot::IS_BOOT.store(false, core::sync::atomic::Ordering::Relaxed);
    unsafe extern "Rust" {
        fn kernel_main(params: KernelParams) -> !;
    }
    unsafe { kernel_main(params) };
}

/// Returns the size of the kernel text and data sections as a tuple
fn get_kernel_size() -> (usize, usize) {
    unsafe extern "C" {
        unsafe static _kernel_text_start: u8;
        unsafe static _kernel_data_start: u8;
        unsafe static _kernel_end: u8;
    }

    unsafe {
        let start = &_kernel_text_start as *const u8 as usize;
        let end = &_kernel_end as *const u8 as usize;
        let data_start = &_kernel_data_start as *const u8 as usize;
        (
            (data_start - start + 0xFFF) & !0xFFF,
            (end - data_start + 0xFFF) & !0xFFF,
        )
    }
}
