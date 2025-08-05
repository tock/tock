// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Support for the VirtIO Input Device
//!
//! <https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html#x1-3850008>
//!
//! This implementation assumes the input device is a keyboard.

use kernel::utilities::cells::OptionalCell;

use crate::devices::{VirtIODeviceDriver, VirtIODeviceType};
use crate::queues::split_queue::{SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer};

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;

pub struct VirtIOInput<'a> {
    eventq: &'a SplitVirtqueue<'static, 'static, 3>,
    statusq: &'a SplitVirtqueue<'static, 'static, 1>,
    status_buffer: OptionalCell<&'static mut [u8]>,

    keys: [OptionalCell<u16>; 2],

    client: OptionalCell<&'a dyn kernel::hil::keyboard::Client>,
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
        if queue_number == self.eventq.queue_number().unwrap() {
            // Received an input device event
            kernel::debug!("bcr input event");

            // Process the incoming key. If this is the SYN_REPORT then our key
            // press is finished and we can call the client.
            let end = if let Some(event_buffer) = &buffer_chain[0] {
                let event_type = u16::from_le_bytes([event_buffer.buf[0], event_buffer.buf[1]]);
                let event_code = u16::from_le_bytes([event_buffer.buf[2], event_buffer.buf[3]]);
                let event_value = u32::from_le_bytes([
                    event_buffer.buf[4],
                    event_buffer.buf[5],
                    event_buffer.buf[6],
                    event_buffer.buf[7],
                ]);

                kernel::debug!(
                    "VirtIO Input Event: t:{}, c:{}, v:{}",
                    event_type,
                    event_code,
                    event_value
                );

                if event_type == EV_KEY && event_value == 1 {
                    // This is a key down press. Save in the next available
                    // slot.
                    if self.keys[0].is_none() {
                        self.keys[0].set(event_code);
                    } else {
                        if self.keys[1].is_none() {
                            self.keys[1].set(event_code);
                        }
                    }
                }

                // If this is a SYN_REPORT return true
                event_type == EV_SYN && event_code == 0 && event_value == 0
            } else {
                false
            };

            self.eventq.provide_buffer_chain(buffer_chain).unwrap();

            if end {
                // Signal to client that we got key presses.
                let mut key_presses: [u16; 2] = [0; 2];
                let mut length = 0;
                self.keys[0].take().map(|key| {
                    key_presses[length] = key;
                    length += 1;
                });
                self.keys[1].take().map(|key| {
                    key_presses[length] = key;
                    length += 1;
                });
                kernel::debug!("KEYS PRESSED {:?}", key_presses);
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
    fn set_client(&self, client: &'a dyn kernel::hil::keyboard::Client) {
        self.client.set(client);
    }
}
