use core::panic::PanicInfo;

use alloc::boxed::Box;

use crate::{
    arch::{
        instructions::interrupts, registers::control::Cr3, x86_64::{cpu::cpu_info, io::uart::Uart16550}, PhysAddr, VirtAddr
    },
    boot::{
        frame_allocator::BootstrapFrameAllocator,
        info::BOOT_INFO,
        memory_map::{MainMemoryMap, UsableRegion},
        page_table::BootstrapPageTable,
    },
    dev::drivers::platform::fb::FramebufferInfoAddr,
    kprintln,
    mm::{
        allocator::{bump::BumpAllocator, Locked},
        mappings,
        memory_map::MemoryMap,
        page_table::{KernelPageTable, PageTableFlags},
        paging::{FrameAllocator, PageSize, PhysFrame, Size2MiB, Size4KiB},
    },
    sync::cell::RacyCell,
    util::panicking::set_alternate_panic_handler,
};

mod memory_map;
mod request;

static SERIAL: RacyCell<Option<Uart16550>> = RacyCell::new(None);

pub fn debug_write_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if let Some(serial) = SERIAL.get_mut() {
        serial.write_fmt(args).unwrap();
    }
}

macro_rules! debug_print {
    ($expr:expr) => {
        debug_write_fmt(format_args!(concat!("[BOOT] ", $expr)))
    };
    ($expr:expr, $($arg:tt)*) => {
        debug_write_fmt(format_args!(concat!("[BOOT] ", $expr), $($arg)*))
    };
}

macro_rules! boot_println {
    () => {
        debug_print!("\n")
    };
    ($expr:expr) => {
        debug_print!(concat!($expr, "\n"))
    };
    ($expr:expr, $($arg:tt)*) => {
        debug_print!(concat!($expr, "\n"), $($arg)*)
    }
}

fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { Uart16550::new(0x3F8) };
    unsafe { serial.init() };
    _ = writeln!(serial, "\n--- BOOT PANIC ---");
    _ = writeln!(serial, "message: {}", info);
    _ = writeln!(serial, "\n--- END PANIC ---");
    loop {}
}

pub unsafe fn entry() -> ! {
    // Register Alternate Panic Handler
    crate::util::panicking::set_alternate_panic_handler(Some(panic));
    unsafe {
        interrupts::disable();

        init_core();
        populate_boot_info();
        allocate_pages();
    }
}

unsafe fn init_core() {
    unsafe { init_serial() };

    if !request::BASE_REVISION.is_supported() {
        panic!(
            "Limine base revision {} is not supported!\n",
            request::BASE_REVISION.revision(),
        );
    }

    if let Some(info) = request::BOOTLOADER_INFO.response() {
        boot_println!("info: kernel booted from {} {}", info.name(), info.version());
    }

    unsafe {
        boot_println!("info: initializing GDT...");
        crate::arch::x86_64::core::gdt::init();
        boot_println!("info: initializing IDT...");
        crate::arch::x86_64::core::idt::init();
        boot_println!("info: getting CPU info...");
        crate::arch::x86_64::cpu::init();
    }
}

unsafe fn init_serial() {
    let mut writer = unsafe { Uart16550::new(0x3F8) };
    unsafe { writer.init() };
    SERIAL.replace(Some(writer));
    boot_println!("info: initialized serial COMM1");
}

unsafe fn populate_boot_info() {
    let boot_info = BOOT_INFO.get_mut();
    match request::HHDM.response() {
        Some(hhdm) => boot_info.hhdm_offset = hhdm.offset,
        None => panic!("bootloader did not send HHDM response"),
    }

    match request::EXECUTABLE_ADDRESS.response() {
        Some(kernel_addr) => {
            boot_info.kernel_phys = PhysAddr::new(kernel_addr.physical_address as usize);
            boot_info.kernel_virt = VirtAddr::new(kernel_addr.virtual_address as usize);
        }
        None => panic!("bootloader did not send executable address response"),
    }

    match request::MEMORY_MAP.response() {
        Some(memory_map) => {
            boot_info
                .memory_map
                .parse_from_limine(&memory_map, boot_info.hhdm_offset as usize);
            boot_println!("memory_map: {:#?}", boot_info.memory_map);
        }
        None => panic!("bootloader did not send memory map response"),
    }

    if let Some(framebuffers) = request::FRAMEBUFFER.response() {
        let framebuffers = framebuffers.framebuffers();
        boot_println!("info: found {} framebuffers", framebuffers.len());
        if let Some(fb) = framebuffers.first() {
            use crate::dev::drivers::platform::fb::{FramebufferInfoAddr, PixelFormat};
            boot_info.framebuffer = FramebufferInfoAddr {
                width: fb.width() as u32,
                height: fb.height() as u32,
                pixel_format: PixelFormat::RGB,
                stride: fb.pitch() as u32,
                bpp: (fb.bpp() / 8) as u32,
                addr: fb.address() as *mut u8,
            };
        }
    } else {
        boot_println!("warn: bootloader did not send any framebuffers");
    }

    match request::RSDP.response() {
        Some(rsdp) => boot_info.rsdp_addr = PhysAddr::new(rsdp.address as usize),
        None => panic!("bootloader did not send rsdp response"),
    }

    boot_println!("info: Boot Info");
    boot_println!(" - HHDM offset: {:#x}", boot_info.hhdm_offset);
    boot_println!(" - kernel virt: {:#x}", boot_info.kernel_virt);
    boot_println!(" - kernel phys: {:#x}", boot_info.kernel_phys);
    boot_println!(" - memory map: {}b available", boot_info.memory_map.total_size());
    boot_println!(" - RSDP address: {:#x}", boot_info.rsdp_addr);
}

