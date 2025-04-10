use core::arch::asm;

pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: ExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

struct Port {
    port: u16,
}

impl Port {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    pub unsafe fn write(&mut self, data: u32) {
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") self.port,
                in("eax") data,
                options(nostack, preserves_flags)
            );
        }
    }
}
