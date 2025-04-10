#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hadron_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod gdt;

#[cfg(test)]
mod tests {
    use super::*;

    hadron_test::test_entry!(_start);

    #[test_case]
    fn test_gdt() {
        assert!(false);
    }
}
