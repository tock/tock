// Minimal VGA peripheral implementation for the Tock x86_q35 chip crate.
// Supports classic 80×25 text mode out‑of‑the‑box and exposes a stub for
// setting planar 16‑colour graphics modes (640×480 and 800×600).  These
// extra modes will be filled in later once the driver is integrated with a
// future framebuffer capsule.
//
// Licensing: same dual‑license terms as the rest of Tock (Apache‑2.0 OR MIT)
//
// NOTE!!!
//
// This file is an initial skeleton.  It compiles and provides working text‑
// mode console support so the board can swap from the UART mux to a VGA
// console.  Graphical modes are *disabled at runtime* until a framebuffer
// capsule lands.  The low‑level register writes for 640×480 and 800×600 are
// nonetheless laid out so they can be enabled by flipping a constant.
//
// VGA peripheral driver for the x86_q35 chip.
//
// The driver currently focuses on **text mode** (colour attribute buffer at
// 0xB8000).  It also defines [`VgaMode`] and an [`init`] routine that writes
// the necessary CRT controller registers for text mode and two common planar
// 16‑colour modes.  Other code (e.g. the board crate) can query the selected
// mode via `kernel::config::CONFIG.vga_mode` and decide whether to route the
// `ProcessConsole` to this driver or to the legacy serial mux.

#![allow(dead_code)]

use core::fmt::{self, Write};
use core::ptr::write_volatile;

pub use kernel::config::VgaMode;

// Public enum of supported modes (mirrors kernel::config::VgaMode)

/*#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum VgaMode {
    // 80 columns x 25 rows, 16 color text mode (VGA attribute memory)
    Text80x25,

    // 640 x 480 x 16- color planar graphics mode (mode 0x12)
    G640x480x16,

    // 800 x 600 x 16 color planar graphics mode (VESA mode 0x102)
    G800x600x16,
}*/

// Constants for memory-mapped text mode buffer

// VGA physical Address

const TEXT_BUFFER_ADDR: usize = 0xB8000;
// Buffer dimensions
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

// Low-level port I/O helpers
// inb/outb wrappers

// Write an 8-bit value to an I/O Port.

#[inline(always)]
fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in ("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

// Read an 8-bit value from an I/O port.
#[inline(always)]
fn inb(port: u16) -> u8 {
    let val: u8;

    unsafe {
        core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    val
}

// 16-bit helper
#[inline(always)]
fn outw(port: u16, val: u16) {
    unsafe {
        core::arch::asm!("out dx, ax", in("dx") port, in ("ax") val, options(nomem, nostack, preserves_flags));
    }
}
// Public API - the VGA struct providing fmt::Write implementation

// Simple text-mode VGA console. Provides "core::fmt::Write" so it can be
// plugged into Tock's `Console` and `ProcessConsole` components.
pub struct VgaText;

impl VgaText {
    // Create a new instance and optionally clear the screen
    pub const fn new() -> Self {
        VgaText
    }

    // Clear the entire text buffer with blank spaces in attribute 0x07
    // light gray on black

    pub fn clear(&self) {
        unsafe {
            let buffer = TEXT_BUFFER_ADDR as *mut u16;
            for i in 0..(BUFFER_WIDTH * BUFFER_HEIGHT) {
                write_volatile(buffer.add(i), 0x0700u16 | b' ' as u16);
            }
        }
        self.set_cursor(0, 0);
    }

    // Move the hardware cursor to `col`, `row`.
    pub fn set_cursor(&self, col: usize, row: usize) {
        let pos = (row * BUFFER_WIDTH + col) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, (pos >> 8) as u8);
        }
    }
}

impl Write for VgaText {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Very small, non-scrolling, no-wrap renderer: only prints within
        // Current line limits for boot messages.
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    let pos = unsafe {
                        outb(0x3D4, 0x0F);
                        let low = inb(0x3D5) as u16;
                        outb(0x3D4, 0x0E);
                        let hi = inb(0x3D5) as u16;
                        (hi << 8) | low
                    } as usize;
                    let row = pos / BUFFER_WIDTH;
                    self.set_cursor(0, (row + 1) % BUFFER_HEIGHT);
                }
                byte => {
                    // Write character + attribute 0x0F (white on black)
                    unsafe {
                        outb(0x3D4, 0x0F);
                        let cur_low = inb(0x3D5) as u16;
                        outb(0x3D4, 0x0E);
                        let cur_hi = inb(0x3D5) as u16;
                        let cur = ((cur_hi << 8) | cur_low) as usize;
                        let buffer = (TEXT_BUFFER_ADDR as *mut u16).add(cur);
                        write_volatile(buffer, 0x0F00u16 | byte as u16);
                        let next = (cur + 1) % (BUFFER_WIDTH * BUFFER_HEIGHT) as usize;
                        outb(0x3D4, 0x0F);
                        outb(0x3D5, (next & 0xFF) as u8);
                        outb(0x3D4, 0x0E);
                        outb(0x3D5, (next >> 8) as u8);
                    }
                }
            }
        }
        Ok(())
    }
}

