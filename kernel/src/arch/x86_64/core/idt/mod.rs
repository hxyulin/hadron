//! An IDT Implementaion in Rust
//! Inspired by the x86_64 crate

use core::{fmt, marker::PhantomData};

use spin::Mutex;

use crate::{
    arch::{
        VirtAddr,
        registers::{RFlags, segmentation::SegmentSelector},
        x86_64::core::gdt::Selectors,
    },
    util::bits::BitHelper,
};

mod handlers;

/// A Basic Handler for a x86-interrupt
/// Arguments:
/// stack_frame: InterruptStackFrame
type HandlerFn = extern "x86-interrupt" fn(stack_frame: InterruptStackFrame);
type DivergingHandlerFn = extern "x86-interrupt" fn(stack_frame: InterruptStackFrame) -> !;
type HandlerFnWithErrCode = extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: u64);
type DivergingHandlerFnWithErrCode = extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: u64) -> !;

pub unsafe trait HandlerFunc {
    fn to_addr(self) -> VirtAddr;
}

macro_rules! handler_func_impl {
    ($name:ty) => {
        unsafe impl HandlerFunc for $name {
            fn to_addr(self) -> VirtAddr {
                VirtAddr::new(self as usize)
            }
        }
    };
}

handler_func_impl!(HandlerFn);
handler_func_impl!(DivergingHandlerFn);
handler_func_impl!(HandlerFnWithErrCode);
handler_func_impl!(DivergingHandlerFnWithErrCode);

#[repr(C, align(16))]
pub struct InterruptDescriptorTable<'a> {
    pub divide_error: Entry<HandlerFn>,
    pub debug: Entry<HandlerFn>,
    pub non_maskable_interrupt: Entry<HandlerFn>,
    pub breakpoint: Entry<HandlerFn>,
    pub overflow: Entry<HandlerFn>,
    pub bound_range_exceeded: Entry<HandlerFn>,
    pub invalid_opcode: Entry<HandlerFn>,
    pub device_not_available: Entry<HandlerFn>,
    pub double_fault: Entry<DivergingHandlerFnWithErrCode>,
    coprocessor_segment_overrun: Entry<HandlerFn>,
    pub invalid_tss: Entry<HandlerFnWithErrCode>,
    pub segment_not_present: Entry<HandlerFnWithErrCode>,
    pub stack_segment_fault: Entry<HandlerFnWithErrCode>,
    pub general_protection_fault: Entry<HandlerFnWithErrCode>,
    pub page_fault: Entry<HandlerFnWithErrCode>,
    reserved_1: Entry<HandlerFn>,
    pub x87_floating_point: Entry<HandlerFn>,
    pub alignment_check: Entry<HandlerFnWithErrCode>,
    pub machine_check: Entry<DivergingHandlerFn>,
    pub simd_floating_point: Entry<HandlerFn>,
    pub virtualization: Entry<HandlerFn>,
    pub cp_protection_exception: Entry<HandlerFnWithErrCode>,
    reserved_2: Entry<HandlerFn>,
    pub hv_injection_exception: Entry<HandlerFn>,
    pub vmm_communication_exception: Entry<HandlerFnWithErrCode>,
    pub security_exception: Entry<HandlerFnWithErrCode>,
    reserved_3: Entry<HandlerFn>,
    /// User-defined Interrupts
    interrupts: [Entry<HandlerFn>; 256 - 32],
    _marker: PhantomData<&'a u8>,
}

impl InterruptDescriptorTable<'_> {
    pub const fn new() -> Self {
        Self {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_1: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            cp_protection_exception: Entry::missing(),
            reserved_2: Entry::missing(),
            hv_injection_exception: Entry::missing(),
            vmm_communication_exception: Entry::missing(),
            security_exception: Entry::missing(),
            reserved_3: Entry::missing(),
            interrupts: [Entry::missing(); 256 - 32],
            _marker: PhantomData,
        }
    }
}

impl InterruptDescriptorTable<'static> {
    pub fn load(&self) {
        use crate::arch::instructions::lidt;
        let idtr = IDTRegister {
            size: (size_of::<Entry<HandlerFn>>() * 256 - 1) as u16,
            offset: self as *const _ as usize,
        };
        unsafe { lidt(&idtr) };
    }
}

#[repr(C, packed)]
pub struct IDTRegister {
    size: u16,
    offset: usize,
}

#[repr(C)]
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
pub struct Entry<F> {
    pointer_low: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    _marker: PhantomData<F>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    pub const fn from_u16(val: u16) -> Self {
        match val {
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}

impl<F> Entry<F> {
    pub const fn missing() -> Self {
        Self {
            pointer_low: 0,
            options: EntryOptions::default(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
            _marker: PhantomData,
        }
    }

    pub const fn handler_addr(&self) -> VirtAddr {
        let addr =
            ((self.pointer_high as usize) << 32) | ((self.pointer_middle as usize) << 16) | (self.pointer_low as usize);
        VirtAddr::new(addr)
    }

    pub fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut EntryOptions {
        use crate::arch::registers::segmentation::CS;
        let addr = addr.as_u64();
        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.options = EntryOptions::default();
        unsafe { self.options.set_cs(CS::get_seg()) };
        self.options.set_present(true);
        &mut self.options
    }
}

impl<F: HandlerFunc> Entry<F> {
    pub fn set_handler_fn(&mut self, func: F) -> &mut EntryOptions {
        self.set_handler_addr(func.to_addr())
    }
}

/// Options for Interrupt Entries
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl const Default for EntryOptions {
    /// Creates an empty interrupt entry
    fn default() -> Self {
        Self {
            cs: SegmentSelector(0),
            bits: 0b0000_1110_0000_0000,
        }
    }
}

impl EntryOptions {
    /// Set the Code Segment used by the Interrupt
    pub unsafe fn set_cs(&mut self, cs: SegmentSelector) -> &mut Self {
        self.cs = cs;
        self
    }

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.bits.set_bit(15, present);
        self
    }

    /// Disables Interrupts
    /// This converts the gate type from a trap to a interrupt,
    /// which cleans the IF flag
    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.bits.set_bit(8, !disable);
        self
    }

    #[inline]
    pub fn set_privilege_level(&mut self, dpl: PrivilegeLevel) -> &mut Self {
        self.bits.set_bits(13..15, dpl as u16);
        self
    }

    #[inline]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.bits.set_bits(0..3, index + 1);
        self
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtAddr,
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    pub cpu_flags: RFlags,
    pub stack_pointer: VirtAddr,
    pub stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}

impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterruptStackFrame")
            .field("instruction_pointer", &self.instruction_pointer)
            .field("code_segment", &self.code_segment)
            .field("cpu_flags", &self.cpu_flags)
            .field("stack_pointer", &self.stack_pointer)
            .field("stack_segment", &self.stack_segment)
            .finish()
    }
}

static_assertions::assert_eq_size!(Entry<HandlerFn>, [u8; 16]);

pub static IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());

pub fn init() {
    let mut idt = IDT.lock();

    idt.divide_error.set_handler_fn(handlers::divide_by_zero);
    idt.breakpoint.set_handler_fn(handlers::breakpoint);
    unsafe {
        idt.double_fault
            .set_handler_fn(handlers::double_fault)
            .set_stack_index(Selectors::DOUBLE_FAULT_IST_INDEX as u16);
    };
    idt.page_fault.set_handler_fn(handlers::page_fault);

    idt.load();
}
