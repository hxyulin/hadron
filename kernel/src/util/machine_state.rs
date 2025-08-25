use core::fmt::Display;

#[cfg(target_arch = "x86_64")]
#[derive(Debug)]
pub struct MachineState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,

    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    pub rdi: u64,
    pub rsi: u64,

    pub rsp: u64,
    pub rbp: u64,

    pub rip: u64,
    pub rflags: u64,

    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,

    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,

    pub efer: u64,
}

#[cfg(target_arch = "aarch64")]
pub struct MachineState {}

impl MachineState {
    #[inline(always)]
    pub fn here() -> Self {
        todo!();
        //let rip = x86_64::registers::read_rip().as_u64();
        //Self::with_rip(rip)
    }
}

#[cfg(target_arch = "x86_64")]
impl MachineState {
    pub fn from_stack_frame(stack_frame: &crate::arch::x86_64::core::idt::InterruptStackFrame) -> Self {
        let rip = stack_frame.instruction_pointer.as_u64();
        let mut state = Self::with_rip(rip);
        // Because we might have switched to a different stack, we need to
        // update the stack pointer.
        state.rsp = stack_frame.stack_pointer.as_u64();
        state.ss = stack_frame.stack_segment.0;
        state
    }

    pub fn with_rip(rip: u64) -> Self {
        let rsp;
        let rbp;
        let cs;
        let ss;
        let ds;
        let es;
        let fs;
        let gs;

        unsafe {
            core::arch::asm!(
                "mov    {}, rsp",
                "mov    {}, rbp",
                "mov    {:x}, cs",
                "mov    {:x}, ss",
                "mov    {:x}, ds",
                "mov    {:x}, es",
                "mov    {:x}, fs",
                "mov    {:x}, gs",
                out(reg) rsp,
                out(reg) rbp,
                out(reg) cs,
                out(reg) ss,
                out(reg) ds,
                out(reg) es,
                out(reg) fs,
                out(reg) gs,
            );
        }

        /*
        let cr0 = x86_64::registers::control::Cr0::read_raw();
        let cr2 = x86_64::registers::control::Cr2::read_raw();
        let (page_table, flags) = x86_64::registers::control::Cr3::read_raw();
        let cr3 = page_table.start_address().as_u64() | ((flags as u64) << 48);
        let cr4 = x86_64::registers::control::Cr4::read_raw();

        let efer = x86_64::registers::control::Efer::read_raw();
        */
        let cr0 = 0;
        let cr2 = 0;
        let cr3 = 0;
        let cr4 = 0;
        let efer = 0;

        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rdi: 0,
            rsi: 0,
            rip,
            rsp,
            rbp,
            rflags: 0,
            cs,
            ss,
            ds,
            es,
            fs,
            gs,

            cr0,
            cr2,
            cr3,
            cr4,

            efer,
        }
    }
}

impl Display for MachineState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(target_arch = "x86_64")]
        {
            writeln!(f, "rsp:   0x{:016X}    rbp:   0x{:016X}", self.rsp, self.rbp,)?;
            writeln!(f, "rax:   0x{:016X}    rbx:   0x{:016X}", self.rax, self.rbx,)?;
            writeln!(f, "rcx:   0x{:016X}    rdx:   0x{:016X}", self.rcx, self.rdx,)?;
            writeln!(f, "rdi:   0x{:016X}    rsi:   0x{:016X}", self.rdi, self.rsi,)?;
            writeln!(f, "r8:    0x{:016X}    r9:    0x{:016X}", self.r8, self.r9,)?;
            writeln!(f, "r10:   0x{:016X}    r11:   0x{:016X}", self.r10, self.r11,)?;
            writeln!(f, "r12:   0x{:016X}    r13:   0x{:016X}", self.r12, self.r13,)?;
            writeln!(f, "r14:   0x{:016X}    r15:   0x{:016X}", self.r14, self.r15,)?;
            writeln!(f, "rip:   0x{:016X}    rflags:0x{:016X}", self.rip, self.rflags,)?;
            writeln!(
                f,
                "cs:    0x{:04X}    ss:    0x{:04X}    ds:    0x{:04X}",
                self.cs, self.ss, self.ds
            )?;
            writeln!(
                f,
                "es:    0x{:04X}    fs:    0x{:04X}    gs:    0x{:04X}",
                self.es, self.fs, self.gs
            )?;
            writeln!(f, "cr0:   0x{:016X}    cr2:   0x{:016X}", self.cr0, self.cr2)?;
            writeln!(f, "cr3:   0x{:016X}    cr4:   0x{:016X}", self.cr3, self.cr4)?;
            writeln!(f, "efer:  0x{:016X}", self.efer)?;
        }

        Ok(())
    }
}