/// Calculates the number of pages needed for the page table
/// when mapping the given number of pages
#[inline]
fn calculate_pages_needed(pages: usize) -> usize {
    let pds = pages / (4096 * 4096) + 1;
    let pts = (pages % (4096 * 4096)).div_ceil(4096);
    // 1 for the PDPT table
    pds + pts + 1
}

fn allocate_pages() -> ! {
    let boot_info = BOOT_INFO.get_mut();
    let (mm_start, mm_len) = boot_info.memory_map.mapped_range();
    let mut frame_allocator = BootstrapFrameAllocator::new(&mut boot_info.memory_map);

    let kernel_size = get_kernel_size();
    let mut pages_to_allocate = 0;

    pages_to_allocate += calculate_pages_needed(kernel_size.0 / Size4KiB::SIZE as usize);
    pages_to_allocate += calculate_pages_needed(kernel_size.1 / Size4KiB::SIZE as usize);
    let stack_frames = request::KERNEL_STACK_SIZE / Size4KiB::SIZE as usize;
    pages_to_allocate += calculate_pages_needed(stack_frames);
    const HEAP_SIZE: usize = 512 * 1024;
    let heap_frames = HEAP_SIZE / Size4KiB::SIZE;
    pages_to_allocate += calculate_pages_needed(heap_frames as usize);
    let mmap_frames = mm_len.div_ceil(Size4KiB::SIZE);
    pages_to_allocate += calculate_pages_needed(mmap_frames as usize);
    {
        let size = (boot_info.framebuffer.stride as usize) * (boot_info.framebuffer.height as usize);
        pages_to_allocate += calculate_pages_needed(size.div_ceil(Size4KiB::SIZE as usize));
    }

    for region in request::MEMORY_MAP.response().unwrap().entries() {
        if let Some(region) = UsableRegion::from_region(region) {
            pages_to_allocate += region.pages_needed();
        }
    }

    let frame = frame_allocator.allocate_mapped_contiguous(pages_to_allocate).unwrap();
    let allocator = Locked::new(unsafe {
        BumpAllocator::new(
            VirtAddr::new(frame.start_address().as_usize() + boot_info.hhdm_offset as usize),
            pages_to_allocate * Size4KiB::SIZE,
        )
    });
    let mut page_table = BootstrapPageTable::new(boot_info.hhdm_offset as usize, &mut frame_allocator, &allocator);

    let start_phys = boot_info.kernel_phys;
    let kernel_virt = boot_info.kernel_virt;
    assert!(
        (kernel_size.0 + kernel_size.1) < mappings::KERNEL_TEXT_SIZE,
        "Kernel is too large\n"
    );

    // Map text section with execute permissions
    for i in 0..kernel_size.0 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            kernel_virt + offset,
            PhysFrame::from_start_address(start_phys + offset),
            PageTableFlags::PRESENT,
            &mut frame_allocator,
        );
    }

    // Map data section with writable permissions but no execute permissions
    for i in 0..kernel_size.1 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE + kernel_size.0;
        page_table.map(
            kernel_virt + offset,
            PhysFrame::from_start_address(start_phys + offset),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            &mut frame_allocator,
        );
    }

    let stack_virt = mappings::KERNEL_STACK_START + mappings::TOTAL_KERNEL_STACK_SIZE - request::KERNEL_STACK_SIZE;
    for i in 0..stack_frames {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            stack_virt + offset,
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }

    for i in 0..heap_frames {
        let offset = i * Size4KiB::SIZE;
        let frame = frame_allocator.allocate_frame().unwrap();
        page_table.map(
            mappings::KERNEL_HEAP_START + offset,
            frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }
    boot_info.heap = (mappings::KERNEL_HEAP_START, HEAP_SIZE);

    {
        let framebuffer = &mut boot_info.framebuffer;
        let fb_virt = VirtAddr::new(framebuffer.addr as usize);
        let fb_phys = PhysAddr::new(fb_virt.as_usize() - boot_info.hhdm_offset as usize);
        let fb_pages = ((framebuffer.stride as usize) * (framebuffer.height as usize)).div_ceil(Size4KiB::SIZE);

        for i in 0..fb_pages {
            let offset = i * Size4KiB::SIZE;
            page_table.map(
                mappings::FRAMEBUFFER_START + offset,
                PhysFrame::from_start_address(fb_phys + offset),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::NO_CACHE,
                &mut frame_allocator,
            );
        }

        framebuffer.addr = mappings::FRAMEBUFFER_START.as_mut_ptr();
    }
    // Allocate memory map
    for i in 0..mmap_frames {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            mm_start + offset,
            PhysFrame::from_start_address(PhysAddr::new(
                mm_start.as_usize() - boot_info.hhdm_offset as usize + offset,
            )),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        )
    }

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
    for region in request::MEMORY_MAP.response().unwrap().entries() {
        if let Some(region) = UsableRegion::from_region(region) {
            let mut start = region.base;
            for i in 0..region.f_pad_4kib {
                let virt = KernelPageTable::DIRECT_MAP_START + start.as_usize();
                page_table.map(virt, PhysFrame::from_start_address(start), flags, &mut frame_allocator);
                start += i * Size4KiB::SIZE;
            }

            for i in 0..region.f_pad_2mib {
                let virt = KernelPageTable::DIRECT_MAP_START + start.as_usize();
                page_table.map(virt, PhysFrame::<Size2MiB>::from_start_address(start), flags, &mut frame_allocator);
                start += i * Size4KiB::SIZE;
            }
        }
    }

    page_table.direct_map(&mut frame_allocator);

    let page_table_ptr = page_table.as_phys_addr().as_u64();

    boot_println!("info: jumping to limine stage 2...");
    // Setup the page tables, switch to the new stack, and push a null pointer to the stack
    unsafe {
        core::arch::asm!(
            "mov cr3, {ptr}",
            "mov rsp, {stack}",
            "push 0",
            "jmp {entry}",
            ptr = in(reg) page_table_ptr,
            stack = in(reg) mappings::KERNEL_STACK_END.as_u64(),
            entry = sym stage_2,
            options(noreturn, preserves_flags)
        )
    }
}

