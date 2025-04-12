use core::ptr::addr_of;

use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    registers::segmentation::Segment,
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 8;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            #[allow(unused_unsafe)]
            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

struct Selectors {
    kernel_code: SegmentSelector,
    kernel_data: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernel_code,
                kernel_data,
                tss_selector,
            },
        )
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        // Load the code segment and the TSS.
        x86_64::registers::segmentation::CS::set_reg(GDT.1.kernel_code);
        x86_64::registers::segmentation::DS::set_reg(GDT.1.kernel_data);
        x86_64::registers::segmentation::SS::set_reg(GDT.1.kernel_data); // Set SS to kernel_data, not NULL
        x86_64::instructions::tables::load_tss(GDT.1.tss_selector);

        // Clear other segment registers.
        x86_64::registers::segmentation::ES::set_reg(SegmentSelector::NULL);
        x86_64::registers::segmentation::FS::set_reg(SegmentSelector::NULL);
        x86_64::registers::segmentation::GS::set_reg(SegmentSelector::NULL);
    }
}
