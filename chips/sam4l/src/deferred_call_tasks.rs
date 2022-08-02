//! Definition of Deferred Call tasks for SAM4L chip peripherals.
//!
//! Deferred calls allow peripheral drivers to register pseudo interrupts.
//! These are the definitions of which deferred calls this chip needs.

use core::convert::Into;
use core::convert::TryFrom;
use kernel::deferred_call::PeripheralTask;

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum Task {
    Flashcalw = 0xf0000,
    CRCCU = 0xf0001,
}

impl TryFrom<usize> for Task {
    type Error = ();

    fn try_from(value: usize) -> Result<Task, ()> {
        match value {
            0xf0000 => Ok(Task::Flashcalw),
            0xf0001 => Ok(Task::CRCCU),
            _ => Err(()),
        }
    }
}

impl Into<usize> for Task {
    fn into(self) -> usize {
        self as usize
    }
}

impl PeripheralTask for Task {}
