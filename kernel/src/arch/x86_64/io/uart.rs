use core::fmt;

use crate::arch::x86_64::io::{inb, outb};

/// Offsets from the base port for key UART registers.
const DATA_REG: u16 = 0; // Data Register (RW) / Divisor Latch Low (DLL) (RW)
const INT_ENABLE_REG: u16 = 1; // Interrupt Enable Register (IER) (RW) / Divisor Latch High (DLH) (RW)
const FIFO_CONTROL_REG: u16 = 2; // FIFO Control Register (FCR) (W) / Interrupt Identification Register (IIR) (R)
const LINE_CONTROL_REG: u16 = 3; // Line Control Register (LCR) (RW)
const MODEM_CONTROL_REG: u16 = 4; // Modem Control Register (MCR) (RW)
const LINE_STATUS_REG: u16 = 5; // Line Status Register (LSR) (R)
const MODEM_STATUS_REG: u16 = 6; // Modem Status Register (MSR) (R)
const SCRATCHPAD_REG: u16 = 7; // Scratchpad Register (SR) (RW)

pub struct Uart16550 {
    port_base: u16,
}

impl Uart16550 {
    /// Creates a new `Uart16550` instance for the given base port.
    ///
    /// # Safety
    /// This function is unsafe because it involves direct hardware access and
    /// assumes the given port is a valid UART base address.
    pub const unsafe fn new(port_base: u16) -> Self {
        Uart16550 { port_base }
    }

    /// Initializes the UART for basic polling mode.
    ///
    /// # Safety
    /// This function is unsafe because it performs direct I/O port writes
    /// and should only be called once during early kernel initialization.
    pub unsafe fn init(&mut self) {
        unsafe {
            // 1. Disable all interrupts
            outb(self.port_base + INT_ENABLE_REG, 0x00);

            // 2. Enable DLAB (Divisor Latch Access Bit) to set baud rate
            //    LCR bit 7 = 1
            outb(self.port_base + LINE_CONTROL_REG, 0x80);

            // 3. Set Baud Rate (e.g., 115200 bps for a 1.8432 MHz crystal)
            //    Divisor = 115200 / (16 * Baud Rate) = 115200 / (16 * 115200) = 1
            //    For 38400 bps, Divisor = 3
            //    For 9600 bps, Divisor = 12
            //    Let's use 38400 for qemu, it's often more stable than 115200
            let baud_rate_divisor: u16 = 3; // For 38400 baud
            outb(self.port_base + DATA_REG, (baud_rate_divisor & 0xFF) as u8); // Divisor Low
            outb(self.port_base + INT_ENABLE_REG, (baud_rate_divisor >> 8) as u8); // Divisor High

            // 4. Disable DLAB (LCR bit 7 = 0) and set 8 data bits, no parity, 1 stop bit (8N1)
            //    LCR = 0x03 (8 data bits, 1 stop bit, no parity)
            outb(self.port_base + LINE_CONTROL_REG, 0x03);

            // 5. Enable FIFOs, clear them, and set interrupt threshold
            //    FCR = 0x01 (enable FIFOs) | 0x02 (clear Rx FIFO) | 0x04 (clear Tx FIFO) | 0x08 (use DMA mode, but not needed here) | 0xC0 (Rx trigger level 14 bytes)
            outb(self.port_base + FIFO_CONTROL_REG, 0xC7); // Enable, Clear Rx/Tx FIFO, 14-byte threshold

            // 6. Set Modem Control Register (MCR)
            //    MCR = 0x03 (DTR=1, RTS=1), or 0x0B for loopback mode (for testing)
            outb(self.port_base + MODEM_CONTROL_REG, 0x03);

            // 7. Read LSR to clear any pending interrupts/errors
            inb(self.port_base + LINE_STATUS_REG);
        }
    }

    /// Checks if the transmit holding register is empty.
    /// This means the UART is ready to accept a new byte for transmission.
    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(self.port_base + LINE_STATUS_REG) & 0x20) != 0 } // LSR bit 5: THRE
    }

    /// Writes a single byte to the serial port, blocking until it can be sent.
    pub fn write_byte(&mut self, byte: u8) {
        // Wait until the transmit buffer is empty
        while !self.is_transmit_empty() {
            // spin loop
        }
        // Send the byte
        unsafe { outb(self.port_base + DATA_REG, byte) };
    }
}

impl fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            self.write_byte(*b);
        }
        Ok(())
    }
}
