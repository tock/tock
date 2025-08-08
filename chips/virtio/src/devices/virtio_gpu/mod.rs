// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::cell::Cell;
use core::ops::Range;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::screen::{Screen, ScreenClient, ScreenPixelFormat, ScreenRotation};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

use super::super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::super::queues::split_queue::{SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer};

mod deferred_call;
mod helpers;
mod messages;

use messages::{
    ctrl_header::CtrlHeader,
    resource_attach_backing::{MemEntry, ResourceAttachBackingReq, ResourceAttachBackingResp},
    resource_create_2d::{ResourceCreate2DReq, ResourceCreate2DResp, VideoFormat},
    resource_detach_backing::{ResourceDetachBackingReq, ResourceDetachBackingResp},
    resource_flush::{ResourceFlushReq, ResourceFlushResp},
    set_scanout::{SetScanoutReq, SetScanoutResp},
    transfer_to_host_2d::{TransferToHost2DReq, TransferToHost2DResp},
    Rect, VirtIOGPUReq, VirtIOGPUResp,
};

/// The total number of bytes occupied by a pixel in memory.
///
/// All [`VideoFormat`]s supported by VirtIO have a pixel stride of 4.
pub const PIXEL_STRIDE: usize = 4;

/// How many individual memory regions a backing buffer for a resouce
/// can be split over.
///
/// This constant is used in calculating the maxium size of a
/// [`ResourceAttachBackingReq`], which in turn is used to calculate
/// the overall maximum request size issued by the [`VirtIOGPU`]
/// driver.
pub const MAX_ATTACH_BACKING_REQ_MEMORY_ENTRIES: usize = 1;

/// Maximum size of any single request issued by the [`VirtIOGPU`]
/// driver.
pub const MAX_REQ_SIZE: usize = helpers::max(&[
    ResourceCreate2DReq::ENCODED_SIZE,
    ResourceAttachBackingReq::<{ MAX_ATTACH_BACKING_REQ_MEMORY_ENTRIES }>::ENCODED_SIZE,
    SetScanoutReq::ENCODED_SIZE,
    TransferToHost2DReq::ENCODED_SIZE,
    ResourceFlushReq::ENCODED_SIZE,
    ResourceDetachBackingReq::ENCODED_SIZE,
]);

/// Maximum size of any single response returned by the device to the
/// [`VirtIOGPU`] driver.
pub const MAX_RESP_SIZE: usize = helpers::max(&[
    ResourceCreate2DResp::ENCODED_SIZE,
    ResourceAttachBackingResp::ENCODED_SIZE,
    SetScanoutResp::ENCODED_SIZE,
    ResourceFlushResp::ENCODED_SIZE,
    ResourceDetachBackingResp::ENCODED_SIZE,
]);

/// State machine states for the [`VirtIOGPU`] driver.
#[derive(Copy, Clone, Debug)]
pub enum VirtIOGPUState {
    Uninitialized,
    InitializingResourceCreate2D,
    InitializingResourceAttachBacking,
    InitializingSetScanout,
    InitializingResourceDetachBacking,
    Idle,
    SettingWriteFrame,
    DrawResourceAttachBacking,
    DrawTransferToHost2D,
    DrawResourceFlush,
    DrawResourceDetachBacking,
}

/// Driver for a VirtIO `GPUDevice`-class device.
///
/// Implements Tock's `Screen` HIL, and supports a single head with
/// the `ARGB_8888` pixel mode.
pub struct VirtIOGPU<'a, 'b> {
    // Misc driver state:
    client: OptionalCell<&'a dyn ScreenClient>,
    state: Cell<VirtIOGPUState>,
    deferred_call: DeferredCall,
    pending_deferred_call_mask: deferred_call::PendingDeferredCallMask,

    // VirtIO bus and buffers:
    control_queue: &'a SplitVirtqueue<'a, 'b, 2>,
    req_resp_buffers: OptionalCell<(&'b mut [u8; MAX_REQ_SIZE], &'b mut [u8; MAX_RESP_SIZE])>,

    // Video output parameters:
    width: u32,
    height: u32,

