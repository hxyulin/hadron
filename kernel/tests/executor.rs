#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hadron_test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(static_mut_refs)]

use core::sync::atomic::{AtomicU32, Ordering};

use hadron_kernel::base::{
    mem::sync::RacyCell,
    task::{KernelTask, executor::BasicExecutor, yield_now},
};
use hadron_test::{println, test_entry};

test_entry!(kernel_entry, init);

fn init() {
    static HEAP: RacyCell<[u8; 4096]> = RacyCell::new([0; 4096]);
    unsafe { hadron_kernel::ALLOCATOR.init_generic(HEAP.get_mut().as_mut_ptr(), 4096) };
}

/// A basic test case to test cooperative multitasking.
#[test_case]
fn test_executor() {
    static DATA: AtomicU32 = AtomicU32::new(0);
    let mut task_executor = BasicExecutor::new();
    task_executor.spawn(KernelTask::new(async {
        loop {
            if DATA.fetch_add(1, Ordering::Relaxed) >= 100 {
                break;
            }
            yield_now().await;
        }
    }));
    task_executor.spawn(KernelTask::new(async {
        loop {
            static COUNTER: AtomicU32 = AtomicU32::new(1);
            let data = DATA.load(Ordering::Relaxed);
            assert_eq!(data, COUNTER.fetch_add(1, Ordering::Relaxed));
            if data >= 100 {
                break;
            }
            yield_now().await;
        }
    }));
    // We only run the ready tasks once, because we don't want to block the test.
    task_executor.run_ready_tasks();
    assert_eq!(DATA.load(Ordering::Relaxed), 101);
}
