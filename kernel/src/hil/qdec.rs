//! Interface for a Qdec compatible chip
//!
//! This trait provides a stanfard interface for chips with a
//! quadrature decoder. Note this interface is experimental and
//! may need further updates once implemented on additional chips

use crate::returncode::ReturnCode;

pub trait QdecDriver {
    /// Sets the client which will receive interrupts
    fn set_client(&self, client: &'static dyn QdecClient);

    /// Enables the SAMPLERDY interrupt
    fn enable_interrupts(&self) -> ReturnCode;

    /// Enables the Qdec, returning error if QDEC is not working
    fn enable_qdec(&self) -> ReturnCode;

    /// Checks if the qdec has been enabled
    fn enabled(&self) -> ReturnCode;

    /// Reads the accumulator value and resets it
    /// Note accumulator means the measure of how many ticks the
    /// QDEC has moved since the last time the function was called
    fn get_acc(&self) -> i32;
}

pub trait QdecClient {
    /// Indicate to the client that the status of the accumulator has changed
    fn sample_ready(&self);
    /// Indicate to the client that an overflow has occurred
    fn overflow(&self);
}