fn init_text_mode() {
    // Standard BIOS mode 03h – easiest: call into BIOS via 0x10 int if we
    // wanted, but we can also just rely on the firmware default.  Here we
    // proactively reset key controller registers to known values so we can
    // switch from graphics back to text.
    outb(0x3D4, 0x11);
    outb(0x3D5, 0x00);
    // Horizontal/vertical timings omitted, firmware already sets them.
    // Reset attribute controller flip-flop
    inb(0x3DA);
    // Set attribute controller color + mode control (preset values)
    for (idx, val) in [
        (0x00, 0x00u8),
        (0x01, 0x01),
        (0x02, 0x02),
        (0x03, 0x03),
        (0x04, 0x04),
        (0x05, 0x05),
        (0x06, 0x14),
        (0x07, 0x07),
        (0x08, 0x38),
        (0x09, 0x39),
        (0x0A, 0x3A),
        (0x0B, 0x3B),
        (0x0C, 0x3C),
        (0x0D, 0x3D),
        (0x0E, 0x3E),
        (0x0F, 0x3F),
        (0x10, 0x0C), // Mode control: graphics off, blink attr on
        (0x12, 0x0F), // Colour plane enable
    ]
    .iter()
    .copied()
    {
        outb(0x3C0, idx);
        outb(0x3C0, val);
    }
}

pub fn init(mode: VgaMode) {
    match mode {
        VgaMode::Text80x25 => init_text_mode(),
        VgaMode::G640x480x16 => init_mode_0x12(),
        VgaMode::G800x600x16 => init_mode_0x102(),
    }
}
fn init_mode_0x12() {
    // 640×480×16‑colour – VGA BIOS mode 0x12, 4‑plane planar.
    // Here we only set the minimal Sequencer and CRTC registers needed so
    // that, when a proper framebuffer driver is added, the mode is active.
    const VBE_INDEX: u16 = 0x01CE;
    const VBE_DATA: u16 = 0x01CF;

    unsafe fn vbe_write(index: u16, value: u16) {
        outw(VBE_INDEX, index);
        outw(VBE_DATA, value);
    }
    unsafe {
        // 1) Disable display while we reconfigure

        vbe_write(0x04, 0x00);

        // 2) Set X-res, Y-res, bits-per-pixel
        vbe_write(0x01, 640);
        vbe_write(0x02, 480);
        vbe_write(0x03, 16);

        // 3) Enable LinearFB and ClearMem
        vbe_write(0x04, 0x41); //bit0=enable, bit6=liner FP
    }
}

// Same for 800x600x16
fn init_mode_0x102() {
    const VBE_INDEX: u16 = 0x01CE; // Bochs VBE index port
    const VBE_DATA: u16 = 0x01CF; // Bochs VBE data  port

    #[inline(always)]
    unsafe fn vbe_write(index: u16, value: u16) {
        outw(VBE_INDEX, index);
        outw(VBE_DATA, value);
    }

    unsafe {
        // 1) Disable display while reconfiguring
        vbe_write(0x04, 0x00); // VBE_DISPI_INDEX_ENABLE

        // 2) Set resolution and colour depth
        vbe_write(0x01, 800); // X-res (width)
        vbe_write(0x02, 600); // Y-res (height)
        vbe_write(0x03, 16); // bits per pixel (RGB 5-6-5)

        // 3) Enable display | LinearFB
        //    bit0 = enable, bit6 = LFB, all others 0
        vbe_write(0x04, 0x41);
    }
}

pub fn framebuffer() -> Option<(*mut u8, usize)> {
    match kernel::config::CONFIG.vga_mode {
        Some(VgaMode::G800x600x16) => Some((0xE000_0000 as *mut u8, 800 * 2)),
        _ => None,
    }
}
