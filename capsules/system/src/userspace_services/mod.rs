// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

/*! Services in userspace.

This module provides a framework for running service-level functionality as applications in userspace,
offering a middle-ground between adding or modifying a capsule and bundling code within a single application.
The service application is available for use by the entire platform and is separately deployable.
Since the code implementing the service exists in an application,
changing the service's operation does not require an OS-level modification and update.
More specifically,
decoupling particular operations from the application
while simultaneously avoiding OS-level implementation enables:
sharing common, necessary function;
isolating less stable code from an application;
writing modular, swappable components;
etc.

# Architecture

Support for userspace services builds on capsules and HILs.
Central to this framework is the **userspace service registry** capsule ([`Registry`]),
which tracks and mediates communication with userspace service applications.
Userspace services register with the registry by sending it a syscall,
thereafter communicating exclusively with the registry to fulfill its function.
Coordinated syscalls and upcalls between the registry and userspace service application,
*usercalls*,
define the operations the userspace service exposes.

In order to use the userspace service,
clients interact with a **service interface** that implements a HIL defining its function.
Acting as a mapper between HIL functions and usercalls,
the service interface invokes userspace service operations through the userspace service registry.
The two communicate through the [`UserspaceServiceClient`] and [`UserspaceServiceAccess`] traits
to send data between the consumer of the userspace service and the userspace service.

Because service interfaces implement a HIL trait,
existing capsules can consume them and,
in turn,
offer the functionality of the userspace service to other userspace applications transparently through their syscall driver definition.
The userspace service application can be transparently updated and swapped out without changing the OS.

The following diagram gives a visual overview of the architecture and communication flow:
```text
        +--------------------+              +-------------------+
        | Client Application |              | Userspace Service |
        +--------------------+              +-------------------+
                  |                             |        ^ |
                  | Syscalls         register() |        | | usercalls and returns
                  |                     syscall |        | | (upcalls and syscalls)
                  v                             v        | v
   =================================================================
     KERNEL       |                             |        ^ |
                  v                             |        | |
        +--------------------+                  |        | |
        |      Capsule       |                  |        | |
        +--------------------+                  |        | |
                  |                             |        | |
                  v                             |        | |
             +---------+                        |        | |
             |HIL trait|                        |        | |
        +----+---------+------------------+     |        | |
        |        Service Interface        |     |        | |
        +---------------------------------+     |        | |
           | |UserspaceServiceClient trait|     |        | |
           | +----------------------------+     |        | |
usercall() |                            ^       |        | |
           v            usercall_done() |       |        | |
        +----------------------------+  |       |        | |
        |UserspaceServiceAccess trait|  |       v        | v
        +----------------------------------------------------------+
        |                          Registry                        |
        +----------------------------------------------------------+
```

# Communication

Usercalls and their returns combine
syscalls,
upcalls,
and buffer `allow`s
to move data between the userspace service application and its client.
To invoke a userspace service operation,
the registry sends an upcall to the userspace service application,
placing arguments as upcall arguments
or in the service application's read-write `allow` buffers.
The first argument of the upcall is an **operation ID** identifying the operation the client is requesting the userspace service run.
The userspace service application returns the result of the operation with a syscall to the registry,
placing return data as syscall arguments
or in its read-only `allow` buffers.

To support more than a single userspace service application,
and to disambiguate interactions of one userspace service from another,
the registry addresses each userspace service with a unique **role ID**.
The role ID identifies the userspace service's function at a high level,
for example,
a role providing cryptographic hashing.
When registering,
the userspace service provides the registry with its role ID.
No two userspace service applications running on the same device may fulfill the same role.

# Writing a Userspace Service

To function as a userspace service,
an application must do the following:

1. **`allow_*` argument and return data buffers.**
The userspace service application and registry move data between each other through buffers in the userspace service application's memory space.
The userspace service application should `allow_readwrite()` its argument buffers
and `allow_readonly()` its return data buffer(s).

2. **Register at startup.**
A userspace service announces its availability through the registration `command` syscall to the registry,
command no. `0x10`,
providing its role ID as the first and only argument.
In the event that the userspace service application crashes,
it must re-register with the registry.

2. **`subscribe()` to incoming operations.**
Essentially,
a userspace service is an application that issues a syscall in response to an upcall.
The registry capsule defines a single upcall,
and its arguments differentiate the intent of the upcall from others.
The userspace service application should `subscribe()` to subscription no. 0 to receive the upcalls.

3. **Respond to upcalls with a `command()`.**
The userspace service application must issue a `command` in return for each upcall it receives from the registry.
The `command` indicates either
success (command no. `0x20`)
or failure (command no. `0x21`).
The two arguments of the success `command` can carry return data.
If the userspace service application is returning additional data through its buffer(s),
it should `allow_readonly()` the buffer(s) prior to issuing the return success `command`.
The failure `command` should return an error code as its first argument.
 */

pub mod data;
pub mod grant;
pub mod registry;
pub mod services;
pub mod usercall;
