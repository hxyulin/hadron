use core::{pin::Pin, task::Context};

pub mod executor;

use alloc::boxed::Box;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelTaskId(u64);

impl KernelTaskId {
    pub fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static ID: AtomicU64 = AtomicU64::new(0);
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct KernelTask {
    pub id: KernelTaskId,
    fut: Pin<Box<dyn Future<Output = ()>>>,
}

impl KernelTask {
    pub fn new(fut: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: KernelTaskId::new(),
            fut: Box::pin(fut),
        }
    }

    pub fn poll(&mut self, ctx: &mut Context) -> core::task::Poll<()> {
        self.fut.as_mut().poll(ctx)
    }
}

pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> core::task::Poll<Self::Output> {
        use core::task::Poll;
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            ctx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Yields the current task
///
/// This means that the current task will be scheduled to run again,
/// but giving up the CPU for other tasks.
pub async fn yield_now() {
    YieldNow { yielded: false }.await
}