    // Set up by `Screen::set_write_frame`, and then later written to with a
    // call to `Screen::write`. It contains the `Rect` being written to, and the
    // current write offset in (x, y) coordinates:
    current_draw_area: Cell<(
        // Draw area:
        Rect,
        // Current draw offset, relative to the draw area itself:
        (u32, u32),
        // Optimization -- count the number of pixels remaining undrawn:
        usize,
    )>,

    // The client provides us a subslice, but we need to place a `&'static mut`
    // buffer into the VirtQueue. We store the client's bounds here. We can't
    // use a `Range<usize>` as it isn't `Copy`, and so have to store
    // `rnage.start` and `range.end` instead.
    write_buffer_subslice_range: Cell<(usize, usize)>,

    // We can only draw rectangles, but the client can ask us to do arbitrarily
    // sized partial writes. This means that sometimes we might need to perform
    // multiple writes in response to a single client request. This stores the
    // offset into the client's buffer we've processed so far:
    write_buffer_offset: Cell<usize>,

    // Slot for the client's write buffer, while it's attached to the GPU:
    write_buffer: TakeCell<'static, [u8]>,

    // Current rect being transfered to the host:
    current_transfer_area_pixels: Cell<(Rect, usize)>,
}

impl<'a, 'b> VirtIOGPU<'a, 'b> {
    pub fn new(
        control_queue: &'a SplitVirtqueue<'a, 'b, 2>,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
        width: usize,
        height: usize,
    ) -> Result<VirtIOGPU<'a, 'b>, ErrorCode> {
        let width: u32 = width.try_into().map_err(|_| ErrorCode::SIZE)?;
        let height: u32 = height.try_into().map_err(|_| ErrorCode::SIZE)?;

        Ok(VirtIOGPU {
            client: OptionalCell::empty(),
            state: Cell::new(VirtIOGPUState::Uninitialized),
            deferred_call: DeferredCall::new(),
            pending_deferred_call_mask: deferred_call::PendingDeferredCallMask::new(),

            control_queue,
            req_resp_buffers: OptionalCell::new((req_buffer, resp_buffer)),

            width,
            height,

            current_draw_area: Cell::new((Rect::empty(), (0, 0), 0)),
            write_buffer_subslice_range: Cell::new((0, 0)),
            write_buffer_offset: Cell::new(0),
            write_buffer: TakeCell::empty(),
            current_transfer_area_pixels: Cell::new((Rect::empty(), 0)),
        })
    }

