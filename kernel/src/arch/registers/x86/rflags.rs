bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct RFlags: u64 {
        const _ = !0;
    }
}
