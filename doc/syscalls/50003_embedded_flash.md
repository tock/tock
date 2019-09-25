---
driver number: 0x50003
---

# Embedded flash

## Overview

The embedded flash driver provides low-level control of the writeable flash
regions of the application.

## Command

  * ### Command number: `0`

    **Description**: Check if the driver is present on the board.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `SUCCESS` if the driver is present on the board, `ENODEVICE`
    otherwise.

  * ### Command number: `1`

    **Description**: Get information about the driver.

    **Argument 1**:
      - 0: Get the word size.
      - 1: Get the page size.
      - 2: Get the maximum number of word writes between page erasures.
      - 3: Get the maximum number page erasures in the lifetime of the flash.

    **Argument 2**: unused

    **Returns**: The corresponding value if the first argument is valid,
    `EINVAL` otherwise.

  * ### Command number: `2`

    **Description**: Write a slice to a flash region. Before calling this
    command, a slice to read from should be allowed (see allow number 0).

    **Argument 1**: The start address of the flash region to write to. The
    length is defined by the length of the allow slice. The address and length
    must be word-aligned. The flash region must be in a writeable flash region.

    **Argument 2**: unused

    **Returns**: `SUCCESS` if the slice was successfully written, `EINVAL`
    otherwise.

  * ### Command number: `3`

    **Description**: Erase a page.

    **Argument 1**: The start address of the flash page to erase. The length is
    implicitly the page size. The flash page must be in a writeable flash
    region.

    **Argument 2**: unused

    **Returns**: `SUCCESS` if the page was successfully erased, `EINVAL`
    otherwise.

## Subscribe

Unused for the embedded flash driver. Will always return `ENOSUPPORT`.

## Allow

  * ### Allow number: `0`

    **Description**: The slice to read from when writing to a flash region.
    Should be called before calling command number 2.

    **Argument**: The slice whose content should be written to flash. The
    slice should be in RAM (until [#1274] is fixed) and will not be modified.

    **Returns**: `SUCCESS` if the slice was successfully allowed, an error
    otherwise.

[#1274]: https://github.com/tock/tock/issues/1274
