use core::arch::asm;

pub mod uart;

/// # Safety
///
/// This function is unsafe because it directory interacts with hardware and does not check if the port is valid.
#[inline]
unsafe fn outb(port: u16, data: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") data);
    }
}

/// # Safety
///
/// This function is unsafe because it directory interacts with hardware and does not check if the port is valid.
#[inline]
unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let mut data: u8;
        asm!("in al, dx", out("al") data, in("dx") port);
        data
    }
}
