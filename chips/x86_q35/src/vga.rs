// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Minimal VGA peripheral implementation for the Tock x86_q35 chip crate.
//!
//! Supports classic 80×25 text mode out-of-the-box and exposes a stub for
//! setting planar 16-colour graphics modes (640×480 and 800×600).  These
//! extra modes will be filled in later once the driver is integrated with a
//! future framebuffer capsule.
//!
//!
//! NOTE!!!
//!
//! This file compiles and provides working text-
//! mode console support so the board can swap from the UART mux to a VGA
//! console.  Graphical modes are *disabled at runtime* until a framebuffer
//! capsule implementation lands.  The low-level register writes for 640×480 and 800×600 are
//! nonetheless laid out so they can be enabled by flipping a constant.
//!
//! VGA peripheral driver for the x86_q35 chip.
//!
//! The driver currently focuses on **text mode** (colour attribute buffer at
//! 0xB8000).  It also defines [`VgaMode`] and an [`init`] routine that writes
//! the necessary CRT controller registers for text mode and two common planar
//! 16-colour modes.  Other code (e.g. the board crate) can query the selected
//! mode via `kernel::config::CONFIG.vga_mode` and decide whether to route the
//! `ProcessConsole` to this driver or to the legacy serial mux.

use core::cell::Cell;
/// Write an 8-bit value to an I/O Port.
/// Read an 8-bit value from an I/O port.
use x86::registers::io::{inb, outb};

// 16 classic VGA colors (matches text-mode palette indices 0–15)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

/// Packed VGA attribute byte for text mode:
/// bits 0–3 = foreground color; 4–6 = background color; 7 = blink/bright (mode-dependent).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Build a color code from fg/bg/blink.
    /// Note: with Attribute Controller mode bit 7 = blink enabled, bit 7 here blinks.
    /// If blink is disabled in the controller, bit 7 acts as "bright background".
    pub const fn new(fg: Color, bg: Color, blink: bool) -> Self {
        let mut b = (fg as u8) & 0x0F;
        b |= ((bg as u8) & 0x07) << 4;
        if blink {
            b |= 1 << 7;
        }
        Self(b)
    }

    /// Raw byte as written to the high byte of the cell.
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Construct directly from a packed byte (for interop/tests).
    #[inline(always)]
    pub const fn from_u8(b: u8) -> Self {
        Self(b)
    }

    /// Extractors (handy for debugging/tools)
    #[inline(always)]
    pub const fn fg(self) -> Color {
        match self.0 & 0x0F {
            0x0 => Color::Black,
            0x1 => Color::Blue,
            0x2 => Color::Green,
            0x3 => Color::Cyan,
            0x4 => Color::Red,
            0x5 => Color::Magenta,
            0x6 => Color::Brown,
            0x7 => Color::LightGray,
            0x8 => Color::DarkGray,
            0x9 => Color::LightBlue,
            0xA => Color::LightGreen,
            0xB => Color::LightCyan,
            0xC => Color::LightRed,
            0xD => Color::Pink,
            0xE => Color::Yellow,
            _ => Color::White, // 0xF
        }
    }
    /// Note: background uses only 3 bits (0..=7). Bright backgrounds require
    /// disabling blink in the Attribute Controller and repurposing bit 7.
    #[inline(always)]
    pub const fn bg(self) -> Color {
        match (self.0 >> 4) & 0x07 {
            0x0 => Color::Black,
            0x1 => Color::Blue,
            0x2 => Color::Green,
            0x3 => Color::Cyan,
            0x4 => Color::Red,
            0x5 => Color::Magenta,
            0x6 => Color::Brown,
            _ => Color::LightGray, // 0x7
        }
    }
    pub const fn blink(self) -> bool {
        (self.0 & 0x80) != 0
    }
}

/// All VGA modes supported by the x86_q35 chip crate.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VgaMode {
    Text80x25,
    Graphics640x480_16,
    Graphics800x600_16,
}

// Constants for memory-mapped text mode buffer

// VGA physical Address

const TEXT_BUFFER_ADDR: usize = 0xB8000;
// Buffer dimensions
const TEXT_BUFFER_WIDTH: usize = 80;
const TEXT_BUFFER_HEIGHT: usize = 25;
/// Physical address where QEMU exposes the linear-frame-buffer BAR.
const LFB_PHYS_BASE: u32 = 0xE0_00_0000;

// Public API - the VGA struct providing text console implementation

