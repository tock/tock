---
driver number: 0x90001
---

# Screen

## Overview

The screen driver allows the process to write data to a framebuffer of a screen.

This syscall driver is designed for single-client usage,
for the sake of simplicity, and because a single client covers
a significant portion of display use cases.
While several clients may use this interface simultaneously,
each may independently modify parameters defining the usage of the display,
like resolution, or power. Those changes will not be broadcast
to every client, requiring external synchronization.
This syscall interface does not expose the current power state of the display,
so each usage should start by calling the "Set power" syscall.
All commands except "Does the driver exist?" and "Set power"
may return OFF when power is not enabled (see screen HIL for details).
## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Success if it exists, otherwise NODEVICE

  * ### Command number: `1`

    **Description**: Checks if the Setup API is available

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with 1 if yes, and 0 if no

  * ### Command number: `2`

    **Description**: Set power

    **Argument 1**: 0 if off, nonzero if on.

    **Argument 2**: unused

    **Returns**: Ok(()) followed by the ready callback if the command was successful,
    BUSY if another command is in progress.
    
    The callback will carry 1 as an argument if the display was turned on,
    but configuration not fully applied. Otherwise, the argument is 0.

  * ### Command number: `3`

    **Description**: Set brightness

    **Argument 1**: Lightness value, relative to minimum and maximum supported.
    0 should turn off the light if available, greater than 0 should set it to minimum.
    65535 and above turn lightness to the maximum supported.
    Intermediate values approximate intermediate lightness levels.
    May take effect only after power is set (e.g. for LED displays).

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if another command is in progress.

  * ### Command number: `4` (deprecated)

    **Description**: Turn on invert colors mode

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if another command is in progress.
  
  * ### Command number: `5` (deprecated)

    **Description**: Turn off invert colors mode

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if another command is in progress.
    
  * ### Command number: `6`

    **Description**: Control invert colors mode.
    Color inversion will affect all pixels already submitted, and submitted in the future.
    It may get reset when in case of switching pixel formats.

    **Argument 1**: 0 if off, nonzero if on.

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if another command is in progress.

  * ### Command number: `11` 

    **Description**: Get the number of supported resolutions (Setup API)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported resolutions (minimum 1).

  * ### Command number: `12` 

    **Description**: Get the size of a supported resolution (Setup API)

    **Argument 1**: index of the resolution (0, return of command 11)

    **Argument 2**: unused

    **Returns**: A pair of u32 values: width, height.
  
  * ### Command number: `13` 

    **Description**: Get the number of supported pixel formats (Setup API)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported pixel formats (minimum 1).

  * ### Command number: `14` 

    **Description**: Get the type of a supported pixel format (Setup API)

    **Argument 1**: index of the pixel formats (0, return of command 13)

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with the pixel format value, INVAL if index out of bounds.

  * ### Command number: `21` 

    **Description**: Get the screen's rotation.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with the rotation value:
    Normal = 0,
    Rotated90 = 1,
    Rotated180 = 2,
    Rotated270 = 3.
  
    Rotation is measured counterclockwise.

  * ### Command number: `22` 

    **Description**: Set the screen's rotation (Setup API)

    **Argument 1**: rotation value, counterclockwise (0 - normal, 1 - 90deg, 2 - 180deg, 3 - 270deg)

    **Argument 2**: unused

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

  * ### Command number: `23` 

    **Description**: Get the screen's resolution

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: A pair of u32 values: width, height.

  * ### Command number: `24` 

    **Description**: Set the screen's resolution (Setup API)

    **Argument 1**: width (pixels)

    **Argument 2**: height (pixels)

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

  * ### Command number: `25` 

    **Description**: Get the screen's pixel format

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: A single u32 value.
    - 0: 8 pixels per byte monochromatic, pixels more to the left are more significant bits. 1 is light, 0 is dark.
    - 1: RGB_233, 2-bit red channel, 3-bit green channel, 3-bit blue channel.
    - 2: RGB_565, 5-bit red channel, 6-bit green channel, 5-bit blue channel.
    - 3: RGB_888
    - 4: ARGB_8888 (RGB with transparency)
    - 5: RGB_4BIT, 1-bit blue channel, 1-bit green, 1-bit red, 1-bit for opaque (1) vs transparent (0)
    - 6: Mono_8BitPage, 8 pixels per byte monochromatic, each byte is displayed
      vertically (pixels above are less significant bits) and tile
      horizontally.

  * ### Command number: `26` 

    **Description**: Set the screen's pixel format (Setup API)

    **Argument 1**: pixel format specifier (see above)

    **Argument 2**: unused

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

  * ### Command number: `100` 

    **Description**: Set the framebuffer write frame

    **Argument 1**: x | y (pixels, 16 bit LE)

    **Argument 2**: width | height (pixels, 16 bit LE)

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

  * ### Command number: `200`

    **Description**: Initiate a write transaction of a buffer shared using `allow_readonly`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: buffer length (in bytes)

    **Argument 2**: unused

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

  * ### Command number: `300`

    **Description**: Initiate a fill transaction of a buffer shared using `allow_readonly`. This will fill the write frame with the first pixel in thhe buffer.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) followed by a callback when it is done, BUSY if another command is in progress.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe to to all commands.

    **Callback signature**: The callback receives different arguments 
    depending on the issued command.

    **Returns**: Ok(()) if the subscribe was successful.

## Allow ReadOnly

  * ### Allow number: `0`

    **Description**: Sets a shared buffer to be used as a source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined (most likely either the original buffer or
    new buffer will be written in its entirety but not both).

    **Returns**: Ok(()) if the subscribe was successful.
