// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Support for the VirtIO Input Device
//!
//! <https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html#x1-3850008>
//!
//! This implementation assumes the input device is a keyboard that follows the
//! [event code](https://www.kernel.org/doc/Documentation/input/event-codes.txt)
//! format that Linux uses.

use kernel::utilities::cells::OptionalCell;

use crate::devices::{VirtIODeviceDriver, VirtIODeviceType};
use crate::queues::split_queue::{SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer};

/// Event: separate events.
const EV_SYN: u16 = 0;
/// Event: state change of keyboard.
const EV_KEY: u16 = 1;

/// VirtIO for input devices (e.g., keyboards).
pub struct VirtIOInput<'a> {
    /// Queue of events from the device (e.g., keyboard).
    eventq: &'a SplitVirtqueue<'static, 'static, 3>,
    /// Queue of status updates from this driver (e.g., turn on LED).
    statusq: &'a SplitVirtqueue<'static, 'static, 1>,
    /// Buffer to hold status updates.
    status_buffer: OptionalCell<&'static mut [u8]>,
    /// Store keys sent across multiple events.
    keys: [OptionalCell<(u16, bool)>; 2],
    /// Keyboard callback client.
    client: OptionalCell<&'a dyn kernel::hil::keyboard::KeyboardClient>,
}

impl<'a> VirtIOInput<'a> {
    pub fn new(
        eventq: &'a SplitVirtqueue<'static, 'static, 3>,
        statusq: &'a SplitVirtqueue<'static, 'static, 1>,
        status_buffer: &'static mut [u8],
    ) -> Self {
        eventq.enable_used_callbacks();

        Self {
            eventq,
            statusq,
            status_buffer: OptionalCell::new(status_buffer),
            keys: [const { OptionalCell::empty() }; 2],
            client: OptionalCell::empty(),
        }
    }

    pub fn provide_buffers(
        &self,
        event_buffer1: &'static mut [u8],
        event_buffer2: &'static mut [u8],
        event_buffer3: &'static mut [u8],
    ) {
        // Provide the device three buffers to hold up to two keys and a sync
        // event.
        let event_buffer_len = event_buffer1.len();
        let mut buffer_chain = [Some(VirtqueueBuffer {
            buf: event_buffer1,
            len: event_buffer_len,
            device_writeable: true,
        })];
        self.eventq.provide_buffer_chain(&mut buffer_chain).unwrap();

        let event_buffer_len = event_buffer2.len();
        let mut buffer_chain = [Some(VirtqueueBuffer {
            buf: event_buffer2,
            len: event_buffer_len,
            device_writeable: true,
        })];
        self.eventq.provide_buffer_chain(&mut buffer_chain).unwrap();

        let event_buffer_len = event_buffer3.len();
        let mut buffer_chain = [Some(VirtqueueBuffer {
            buf: event_buffer3,
            len: event_buffer_len,
            device_writeable: true,
        })];
        self.eventq.provide_buffer_chain(&mut buffer_chain).unwrap();
    }
}

impl SplitVirtqueueClient<'static> for VirtIOInput<'_> {
    fn buffer_chain_ready(
        &self,
        queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueBuffer<'static>>],
        _bytes_used: usize,
    ) {
        fn parse_event(buf: &[u8]) -> Result<(u16, u16, u32), ()> {
            let event_type = u16::from_le_bytes(buf.get(0..2).ok_or(())?.try_into().or(Err(()))?);
            let event_code = u16::from_le_bytes(buf.get(2..4).ok_or(())?.try_into().or(Err(()))?);
            let event_value = u32::from_le_bytes(buf.get(4..8).ok_or(())?.try_into().or(Err(()))?);
            Ok((event_type, event_code, event_value))
        }

        if queue_number == self.eventq.queue_number().unwrap() {
            // Received an input device event

            // Process the incoming key. If this is the SYN_REPORT then our key
            // press is finished and we can call the client.
            let end = if let Some(event_buffer) = &buffer_chain[0] {
                if let Ok((event_type, event_code, event_value)) = parse_event(event_buffer.buf) {
                    if event_type == EV_KEY {
                        // This is a key down press. Save in the next available
                        // slot.
                        if self.keys[0].is_none() {
                            self.keys[0].set((event_code, event_value == 1));
                        } else {
                            if self.keys[1].is_none() {
                                self.keys[1].set((event_code, event_value == 1));
                            }
                        }
                    }

                    // If this is a SYN_REPORT return true
                    event_type == EV_SYN && event_code == 0 && event_value == 0
                } else {
                    false
                }
            } else {
                false
            };

            self.eventq.provide_buffer_chain(buffer_chain).unwrap();

            if end {
                // Signal to client that we got key presses.
                let mut key_presses: [(u16, bool); 2] = [(0, false); 2];
                let mut length = 0;
                self.keys[0].take().map(|key| {
                    key_presses[length] = key;
                    length += 1;
                });
                self.keys[1].take().map(|key| {
                    key_presses[length] = key;
                    length += 1;
                });
                self.client.map(|client| {
                    client.keys_pressed(&key_presses[0..length], Ok(()));
                });
            }
        } else if queue_number == self.statusq.queue_number().unwrap() {
            // Sent a status update

            let status_buffer = buffer_chain[0].take().expect("No status buffer").buf;

            self.status_buffer.replace(status_buffer);
        }
    }
}

impl VirtIODeviceDriver for VirtIOInput<'_> {
    fn negotiate_features(&self, _offered_features: u64) -> Option<u64> {
        // We don't support any special features and do not care about
        // what the device offers.
        Some(0)
    }

    fn device_type(&self) -> VirtIODeviceType {
        VirtIODeviceType::InputDevice
    }
}

impl<'a> kernel::hil::keyboard::Keyboard<'a> for VirtIOInput<'a> {
    fn set_client(&self, client: &'a dyn kernel::hil::keyboard::KeyboardClient) {
        self.client.set(client);
    }
}
