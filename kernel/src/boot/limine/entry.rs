use core::fmt::Arguments;

use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, PageSize, PageTableFlags, PhysFrame, Size4KiB},
};

use super::requests;
use crate::{
    KernelParams,
    base::{
        info::{KernelInfo, RuntimeInfo},
        mem::{
            frame_allocator::KernelFrameAllocator,
            mappings,
            memory_map::{MemoryMap, MemoryRegionTag},
            page_table::KernelPageTable,
        },
    },
    boot::{
        arch::x86_64::{frame_allocator::BasicFrameAllocator, page_table::BootstrapPageTable},
        drivers::{framebuffer::FramebufferWriter, serial::SerialWriter},
        info::boot_info_mut,
    },
    devices::{
        fb::{Framebuffer, FramebufferInfo, PixelFormat},
        tty::{fb::VirtFbTtyDevice, serial::SerialDevice},
    },
    util::logger::LOGGER,
};

macro_rules! print {
    ($bi:ident, $($arg:tt)*) => {
        write_fmt(
            $bi.serial.as_mut(),
            $bi.framebuffer.as_mut(),
            format_args!($($arg)*),
        )
    }
}

fn write_fmt(serial: Option<&mut SerialWriter>, fb: Option<&mut FramebufferWriter>, args: Arguments) {
    use core::fmt::Write;
    if let Some(serial) = serial {
        _ = serial.write_fmt(args);
    }
    if let Some(framebuffer) = fb {
        _ = framebuffer.write_fmt(args);
    }
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
    let boot_info = unsafe { boot_info_mut() };
    print!(boot_info, "Kernel panicked at '{:?}'\n", info);
}

/// Initializes the core of the kernel.
///
/// This includes:
/// - Initializing the serial port
/// - Initializing the GDT
/// - Initializing the IDT
fn init_core() {
    let boot_info = unsafe { boot_info_mut() };
    // We initialize the seiral port so it is available for printing
    boot_info.serial = {
        let mut serial = SerialWriter::new(0x3F8);
        serial.init();
        Some(serial)
    };

    if !requests::BASE_REVISION.is_supported() {
        print!(
            boot_info,
            "Limine Base Revision {} is not supported\n",
            requests::BASE_REVISION.revision()
        );
        panic!();
    }

    // We want the framebuffer as early as possible
    let fb = requests::FRAMEBUFFER
        .get_response()
        .unwrap()
        .framebuffers()
        .next()
        .unwrap();
    let fb_info = FramebufferInfo {
        width: fb.width() as u32,
        height: fb.height() as u32,
        pixel_format: PixelFormat::RGB,
        stride: fb.pitch() as u32,
        bpp: fb.bpp() as u32 / 8,
    };
    let fb =
        unsafe { core::slice::from_raw_parts_mut(fb.address() as *mut u8, fb.pitch() as usize * fb.height() as usize) };
    boot_info.framebuffer = Some(FramebufferWriter::new(Framebuffer::new(fb_info, fb)));

    let response = requests::BOOTLOADER_INFO.get_response().unwrap();
    print!(boot_info, "Booted from {} {}\n", response.name(), response.version());

    print!(boot_info, "Initializing GDT...\n");
    crate::base::arch::gdt::init();
    print!(boot_info, "Initializing IDT...\n");
    crate::base::arch::idt::init();
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
    let kernel_addr = requests::EXECUTABLE_ADDRESS.get_response().unwrap();
    boot_info.kernel_start_phys = PhysAddr::new(kernel_addr.physical_address);
    boot_info.kernel_start_virt = VirtAddr::new(kernel_addr.virtual_address);
    print!(boot_info, "Parsing memory map...\n");
    boot_info
        .memory_map
        .parse_from_limine(requests::MEMORY_MAP.get_response().unwrap());
    let rsdp = requests::RSDP.get_response().unwrap();
    boot_info.rsdp_addr = PhysAddr::new(rsdp.address);
}

