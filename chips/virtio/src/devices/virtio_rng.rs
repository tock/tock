use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::rng::{Client as RngClient, Continue as RngCont, Rng};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use super::super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::super::queues::split_queue::{SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer};

pub struct VirtIORng<'a, 'b> {
    virtqueue: &'a SplitVirtqueue<'a, 'b, 1>,
    buffer_capacity: Cell<usize>,
    callback_pending: Cell<bool>,
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn RngClient>,
}

impl<'a, 'b> VirtIORng<'a, 'b> {
    pub fn new(virtqueue: &'a SplitVirtqueue<'a, 'b, 1>) -> VirtIORng<'a, 'b> {
        VirtIORng {
            virtqueue,
            buffer_capacity: Cell::new(0),
            callback_pending: Cell::new(false),
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
        }
    }

    pub fn provide_buffer(&self, buf: &'b mut [u8]) -> Result<usize, (&'b mut [u8], ErrorCode)> {
        let len = buf.len();
        if len < 4 {
            // We don't yet support merging of randomness of multiple buffers
            //
            // Allowing a buffer with less than 4 elements will cause
            // the callback to never be called, while the buffer is
            // reinserted into the queue
            return Err((buf, ErrorCode::INVAL));
        }

        let mut buffer_chain = [Some(VirtqueueBuffer {
            buf,
            len,
            device_writeable: true,
        })];

        let res = self.virtqueue.provide_buffer_chain(&mut buffer_chain);

        match res {
            Err(ErrorCode::NOMEM) => {
                // Hand back the buffer, the queue MUST NOT write partial
                // buffer chains
                let buf = buffer_chain[0].take().unwrap().buf;
                Err((buf, ErrorCode::NOMEM))
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(()) => {
                let mut cap = self.buffer_capacity.get();
                cap += len;
                self.buffer_capacity.set(cap);
                Ok(cap)
            }
        }
    }

    fn buffer_chain_callback(
        &self,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
        bytes_used: usize,
    ) {
        // Disable further callbacks, until we're sure we need them
        //
        // The used buffers should stay in the queue until a client is
        // ready to consume them
        self.virtqueue.disable_used_callbacks();

        // We only have buffer chains of a single buffer
        let buf = buffer_chain[0].take().unwrap().buf;

        // We have taken out a buffer, hence decrease the available capacity
        assert!(self.buffer_capacity.get() >= buf.len());

        // It could've happened that we don't require the callback any
        // more, hence check beforehand
        let cont = if self.callback_pending.get() {
            // The callback is no longer pending
            self.callback_pending.set(false);

            let mut u32randiter = buf[0..bytes_used].chunks(4).filter_map(|slice| {
                if slice.len() < 4 {
                    None
                } else {
                    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
                }
            });

            // For now we don't use left-over randomness and assume the
            // client has consumed the entire iterator
            self.client
                .map(|client| client.randomness_available(&mut u32randiter, Ok(())))
                .unwrap_or(RngCont::Done)
        } else {
            RngCont::Done
        };

        if let RngCont::More = cont {
            // Returning more is the equivalent of calling .get() on
            // the Rng trait.

            // TODO: what if this call fails?
            let _ = self.get();
        }

        // In any case, reinsert the buffer for further processing
        self.provide_buffer(buf).expect("Buffer reinsertion failed");
    }
}

impl<'a, 'b> Rng<'a> for VirtIORng<'a, 'b> {
    fn get(&self) -> Result<(), ErrorCode> {
        // Minimum buffer capacity must be 4 bytes for a single 32-bit
        // word
        if self.buffer_capacity.get() < 4 {
            Err(ErrorCode::FAIL)
        } else if self.client.is_none() {
            Err(ErrorCode::FAIL)
        } else if self.callback_pending.get() {
            Err(ErrorCode::OFF)
        } else if self.virtqueue.used_descriptor_chains_count() < 1 {
            // There is no buffer ready in the queue, so let's rely
            // purely on queue callbacks to notify us of the next
            // incoming one
            self.callback_pending.set(true);
            self.virtqueue.enable_used_callbacks();
            Ok(())
        } else {
            // There is a buffer in the virtqueue, get it and return
            // it to a client in a deferred call
            self.callback_pending.set(true);
            self.deferred_call.set();
            Ok(())
        }
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        // Cancel by setting the callback_pending flag to false which
        // MUST be checked prior to every callback
        self.callback_pending.set(false);

        // For efficiency reasons, also unsubscribe from the virtqueue
        // callbacks, which will let the buffers remain in the queue
        // for future use
        self.virtqueue.disable_used_callbacks();

        Ok(())
    }

    fn set_client(&self, client: &'a dyn RngClient) {
        self.client.set(client);
    }
}

impl<'a, 'b> SplitVirtqueueClient<'b> for VirtIORng<'a, 'b> {
    fn buffer_chain_ready(
        &self,
        _queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
        bytes_used: usize,
    ) {
        self.buffer_chain_callback(buffer_chain, bytes_used)
    }
}

impl<'a, 'b> DeferredCallClient for VirtIORng<'a, 'b> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        // Try to extract a descriptor chain
        if let Some((mut chain, bytes_used)) = self.virtqueue.pop_used_buffer_chain() {
            self.buffer_chain_callback(&mut chain, bytes_used)
        } else {
            // If we don't get a buffer, this must be a race condition
            // which should not occur
            //
            // Prior to setting a deferred call, all virtqueue
            // interrupts must be disabled so that no used buffer is
            // removed before the deferred call callback
            panic!("VirtIO RNG: deferred call callback with empty queue");
        }
    }
}

impl<'a, 'b> VirtIODeviceDriver for VirtIORng<'a, 'b> {
    fn negotiate_features(&self, _offered_features: u64) -> Option<u64> {
        // We don't support any special features and do not care about
        // what the device offers.
        Some(0)
    }

    fn device_type(&self) -> VirtIODeviceType {
        VirtIODeviceType::EntropySource
    }
}
