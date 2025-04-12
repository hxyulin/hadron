use alloc::boxed::Box;
use core::slice;
use gimli::{
    BaseAddresses, CfaRule, EhFrame, EhFrameHdr, EhHdrTable, EndianSlice, LittleEndian, ParsedEhFrameHdr, Pointer,
    Register, RegisterRule, UnwindContext, UnwindSection, X86_64,
};

use crate::util::machine_state::MachineState;

unsafe extern "C" {
    /// The start of the `.eh_frame_hdr` section.
    static __EH_FRAME_HDR_START: u8;
    /// The end of the `.eh_frame_hdr` section.
    static __EH_FRAME_HDR_END: u8;
    /// The start of the `.eh_frame` section.
    static __EH_FRAME_START: u8;
    /// The end of the `.eh_frame` section.
    static __EH_FRAME_END: u8;
}

/// The size of the `.eh_frame_hdr` section in bytes.
#[inline]
fn eh_frame_hdr_size() -> usize {
    let start = &raw const __EH_FRAME_HDR_START;
    let end = &raw const __EH_FRAME_HDR_END;
    (end as usize) - (start as usize)
}

/// The size of the `.eh_frame` section in bytes.
#[inline]
fn eh_frame_size() -> usize {
    let start = &raw const __EH_FRAME_START;
    let end = &raw const __EH_FRAME_END;
    (end as usize) - (start as usize)
}

/// A struct containing information about EhFrame.
#[derive(Debug)]
struct EhInfo<'a> {
    base_addrs: BaseAddresses,
    hdr: &'a ParsedEhFrameHdr<EndianSlice<'a, LittleEndian>>,
    hdr_table: EhHdrTable<'a, EndianSlice<'a, LittleEndian>>,
    eh_frame: gimli::EhFrame<EndianSlice<'a, LittleEndian>>,
}

impl EhInfo<'static> {
    /// Create a new `EhInfo` from a pointer to the `.eh_frame_hdr` section.
    ///
    /// # Safety
    /// The pointer must point to a valid `.eh_frame_hdr` section.
    unsafe fn from_hdr_ptr(eh_frame_hdr: *const u8) -> Self {
        let mut base_addrs = BaseAddresses::default();
        base_addrs = base_addrs.set_eh_frame_hdr(eh_frame_hdr as u64);

        // FIXME: Can we make this maybe not leak?
        let hdr = Box::leak(Box::new(
            EhFrameHdr::new(
                unsafe { slice::from_raw_parts(eh_frame_hdr, eh_frame_hdr_size()) },
                LittleEndian,
            )
            .parse(&base_addrs, 8)
            .unwrap(),
        ));

        // We deduce the `.eh_frame` address, only direct pointers are implemented.
        let eh_frame = match hdr.eh_frame_ptr() {
            Pointer::Direct(addr) => addr as *mut u8,
            _ => unimplemented!(),
        };

        // We then add the `.eh_frame` address for addresses relative to that
        // section.
        base_addrs = base_addrs.set_eh_frame(eh_frame as u64);

        // The `.eh_frame` section is then parsed.
        let eh_frame = EhFrame::new(
            unsafe { slice::from_raw_parts(eh_frame, eh_frame_size()) },
            LittleEndian,
        );

        Self {
            base_addrs,
            hdr,
            hdr_table: hdr.table().unwrap(),
            eh_frame,
        }
    }
}

/// A struct containing the register values for a call frame.
///
/// This does not include all the registers, only the ones that are needed for unwinding.
#[derive(Debug, Default)]
struct RegisterSet {
    rip: Option<u64>,
    rsp: Option<u64>,
    rbp: Option<u64>,
    ret: Option<u64>,
}

impl RegisterSet {
    fn get(&self, reg: Register) -> Option<u64> {
        match reg {
            X86_64::RSP => self.rsp,
            X86_64::RBP => self.rbp,
            X86_64::RA => self.ret,
            _ => None,
        }
    }

    fn set(&mut self, reg: Register, val: u64) -> Result<(), UnwinderError> {
        *match reg {
            X86_64::RSP => &mut self.rsp,
            X86_64::RBP => &mut self.rbp,
            X86_64::RA => &mut self.ret,
            _ => return Err(UnwinderError::UnexpectedRegister(reg)),
        } = Some(val);

        Ok(())
    }

    fn undef(&mut self, reg: Register) {
        *match reg {
            X86_64::RSP => &mut self.rsp,
            X86_64::RBP => &mut self.rbp,
            X86_64::RA => &mut self.ret,
            _ => return,
        } = None;
    }

    fn get_pc(&self) -> Option<u64> {
        self.rip
    }

    fn set_pc(&mut self, val: u64) {
        self.rip = Some(val);
    }

    fn get_ret(&self) -> Option<u64> {
        self.ret
    }

