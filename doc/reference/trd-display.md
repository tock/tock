Display HIL
===========

**TRD:** <br/>
**Working Group:** <br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Alexandru Radovici, Dorota Czaplejewicz<br/>
**Draft-Created:** 2022/06/10 <br/>
**Draft-Modified:** 2022/06/10 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** tock-dev@googlegroups.com<br/>

Abstract
-------------------------------

This document proposes hardware independent layer interface (HIL) for 
display interface in the Tock operating system kernel. It describes 
the Rust traits and other definitions for this service as well as the 
reasoning behind them. 

This document is in full compliance with [TRD1](./trd1-trds.md).

1 Introduction
===============================

The Display HIL defines three main items that are present in all the displays:

  1. The `FrameBuffer` that handles pixel data that is being displayed, used for graphics
  2. The `TextBuffer` that handles text data that is being displayed, used for text
  3. The `Screen` that handles the parameters of the actual display hardware

The separation between framebuffer, textbuffer and screen has been made as they may function
as independant systems. For instance, the framebuffer could be used either for a
screen or fopr a virtual framebuffer, while the screen could be used either for
graphic screens or text screens.

The Display HIL is in the kernel crate, in module `hil::display`. It provides seven main
traits:

  * `kernel::hil::display::FrameBuffer`: provides an abstraction of
    the a framebuffer.
  * `kernel::hil::display::FrameBufferSetup`: provides an abstraction
  of a framebuffer that the configuration of several parameters such as
  resolution, color mode, etc.
  * `kernel::hil::display::TextBuffer`: provides an abstraction of
    the a textbuffer.
  * `kernel::hil::display::Screen`: provides an abstraction of an actual
  screen device and deals with functionallity like power, color inversion
  and brightness.
  * `kernel::hil::display::Display`: combines the `FrameBuffer` and 
  the `Screen` trait.
  * `kernel::hil::display::DisplayAdvanced`: combines the `Display` and 
  the `FrameBufferSetup` trait.

The Display HIL povides ... client traits:
  * `kernel::hil::display::FrameBufferClient`: provides an abstraction of
    the a framebuffer.
  * `kernel::hil::display::FrameBufferSetup`: provides an abstraction
  of a framebuffer that the configuration of several parameters such as
  resolution, color mode, etc.

This document describes these traits and their semantics.

2 `FrameBuffer` and `FrameBufferClient` traits
===============================

The `FrameBuffer` trait allows a client to write data to a framebuffer and 
provides information about the frmaebuffer's geometry (resolution) and 
color mode.

```rust
#[non_exhaustive]
pub enum PixelFormat {
    Mono,
    RGB_233,
    RGB_565,
    RGB_888,
    ARGB_8888,
}

impl PixelFormat {
    pub fn get_bits_per_pixel(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::RGB_233 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
        }
    }
}

pub struct GraphicsMode {
    width: usize,
    height: usize,
    pixel_format: PixelFormat,
}

pub trait FrameBuffer {
    fn get_mode(&self) -> GraphicsMode;
    fn get_tile_size(&self) -> usize;
    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode>;
    fn write(
        &self,
        buffer: &'static mut [u8],
        len: usize,
        reset_position: bool,
    ) -> Result<(), ErrorCode>;
    fn flush(&self) -> Result<(), ErrorCode>;
    fn set_client(&self, client: Option<&'static dyn ScreenClient>);
}

pub trait FrameBufferClient {
    fn write_complete(&self, buffer: &'static mut [u8], r: Result<(), ErrorCode>);
    fn command_complete(&self, r: Result<(), ErrorCode>);
}
```

The `get_mode` method in `FrameBuffer` returns the current framebuffer's `GraphicsMode`:
  - `width` - the width in pixels
  - `height` - the height in pixels
  - `pixel_format` - the way in which the data written to the framebuffer will be interpreded

Framebuffers are block devices, they write data in blocks instead of bytes.
The `get_block_size` informs the client about the block size that the
framebuffer is capable of accepting. The buffer length used by the 
write functions has to be a multiple of this block size. The block size
is returned in bytes (not pixels). It is up to the framebuffer driver
to make sure that the block size corresponds to the pixel format.

The framebuffer allows clients to set a specific write are. When the
client issues a write command, the framebuffer will update only the
area that has been selected by `set_write_frame`. When reacing
the end of the write area, framebuffers SHOULD wrap around and continue
writing from the begining of the write area.

The `write` method writes data to the framebuffer. The framebuffer keeps
a write position counter to memorize the position whithin the write area.
Setting the `reset_position` to `true` will reset this counter to the
start of the write area.

The write-related methods in `FrameBuffer` can return an error. Valid errors are:
  - INVAL (`set_write_frame`, `write`): the parameters of the write area are impossible or 
    the length of the buffer is not a block multiple
  - OFF (all): the framebuffer device has not been initialized or is powered off
  - BUSY (all): there is another action in progress, the client should wait a 
    call to the `command_complete` or `write_complete` before requesting another action
  - FAIL (all): some other error occurred
  - ENOSUPPORT (`flush`): there is no need to flush the framebuffer.

2 `TextBuffer` and `TextBuffer` traits
===============================

3 `Screen` and `ScreenClient` traits
===============================

4 The `FrameBufferSetup` trait
==============================

5 Capsules
===============================

This section describes the standard Tock capsules for displays.

6 Implementation Considerations
===============================

7 Authors' Address
=================================
```
Alexandru Radovici
Wyliodrin SRL
Bucharest, 061103
Romania
alexandru.radovici@wyliodrin.com

Dorota Czaplejewicz <gihu.dcz@porcupinefactory.org>
```
