//! LiteX event manager (IRQ and CSR management interface for
//! controlling interrupts of cores)
//!
//! Documentation in `litex/soc/interconnect/csr_eventmanager.py`.

use crate::litex_registers::{IntLike, Read, ReadWrite};
use core::marker::PhantomData;

pub struct LiteXEventManager<'a, T, S, P, E>
where
    T: IntLike,
    S: Read<T>,
    P: ReadWrite<T>,
    E: ReadWrite<T>,
{
    status: &'a S,
    pending: &'a P,
    enable: &'a E,
    _base_type: PhantomData<T>,
}

impl<'a, T, S, P, E> LiteXEventManager<'a, T, S, P, E>
where
    T: IntLike,
    S: Read<T>,
    P: ReadWrite<T>,
    E: ReadWrite<T>,
{
    pub const fn new(status: &'a S, pending: &'a P, enable: &'a E) -> Self {
        LiteXEventManager {
            status,
            pending,
            enable,
            _base_type: PhantomData,
        }
    }

    pub fn disable_all(&self) {
        self.enable.set(T::zero());
    }

    pub fn enable_all(&self) {
        self.enable.set(T::max());
    }

    pub fn enable_event(&self, index: usize) {
        self.enable.set(self.enable.get() | (T::one() << index));
    }

    pub fn disable_event(&self, index: usize) {
        self.enable.set(self.enable.get() & !(T::one() << index));
    }

    pub fn event_enabled(&self, index: usize) -> bool {
        self.enable.get() & (T::one() << index) != T::zero()
    }

    pub fn events_enabled(&self) -> T {
        self.enable.get()
    }

    pub fn event_source_asserted(&self, index: usize) -> bool {
        self.status.get() & (T::one() << index) != T::zero()
    }

    pub fn any_event_pending(&self) -> bool {
        self.pending.get() != T::zero()
    }

    pub fn event_pending(&self, index: usize) -> bool {
        self.pending.get() & (T::one() << index) != T::zero()
    }

    pub fn events_pending(&self) -> T {
        self.pending.get()
    }

    pub fn clear_event(&self, index: usize) {
        self.pending.set(T::one() << index);
    }

    pub fn clear_all(&self) {
        self.pending.set(T::max());
    }
}