/// Allocates the pages for the kernel.
/// This creates the frame allocator, page table, and allocates pages
fn allocate_pages() -> ! {
    let boot_info = unsafe { boot_info_mut() };
    let mut frame_allocator = BasicFrameAllocator::new(&mut boot_info.memory_map);
    let mut page_table = BootstrapPageTable::new(boot_info.hhdm_offset, &mut frame_allocator);

    let start_phys = boot_info.kernel_start_phys;
    let kernel_size = get_kernel_size();
    // Map text section with execute permissions
    for i in 0..kernel_size.0 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            mappings::KERNEL_TEXT + offset,
            PhysFrame::from_start_address(start_phys + offset).unwrap().into(),
            PageTableFlags::PRESENT,
            &mut frame_allocator,
        );
    }

    // Map data section with writable permissions but no execute permissions
    for i in 0..kernel_size.1 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE + kernel_size.0 as u64;
        page_table.map(
            mappings::KERNEL_TEXT + offset,
            PhysFrame::from_start_address(start_phys + offset).unwrap().into(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }

    let stack_virt = mappings::KERNEL_STACK_START + mappings::KERNEL_STACK_SIZE - requests::KERNEL_STACK_SIZE as u64;
    let stack_frames = requests::KERNEL_STACK_SIZE / Size4KiB::SIZE as usize;
    for i in 0..stack_frames {
        let offset = i as u64 * Size4KiB::SIZE;
        page_table.map(
            stack_virt + offset,
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }

    const HEAP_SIZE: u64 = 512 * 1024;
    let heap_frames = HEAP_SIZE / Size4KiB::SIZE;
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

    // Map framebuffer
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let fb_virt = VirtAddr::new(framebuffer.fb_addr() as u64);
        let fb_phys = PhysAddr::new(fb_virt.as_u64() - boot_info.hhdm_offset);
        let fb_pages = framebuffer.fb_size().div_ceil(Size4KiB::SIZE as usize);

        write_fmt(
            boot_info.serial.as_mut(),
            Some(framebuffer),
            format_args!(
                "Framebuffer: (VirtAddr: {:#x}, PhysAddr: {:#x}, Size: {})\n",
                fb_virt.as_u64(),
                fb_phys.as_u64(),
                fb_pages
            ),
        );

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
        unsafe { framebuffer.set_fb_addr(mappings::FRAMEBUFFER.as_u64() as usize) };
    }

    let page_table_ptr = page_table.phys_addr().as_u64();

    write_fmt(
        boot_info.serial.as_mut(),
        None, // We can't use it yet, since we set the virt addr to the new mapping
        format_args!("Switching to new page table... (PML4: {:#x})\n", page_table_ptr),
    );

    // Setup the page tables, switch to the new stack, and push a null pointer to the stack
    unsafe {
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

    let boot_info = unsafe { boot_info_mut() };
    print!(boot_info, "Constructing frame allocator...\n");
    let memory_map = MemoryMap::from_bootstrap(&boot_info.memory_map);
    let mut frame_allocator = KernelFrameAllocator::new(memory_map);
    print!(boot_info, "Constructing frame allocator...\n");
    let page_table = KernelPageTable::new();
    let framebuffers = create_framebuffers();

    let rsdp = boot_info.rsdp_addr;

    // PERF: Either remove this or make it better performing (inside `free_special_region`)
    let total_pages = frame_allocator.total_pages();
    frame_allocator.free_special_region(MemoryRegionTag::BootloaderReclaimable);
    let new_total_pages = frame_allocator.total_pages();
    print!(
        boot_info,
        "Reclaimed {} pages (total memory {})\n",
        new_total_pages - total_pages,
        new_total_pages * Size4KiB::SIZE
    );

    let runtime_info = RuntimeInfo::new(Mutex::new(frame_allocator), Mutex::new(page_table));

    // Now we need to convert our serial port to a TTY device
    // and the framebuffer to a framebuffer device, and a virtual TTY device
    let serial_port = boot_info.serial.take().unwrap().as_port();
    let serial_device = SerialDevice::from_initialized_port(serial_port);
    let serial_id = runtime_info.devices.add_tty_device(Arc::new(Mutex::new(serial_device)));
    LOGGER.add_output(serial_id);

    let (fb, writer) = boot_info.framebuffer.take().unwrap().to_inner();
    let fb_id = runtime_info.devices.add_fb_device(Arc::new(Mutex::new(fb)));
    let mut virt_fb = VirtFbTtyDevice::new(fb_id);
    virt_fb.set_pos(writer.x_pos() as u32, writer.y_pos() as u32);
    let virt_fb_id = runtime_info.devices.add_tty_device(Arc::new(Mutex::new(virt_fb)));
    LOGGER.add_output(virt_fb_id);

    unsafe {
        use crate::base::info::KERNEL_INFO;
        KERNEL_INFO = KernelInfo::Kernel(runtime_info)
    };
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&LOGGER).unwrap();

    jump_to_kernel_main(KernelParams { rsdp });
}

fn init_heap() {
    let boot_info = unsafe { boot_info_mut() };
    print!(boot_info, "Setting up kernel heap...\n");
    let mut allocator = crate::ALLOCATOR.lock();
    unsafe { allocator.init(mappings::KERNEL_HEAP.as_mut_ptr(), boot_info.heap.1 as usize) };
}

fn create_framebuffers() -> Vec<Mutex<Framebuffer>> {
    let boot_info = unsafe { boot_info_mut() };
    print!(boot_info, "Creating framebuffers...\n");
    Vec::new()
}

fn jump_to_kernel_main(params: KernelParams) -> ! {
    // It is not considered not 'boot' anymore, since we are jumping to the kernel afterwards
    crate::boot::IS_BOOT.store(false, core::sync::atomic::Ordering::Relaxed);
    unsafe extern "C" {
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
