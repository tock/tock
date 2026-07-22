// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil::screen::{Screen, ScreenClient, ScreenPixelFormat, ScreenRotation};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;

use super::utils::Frame;

/// Expose an underlying ARGB8888-formatted screen as a Mono VLSB
/// (a.k.a. `Mono_8BitPage`) screen of the same resolution.
///
/// # Mono VLSB source layout
///
/// Mono VLSB packs 8 vertically-adjacent pixels into a single byte,
/// from top to bottom, grouped into width-sized "pages", from left to
/// right. A page is a row of width `N` and height 8 pixels, formed by
/// `N` successive bytes. So, bit `k` of byte `col + (row / 8) *
/// width` is the pixel at (`col`, row-within-frame `row`).
///
/// For a write frame of height `h`, the source needs `width * ceil(h
/// / 8)` bytes; if the height is not a multiple of 8, the high bits
/// of each column in the last page are unused.
///
/// # ARGB8888 output layout
///
/// One pixel per four bytes, in byte order `[A, R, G, B]` with A =
/// 0xFF.
///
/// # Chunking and draw-buffer sizing
///
/// The `Screen` HIL allows large writes to be split across multiple
/// `write(_, continue_write=true)` calls. This adapter accepts any
/// chunk size (in Mono VLSB bytes); internally it processes one
/// sub-rectangle at a time on the underlying screen, so the draw
/// buffer only needs to hold one sub-op's ARGB output. A larger draw
/// buffer lets one client `write` be satisfied in fewer underlying
/// `set_write_frame + write` round-trips (at most one per full-width
/// page held by the buffer) at the cost of more static RAM.
///
/// The buffer must be able to hold at least one row of the widest
/// sub-op the adapter will ever emit --- i.e. one row of the client's
/// write-frame width (`frame.width * 4` bytes). Boards that expect
/// full-width writes should size it to at least `resource_width * 8 *
/// 4` (one full-width page) so head/tail partial sub-ops always fit
/// in a single underlying write.
pub struct ScreenARGB8888ToMono8BitPage<'a, S: Screen<'a>> {
    screen: &'a S,
    /// ARGB pixel data buffer
    draw_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    /// Underlying-array length of the draw buffer, cached so that
    /// sub-op sizing can consult it without taking the buffer out.
    draw_buffer_capacity: usize,
    /// The client-provided source buffer, held for the duration of a
    /// single `write` call's chunk.
    client_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    client: OptionalCell<&'a dyn ScreenClient>,

    /// Client's logical write frame. Set by `set_write_frame`.
    display_frame: Cell<Frame>,

    /// Byte cursor into the frame's Mono VLSB stream. Advanced by
    /// each sub-op as it completes and reset by `set_write_frame` and
    /// `write(_, false)`.
    frame_cursor: Cell<usize>,
    /// Value of `frame_cursor` at the start of the current chunk. The
    /// next byte to consume in the client buffer is at offset
    /// `frame_cursor - chunk_start`.
    chunk_start: Cell<usize>,
    /// Frame position at which the current chunk ends. When
    /// `frame_cursor` reaches this, the client-visible write is done.
    chunk_end: Cell<usize>,

    /// The underlying-screen rectangle for the sub-op currently in
    /// flight (between `start_next_sub_op` and
    /// `write_complete`). Must only be [Option::Some] if there is an
    /// outstanding write operation to the underlying device. That is,
    /// on [ScreenClient::command_complete], if this is
    /// [Option::None], the completion is for a client initiated
    /// `set_frame`, if this is `Option::Some` it, the completion is
    /// for an internal operation's `set_frame`.
    current_op: Cell<Option<Frame>>,
}

