use crate::base::io::mmio::OffsetMmio;
use alloc::vec::Vec;
use core::fmt::Debug;
use x86_64::VirtAddr;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IoApicDeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    SMI = 0b010,
    NMI = 0b100,
    INIT = 0b101,
    ExtINT = 0b111,
}

impl IoApicDeliveryMode {
    pub fn from_flags(flags: u16) -> Self {
        Self::try_from((flags & 0b111) as u8).unwrap()
    }
}

impl TryFrom<u8> for IoApicDeliveryMode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(Self::Fixed),
            0b001 => Ok(Self::LowestPriority),
            0b010 => Ok(Self::SMI),
            0b100 => Ok(Self::NMI),
            0b101 => Ok(Self::INIT),
            0b111 => Ok(Self::ExtINT),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IoApicDestinationMode {
    Physical = 0,
    Logical = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IoApicTriggerMode {
    Edge = 0,
    Level = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum IoApicEntryDestination {
    Cpu(u8),
}

/// The redirection table entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoApicEntry {
    irq: u8,
    flags: u16,
    reserved2: u16,
    reserved3: u8,
    destination: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct IoApicSourceOverrideEntry {
    pub bus: u8,
    pub irq: u8,
    pub flags: u16,
    pub gsi: u32,
}

impl Debug for IoApicEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IoApicEntry")
            .field("irq", &self.irq)
            .field("delivery_mode", &IoApicDeliveryMode::from_flags(self.flags))
            .field("destination_mode", &((self.flags >> 3) & 0b1))
            .field("delivery_status", &((self.flags >> 4) & 0b1))
            .field("pin_polarity", &((self.flags >> 5) & 0b1))
            .field("remote_irr", &((self.flags >> 6) & 0b1))
            .field("trigger_mode", &((self.flags >> 7) & 0b1))
            .field("enabled", &((self.flags >> 8) & 0b1))
            .field("destination", &self.destination)
            .finish_non_exhaustive()
    }
}

impl Default for IoApicEntry {
    /// The default entry is disabled, edge trigger, physical destination, and fixed delivery mode
    fn default() -> Self {
        Self {
            irq: 0,
            flags: (1 << 8),
            reserved2: 0,
            reserved3: 0,
            destination: 0,
        }
    }
}

impl IoApicEntry {
    pub fn new(irq: u8, dest: IoApicEntryDestination) -> Self {
        match dest {
            IoApicEntryDestination::Cpu(cpu) => Self {
                irq,
                flags: 0,
                destination: cpu,
                ..Self::default()
            },
        }
    }

    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    pub fn set_delivery_mode(&mut self, mode: IoApicDeliveryMode) {
        self.flags &= !0b111;
        self.flags |= mode as u16;
    }

    pub fn set_destination_mode(&mut self, mode: IoApicDestinationMode) {
        self.flags &= !0b100;
        self.flags |= (mode as u16) << 3;
    }

    pub fn set_delivery_status(&mut self, status: bool) {
        if status {
            self.flags |= 1 << 4;
        } else {
            self.flags &= !(1 << 4);
        }
    }

    pub fn set_pin_polarity(&mut self, polarity: bool) {
        if polarity {
            self.flags |= 1 << 5;
        } else {
            self.flags &= !(1 << 5);
        }
    }

    pub fn set_remote_irr(&mut self, _irr: bool) {
        unimplemented!();
    }

    pub fn set_trigger_mode(&mut self, mode: u8) {
        self.flags = (self.flags & !(0b1 << 7)) | ((mode as u16) << 7);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.flags &= !(1 << 8);
        } else {
            self.flags |= 1 << 8;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IoApic {
    base: VirtAddr,
    gsi_base: u32,
    /// Number of entries in the redirection table
    size: u32,

    sel: OffsetMmio<u32, 0x00>,
    win: OffsetMmio<u32, 0x10>,
}

impl IoApic {
    pub const SIZE_ALIGNED: u64 = 4096;

    const IDX_REG_ID: u32 = 0x00;
    const IDX_REG_VER: u32 = 0x01;
    const IDX_REG_ARB: u32 = 0x02;

    fn read_reg(&self, index: u32) -> u32 {
        self.sel.write(self.base, index);
        self.win.read(self.base)
    }

    fn write_reg(&self, index: u32, value: u32) {
        self.sel.write(self.base, index);
        self.win.write(self.base, value);
    }

    fn table_entry(index: usize) -> usize {
        0x10 + 2 * index
    }

    pub fn new(base: VirtAddr, gsi_base: u32) -> Self {
        let mut io_apic = Self {
            base,
            gsi_base,
            size: 0,
            sel: OffsetMmio::default(),
            win: OffsetMmio::default(),
        };

        io_apic.size = io_apic.read_reg(Self::IDX_REG_VER) & 0xFF;

        io_apic
    }

    pub fn write_entry(&mut self, index: usize, entry: IoApicEntry) {
        let data = unsafe { core::mem::transmute::<IoApicEntry, u64>(entry) };
        self.write_reg(Self::table_entry(index) as u32, (data & 0xFFFF_FFFF) as u32);
        self.write_reg(Self::table_entry(index) as u32 + 1, (data >> 32) as u32);
    }

    pub fn read_entry(&self, index: usize) -> IoApicEntry {
        let data_low = self.read_reg(Self::table_entry(index) as u32);
        let data_high = self.read_reg(Self::table_entry(index) as u32 + 1);
        unsafe { core::mem::transmute::<u64, IoApicEntry>(data_low as u64 | (data_high as u64) << 32) }
    }
}

#[derive(Debug, Clone)]
pub struct IoApics {
    apics: Vec<IoApic>,
    source_overrides: Vec<IoApicSourceOverrideEntry>,
    /// Bitmap of GSIs that are in use
    gsi_bitmap: u64,
}

impl IoApics {
    pub const fn empty() -> Self {
        Self {
            apics: Vec::new(),
            source_overrides: Vec::new(),
            gsi_bitmap: 0,
        }
    }

    pub fn add_apic(&mut self, apic: IoApic) {
        self.apics.push(apic);
    }

    pub fn add_source_override(&mut self, entry: IoApicSourceOverrideEntry) {
        self.gsi_bitmap |= 1 << entry.gsi;
        self.source_overrides.push(entry);
    }

    fn allocate_gsi(&mut self, irq: u8) -> u32 {
        let index = irq as usize;
        if self.gsi_bitmap & (1 << index) == 0 {
            self.gsi_bitmap |= 1 << index;
            return index as u32;
        }
        panic!("GSI {} already allocated", index);
    }

    /// Map an IRQ to a GSI, returns the GSI
    pub fn map_irq(&mut self, irq: u8) -> u32 {
        let (gsi, mut entry) = match self.source_overrides.iter().find(|entry| entry.irq == irq) {
            Some(source_override) => {
                let mut entry = IoApicEntry::default();
                entry.set_flags(source_override.flags);
                entry.destination = source_override.bus;
                (source_override.gsi, IoApicEntry::default())
            }
            None => {
                let gsi = self.allocate_gsi(irq);
                (gsi, IoApicEntry::default())
            }
        };

        let apic = self
            .apics
            .iter_mut()
            .find(|apic| apic.gsi_base <= gsi && gsi < apic.gsi_base + apic.size)
            .unwrap();
        let index = gsi - apic.gsi_base;
        entry.irq = irq + 32;
        entry.set_enabled(true);
        apic.write_entry(index as usize, entry);
        log::trace!("Mapped IRQ {} to GSI {}", irq, gsi);
        let entry = apic.read_entry(index as usize);
        log::debug!("Read entry: {:?}", entry);
        gsi
    }
}
