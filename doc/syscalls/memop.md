# Memop

## Overview

`memop` is a core Tock syscall. Most memop syscalls are read-only information
such as where a process was loaded in flash and ram. Processes also use memop
to grow the application heap (`brk` and `sbrk`) or to provide optional
debugging information such as the top of the stack for processes that manage
their own stack.

All memop calls pass an operation type as the first parameter. Some include
an argument in the second parameter:

```rust
memop(op_type: u32, argument: u32) -> [[ VARIES ]] as u32
```

## Memory Operations

  * ### Operation type `0`: `brk`

    **Description**: Change the location of the program break to the absolute
    address provided.

    **Argument 1** `as *u8`: Address of the new program break (aka maximum
    accessible value).

    **Returns** `ReturnCode as u32`: `SUCCESS` or `ENOMEM`.

  * ### Operation type `1`: `sbrk`

    **Description**: Move the program break up or down by the specified number
    of bytes.

    **Argument 1** `as i32`: Number of bytes to move the program break.

    **Returns** `as *u8`: The previous program break (the start of the newly allocated memory) or `ENOMEM`.

  * ### Operation type `2`: Memory start

    **Description**: Get the address of the start of the application's RAM
    allocation.

    **Argument 1**: unused

    **Returns** `as *u8`: The address.

  * ### Operation type `3`: Memory end

    **Description**: Get the address pointing to the first address after the
    end of the application's RAM allocation.

    **Argument 1**: unused

    **Returns** `as *u8`: The address.

  * ### Operation type `4`: Flash start

    **Description**: Get the address of the start of the application's flash
    region. This is where the TBF header is located.

    **Argument 1**: unused

    **Returns** `as *u8`: The address.

  * ### Operation type `5`: Flash end

    **Description**: Get the address pointing to the first address after the
    end of the application's flash region.

    **Argument 1**: unused

    **Returns** `as *u8`: The address.

  * ### Operation type `6`: Grant start

    **Description**: Get the address of the lowest address of the grant region
    for the app. (Note: the grant end is by definition the memory end, so there
    is no corresponding grant end syscall.)

    **Argument 1**: unused

    **Returns** `as *u8`: The address.

  * ### Operation type `7`: Flash regions

    **Description**: Get the number of writeable flash regions defined in the
    header of this app.

    **Argument 1**: unused

    **Returns** `as u32`: The number of regions.

  * ### Operation type `8`: Flash region start address

    **Description**: Get the start address of the writeable region indexed
    from 0.

    **Argument 1** `as u32`: Which region.

    **Returns** `as *u8`: The start address of the selected region, or `(void*)
    -1` if the requested region does not exist.

  * ### Operation type `9`: Flash region end address

    **Description**: Get the end address of the writeable region indexed
    from 0.

    **Argument 1** `as u32`: Which region.

    **Returns** `as *u8`: The address immediately after the selected region, or
    `(void*) -1` if the requested region does not exist.

  * ### Operation type `10`: (debug) Specify stack location

    **Description**: Specify the top of the application stack.

    **Argument 1** `as *const u8`: Address of the stack top.

    **Returns** `ReturnCode as u32`: Always `SUCCESS`.

  * ### Operation type `11`: (debug) Specify heap location

    **Description**: Specify the start of the application heap.

    **Argument 1** `as *const u8`: Address of the heap start.

    **Returns** `ReturnCode as u32`: Always `SUCCESS`.
