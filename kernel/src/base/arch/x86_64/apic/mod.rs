use local::LocalApic;
use pic::LegacyPic;
use x86_64::PhysAddr;

use crate::base::info::kernel_info;

pub mod io;
pub mod local;
pub mod pic;

pub struct Apics {
    legacy: Option<LegacyPic>,
    lapic: LocalApic,
}

impl Apics {
    pub fn new(apic: &acpi::platform::interrupt::Apic<'_, alloc::alloc::Global>) -> Self {
        x86_64::instructions::interrupts::disable();
        let legacy = if apic.also_has_legacy_pics {
            log::debug!("ACPI: also has legacy PICs");
            let mut pic = LegacyPic::new();
            unsafe { pic.disable() };
            Some(pic)
        } else {
            None
        };

        // A page aligned size for the local APIC, so we round up to the next page size
        let mmio_addr = kernel_info()
            .mmio
            .lock()
            .allocate_persistant(PhysAddr::new(apic.local_apic_address), 4096);
        log::debug!(
            "ACPI: local APIC at {:#x} (base = {:#x})",
            mmio_addr,
            apic.local_apic_address
        );
        let mut lapic = LocalApic::new(mmio_addr);
        lapic.init(apic.local_apic_address);

        // TODO: IO APICs

        Self { legacy, lapic }
    }
}