impl<'a, S: Screen<'a>> ScreenARGB8888ToMono8BitPage<'a, S> {
    pub fn new(screen: &'a S, draw_buffer: &'static mut [u8]) -> Self {
        // Draw buffer must hold whole 4-byte pixels.
        assert!(draw_buffer.len().is_multiple_of(4));
        let capacity = draw_buffer.len();
        ScreenARGB8888ToMono8BitPage {
            screen,
            draw_buffer: OptionalCell::new(SubSliceMut::new(draw_buffer)),
            draw_buffer_capacity: capacity,
            client_buffer: OptionalCell::empty(),
            client: OptionalCell::empty(),
            display_frame: Cell::new(Frame::default()),
            frame_cursor: Cell::new(0),
            chunk_start: Cell::new(0),
            chunk_end: Cell::new(0),
            current_op: Cell::new(None),
        }
    }
}

/// Convert one Mono VLSB sub-rectangle's worth of source bytes into
/// row-major ARGB8888.
///
/// The sub-rectangle is `sub_cols` columns wide by `dst.len() /
/// (sub_cols * 4)` rows tall. `src` provides Mono VLSB bytes relative
/// to the sub-rectangle (page 0 first, then page 1, ...); it must be
/// at least `sub_cols * ceil(rows / 8)` bytes. Unused high bits of
/// the final page (when the row count is not a multiple of 8) are
/// ignored.
///
/// A set bit becomes an opaque white ARGB pixel; a cleared bit an opaque
/// black one. Output byte order is `[A, R, G, B]` with A = 0xFF.
fn convert_mvlsb_sub_rect(src: &[u8], dst: &mut [u8], sub_cols: usize) {
    // destination bytes per row
    let row_bytes = sub_cols * 4;
    // destination bytes per page
    let page_bytes = 8 * row_bytes;
    // Iterate over each source and destination "page" (columns * 8)
    for (src_page, dst_page) in src.chunks(sub_cols).zip(dst.chunks_mut(page_bytes)) {
        // Within a page, iterate on each row and corresponding bit
        // position in src bytes.
        for (bit, dst_row) in dst_page.chunks_exact_mut(row_bytes).enumerate() {
            // Extract the row of source pixels
            let src_row_pixels = src_page.iter().map(|b| (b >> bit) & 1 != 0);
            // Within a row, each source byte fills one 4-byte pixel.
            let dst_row_pixels = dst_row.chunks_exact_mut(4);

            for (src_pixel, dst_pixel) in src_row_pixels.zip(dst_row_pixels) {
                let v = if src_pixel { !0 } else { 0 };
                // Yellow foreground, black background.
                dst_pixel.copy_from_slice(&[0xFF, v, v, 0x00]);
            }
        }
    }
}

/// How tall, in rows, is Mono VLSB page `page` given the total frame
/// height (accounts for the final page being partial when
/// `frame_h % 8 != 0`).
fn page_rows(page: usize, frame_h: usize) -> usize {
    core::cmp::min(8, frame_h.saturating_sub(page * 8))
}

/// Number of Mono VLSB source bytes a sub-op of shape `op` consumes.
fn sub_op_src_bytes(op: Frame) -> usize {
    op.width * op.height.div_ceil(8)
}

/// Number of ARGB destination bytes a sub-op of shape `op` writes.
fn sub_op_dst_bytes(op: Frame) -> usize {
    op.width * op.height * 4
}

