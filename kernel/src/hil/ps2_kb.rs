///Hill kb driber
use crate::ErrorCode;

pub trait KBReceiver {
    /// Attempt to pull one decoded byte (ASCII or keycode) from the keyboard.
    ///
    /// Returns `Some(u8)` on success, or `None` if no data is available.
    fn receive(&self) -> Option<u8>;
}