    pub fn initialize(&self) -> Result<(), ErrorCode> {
        // We can't double-initialize this device:
        let VirtIOGPUState::Uninitialized = self.state.get() else {
            return Err(ErrorCode::ALREADY);
        };

        // Enable callbacks for used descriptors:
        self.control_queue.enable_used_callbacks();

        // Take the request and response buffers. They must be available during
        // initialization:
        let (req_buffer, resp_buffer) = self.req_resp_buffers.take().unwrap();

        // Step 1: Create host resource
        let cmd_resource_create_2d_req = ResourceCreate2DReq {
            ctrl_header: CtrlHeader {
                ctrl_type: ResourceCreate2DReq::CTRL_TYPE,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id: 1,
            format: VideoFormat::A8R8G8B8Unorm,
            width: self.width,
            height: self.height,
        };
        cmd_resource_create_2d_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: ResourceCreate2DReq::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: ResourceCreate2DResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state.set(VirtIOGPUState::InitializingResourceCreate2D);

        Ok(())
    }

    fn initialize_resource_create_2d_resp(
        &self,
        _resp: ResourceCreate2DResp,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // Step 2: Attach backing memory (our framebuffer)

        // At first, we attach a zero-sized dummy buffer:
        const ENTRIES: usize = 1;
        let cmd_resource_attach_backing_req: ResourceAttachBackingReq<{ ENTRIES }> =
            ResourceAttachBackingReq {
                ctrl_header: CtrlHeader {
                    ctrl_type: ResourceAttachBackingReq::<{ ENTRIES }>::CTRL_TYPE,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                resource_id: 1,
                nr_entries: ENTRIES as u32,
                entries: [MemEntry {
                    // TODO: use dummy buffer!
                    addr: 1,
                    length: 1,
                    padding: 0,
                }],
            };
        cmd_resource_attach_backing_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: ResourceAttachBackingReq::<{ ENTRIES }>::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: ResourceAttachBackingResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state
            .set(VirtIOGPUState::InitializingResourceAttachBacking);
    }

    fn initialize_resource_attach_backing_resp(
        &self,
        _resp: ResourceAttachBackingResp,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // Step 3: Set scanout
        let cmd_set_scanout_req = SetScanoutReq {
            ctrl_header: CtrlHeader {
                ctrl_type: SetScanoutReq::CTRL_TYPE,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: self.height,
            },
            scanout_id: 0,
            resource_id: 1,
        };
        cmd_set_scanout_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: SetScanoutReq::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: SetScanoutResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state.set(VirtIOGPUState::InitializingSetScanout);
    }

    fn initialize_set_scanout_resp(
        &self,
        _resp: SetScanoutResp,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // Step 4: Detach resource
        let cmd_resource_detach_backing_req = ResourceDetachBackingReq {
            ctrl_header: CtrlHeader {
                ctrl_type: ResourceDetachBackingReq::CTRL_TYPE,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id: 1,
            padding: 0,
        };
        cmd_resource_detach_backing_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: ResourceDetachBackingReq::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: ResourceDetachBackingResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state
            .set(VirtIOGPUState::InitializingResourceDetachBacking);
    }

    fn initialize_resource_detach_backing_resp(
        &self,
        _resp: ResourceDetachBackingResp,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // Initialization done! Return the buffers:
        self.req_resp_buffers.replace((req_buffer, resp_buffer));

        // Set the device state:
        self.state.set(VirtIOGPUState::Idle);

        // Then issue the appropriate callback:
        self.client.map(|c| c.screen_is_ready());
    }

    fn continue_draw_transfer_to_host_2d(
        &self,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // Now, the `TRANSFER_TO_HOST_2D` command can only copy rectangles.
        // However, when we performed a partial write (let's say of just one
        // pixel), then the current x offset will not perfectly line up with the
        // left boundary of the overall draw rectangle. Similarly, when the
        // buffer doesn't perfectly fill up the last row of pixels, we can't
        // draw them together with the previous rows of the rectangle. Thus, a
        // single `write` call may result in at most three underlying
        // `TRANSFER_TO_HOST_2D` commands.
        //
        // At this stage, we have the `write_buffer_subslice_range` set to the
        // client's range, `write_buffer_offset` contains the offset into this
        // subslice range that we've already drawn, and `current_draw_area` has
        // the correct offset into the rectangle on the host.
        let (draw_rect, current_draw_offset, remaining_pixels) = self.current_draw_area.get();
        let (write_buffer_subslice_range_start, write_buffer_subslice_range_end) =
            self.write_buffer_subslice_range.get();
        let write_buffer_subslice_range = Range {
            start: write_buffer_subslice_range_start,
            end: write_buffer_subslice_range_end,
        };
        let write_buffer_offset = self.write_buffer_offset.get();

        // Compute the remaining bytes left in the client-supplied buffer:
        let write_buffer_remaining_bytes = write_buffer_subslice_range
            .len()
            .checked_sub(write_buffer_offset)
            .unwrap();
        assert!(write_buffer_remaining_bytes % PIXEL_STRIDE == 0);
        let write_buffer_remaining_pixels = write_buffer_remaining_bytes / PIXEL_STRIDE;
        assert!(write_buffer_remaining_pixels <= remaining_pixels);

        // Check whether the current draw offset within the rectangle has an `x`
        // coordinate of zero. That means we can copy one or more full rows, or
        // the last partial row of the draw area:
        let transfer_pixels = if draw_rect.is_empty() {
            // Short-circuit an empty draw_rect, to avoid divide by zero
            // areas when using `rect.width` or `rect.height` as a divisor:
            0
        } else if current_draw_offset.0 == 0 {
            // Okay, we can start drawing the full rectangle. We want to try
            // drawing any full rows, if there are any left, and if not the
            // last partial row:
            assert!(current_draw_offset.1 <= draw_rect.height || remaining_pixels == 0);
            if current_draw_offset.1 >= draw_rect.height {
                // Just one row left to draw, and we start from `x ==
                // 0`. This means we can just copy however much more data
                // the client buffer holds. We've previously checked that
                // the client buffer fully fits into the draw area, but
                // re-check that assertion here:
                assert!(draw_rect.width as usize >= write_buffer_remaining_pixels);
                write_buffer_remaining_pixels
            } else {
                // There is more than one row left to copy, and we start
                // from `x == 0`. If the client buffer lines up with the end
                // of a row, we can copy them as a single
                // rectangle. Otherwise, we need two copies:
                write_buffer_remaining_pixels / (draw_rect.width as usize)
                    * (draw_rect.width as usize)
            }
        } else {
            // Our current draw offset is not zero. This means we must copy
            // the current row, and then potentially any subsequent
            // rows. Determine how much to copy based on the lower of the
            // remaining data in the slice, or the remaining row width:
            let remaining_row_width = draw_rect.width.checked_sub(current_draw_offset.0).unwrap();
            core::cmp::min(remaining_row_width as usize, write_buffer_remaining_pixels)
        };

        // If we've got nothing left to copy, great! We're done drawing, but
        // still need to detach the resource:
        if transfer_pixels == 0 {
            let cmd_resource_detach_backing_req = ResourceDetachBackingReq {
                ctrl_header: CtrlHeader {
                    ctrl_type: ResourceDetachBackingReq::CTRL_TYPE,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                resource_id: 1,
                padding: 0,
            };
            cmd_resource_detach_backing_req
                .write_to_byte_iter(&mut req_buffer.iter_mut())
                .unwrap();

            let mut buffer_chain = [
                Some(VirtqueueBuffer {
                    buf: req_buffer,
                    len: ResourceDetachBackingReq::ENCODED_SIZE,
                    device_writeable: false,
                }),
                Some(VirtqueueBuffer {
                    buf: resp_buffer,
                    len: ResourceDetachBackingResp::ENCODED_SIZE,
                    device_writeable: true,
                }),
            ];
            self.control_queue
                .provide_buffer_chain(&mut buffer_chain)
                .unwrap();

            self.state.set(VirtIOGPUState::DrawResourceDetachBacking);

            return;
        }

        // Otherwise, build the transfer rect from `transfer_pixels`,
        // `draw_rect` and the current draw offset:
        let transfer_rect = Rect {
            x: draw_rect.x.checked_add(current_draw_offset.0).unwrap(),
            y: draw_rect.y.checked_add(current_draw_offset.1).unwrap(),
            width: core::cmp::min(transfer_pixels, draw_rect.width as usize) as u32,
            height: transfer_pixels.div_ceil(draw_rect.width as usize) as u32,
        };
        self.current_transfer_area_pixels
            .set((transfer_rect, transfer_pixels));

        // Attach write buffer
        let cmd_transfer_to_host_2d_req = TransferToHost2DReq {
            ctrl_header: CtrlHeader {
                ctrl_type: TransferToHost2DReq::CTRL_TYPE,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: transfer_rect,
            offset: write_buffer_offset as u64,
            resource_id: 1,
            padding: 0,
        };
        // kernel::debug!(
        //     "Transfer to host {:?}, {:?}",
        //     transfer_rect,
        //     write_buffer_offset
        // );
        cmd_transfer_to_host_2d_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: TransferToHost2DReq::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: TransferToHost2DResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state.set(VirtIOGPUState::DrawTransferToHost2D);
    }

    fn continue_draw_resource_flush(
        &self,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        let (current_transfer_area, _) = self.current_transfer_area_pixels.get();

        let cmd_resource_flush_req = ResourceFlushReq {
            ctrl_header: CtrlHeader {
                ctrl_type: ResourceFlushReq::CTRL_TYPE,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: current_transfer_area,
            resource_id: 1,
            padding: 0,
        };
        cmd_resource_flush_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: ResourceFlushReq::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: ResourceFlushResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state.set(VirtIOGPUState::DrawResourceFlush);
    }

    fn continue_draw_resource_flushed(
        &self,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        // We've finished one write command, but there might be more to
        // come. Increment `current_draw_offset` and `write_buffer_offset`, and
        // decrement `remaining_pixels` accordingly.
        let (draw_rect, mut current_draw_offset, mut remaining_pixels) =
            self.current_draw_area.get();
        let mut write_buffer_offset = self.write_buffer_offset.get();

        // This is what we've just drawn:
        let (drawn_area, drawn_pixels) = self.current_transfer_area_pixels.get();

        // We always draw left -> right, top -> bottom, so we can simply set the
        // current `x` and `y` coordinates to the bottom-right most coordinates
        // we've just drawn (while wrapping and carrying the one):
        current_draw_offset.0 = drawn_area
            .x
            .checked_add(drawn_area.width)
            .and_then(|drawn_x1| drawn_x1.checked_sub(draw_rect.x))
            .unwrap();
        current_draw_offset.1 = drawn_area
            .y
            .checked_add(drawn_area.height)
            .and_then(|drawn_y1| drawn_y1.checked_sub(draw_rect.y))
            .unwrap();

        // Wrap to the next line when we've finished writing the column of our
        // last row drawn:
        assert!(current_draw_offset.0 <= draw_rect.width);
        if current_draw_offset.0 == draw_rect.width {
            current_draw_offset.0 = 0;
            current_draw_offset.1 = current_draw_offset.1.checked_add(1).unwrap();
        }

        // Subtract our drawn_pixels from `remaining_pixels`:
        assert!(remaining_pixels >= drawn_pixels);
        remaining_pixels -= drawn_pixels;

        // Add our drawn pixels * PIXEL_STRIDE to the buffer offset:
        write_buffer_offset += drawn_pixels.checked_mul(PIXEL_STRIDE).unwrap();

        // Write all of this back:
        self.current_draw_area
            .set((draw_rect, current_draw_offset, remaining_pixels));
        self.write_buffer_offset.set(write_buffer_offset);

        // And continue drawing:
        self.continue_draw_transfer_to_host_2d(req_buffer, resp_buffer);
    }

    fn continue_draw_resource_detached_backing(
        &self,
        req_buffer: &'b mut [u8; MAX_REQ_SIZE],
        resp_buffer: &'b mut [u8; MAX_RESP_SIZE],
    ) {
        self.req_resp_buffers.replace((req_buffer, resp_buffer));
        self.state.set(VirtIOGPUState::Idle);

        let (write_buffer_subslice_range_start, write_buffer_subslice_range_end) =
            self.write_buffer_subslice_range.get();
        let write_buffer_subslice_range = Range {
            start: write_buffer_subslice_range_start,
            end: write_buffer_subslice_range_end,
        };

        let mut subslice = SubSliceMut::new(self.write_buffer.take().unwrap());
        subslice.slice(write_buffer_subslice_range);

        self.client.map(|c| c.write_complete(subslice, Ok(())));
    }

    fn buffer_chain_callback(
        &self,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
        _bytes_used: usize,
    ) {
        // Every response should return exactly two buffers: one
        // request buffer, and one response buffer.
        let req_buffer = buffer_chain
            .get_mut(0)
            .and_then(|opt_buf| opt_buf.take())
            .expect("Missing request buffer in VirtIO GPU buffer chain");
        let resp_buffer = buffer_chain
            .get_mut(1)
            .and_then(|opt_buf| opt_buf.take())
            .expect("Missing request buffer in VirtIO GPU buffer chain");

        // Convert the buffer slices back into arrays:
        let req_array: &mut [u8; MAX_REQ_SIZE] = req_buffer
            .buf
            .try_into()
            .expect("Returned VirtIO GPU request buffer has unexpected size!");

        let resp_length = resp_buffer.len;
        let resp_array: &mut [u8; MAX_RESP_SIZE] = resp_buffer
            .buf
            .try_into()
            .expect("Returned VirtIO GPU response buffer has unexpected size!");

        // Check that the response has a length we can parse into a CtrlHeader:
        if resp_length < CtrlHeader::ENCODED_SIZE {
            panic!(
                "VirtIO GPU returned response smaller than the CtrlHeader, \
                 which we cannot parse! Returned bytes: {}",
                resp_length
            )
        }

        // We progressively parse the response, starting with the CtrlHeader
        // shared across all messages, checking its type, and then parsing the
        // rest. We do so by reusing a common iterator across these operations:
        let mut resp_iter = resp_array.iter().copied();
        let ctrl_header = CtrlHeader::from_byte_iter(&mut resp_iter)
            .expect("Failed to parse VirtIO response CtrlHeader");

        // We now match the current device state with the ctrl_type
        // that was returned to continue parsing:
        match (self.state.get(), ctrl_header.ctrl_type) {
            (
                VirtIOGPUState::InitializingResourceCreate2D,
                ResourceCreate2DResp::EXPECTED_CTRL_TYPE,
            ) => {
                // Parse the remainder of the response:
                let resp = ResourceCreate2DResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU ResourceCreate2DResp");

                // Continue the initialization routine:
                self.initialize_resource_create_2d_resp(resp, req_array, resp_array);
            }

            (
                VirtIOGPUState::InitializingResourceAttachBacking,
                ResourceAttachBackingResp::EXPECTED_CTRL_TYPE,
            ) => {
                // Parse the remainder of the response:
                let resp = ResourceAttachBackingResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU ResourceAttachBackingResp");

                // Continue the initialization routine:
                self.initialize_resource_attach_backing_resp(resp, req_array, resp_array);
            }

            (VirtIOGPUState::InitializingSetScanout, SetScanoutResp::EXPECTED_CTRL_TYPE) => {
                // Parse the remainder of the response:
                let resp =
                    SetScanoutResp::from_byte_iter_post_ctrl_header(ctrl_header, &mut resp_iter)
                        .expect("Failed to parse VirtIO GPU SetScanoutResp");

                // Continue the initialization routine:
                self.initialize_set_scanout_resp(resp, req_array, resp_array);
            }

            (
                VirtIOGPUState::InitializingResourceDetachBacking,
                ResourceDetachBackingResp::EXPECTED_CTRL_TYPE,
            ) => {
                // Parse the remainder of the response:
                let resp = ResourceDetachBackingResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU ResourceDetachBackingResp");

                // Continue the initialization routine:
                self.initialize_resource_detach_backing_resp(resp, req_array, resp_array);
            }

            (
                VirtIOGPUState::DrawResourceAttachBacking,
                ResourceAttachBackingResp::EXPECTED_CTRL_TYPE,
            ) => {
                // Parse the remainder of the response:
                let _resp = ResourceAttachBackingResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU ResourceAttachBackingResp");

                // Continue the initialization routine:
                self.continue_draw_transfer_to_host_2d(req_array, resp_array);
            }

            (VirtIOGPUState::DrawTransferToHost2D, TransferToHost2DResp::EXPECTED_CTRL_TYPE) => {
                // Parse the remainder of the response:
                let _resp = TransferToHost2DResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU TransferToHost2DResp");

                // Continue the initialization routine:
                self.continue_draw_resource_flush(req_array, resp_array);
            }

            (VirtIOGPUState::DrawResourceFlush, ResourceFlushResp::EXPECTED_CTRL_TYPE) => {
                // Parse the remainder of the response:
                let _resp =
                    ResourceFlushResp::from_byte_iter_post_ctrl_header(ctrl_header, &mut resp_iter)
                        .expect("Failed to parse VirtIO GPU ResourceFlushResp");

                // Continue the initialization routine:
                self.continue_draw_resource_flushed(req_array, resp_array);
            }

            (
                VirtIOGPUState::DrawResourceDetachBacking,
                ResourceDetachBackingResp::EXPECTED_CTRL_TYPE,
            ) => {
                // Parse the remainder of the response:
                let _resp = ResourceDetachBackingResp::from_byte_iter_post_ctrl_header(
                    ctrl_header,
                    &mut resp_iter,
                )
                .expect("Failed to parse VirtIO GPU ResourceDetachBackingResp");

                // Continue the initialization routine:
                self.continue_draw_resource_detached_backing(req_array, resp_array);
            }

            (VirtIOGPUState::Uninitialized, _)
            | (VirtIOGPUState::InitializingResourceCreate2D, _)
            | (VirtIOGPUState::InitializingResourceAttachBacking, _)
            | (VirtIOGPUState::InitializingSetScanout, _)
            | (VirtIOGPUState::InitializingResourceDetachBacking, _)
            | (VirtIOGPUState::Idle, _)
            | (VirtIOGPUState::SettingWriteFrame, _)
            | (VirtIOGPUState::DrawResourceAttachBacking, _)
            | (VirtIOGPUState::DrawTransferToHost2D, _)
            | (VirtIOGPUState::DrawResourceFlush, _)
            | (VirtIOGPUState::DrawResourceDetachBacking, _) => {
                panic!(
                    "Received unexpected VirtIO GPU device response. Device \
                     state: {:?}, ctrl hader: {:?}",
                    self.state.get(),
                    ctrl_header
                );
            }
        }
    }
}

impl<'a> Screen<'a> for VirtIOGPU<'a, '_> {
    fn set_client(&self, client: &'a dyn ScreenClient) {
        self.client.replace(client);
    }

    fn get_resolution(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::ARGB_8888
    }

    fn get_rotation(&self) -> ScreenRotation {
        ScreenRotation::Normal
    }

    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode> {
        // Make sure we're idle:
        let VirtIOGPUState::Idle = self.state.get() else {
            return Err(ErrorCode::BUSY);
        };

        // We first convert the coordinates to u32s:
        let x: u32 = x.try_into().map_err(|_| ErrorCode::INVAL)?;
        let y: u32 = y.try_into().map_err(|_| ErrorCode::INVAL)?;
        let width: u32 = width.try_into().map_err(|_| ErrorCode::INVAL)?;
        let height: u32 = height.try_into().map_err(|_| ErrorCode::INVAL)?;

        // Ensure that the draw area actually fits our screen:
        let x1 = x.checked_add(width).ok_or(ErrorCode::INVAL)?;
        let y1 = y.checked_add(height).ok_or(ErrorCode::INVAL)?;
        if x1 > self.width || y1 > self.height {
            return Err(ErrorCode::INVAL);
        }

        // Store the new drawing area as the bounding box and offset coordinates
        // for `write`:
        self.current_draw_area.set((
            // Draw area:
            Rect {
                x,
                y,
                width,
                height,
            },
            // Current draw offset, relative to the draw area itself:
            (0, 0),
            // Precompute the number of pixels in this draw area:
            (width as usize)
                .checked_mul(height as usize)
                .ok_or(ErrorCode::INVAL)?,
        ));

        // Set the device state to busy and issue the callback in a deferred
        // call:
        self.state.set(VirtIOGPUState::SettingWriteFrame);
        self.pending_deferred_call_mask
            .set(deferred_call::PendingDeferredCall::SetWriteFrame);
        self.deferred_call.set();

        Ok(())
    }

    fn write(
        &self,
        buffer: SubSliceMut<'static, u8>,
        continue_write: bool,
    ) -> Result<(), ErrorCode> {
        // Make sure we're idle:
        let VirtIOGPUState::Idle = self.state.get() else {
            return Err(ErrorCode::BUSY);
        };

        // If `continue_write` is false, we must reset `x_off` and
        // `y_off`. Otherwise we start at the stored offset.
        let (draw_rect, mut current_draw_offset, mut remaining_pixels) =
            self.current_draw_area.get();
        if !continue_write {
            current_draw_offset = (0, 0);
            // This multiplication must not overflow, as we've already performed
            // it before in `set_write_area`:
            remaining_pixels = (draw_rect.width as usize)
                .checked_mul(draw_rect.height as usize)
                .unwrap();
        }
        self.current_draw_area
            .set((draw_rect, current_draw_offset, remaining_pixels));

        // Ensure that this buffer is evenly divisible by PIXEL_STRIDE and that
        // it can fit into the remaining part of the draw area:
        if buffer.len() % PIXEL_STRIDE != 0 {
            return Err(ErrorCode::INVAL);
        }
        if buffer.len() / PIXEL_STRIDE > remaining_pixels {
            return Err(ErrorCode::SIZE);
        }

        // Now, the `TRANSFER_TO_HOST_2D` command can only copy rectangles.
        // However, when we performed a partial write (let's say of just one
        // pixel), then the current x offset will not perfectly line up with the
        // left boundary of the overall draw rectangle. Similarly, when the
        // buffer doesn't perfectly fill up the last row of pixels, we can't
        // draw them together with the previous rows of the rectangle. Thus, a
        // single `write` call may result in at most three underlying
        // `TRANSFER_TO_HOST_2D` commands.
        //
        // We use a common subroutine to identify the next data to copy. We
        // first store the overall subslice active range, and the offset in this
        // subslice (0 right now!), and then let that subroutine handle the rest:
        let write_buffer_subslice_range = buffer.active_range();
        self.write_buffer_subslice_range.set((
            write_buffer_subslice_range.start,
            write_buffer_subslice_range.end,
        ));
        self.write_buffer_offset.set(0);

        let (req_buffer, resp_buffer) = self.req_resp_buffers.take().unwrap();

        // Now, attach the user-supplied buffer to this device:
        let buffer_slice = buffer.take();

        const ENTRIES: usize = 1;
        let cmd_resource_attach_backing_req: ResourceAttachBackingReq<{ ENTRIES }> =
            ResourceAttachBackingReq {
                ctrl_header: CtrlHeader {
                    ctrl_type: ResourceAttachBackingReq::<{ ENTRIES }>::CTRL_TYPE,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                resource_id: 1,
                nr_entries: ENTRIES as u32,
                entries: [MemEntry {
                    addr: buffer_slice.as_ptr() as u64 + write_buffer_subslice_range.start as u64,
                    length: write_buffer_subslice_range.len() as u32,
                    padding: 0,
                }],
            };
        cmd_resource_attach_backing_req
            .write_to_byte_iter(&mut req_buffer.iter_mut())
            .unwrap();

        assert!(self.write_buffer.replace(buffer_slice).is_none());

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: req_buffer,
                len: ResourceAttachBackingReq::<{ ENTRIES }>::ENCODED_SIZE,
                device_writeable: false,
            }),
            Some(VirtqueueBuffer {
                buf: resp_buffer,
                len: ResourceAttachBackingResp::ENCODED_SIZE,
                device_writeable: true,
            }),
        ];
        self.control_queue
            .provide_buffer_chain(&mut buffer_chain)
            .unwrap();

        self.state.set(VirtIOGPUState::DrawResourceAttachBacking);

        Ok(())
    }

    fn set_brightness(&self, _brightness: u16) -> Result<(), ErrorCode> {
        // nop, not supported
        Ok(())
    }

    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode> {
        if !enabled {
            Err(ErrorCode::INVAL)
        } else {
            Ok(())
        }
    }

    fn set_invert(&self, _enabled: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'b> SplitVirtqueueClient<'b> for VirtIOGPU<'_, 'b> {
    fn buffer_chain_ready(
        &self,
        _queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
        bytes_used: usize,
    ) {
        self.buffer_chain_callback(buffer_chain, bytes_used)
    }
}

impl DeferredCallClient for VirtIOGPU<'_, '_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        let calls = self.pending_deferred_call_mask.get_copy_and_clear();
        calls.for_each_call(|call| match call {
            deferred_call::PendingDeferredCall::SetWriteFrame => {
                let VirtIOGPUState::SettingWriteFrame = self.state.get() else {
                    panic!(
                        "Unexpected VirtIOGPUState {:?} for SetWriteFrame \
                         deferred call",
                        self.state.get()
                    );
                };

                // Set the device staste back to idle:
                self.state.set(VirtIOGPUState::Idle);

                // Issue callback:
                self.client.map(|c| c.command_complete(Ok(())));
            }
        })
    }
}

impl VirtIODeviceDriver for VirtIOGPU<'_, '_> {
    fn negotiate_features(&self, _offered_features: u64) -> Option<u64> {
        // We don't support any special features and do not care about
        // what the device offers.
        Some(0)
    }

    fn device_type(&self) -> VirtIODeviceType {
        VirtIODeviceType::GPUDevice
    }
}
