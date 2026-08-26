# Overview

The userspace service architecture enables application-layer code to offer service-level functionality to clients with a structured interface for invoking operations and communicating data. This allows both the service and the client to evolve and change independent of each other due to abiding by a well-defined interface.
Applications offering service-level functionality, userspace service applications, are available for use by the entire platform and run as application-layer code that is individually deployable just as other applications are.
The architecture achieves a decoupling of service code from both the OS and the consumer of the service.
Changes in the userspace service application do not cascade into requiring changes to client code.
Changes in the userspace service application code do not require an OS-level modification or update.

## Use case examples

- Swappable suites of cryptographic primitives, promoting cryptographic agility.
- Sensor fusion and soft sensing, synthesizing data from multiple sources as a new stream of data.
- Network stacks, connecting multiple applications while keeping networking logic out of the kernel.
- Data storage, providing a structured, on-device data store.

# Architecture

There are three major components to the userspace services architecture:

- the **registry**, which tracks and mediates communication with all userspace service applications;
- the **userspace service application**, which offers service-level functionality from userspace;
- and the **service interface**, which maps HIL functions to calls to the userspace application.

These components build on Tock’s design for applications, capsules, HILs, syscalls, and upcalls to realize userspace services.
Below is an overview diagram of the architecture depicting the three major components of userspace services and a client userspace application using the userspace service.

```
         +--------------------+              +-------------------+
         | Client Application |              | Userspace Service |
         +--------------------+              +-------------------+
                   |                             |        ^ |
                   | Syscalls       registration |        | | usercall invocations and returns
                   |           (command syscall) |        | | (upcalls and syscalls)
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

## The Registry

Central to userspace services is the userspace service registry.
The registry is a capsule that tracks, manages, and mediates communication with all userspace service applications on the platform.
Userspace service applications announce their availability to the platform by registering with the Registry.
Communication with the userspace service application occurs through syscalls (for communication from the userspace service application to the Registry)
and upcalls (for communication from the Registry to the userspace service application).

## Userspace Service Applications

Userspace service applications implement service-level functionality and make that functionality available to clients on the platform.
The userspace service application may be offering a suite of cryptographic operations, hosting a configuration store, or calibrating sensor data among other possibilities.
They enable sharing common, necessary functions; isolating less stable code from an application; writing modular, swappable components; etc.
Userspace service applications install and run just as any other Tock application.

A userspace service application registers and communicates with the Registry to execute operations (usercalls) the userspace service application offers.
At startup, the userspace service application registers with the Registry to announce itself, thereafter awaiting invocations of usercalls to perform work.
The userspace service application indirectly communicate with its clients through the Registry by using command and buffer allow syscalls with the Registry.

## Service Interface

Userspace services offer operations that directly match HIL functions to provide a stable, structured interface to clients consuming the userspace service.
However, because the userspace service application runs in userspace and uses the syscall interface which does not trivially translate into calls to HIL trait functions,
service interfaces implement the HIL interface instead. The service interface maps HIL trait function calls to communication with the userspace service application
(through function calls to the Registry which will ultimately make upcalls to and receive syscalls from the userspace service application).
The service interface is specific to a HIL.

# Data Flow

## Roles and Operations

In order to identify both userspace service applications and the functions they perform across userspace and the kernel,
the userspace services architecture uses role IDs and operation IDs to specify them.
A role describes the HIL trait functionality that a userspace service application implements.
Each role has an identifier that the Registry and service interface both use to specify what userspace service application their communication concerns.
A role ID maps to a single instance of an application running.
An operation describes a specific HIL trait function that a userspace service application can execute.
It is an identifier that is unique among the other functions on that specific HIL trait.
Together, the role ID and operation ID uniquely specify an invocation (to request that the Registry make or to assist in identifying the origin of a callback).

## Userspace Service Application ↔ The Registry

All communication with a userspace service occurs through the Registry.
These two entities communicate through syscalls (for communication from the userspace service to the Registry)
and upcalls (for communication from the Registry to the userspace service application).

```
         +--------------------+              +-------------------+
         | Client Application |              | Userspace Service |
         +--------------------+              +-------------------+
                                                 |        ^ |
                                    registration |        | | usercall invocations and returns
                               (command syscall) |        | | (upcalls and syscalls)
                                                 v        | v
    =================================================================
      KERNEL                                     |        ^ |
                                                 |        | |
         +--------------------+                  |        | |
         |      Capsule       |                  |        | |
         +--------------------+                  |        | |
                                                 |        | |
                                                 |        | |
              +---------+                        |        | |
              |HIL trait|                        |        | |
         +----+---------+------------------+     |        | |
         |        Service Interface        |     |        | |
         +---------------------------------+     |        | |
              |UserspaceServiceClient trait|     |        | |
              +----------------------------+     |        | |
                                                 |        | |
                                                 |        | |
         +----------------------------+          |        | |
         |UserspaceServiceAccess trait|          v        | v
         +----------------------------------------------------------+
         |                          Registry                        |
         +----------------------------------------------------------+
