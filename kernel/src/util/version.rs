#[derive(Clone, Copy)]
pub struct SemVer {
    major: u32,
    minor: u32,
    patch: u32,
}

impl core::fmt::Debug for SemVer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "V{}.{}.{}", self.major, self.minor, self.patch)
    }
}
