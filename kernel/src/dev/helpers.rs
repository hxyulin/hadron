pub fn identity_vendor_id(vendor_id: u16) -> Option<&'static str> {
    match vendor_id {
        0x1234 => Some("QEMU"),
        0x8086 => Some("Intel"),
        0x10DE => Some("Nvidia"),
        _ => None,
    }
}
