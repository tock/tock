use core::convert::TryFrom;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

static DEFERED_CALL: AtomicU32 = AtomicU32::new(0);

/// Represents a way to generate an asynchronous call without a hardware interrupt.
pub struct DeferedCall(Task);

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum Task {
    Flashcalw = 0,
}

impl TryFrom<u32> for Task {
    type Error = ();

    fn try_from(value: u32) -> Result<Task, ()> {
        match value {
            0 => Ok(Task::Flashcalw),
            _ => Err(())
        }
    }
}

impl DeferedCall {
    /// Creates a new DeferedCall
    ///
    /// Only create one per task, preferably in the module that it will be used in.
    pub const unsafe fn new(task: Task) -> DeferedCall {
        DeferedCall(task)
    }

    /// Set the `DeferedCall` as pending
    pub fn set(&self) {
        DEFERED_CALL.fetch_or(1 << self.0 as u32, Ordering::Relaxed);
    }

    /// Are there any pending `DeferedCall`s
    pub fn has_tasks() -> bool {
        DEFERED_CALL.load(Ordering::Relaxed) != 0
    }

    /// Gets and clears the next pending `DeferedCall`
    pub fn next_pending() -> Option<Task> {
        let val = DEFERED_CALL.load(Ordering::Relaxed);
        if val == 0 {
            return None
        } else {
            let bit = val.trailing_zeros();
            let new_val = val & !(1 << bit);
            DEFERED_CALL.store(new_val, Ordering::Relaxed);
            return Task::try_from(bit).ok();
        }
    }
}
