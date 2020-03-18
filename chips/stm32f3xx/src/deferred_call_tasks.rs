//! Definition of Deferred Call tasks.
//!
//! Deferred calls allow peripheral drivers to register pseudo interrupts.
//! These are the definitions of which deferred calls this chip needs.

use core::convert::Into;
use core::convert::TryFrom;

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum Task {
    // ... add deffred tasks here
    // Example = 0,
}

impl TryFrom<usize> for Task {
    type Error = ();

    fn try_from(value: usize) -> Result<Task, ()> {
        match value {
            0 => Ok(Task::Example),
            _ => Err(()),
        }
    }
}

impl Into<usize> for Task {
    fn into(self) -> usize {
        self as usize
    }
}
