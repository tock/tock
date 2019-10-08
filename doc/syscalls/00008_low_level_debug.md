---
driver number: 0x00008
---

# Low-Level Debug

## Overview

The low-level debug driver provides tools to diagnose userspace issues that make
normal debugging workflows (e.g. printing to the console) difficult. It allows
apps to print status codes and numeric information using only the `command`
system call, and is easy to call from handwritten assembly. The driver is in
capsules/src/low\_level\_debug.rs.

## Command

  * Description: command() is used to print status codes and numbers. The driver
    does not provide a way for an app to wait for the print completed. If the
    app prints too many messages in a row, the driver will print a message
    indicating it has dropped some debug messages.

  * ### Command Number: 0

    **Description**: Driver check.

    **Argument 1**: Unused

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 1

    **Description**: Print a predefined status code. The available status codes
    are listed later in this doc.

    **Argument 1**: Status code to print

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 2

    **Description**: Print a single number. The number will be printed in
    hexadecimal.

    **Argument 1**: Number to print

    **Argument 2**: Unused

    **Returns**: SUCCESS

  * ### Command Number: 3

    **Description**: Print two numbers. The numbers will be printed in
    hexadecimal.

    **Argument 1**: First number to print

    **Argument 2**: Second number to print

    **Returns**: SUCCESS

## Predefined Status Codes

The following status codes are defined for use with the predefined status code
command (\#1). As an alternative to this table, the binary in
tools/status\_codes may be used to decode the status codes.

| Status Code | Description                                                                |
|-------------|----------------------------------------------------------------------------|
| 0x01        | Application panic (e.g. panic!() called in Rust code)                      |
| 0x02        | A statically-linked app was not installed in the correct location in flash |
