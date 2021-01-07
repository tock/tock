---
driver number: 0x90001
---

# Screen

## Overview

The screen driver allows the process to write data to a framebuffer of a screen.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if it exists, otherwise ENODEVICE

  * ### Command number: `1`

    **Description**: Checks if the Setup API is available

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with 1 if yes, and 0 if no

  * ### Command number: `3`

    **Description**: Set brightness

    **Argument 1**: Percent of brightness, 0% should turn off the screen, greater than 0% should turn it on.

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `4`

    **Description**: Turn on invert colors mode

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.
  
  * ### Command number: `5`

    **Description**: Turn off invert colors mode

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `11` 

    **Description**: Get the number of supported resolutions (Setup API)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported resolutions (minimum 1), EBUSY if another command is in progress.

  * ### Command number: `12` 

    **Description**: Get the size of a supported resolution (Setup API)

    **Argument 1**: index of the resolution (0, return of command 11)

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the resolution, EBUSY if another command is in progress.
  
  * ### Command number: `13` 

    **Description**: Get the number of supported color depth (Setup API)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported color depth (minimum 1), EBUSY if another command is in progress.

  * ### Command number: `14` 

    **Description**: Get the type of a supported color depth (Setup API)

    **Argument 1**: index of the color depth (0, return of command 13)

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the resolution, EBUSY if another command is in progress.

  * ### Command number: `21` 

    **Description**: Get the screen's rotation

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the rotation value, EBUSY if another command is in progress.

  * ### Command number: `22` 

    **Description**: Set the screen's rotation (Setup API)

    **Argument 1**: rotation value (0 - normal, 1 - 90deg, 2 - 180deg, 3 - 270deg)

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

  * ### Command number: `23` 

    **Description**: Get the screen's resolution

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the rotation value, EBUSY if another command is in progress.

  * ### Command number: `24` 

    **Description**: Set the screen's resolution (Setup API)

    **Argument 1**: width (pixels)

    **Argument 2**: height (pixels)

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

  * ### Command number: `25` 

    **Description**: Get the screen's color depth

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the rotation value, EBUSY if another command is in progress.

  * ### Command number: `26` 

    **Description**: Set the screen's color depth (Setup API)

    **Argument 1**: color depth 

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

  * ### Command number: `100` 

    **Description**: Set the framebuffer write frame

    **Argument 1**: x | y (pixels, 16 bit LE)

    **Argument 2**: width | height (pixels, 16 bit LE)

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

  * ### Command number: `101` 

    **Description**: Initiate a write transaction of a buffer shared using `allow_readonly`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: buffer length (in bytes)

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

  * ### Command number: `102` 

    **Description**: Initiate a fill transaction of a buffer shared using `allow_readonly`. This will fill the write frame with the first pixel in thhe buffer.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe to to all commands.

    **Callback signature**: The callback receives different arguments 
    depending on the issued command.

    **Returns**: SUCCESS if the subscribe was successful.

## Allow ReadOnly

  * ### Allow number: `0`

    **Description**: Sets a shared buffer to be used as a source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined (most likely either the original buffer or
    new buffer will be written in its entirety but not both).

    **Returns**: SUCCESS if the subscribe was successful, INVAL if the buffer's length is not a multiple of the color depth length. 

