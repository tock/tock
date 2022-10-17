//! Definition of Deferred Call tasks.
//!
//! Deferred calls also peripheral drivers to register pseudo interrupts.
//! These are the definitions of which deferred calls this chip needs.

use core::convert::Into;
use core::convert::TryFrom;

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum DeferredCallTask {
    Fsmc = 0,
    Usart1 = 1,
    Usart2 = 2,
    Usart3 = 3,
    // The CAN peripheral is not present on all STM32F4 devices.
    // The CAN specific deferred call will be handled by the board's
    // specific InterruptService implementation.
    Can = 4,
}

impl TryFrom<usize> for DeferredCallTask {
    type Error = ();

    fn try_from(value: usize) -> Result<DeferredCallTask, ()> {
        match value {
            0 => Ok(DeferredCallTask::Fsmc),
            1 => Ok(DeferredCallTask::Usart1),
            2 => Ok(DeferredCallTask::Usart2),
            3 => Ok(DeferredCallTask::Usart3),
            4 => Ok(DeferredCallTask::Can),
            _ => Err(()),
        }
    }
}

impl Into<usize> for DeferredCallTask {
    fn into(self) -> usize {
        self as usize
    }
}
