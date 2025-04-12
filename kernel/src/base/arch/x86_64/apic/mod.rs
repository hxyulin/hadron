use pic::LegacyPic;

pub mod io;
pub mod local;
pub mod pic;

pub struct Apics {
    legacy: Option<LegacyPic>,
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

        Self { legacy }
    }
}