    fn set_stack_ptr(&mut self, val: u64) {
        self.rsp = Some(val);
    }

    fn iter() -> impl Iterator<Item = Register> {
        [X86_64::RSP, X86_64::RBP, X86_64::RA].into_iter()
    }

    fn from_machine_state(machine_state: &MachineState) -> Self {
        Self {
            rip: Some(machine_state.rip),
            rsp: Some(machine_state.rsp),
            rbp: Some(machine_state.rbp),
            ret: None,
        }
    }
}

/// A call frame.
#[derive(Debug)]
pub struct CallFrame {
    pub pc: u64,
}

/// An error that can occur during unwinding.
#[derive(Debug)]
pub enum UnwinderError {
    UnexpectedRegister(Register),
    UnsupportedCfaRule,
    UnimplementedRegisterRule,
    CfaRuleUnknownRegister(Register),
    NoUnwindInfo,
    NoPcRegister,
    NoReturnAddr,
}

/// A virtual unwinder
/// meaning that it doesn't actually call destructors, just unwinds the stack.
/// This is meant to be used for backtraces, and is not meant to be recoverable.
pub struct VirtualUnwinder {
    eh_info: EhInfo<'static>,
    unwind_ctx: UnwindContext<EndianSlice<'static, LittleEndian>>,
    regs: RegisterSet,
    cfa: u64,
    is_first: bool,
}

impl VirtualUnwinder {
    /// Create a new `VirtualUnwinder` from a `EhInfo` and a `RegisterSet`.
    fn new(eh_info: EhInfo<'static>, register_set: RegisterSet) -> Self {
        Self {
            eh_info,
            unwind_ctx: UnwindContext::new(),
            regs: register_set,
            cfa: 0,
            is_first: true,
        }
    }

    /// Get the next call frame.
    ///
    /// Returns `Ok(None)` or `Err(UnwinderError::NoUnwindInfo)` if there are no more frames.
    pub fn next(&mut self) -> Result<Option<CallFrame>, UnwinderError> {
        let pc = self.regs.get_pc().ok_or(UnwinderError::NoPcRegister)?;

        // If it is the first iteration, we dont need to unwind yet, just return the current frame.
        if self.is_first {
            self.is_first = false;
            return Ok(Some(CallFrame { pc }));
        }

        // The row in the unwind table
        let row = self
            .eh_info
            .hdr_table
            .unwind_info_for_address(
                &self.eh_info.eh_frame,
                &self.eh_info.base_addrs,
                &mut self.unwind_ctx,
                pc,
                |section, bases, offset| section.cie_from_offset(bases, offset),
            )
            .map_err(|_| UnwinderError::NoUnwindInfo)?;

        // Upadted the CFA address.
        match row.cfa() {
            CfaRule::RegisterAndOffset { register, offset } => {
                let reg_val = self
                    .regs
                    .get(*register)
                    .ok_or(UnwinderError::CfaRuleUnknownRegister(*register))?;
                self.cfa = (reg_val as i64 + offset) as u64;
            }
            _ => return Err(UnwinderError::UnsupportedCfaRule),
        }

        // Update the registers.
        for reg in RegisterSet::iter() {
            match row.register(reg) {
                RegisterRule::Undefined => self.regs.undef(reg),
                RegisterRule::SameValue => (),
                RegisterRule::Offset(offset) => {
                    let ptr = (self.cfa as i64 + offset) as u64 as *const usize;
                    self.regs.set(reg, unsafe { ptr.read() } as u64)?;
                }
                _ => return Err(UnwinderError::UnimplementedRegisterRule),
            }
        }

        let pc = self.regs.get_ret().ok_or(UnwinderError::NoReturnAddr)? - 1;
        self.regs.set_pc(pc);
        self.regs.set_stack_ptr(self.cfa);

        Ok(Some(CallFrame { pc }))
    }
}

#[inline(always)]
pub fn create_unwinder(state: &MachineState) -> VirtualUnwinder {
    let eh_info = unsafe { EhInfo::from_hdr_ptr(&raw const __EH_FRAME_HDR_START) };
    let registers = RegisterSet::from_machine_state(state);
    VirtualUnwinder::new(eh_info, registers)
}

#[inline(always)]
pub fn panic_backtrace(panic_info: &core::panic::PanicInfo) {
    log::error!("KERNEL PANIC: {}", panic_info.message());
    if let Some(location) = panic_info.location() {
        log::error!("    at {}:{}:{}", location.file(), location.line(), location.column());
    }
    let machine_state = MachineState::here();
    let mut unwinder = create_unwinder(&machine_state);

    // Print the backtrace, ignoring any errors, since we don't care about them.
    while let Ok(Some(frame)) = unwinder.next() {
        log::error!("    at {:#X}", frame.pc);
    }

    log::error!("{}", machine_state);
}