fn setup_platform_dev() {
    use crate::dev::{
        DEVICES,
        platform::{PlatformDev, PlatformDevAddr, PlatformDevType},
    };

    let mut platform_devs = DEVICES.platform();

    if let Some(serial) = SERIAL.get_mut().take() {
        platform_devs.add_device(PlatformDev::new(
            "io_dev",
            PlatformDevType::IoDevice,
            PlatformDevAddr::io_port(serial.port()),
        ));
    }

    let fb = &BOOT_INFO.get_mut().framebuffer;
    platform_devs.add_device(PlatformDev::new(
        "efi_fb",
        PlatformDevType::Framebuffer,
        PlatformDevAddr::addr((fb as *const FramebufferInfoAddr) as usize),
    ));

    let drivers = crate::dev::drivers::platform::available_drivers();

    // Attach Drivers to Devices
    for device in platform_devs.iter_mut() {
        for drv in drivers {
            if drv.matches(device) {
                if !drv.probe(device) {
                    continue;
                }
                drv.attach(device);
                break;
            }
        }
    }
}

fn setup_logger() {
    use crate::dev::DEVICES;
    use crate::util::kprint::ConsoleWriter;
    let mut platform_devs = DEVICES.platform();

    // While the logger is locked, we can't log
    let mut logger = crate::util::kprint::LOGGER.lock();
    for dev in platform_devs.iter() {
        if let Some(drv) = &dev.dev.drv {
            match drv.caps.console {
                None => continue,
                Some(_) => logger.loggers.push(Box::new(ConsoleWriter::new(&dev.dev))),
            }
        }
    }

    // We no longer use our alternate logger
    set_alternate_panic_handler(None);
}

fn stage_2() -> ! {
    let boot_info = BOOT_INFO.get_mut();
    // Initialize the heap
    unsafe { crate::mm::allocator::ALLOCATOR.init(boot_info.heap.0.as_mut_ptr(), boot_info.heap.1) };

    // We setup devices to our proper device system
    setup_platform_dev();
    setup_logger();

    kprintln!(Debug, "Hello World!");
    kprintln!(Debug, "CPU Info: {:#?}", cpu_info());

    {
        let boot_info = BOOT_INFO.get_mut();
        let mut page_table = KernelPageTable::new(Cr3::addr());
        let mut memory_map = MemoryMap::from_bootstrap(&mut boot_info.memory_map, &mut page_table);
    }

    unsafe extern "Rust" {
        fn kernel_main() -> !;
    }
    unsafe { kernel_main() };
}

#[inline]
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
        assert!(
            (data_start - start) % 0x1000 == 0,
            "Kernel text section is not page aligned"
        );
        assert!(
            (end - data_start) % 0x1000 == 0,
            "Kernel data section is not page aligned"
        );
        (data_start - start, end - data_start)
    }
}
