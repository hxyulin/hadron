use core::task::Waker;

use alloc::{borrow::ToOwned, collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam::queue::ArrayQueue;

use super::{KernelTask, KernelTaskId};

struct BasicWaker {
    task_id: KernelTaskId,
    task_queue: Arc<ArrayQueue<KernelTaskId>>,
}

impl BasicWaker {
    fn new(task_id: KernelTaskId, task_queue: Arc<ArrayQueue<KernelTaskId>>) -> Waker {
        let waker = Arc::new(Self { task_id, task_queue });
        Waker::from(waker)
    }
}

impl Wake for BasicWaker {
    fn wake(self: Arc<Self>) {
        self.task_queue.push(self.task_id).unwrap();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.task_queue.push(self.task_id).unwrap();
    }
}

/// A basic round-robin task executor
///
/// This executor is very simple, and does not support any kind of priority scheduling.
/// This is intended for basic async/await support.
pub struct BasicExecutor {
    tasks: BTreeMap<KernelTaskId, KernelTask>,
    task_queue: Arc<ArrayQueue<KernelTaskId>>,
    waker_cache: BTreeMap<KernelTaskId, Waker>,
}

impl BasicExecutor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(128)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: KernelTask) {
        let task_id = task.id;
        let _prev = self.tasks.insert(task_id, task);
        assert!(_prev.is_none(), "task already exists");
        self.task_queue.push(task_id).expect("task queue is full");
    }

    pub fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };

            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| BasicWaker::new(task_id.into(), self.task_queue.clone()));

            let mut ctx = Context::from_waker(waker);
            use core::task::{Context, Poll};
            match task.poll(&mut ctx) {
                Poll::Ready(result) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_ready_tasks();
            // We don't have interrupts yet so we just busy wait
        }
    }
}
