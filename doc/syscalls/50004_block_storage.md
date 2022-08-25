---
driver number: 0x50004
---

# Block storage

## Overview

Exposes a block-based nonvolatile storage device.

The device is formed from equally-sized storage blocks,
which are arranged one after another, without gaps or overlaps,
to form a linear storage of bytes.

The device is split into blocks in two ways, into:
- discard blocks, which are the smallest unit of space
that can be discarded
- write blocks, which are the smallest unit of space that can be written

Every byte on the device belongs to exactly one discard block,
and to exactly one write block at the same time.

Blocks must be discarded before every write.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) if it exists, otherwise NODEVICE

  * ### Command number: `1`

    **Description**: Returns device size in bytes.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U64 with size.

  * ### Command number: `2`

    **Description**: Returns geometry.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: A pair of u32 values: first being the size in bytes of the read/write block, the second being the size in bytes of the discard block.

  * ### Command number: `3`

    **Description**: Reads bytes from the given address range into the READ AllowRW buffer. Calls back the READ subscriber.

    **Argument 1**: Start address

    **Argument 2**: Number of bytes to read

    **Returns**: Ok(()) followed by the callback if the requested bytes lie within the device, INVAL if not, BUSY if another command is in progress.

  * ### Command number: `4`

    **Description**: Reads bytes from the given block into the READ AllowRW buffer. Calls back the READ subscriber.

    **Argument 1**: Read/write block number

    **Argument 2**: unused

    **Returns**: Ok(()) followed by the callback if the requested block lies within the device, INVAL if not, BUSY if another command is in progress.
  
  * ### Command number: `5`

    **Description**: Discards the given block. Calls back the DISCARD subscriber.

    **Argument 1**: Discard block index

    **Argument 2**: unused

    **Returns**: Ok(()) followed by the callback if the requested block lies within the device, INVAL if not, BUSY if another command is in progress.

  * ### Command number: `6` 

    **Description**: Writes bytes from the WRITE AllowRO buffer into the given block. Calls back the WRITE subscriber.

    **Argument 1**: Read/write block index

    **Argument 2**: unused

    **Returns**: Ok(()) followed by the callback if the requested block lies within the device, INVAL if not, BUSY if another command is in progress.

## Subscribe

The first argument of all callbacks is equal to `1` on errors, `0` on success.
Error code, if applicable, is stored in the second argument.

  * ### READ, number: `0`

    **Description**: Read completion notifications.

    **Callback signature**: Result<(), ErrorCode>

    **Returns**: Ok(()) if the subscribe was successful.
    
* ### DISCARD, number: `1`

    **Description**: Discard completion notifications.

    **Callback signature**: Result<(), ErrorCode>

    **Returns**: Ok(()) if the subscribe was successful.

* ### WRITE, number: `2`

    **Description**: Write completion notifications.

    **Callback signature**: Result<(), ErrorCode>

    **Returns**: Ok(()) if the subscribe was successful.
    
## Allow ReadOnly

  * ### WRITE, number: `0`

    **Description**: Sets a shared buffer to be used as the source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined.

    **Returns**: Ok(()) if the allow was successful.

## Allow ReadWrite

  * ### READ, number: `0`

    **Description**: Sets a shared buffer to be used as the destination for data from
    the next read transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a read transaction is completed. Replacing
    the buffer after beginning a read transaction but before receiving a
    completion callback is undefined.

    **Returns**: Ok(()) if the allow was successful.
