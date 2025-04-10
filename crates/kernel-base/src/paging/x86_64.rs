use super::PageSize;

/// Represents a 4KiB page.
pub struct Size4KiB;
/// Represents a 2MiB page.
pub struct Size2MiB;
/// Represents a 1GiB page.
pub struct Size1GiB;

impl PageSize for Size4KiB {
    const SIZE: usize = 4096;
}

impl PageSize for Size2MiB {
    const SIZE: usize = 2 * 1024 * 1024;
}

impl PageSize for Size1GiB {
    const SIZE: usize = 1024 * 1024 * 1024;
}

#[cfg(test)]
mod tests {
    use super::*;

    static_assertions::const_assert_eq!(Size4KiB::SIZE, 4096);
    static_assertions::const_assert_eq!(Size2MiB::SIZE, 2 * 1024 * 1024);
    static_assertions::const_assert_eq!(Size1GiB::SIZE, 1024 * 1024 * 1024);
}