/// Compute the next underlying-screen sub-rectangle given the current
/// position in the client's Mono VLSB stream, the end of the current
/// chunk, and the draw buffer's byte capacity.
///
/// Chooses a sub-op greedily: as many full pages as fit in both the
/// remaining chunk and the draw buffer, or a single partial page (head
/// or tail).
///
/// Returns `None` when (nothing left in this chunk).
fn next_sub_op(
    display_frame: Frame,
    cursor: usize,
    chunk_end: usize,
    draw_buffer_capacity: usize,
) -> Option<Frame> {
    if cursor >= chunk_end {
        return None;
    }
    let start_col = cursor % display_frame.width;
    let start_page = cursor / display_frame.width;
    let this_page_rows = page_rows(start_page, display_frame.height);
    let remaining = chunk_end - cursor;

    // Head partial page: chunk begins/continues mid-page (start_col > 0).
    // One page's worth of rows, up to (W - start_col) columns wide.
    if start_col > 0 {
        let cols = core::cmp::min(display_frame.width - start_col, remaining);
        return Some(Frame {
            x: display_frame.x + start_col,
            y: display_frame.y + start_page * 8,
            width: cols,
            height: this_page_rows,
        });
    }

    // Tail partial page: at page boundary but fewer than W bytes
    // remain, so we can't fill a full-width page.
    if remaining < display_frame.width {
        return Some(Frame {
            x: display_frame.x,
            y: display_frame.y + start_page * 8,
            width: remaining,
            height: this_page_rows,
        });
    }

    // Middle band: at a page boundary with at least one full page
    // remaining. Grab as many full pages as fit in both the chunk and
    // the draw buffer.
    let pages_available = remaining / display_frame.width;
    let mut pages_taken = 0;
    let mut rows_taken = 0;
    for i in 0..pages_available {
        let ph = page_rows(start_page + i, display_frame.height);
        if display_frame.width * (rows_taken + ph) * 4 > draw_buffer_capacity {
            break;
        }
        pages_taken += 1;
        rows_taken += ph;
    }
    debug_assert!(
        pages_taken >= 1,
        "draw_buffer_capacity too small to hold one full-width page",
    );
    Some(Frame {
        x: display_frame.x,
        y: display_frame.y + start_page * 8,
        width: display_frame.width,
        height: rows_taken,
    })
}

impl<'a, S: Screen<'a>> ScreenARGB8888ToMono8BitPage<'a, S> {
    /// Compute the next sub-op, convert its source bytes into the draw
    /// buffer, and issue `set_write_frame` on the underlying screen.
    ///
    /// Must only be called when `frame_cursor < chunk_end`.
    fn start_next_sub_op(&self) -> Result<(), ErrorCode> {
        let cursor = self.frame_cursor.get();
        let chunk_end = self.chunk_end.get();
        // Caller guarantees there's more work.
        let op = next_sub_op(
            self.display_frame.get(),
            cursor,
            chunk_end,
            self.draw_buffer_capacity,
        )
        .ok_or(ErrorCode::FAIL)?;
        let src_bytes = sub_op_src_bytes(op);
        let dst_bytes = sub_op_dst_bytes(op);
        let client_offset = cursor - self.chunk_start.get();

        let mut draw_buffer = self.draw_buffer.take().ok_or(ErrorCode::BUSY)?;
        let client_buffer = match self.client_buffer.take() {
            Some(cb) => cb,
            None => {
                self.draw_buffer.replace(draw_buffer);
                return Err(ErrorCode::FAIL);
            }
        };
        draw_buffer.reset();
        convert_mvlsb_sub_rect(
            &client_buffer.as_slice()[client_offset..client_offset + src_bytes],
            &mut draw_buffer.as_mut_slice()[..dst_bytes],
            op.width,
        );
        draw_buffer.slice(..dst_bytes);
        self.draw_buffer.replace(draw_buffer);
        self.client_buffer.replace(client_buffer);

        self.current_op.set(Some(op));
        self.screen.set_write_frame(op.x, op.y, op.width, op.height)
    }

    /// Hand the pre-converted draw buffer to the underlying screen's
    /// write. Called from `command_complete` after
    /// `start_next_sub_op`'s `set_write_frame` finishes.
    fn write_current_sub_op(&self) -> Result<(), ErrorCode> {
        let draw_buffer = self.draw_buffer.take().ok_or(ErrorCode::FAIL)?;
        self.screen.write(draw_buffer, false)
    }

    /// End of chunk: return the client's buffer and deliver
    /// `write_complete` to them.
    fn finish_chunk(&self, result: Result<(), ErrorCode>) {
        self.current_op.set(None);
        if let Some(cb) = self.client_buffer.take() {
            self.client.map(|c| c.write_complete(cb, result));
        }
    }
}