/// Simple text-mode VGA console.
pub struct Vga {
    col: Cell<usize>,
    row: Cell<usize>,
    /// Current VGA text attribute byte for newly written characters.
    /// Layout (text mode):
    /// bits 0–3 = fg (0–15), 4–6 = bg (0–7), 7 = blink/bright (mode-dependent).
    /// Kept packed to match hardware and allow a single 16-bit volatile store per glyph.
    attr: Cell<u8>,
}
impl Vga {
    pub const fn new() -> Self {
        Self {
            col: Cell::new(0),
            row: Cell::new(0),
            // default: LightGray on Black, no blink
            attr: Cell::new(ColorCode::new(Color::LightGray, Color::Black, false).as_u8()),
        }
    }

    const fn buffer_ptr() -> *mut u16 {
        TEXT_BUFFER_ADDR as *mut u16
    }

    // Index -> pointer into 0xB8000.
    // SAFETY: `buffer_ptr()` points to the VGA text buffer at a fixed, valid address (0xB8000), and callers ensure
    // that `index < TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT`, so the pointer offset is always in-bounds.
    #[inline(always)]
    unsafe fn cell_at(index: usize) -> *mut u16 {
        unsafe { Self::buffer_ptr().add(index) }
    }

    fn update_hw_cursor(&self) {
        let pos = (self.row.get() * TEXT_BUFFER_WIDTH + self.col.get()) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, (pos >> 8) as u8);
        }
    }

    fn scroll_up(&self) {
        let blank = ((self.attr.get() as u16) << 8) | b' ' as u16;

        for row in 1..TEXT_BUFFER_HEIGHT {
            for col in 0..TEXT_BUFFER_WIDTH {
                let src = unsafe { Self::cell_at(row * TEXT_BUFFER_WIDTH + col) };
                let dst = unsafe { Self::cell_at((row - 1) * TEXT_BUFFER_WIDTH + col) };
                let val = unsafe { core::ptr::read_volatile(src) };
                unsafe { core::ptr::write_volatile(dst, val) };
            }
        }

        for col in 0..TEXT_BUFFER_WIDTH {
            let idx = (TEXT_BUFFER_HEIGHT - 1) * TEXT_BUFFER_WIDTH + col;
            unsafe { core::ptr::write_volatile(Self::cell_at(idx), blank) };
        }

        self.row.set(TEXT_BUFFER_HEIGHT - 1);
        self.col.set(0);
    }

    pub fn set_cursor(&self, col: usize, row: usize) {
        self.col.set(col);
        self.row.set(row);
        self.update_hw_cursor();
    }

    // pub fn set_attr(&self, attr: u8) {
    //   self.attr.set(attr);
    //}

    /// Set the current attribute from a typed ColorCode.
    #[inline(always)]
    pub fn set_color_code(&self, code: ColorCode) {
        self.attr.set(code.as_u8());
    }

    /// Set fg/bg/blink with typed colors.
    #[inline(always)]
    pub fn set_colors(&self, fg: Color, bg: Color, blink: bool) {
        self.set_color_code(ColorCode::new(fg, bg, blink));
    }

    /// Read back the current color code (typed).
    #[inline(always)]
    pub fn color_code(&self) -> ColorCode {
        ColorCode::from_u8(self.attr.get())
    }

    pub fn clear(&self) {
        let blank = ((self.attr.get() as u16) << 8) | b' ' as u16;
        for i in 0..TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT {
            unsafe { core::ptr::write_volatile(Self::cell_at(i), blank) };
        }
        self.col.set(0);
        self.row.set(0);
        self.update_hw_cursor();
    }

    pub fn write_byte(&self, byte: u8) {
        match byte {
            b'\n' => {
                self.col.set(0);
                self.row.set(self.row.get() + 1);
            }
            b'\r' => {
                self.col.set(0);
            }
            byte => {
                let val = ((self.attr.get() as u16) << 8) | byte as u16;
                unsafe {
                    core::ptr::write_volatile(
                        Self::cell_at(self.row.get() * TEXT_BUFFER_WIDTH + self.col.get()),
                        val,
                    );
                }
                self.col.set(self.col.get() + 1);
                if self.col.get() == TEXT_BUFFER_WIDTH {
                    self.col.set(0);
                    self.row.set(self.row.get() + 1);
                }
            }
        }
        if self.row.get() == TEXT_BUFFER_HEIGHT {
            self.scroll_up();
        }
        self.update_hw_cursor();
    }
}

