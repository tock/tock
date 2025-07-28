
// Stub of the i8042 / PS-2 controller – compiles, but does nothing

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;

const BUFFER_SIZE: usize = 32;

/// Empty controller – IRQs, ports and clock still disabled.
pub struct Ps2Controller {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head:   Cell<usize>,
    tail:   Cell<usize>,
    _phantom: PhantomData<()>,
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head:   Cell::new(0),
            tail:   Cell::new(0),
            _phantom: PhantomData,
        }
    }

    /// No hardware access yet – later steps will fill this in.
    pub fn init(&self) { /* stub */ }

    /// Never called while IRQ 1 is still masked (next steps).
    pub fn handle_interrupt(&self) { /* stub */ }

    pub fn pop_scan_code(&self) -> Option<u8> { None }
}