impl<'a, S: Screen<'a>> Screen<'a> for ScreenARGB8888ToMono8BitPage<'a, S> {
    fn set_client(&self, client: &'a dyn ScreenClient) {
        self.client.replace(client);
    }

    fn get_resolution(&self) -> (usize, usize) {
        self.screen.get_resolution()
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::Mono_8BitPage
    }

    fn get_rotation(&self) -> ScreenRotation {
        self.screen.get_rotation()
    }

    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode> {
        if self.current_op.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        self.display_frame.set(Frame {
            x,
            y,
            width,
            height,
        });
        self.frame_cursor.set(0);

        self.screen.set_write_frame(x, y, width, height)
    }

    fn write(
        &self,
        buffer: SubSliceMut<'static, u8>,
        continue_write: bool,
    ) -> Result<(), ErrorCode> {
        if self.current_op.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        let frame = self.display_frame.get();
        if frame.width == 0 || frame.height == 0 {
            return Err(ErrorCode::INVAL);
        }

        if !continue_write {
            self.frame_cursor.set(0);
        }

        let cursor = self.frame_cursor.get();
        let total_frame_bytes = frame.width * frame.height.div_ceil(8);
        if cursor >= total_frame_bytes {
            return Err(ErrorCode::SIZE);
        }
        let chunk_len = core::cmp::min(buffer.len(), total_frame_bytes - cursor);
        if chunk_len == 0 {
            return Err(ErrorCode::SIZE);
        }

        // Truncate the client buffer's active window to the portion we
        // will actually consume.
        let mut buffer = buffer;
        buffer.slice(..chunk_len);

        self.chunk_start.set(cursor);
        self.chunk_end.set(cursor + chunk_len);
        assert!(self.client_buffer.replace(buffer).is_none());

        if let Err(e) = self.start_next_sub_op() {
            self.finish_chunk(Err(e));
            return Err(e);
        }
        Ok(())
    }

    fn set_brightness(&self, brightness: u16) -> Result<(), ErrorCode> {
        self.screen.set_brightness(brightness)
    }

    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode> {
        self.screen.set_power(enabled)
    }

    fn set_invert(&self, enabled: bool) -> Result<(), ErrorCode> {
        self.screen.set_invert(enabled)
    }
}

impl<'a, S: Screen<'a>> ScreenClient for ScreenARGB8888ToMono8BitPage<'a, S> {
    fn screen_is_ready(&self) {
        self.client.map(|c| c.screen_is_ready());
    }

    fn command_complete(&self, result: Result<(), ErrorCode>) {
        if self.current_op.get().is_none() {
            self.client.map(|c| c.command_complete(result));
        } else {
            if result.is_err() {
                self.finish_chunk(result);
                return;
            }
            if let Err(e) = self.write_current_sub_op() {
                self.finish_chunk(Err(e));
            }
        }
    }

    fn write_complete(&self, buffer: SubSliceMut<'static, u8>, result: Result<(), ErrorCode>) {
        self.draw_buffer.replace(buffer);

        if result.is_err() {
            self.finish_chunk(result);
            return;
        }

        // Advance the frame cursor past the sub-op we just finished.
        let Some(op) = self.current_op.get() else {
            // Spurious callback; nothing to advance.
            return;
        };
        self.frame_cursor
            .set(self.frame_cursor.get() + sub_op_src_bytes(op));
        self.current_op.set(None);

        // More of this chunk to draw? Kick off the next sub-op.
        // Otherwise finalize the client-visible write.
        if self.frame_cursor.get() < self.chunk_end.get() {
            if let Err(e) = self.start_next_sub_op() {
                self.finish_chunk(Err(e));
            }
        } else {
            self.finish_chunk(Ok(()));
        }
    }
}
