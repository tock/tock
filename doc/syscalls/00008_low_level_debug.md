---
driver number: 0x00008
---

# Low-Level Debug

## Overview

The low-level debug driver provides tools to diagnose userspace issues that make
normal debugging workflows (e.g. printing to the console) difficult. It allows
libraries to print alert codes and apps to print numeric information using only
the `command` system call, and is easy to call from handwritten assembly. The
driver is in capsules/src/low\_level\_debug.rs.

## Command

  * Description: command() is used to print alert codes and numbers. The driver
    does not provide a way for an app to wait for the print to complete. If the
    app prints too many messages in a row, the driver will print a message
    indicating it has dropped some debug messages.

  * ### Command Number: 0

    **Description**: Driver check.

    **Argument 1**: Unused

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 1

    **Description**: Print a predefined alert code. The available alert codes
    are listed later in this doc. Predefined alert codes are intended for use in
    library code, and are defined here to avoid collisions between projects.

    **Argument 1**: Alert code to print

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 2

    **Description**: Print a single number. The number will be printed in
    hexadecimal. In general, this should only be added temporarily for debugging
    and should not be called by released library code.

    **Argument 1**: Number to print

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 3

    **Description**: Print two numbers. The numbers will be printed in
    hexadecimal. Like command 2, this is intended for temporary debugging and
    should not be called by released library code. If you want to print multiple
    values, it is often useful to use the first argument to indicate what value
    is being printed.

    **Argument 1**: First number to print

    **Argument 2**: Second number to print

    **Returns**: SUCCESS

## Predefined Alert Codes

The following alert codes are defined for use with the predefined alert code
command (\#1). As an alternative to this table, the binary in tools/alert\_codes
may be used to decode the alert codes.

| Alert Code | Description                                                                |
|------------|----------------------------------------------------------------------------|
| 0x01       | Application panic (e.g. panic!() called in Rust code)                      |
| 0x02       | A statically-linked app was not installed in the correct location in flash |
