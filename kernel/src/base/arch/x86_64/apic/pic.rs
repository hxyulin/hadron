use crate::base::io::io_wait;
use x86_64::instructions::port::Port;

#[derive(Debug, Clone)]
pub struct LegacyPic {
    pic1_cmd: Port<u8>,
    pic1_data: Port<u8>,
    pic2_cmd: Port<u8>,
    pic2_data: Port<u8>,
}

impl LegacyPic {
    const PIC1_CMD: u16 = 0x20;
    const PIC1_DATA: u16 = 0x21;
    const PIC2_CMD: u16 = 0xA0;
    const PIC2_DATA: u16 = 0xA1;

    pub fn new() -> Self {
        Self {
            pic1_cmd: Port::new(Self::PIC1_CMD),
            pic1_data: Port::new(Self::PIC1_DATA),
            pic2_cmd: Port::new(Self::PIC2_CMD),
            pic2_data: Port::new(Self::PIC2_DATA),
        }
    }

    /// Enables the PIC.
    pub fn enable(&mut self) {
        unsafe {
            self.pic1_cmd.write(0x11);
            io_wait();
            self.pic2_cmd.write(0x11);
            io_wait();
        }
    }

    unsafe fn map_irqs(&mut self, irq: u8, irq2: u8) {
        unsafe {
            self.pic1_data.write(irq);
            io_wait();
            self.pic2_data.write(irq2);
            io_wait();
        }
    }

    unsafe fn register_slave(&mut self, irq: u8, irq2: u8) {
        unsafe {
            self.pic1_data.write(irq);
            io_wait();
            self.pic2_data.write(irq2);
            // We need to wait for the slave to be ready
            io_wait();
        }
    }

    unsafe fn set_x86_mode(&mut self) {
        const MASK: u8 = 0x01;
        unsafe {
            self.pic1_data.write(MASK);
            io_wait();
            self.pic2_data.write(MASK);
            io_wait();
        }
    }

    /// Disables the PIC.
    ///
    /// # Safety
    /// This function is unsafe because it can cause UB if interrupts are used for other purposes.
    pub unsafe fn disable(&mut self) {
        self.enable();
        unsafe {
            self.map_irqs(0x20, 0x28);
            self.register_slave(0x04, 0x02);

            self.set_x86_mode();

            // Mask all interrupts
            self.map_irqs(0xFF, 0xFF);
        }
    }
}
