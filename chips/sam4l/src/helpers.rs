use core::convert::TryFrom;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

static DEFERRED_CALL: AtomicUsize = AtomicUsize::new(0);

/// Represents a way to generate an asynchronous call without a hardware interrupt.
pub struct DeferredCall(Task);

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum Task {
    Flashcalw = 0,
}

impl TryFrom<usize> for Task {
    type Error = ();

    fn try_from(value: usize) -> Result<Task, ()> {
        match value {
            0 => Ok(Task::Flashcalw),
            _ => Err(()),
        }
    }
}

impl DeferredCall {
    /// Creates a new DeferredCall
    ///
    /// Only create one per task, preferably in the module that it will be used in.
    pub const unsafe fn new(task: Task) -> DeferredCall {
        DeferredCall(task)
    }

    /// Set the `DeferredCall` as pending
    pub fn set(&self) {
        DEFERRED_CALL.fetch_or(1 << self.0 as usize, Ordering::Relaxed);
    }

    /// Are there any pending `DeferredCall`s
    pub fn has_tasks() -> bool {
        DEFERRED_CALL.load(Ordering::Relaxed) != 0
    }

    /// Gets and clears the next pending `DeferredCall`
    pub fn next_pending() -> Option<Task> {
        let val = DEFERRED_CALL.load(Ordering::Relaxed);
        if val == 0 {
            return None;
        } else {
            let bit = val.trailing_zeros() as usize;
            let new_val = val & !(1 << bit);
            DEFERRED_CALL.store(new_val, Ordering::Relaxed);
            return Task::try_from(bit).ok();
        }
    }
}