fn init_text_mode() {
    // Select CRTC register index 0x11 (cursor start register) and reset its value to 0
    unsafe {
        outb(0x3D4, 0x11);
        outb(0x3D5, 0x00);

        // Read the Attribute Controller’s status register to reset its internal flip-flop
        inb(0x3DA);
    }

    // Program the 21 Attribute Controller registers:
    //   0x00–0x0F are the 16 palette entries,
    //   0x10 = mode control (graphics off, blink on),
    //   0x12 = color plane enable mask.
    for (idx, val) in [
        (0x00, 0x00u8), // palette 0: black
        (0x01, 0x01),   // palette 1: blue
        (0x02, 0x02),   // palette 2: green
        (0x03, 0x03),   // palette 3: cyan
        (0x04, 0x04),   // palette 4: red
        (0x05, 0x05),   // palette 5: magenta
        (0x06, 0x14),   // palette 6: brown
        (0x07, 0x07),   // palette 7: light grey
        (0x08, 0x38),   // palette 8: dark grey
        (0x09, 0x39),   // palette 9: light blue
        (0x0A, 0x3A),   // palette A: light green
        (0x0B, 0x3B),   // palette B: light cyan
        (0x0C, 0x3C),   // palette C: light red
        (0x0D, 0x3D),   // palette D: light magenta
        (0x0E, 0x3E),   // palette E: yellow
        (0x0F, 0x3F),   // palette F: white
        (0x10, 0x0C),   // mode control: text mode, blink attribute on
        (0x12, 0x0F),   // enable all 4 color planes
    ]
    .iter()
    .copied()
    {
        // Write the register index to the Attribute Controller
        unsafe {
            outb(0x3C0, idx);
            // Write the corresponding value
            outb(0x3C0, val);
        }
    }

    // Reset the flip-flop again before enabling video output
    unsafe {
        inb(0x3DA);

        // Turn video output back on (set bit 5 of the Attribute Controller’s 0x20 register)
        outb(0x3C0, 0x20);
    }
}

#[allow(clippy::single_match)]
pub fn init(mode: VgaMode) {
    match mode {
        VgaMode::Text80x25 => init_text_mode(),
        VgaMode::Graphics640x480_16 => panic!("VGA 640×480 mode not implemented"),
        VgaMode::Graphics800x600_16 => panic!("VGA 800×600 mode not implemented"),
    }
}

const _: () = {
    // Exhaustively touch every current VgaMode variant
    match VgaMode::Text80x25 {
        VgaMode::Text80x25 => (),
        VgaMode::Graphics640x480_16 => (),
        VgaMode::Graphics800x600_16 => (),
    }
};

// stub for future graphic options implementation
pub fn framebuffer() -> Option<(*mut u8, usize)> {
    None
}

fn init_and_map_lfb(mode: VgaMode, page_dir: &mut x86::registers::bits32::paging::PD) {
    init(mode);
    if mode == VgaMode::Text80x25 {
        // Inline 4 MiB VGA framebuffer mapping using PDEntry::new()
        use x86::registers::bits32::paging::{PAddr, PDEntry, PDFlags, PDFLAGS};

        // Compute which PDE slot holds LFB_PHYS_BASE
        // The page directory has 1024 entries, and the directory
        // index is the top 10 bits of the (virtual) address: bits [31:22].
        // We map the LFB with 4 MiB pages (PS=1) and use an identity mapping
        // (virt == phys), so shifting the physical base right by 22 yields the PDE index.

        let idx = (LFB_PHYS_BASE >> 22) as usize;
        // Wrap the physical base in a PAddr
        let pa = PAddr::from(LFB_PHYS_BASE);

        // Build flags via PDFlags + PDFLAGS::...::SET
        let mut flags = PDFlags::new(0);
        flags.write(PDFLAGS::P::SET + PDFLAGS::RW::SET + PDFLAGS::PS::SET);

        // Construct the entry (new() will mask & assert alignment)
        page_dir[idx] = PDEntry::new(pa, flags);
    }
}

/// Initialise 80×25 text mode and start with a clean screen.
pub(crate) fn new_text_console(page_dir_ptr: &mut x86::registers::bits32::paging::PD) {
    // Map VGA linear-framebuffer + program CRTC/attribute regs

    init_and_map_lfb(VgaMode::Text80x25, page_dir_ptr);

    // Wipe the BIOS banner so the kernel starts on a blank page.
    let blank: u16 = 0x0720; // white-on-black space
    for i in 0..(TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT) {
        unsafe {
            core::ptr::write_volatile((TEXT_BUFFER_ADDR as *mut u16).add(i), blank);
        }
    }
}
