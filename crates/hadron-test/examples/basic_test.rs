#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hadron_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

hadron_test::test_entry!(_start);

#[test_case]
fn test_case_1() {
    assert_eq!(0, 1);
}