```

At startup, the userspace service application registers with the Registry with the register command syscall which also identifies the userspace service application’s role with a role ID.
This registration is only necessary once, at startup.
Its return value indicates the success or failure of the registration.

Executing an operation with the userspace service application uses the command, read-write allow, and read-only allow syscalls, as well as upcalls.
The Registry signals to the userspace service application to initiate an operation by first copying arguments for the operation into the userspace service application’s read-write allow buffers
and then issuing a single upcall to the userspace service application.
Upon receiving the upcall, the userspace service reclaims its read-write allow buffers, executes the operation, populates its read-only allow buffers, offers its read-write and read-only buffers to the Registry,
and then signals the operation’s completion (success or failure) with a single command syscall.

## The Registry ↔ Service Interface

The service interface and Registry interact through a pair of traits, `UserspaceServiceAccess` and `UserspaceServiceClient`,
using them to communicate data between the service interface’s HIL client and the userspace service.

```
         +--------------------+              +-------------------+
         | Client Application |              | Userspace Service |
         +--------------------+              +-------------------+




    =================================================================
      KERNEL

         +--------------------+
         |      Capsule       |
         +--------------------+

              +---------+
              |HIL trait|
         +----+---------+------------------+
         |        Service Interface        |
         +---------------------------------+
            | |UserspaceServiceClient trait|
            | +----------------------------+
 usercall() |                            ^
            v            usercall_done() |
         +----------------------------+  |
         |UserspaceServiceAccess trait|  |
         +----------------------------------------------------------+
         |                          Registry                        |
         +----------------------------------------------------------+
```

Upon receiving a function call from its HIL client, the service interface maps arguments received from the HIL client to arguments to pass to the userspace service (through the Registry).
The service interface achieves this by either serializing bytes (for intrinsic data types that fit in a `usize`)
or passing the buffers containing data received from the HIL client.
The Registry receives this data through a call to the `usercall` function on the `UserspaceServiceAccess` trait.

When a userspace service operation completes, the Registry makes a callback to the service interface through the `UserspaceServiceClient` trait function `usercall_done`.
The `usercall_done` function provides the service interface implementing it with access to the read-only buffers provided by the userspace service.
These buffers contain the serialized data returned by the userspace service.
The service interface provides these results to its HIL client through HIL-client-specific functions.

# Implementing a Userspace Service

Creating a new userspace service requires writing the userspace service application to implement the operations of the service
and writing the service interface to map HIL function calls to userspace service operations.

## Writing a Service Interface

The service interface must implement at least two traits:
the HIL trait that the userspace service application implements functionality for and the `UserspaceServiceClient` trait.
By implementing the HIL, the service interface can act as any other HIL for a capsule in Tock
and transparently pass commands and data to the userspace service application on behalf of the HIL client.
By implementing the `UserspaceServiceClient` trait, the service interface can receive callbacks from the Registry
as the service interface communicates with the userspace service application (through the Registry).

The service interface’s HIL trait implementation differs because each HIL trait is different,
but each HIL trait function will generally invoke an operation in the userspace service application.
The service interface uses the Registry’s `UserspaceServiceAccess::usercall` function to invoke an operation in the userspace service application.
This call to `UserspaceServiceAccess::usercall` specifies the data to send to the userspace service application,
either serializing data or copying buffers to the userspace service application’s read-write allow buffers.

The `UserspaceServiceClient` trait is the means by which the service interface asynchronously receives the final result of a userspace service application operation.
It has a single function, `UserspaceServiceClient::usercall_done`, which indicates completion (in both success or failure) of the operation.
In this callback, the service interface can retrieve the resulting data and, in turn, initiate a callback to its HIL client.

## Writing a Userspace Service

At startup, the userspace service application must register its availability with the Registry and offer read-write buffers for receiving argument data.
First, the userspace service application must use read-write allow syscalls to make its buffers available.
The number and size of these buffers will depend on what is necessary for the userspace service application to fulfill its purpose.
Once it provides the read-write buffers to the kernel, the userspace service application should then register with the Registry to announce its availability, providing its role ID and its callback function to receive invocations.
At any point after successful registration, the userspace service may receive invocations of usercalls.

The userspace service application must follow a consistent procedure to receive and respond to usercalls.
After completing the prior described startup sequence, the userspace service application awaits usercall invocations and its read-write argument data buffers are held by the kernel.
A usercall invokes the userspace service application’s defined usercall callback to signal to the userspace service application to perform an operation.
The userspace service application must always follow this sequence:

1. **Reclaim the read-write allow buffers previously offered to the kernel.**
   These contain the argument data that the client is providing to the userspace service application.
2. **Execute the operation the client has invoked.**
   The client provides the userspace service application with argument data provided as direct arguments to the callback function and in the read-write buffers the userspace service application reclaims from the kernel.
3. **Prepare return result data.**
   The userspace service application can return up to two `usize`-sized values and additional data through buffers (separate from the read-write argument buffers).
   Write all data to return to the client through buffers into their respective buffers and provide these buffers to the kernel using read-only allow syscalls.
4. **Make the read-write argument data buffers available to the kernel.**
   These buffers must be immediately available following the completion of the operation.
   Use read-write allow syscalls to make these buffers available to the Registry again.
5. **Signal completion of the operation, both success or failure, back to the Registry using the command syscall.**
   Values the userspace service application is directly returning to the client are arguments to this command syscall.

This sequence implicitly defines a loop, preparing the userspace service application to be available to execute another operation immediately.

## Board Configuration

The board configuration (in the board’s `main.rs` file) for a device defines the OS-side integration of the userspace service.
The board configuration must include a few changes to enable the userspace service.

- **Add the userspace services Registry capsule.**
  The board struct must include the Registry capsule.
  The `SyscallDriverLookup` for the Registry capsule must also include the Registry as a driver option to enable the Registry to receive syscalls.
  This completes the communication path between the userspace service application and the Registry.
- **Initialize the service interface.**
  Perform initialization of the userspace service’s service interface, providing a reference to the Registry.
  This completes the communication path between the Registry and the service interface.
- **Use the service interface as the HIL implementor.**
  Specify the service interface as the HIL implementor to a consuming capsule.
  This completes the communication path between the service interface and the HIL client.
  For capsules offering a syscall interface to userspace, this also completes the communication path between userspace client applications and the service interface, making the userspace service usable for userspace applications.
