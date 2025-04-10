use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, PageSize, PageTableFlags, PhysFrame, Size4KiB},
};

use super::requests;
use crate::{
    base::mem::mappings,
    boot::{
        arch::x86_64::{frame_allocator::BasicFrameAllocator, page_table::BootstrapPageTable},
        drivers::serial,
        info::boot_info_mut,
    },
};

pub fn limine_entry() -> ! {
    init_core();
    populate_boot_info();
    allocate_pages();
    panic!("Kernel entry point reached");
}

/// Initializes the core of the kernel.
///
/// This includes:
/// - Initializing the serial port
/// - Initializing the GDT
/// - Initializing the IDT
fn init_core() {
    // We initialize the seiral port so it is available for printing
    unsafe { serial::init() };
    if !requests::BASE_REVISION.is_supported() {
        serial::write_fmt(format_args!(
            "Limine Base Revision {} is not supported\n",
            requests::BASE_REVISION.revision()
        ));
        panic!();
    }

    let response = requests::BOOTLOADER_INFO.get_response().unwrap();
    serial::write_fmt(format_args!("Booted from {} {}\n", response.name(), response.version()));

    serial::write_str("Initializing GDT...\n");
    crate::base::arch::gdt::init();
    serial::write_str("Initializing IDT...\n");
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
    serial::write_str("Parsing memory map...\n");
    boot_info
        .memory_map
        .parse_from_limine(requests::MEMORY_MAP.get_response().unwrap());
    let rsdp = requests::RSDP.get_response().unwrap();
    boot_info.rsdp_addr = PhysAddr::new(rsdp.address);
}

/// Allocates the pages for the kernel.
/// This creates the frame allocator, page table, and allocates pages
fn allocate_pages() {
    let boot_info = unsafe { boot_info_mut() };
    let mut frame_allocator = BasicFrameAllocator::new(&mut boot_info.memory_map);
    let mut page_table = BootstrapPageTable::new(boot_info.hhdm_offset, &mut frame_allocator);
    serial::write_str("Allocating kernel pages...\n");

    let start_phys = boot_info.kernel_start_phys;
    let kernel_size = get_kernel_size();
    // Map text section with execute permissions
    for i in 0..kernel_size.0 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE;
        page_table.map(
            mappings::KERNEL_TEXT + offset,
            PhysFrame::from_start_address((start_phys + offset).into())
                .unwrap()
                .into(),
            PageTableFlags::PRESENT,
            &mut frame_allocator,
        );
    }

    // Map data section with writable permissions but no execute permissions
    for i in 0..kernel_size.1 as u64 / Size4KiB::SIZE {
        let offset = i * Size4KiB::SIZE + kernel_size.0 as u64;
        page_table.map(
            mappings::KERNEL_TEXT + offset,
            PhysFrame::from_start_address((start_phys + offset).into())
                .unwrap()
                .into(),
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
    let heap_frames = HEAP_SIZE / Size4KiB::SIZE as u64;
    for i in 0..heap_frames {
        let offset = i as u64 * Size4KiB::SIZE;
        page_table.map(
            mappings::KERNEL_HEAP + offset,
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        );
    }
    boot_info.heap = (mappings::KERNEL_HEAP, HEAP_SIZE);

    let page_table_ptr = page_table.phys_addr().as_u64();

    serial::write_fmt(format_args!(
        "Switching to new page table... (PML4: {:#x})\n",
        page_table_ptr
    ));

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
        )
    }
}

fn limine_stage_2() -> ! {
    unsafe extern "C" {
        fn kernel_main() -> !;
    }
    serial::write_fmt(format_args!(
        "Jumping to kernel_main (at {:#x})\n",
        kernel_main as usize
    ));
    unsafe { kernel_main() };
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
