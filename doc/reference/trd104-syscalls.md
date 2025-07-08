System Calls
========================================

**TRD:** 104 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Hudson Ayers, Guillaume Endignoux, Jon Flatley, Philip Levis, Amit Levy, Pat Pannuto, Leon Schuermann, Johnathan Van Why, dcz <br/>
**Draft-Created:** August 31, 2020<br/>
**Draft-Modified:** January 29, 2025<br/>
**Draft-Version:** 11<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the system call application binary interface (ABI)
between user space processes and the Tock kernel for 32-bit ARM Cortex-M
and RISC-V RV32I platforms.

1 Introduction
===============================

The Tock operating system can run multiple independent userspace applications.
Each application image is a separate process: it has its own address space
and thread stack.  Because applications are untrusted, the kernel uses hardware
memory protection to isolate the kernel from processes. This allows applications
written in C (or even assembly) to safely run on Tock.  Applications invoke
operations on and receive upcalls from the Tock kernel through the system call
programming interface.

This document describes Tock's system call programming interface (API)
and application binary interface (ABI) for 32-bit ARM Cortex-M and
RISC-V RV32I platforms. It describes the system calls that Tock
implements, their semantics, and how a userspace process invokes
them. The ABI for other architectures, if supported, will be described
in other documents.

2 Design Considerations
===============================

Three design considerations guide the design of Tock's system call API and
ABI.

  1. Tock is currently supported on the ARM CortexM and RISC-V architectures.
  It may support others in the future. Its ABI must support both architectures
  and be flexible enough to support future ones.
  2. Tock userspace applications can be written in any language. The system
  call API must support their calling semantics in a safe way. Rust is
  especially important.
  3. Both the API and ABI must be efficient and support common call
  patterns in an efficient way.

2.1 Architectural Support and ABIs
--------------------------------

The primary question for the ABI is how many and which registers transfer
data between the kernel and userspace. Passing more registers has the benefit
of the kernel and userspace being able to transfer more information
without relying on pointers to memory structures. It has the cost of requiring
every system call to transfer and manipulate more registers.

2.2 Programming Language APIs
---------------------------------

Userspace support for Rust is an important requirement for Tock. A key
invariant in Rust is that a given memory object can either have multiple
references or a single mutable reference. If userspace
passes a writeable (mutable) buffer into the kernel, it must relinquish
any references to that buffer. As a result, the only way for userspace
to regain a reference to the buffer is for the kernel to pass it back.

2.3 Efficiency
---------------------------------

Programming language calling conventions are
another consideration because they affect efficiency. For
example, the C calling convention in ARM says that the first four arguments
to a function are stored in r0-r3. Additional arguments are stored on
the stack. Therefore, if the system call ABI says that arguments are stored
in different registers than r0-r3, a C function call that invokes a system
call will need to move the C arguments into those registers.

3 System Call ABI
=================================

This section describes the ABI for Tock on 32-bit platforms, including
the exact register mappings for the CortexM and 32-bit RISC-V architectures.
The ABI for 64-bit platforms is currently undefined but may be specified in
a future TRD. The register mappings for future 32-bit architectures can be
specified in supplemental TRDs.

3.1 Registers
---------------------------------

When userspace invokes a system call, it passes 4 registers to the
kernel as arguments. It also pass an 8-bit value of which type of
system call (see Section 4) is being invoked (the Syscall Class
ID). When the system call returns, it returns 4 registers as return
values. When the kernel invokes an upcall on userspace, it passes 4
registers to userspace as arguments and has no return value.

|                        | CortexM | RISC-V |
|------------------------|---------|--------|
| Syscall Arguments      | r0-r3   | a0-a3  |
| Syscall Return Values  | r0-r3   | a0-a3  |
| Syscall Class ID       | svc     | a4     |
| Upcall Arguments       | r0-r3   | a0-a3  |
| Upcall Return Values   | None    | None   |

How registers are mapped to arguments can affect performance and code size.
For system calls implemented by capsules and drivers (`command`, `subscribe`,
and `allow`), arguments that are passed to these calls should be placed
in the same registers that will be used to invoke those calls. This allows
the system call handlers in the kernel to pass them unchanged, rather than
have to move them between registers.

For example, `command` has this signature:

```rust
fn command(&self, minor_num: usize, r2: usize, r3: usize, caller_id: AppId) -> Result<(), ErrorCode>
```

This means that the value which will be passed as `r2` to the command
should be placed in register r2 when userspace invokes the system
call. That way, the system call handler can just leave register r2
unchanged. If, instead, the argument `r2` were passed in register r3,
the system call handler would have to spend an instruction moving
register r3 to register r2.

Driver system call implementations in the Tock kernel typically pass a reference
to `self` as their first argument. Therefore, `r0` is usually used to dispatch
onto the correct driver; this argument is consumed by the system call handler
and replaced with `&self` when the actual system call method is invoked.

3.2 Syscall Argument Types
----------------------------------

The use of registers, and the selection of specific registers, imposes physical
restrictions on the types which can be passed (i.e., this TRD defines the ABI
for platforms with 32-bit, general purpose registers, which limits any single
argument to a maximum of 32-bit storage for its type).

Syscalls SHOULD define as specific a type as is semantically meaningful for
these 32 bits of storage. For example, the `command` syscall uses register 0 as
a Driver Number and register 1 as a Command Number. These are defined as a
`u32`, as Tock expects no more than 2^32 drivers, each with no more than 2^32
commands. A future 64-bit ABI MAY choose to combine these into one syscall
register. In contrast, other syscall argument slots may have more generic
purposes, where the type and underlying storage are likely to move in sync on
future platforms. This document defines the following conceptual types:

<table>
  <tr>
    <th>Type</th>
    <th>C Analog</th>
    <th>Description</th>
  </tr>
  <tr>
     <td><tt>OPAQUE_NUMERIC</tt></td>
     <td><tt>(unsigned) int</tt></td>
     <td>A whole number which is capable of being expressed in a native machine word type.</td>
  </tr>
  <tr>
    <td><tt>SIZE</tt><br/><tt>OPAQUE_SIZE</tt></td>
    <td><tt>size_t</tt></td>
    <td>An unsigned, numeric type with a range capable of expressing in bytes the
   size of any valid <em>contiguous</em> object.</td>
  </tr>
  <tr>
    <td><tt>POINTER_OR_ZERO</tt><br/><tt>OPAQUE_POINTER_OR_ZERO</tt></td>
    <td><tt>void *</tt></td>
    <td>A type capable of holding a pointer to a valid memory location OR the
    value `0`, which is defined as the NULL sentinel.</td>
  </tr>
  <tr>
    <td><tt>C_FUNCTION_POINTER_OR_ZERO</tt></td>
    <td></td>
    <td>This describes an argument which holds a pointer to an executable
    function, where the function adheres to the standard C ABI for the platform
    (akin to <tt>extern C</tt> in C++ or <tt>extern "C" fn</tt> in Rust) OR the
    value <tt>0</tt>, which is defined as the NULL sentinel.</td>
  </tr>
  <tr>
    <td><tt>OPAQUE_GENERIC</tt></td>
    <td></td>
    <td>This describes an argument with no restrictions on type.</td>
  </tr>
</table>

Types prefixed with `OPAQUE` are those where the kernel SHOULD NOT attempt to
read the value. `OPAQUE` types are generally for pass-through use cases, where
userspace passes a value back to itself through the kernel.  `OPAQUE` types,
including `OPAQUE_GENERIC`, MAY have additional metadata associated with them
(e.g. pointer provenance or tags) which the kernel MUST preserve.

_Note:_ There is no `NUMERIC` type, only `OPAQUE_NUMERIC`, as any use by the
kernel of machine-width generic number type almost certainly invites the design
of an ABI which is not portable across architectures.

_Note:_ Historically, Rust conflated some of these concepts with `usize`,
however, legacy platforms (e.g., those with segmented memory) as well as
emerging security-oriented platforms (e.g., those with tagged pointers) require
distinguishing these cases.


3.3 Return Values
----------------------------------

All system calls have the same return value format. A system call can
return one of several variants, having different associated value types,
which are shown here. `r0`-`r3` refer to the return value registers:
for CortexM they are `r0`-`r3` and for RISC-V they are `a0`-`a3`.

| System call return variant | `r0` | `r1`               | `r2`               | `r3`               |
|----------------------------|------|--------------------|--------------------|--------------------|
| Failure                    | 0    | Error code         | -                  | -                  |
| Failure with u32           | 1    | Error code         | Return Value 0     |                    |
| Failure with 2 u32         | 2    | Error code         | Return Value 0     | Return Value 1     |
| Failure with u64           | 3    | Error code         | Return Value 0 LSB | Return Value 0 MSB |
| Success                    | 128  |                    |                    |                    |
| Success with u32           | 129  | Return Value 0     |                    |                    |
| Success with 2 u32         | 130  | Return Value 0     | Return Value 1     |                    |
| Success with u64           | 131  | Return Value 0 LSB | Return Value 0 MSB |                    |
| Success with 3 u32         | 132  | Return Value 0     | Return Value 1     | Return Value 2     |
| Success with u32 and u64   | 133  | Return Value 0     | Return Value 1 LSB | Return Value 1 MSB |

There are many failure and success variants because
different system calls need to pass different amounts of data. A
command that requests a 64-bit timestamp, for example, needs its
success to return a `u64`, but its failure can return nothing. In
contrast, a system call that passes a pointer into the kernel may have
a simple success return value but requires a failure with one 32-bit
value so the pointer can be passed back.

Every system call MUST return only one failure and only one success
variant. Different system calls may use different failure and success
variants, but any specific system call returns exactly one of each. If an
operation might have multiple success return variants or failure return
variants, then it MUST be split into multiple system calls.

This requirement of a single failure variant and a single success
variant is to simplify userspace implementations and preclude them
from having to handle many different cases.  The presence of many
difference cases suggests that the operation should be split up, as
there is non-determinism in its execution or its meaning is
overloaded. The requirement of a single failure and a single success
variant also fits well with Rust's `Result` type.

If userspace tries to invoke a system call that the kernel does not
support, the system call will return a Failure result with an error
code of `NODEVICE` or `NOSUPPORT` (Section 4). As the Allow and
Subscribe system call classes have defined Failure types, the kernel can
produce the expected type with known failure variants. Command, however,
can return any variant. This means that Commands can appear to have
two failure variants: the one expected (e.g., Failure with u32) as well
as Failure. To avoid this ambiguity for `NODEVICE`, userspace can use the
reserved "exists" command (Command Identifier 0), described in Section
4.3.1. If this command returns Success, the driver is installed and will
not return a Failure with `NODEVICE` for Commands. The driver may still
return `NOSUPPORT`, however. Because this implies a misunderstanding of
the system call API by userspace (it is invoking system calls that do not
exist), userspace is responsible for handling this case.

For all Failure types, the passed Error code MUST be placed in `r1`.

All 32-bit values not specified for `r0` in the above table are reserved.
Reserved `r0` values MAY be used by a future TRD and MUST NOT be returned by the
kernel unless specified in a TRD. Therefore, for future compatibility, userspace
code MUST handle `r0` values that it does not recognize.

3.4 Error Codes
---------------------------------

All system call failures return an error code. These error codes are a superset of
kernel error codes. They include all kernel error codes so errors from calls
on kernel HILs can be easily mapped to userspace system calls when suitable. There
are additional error codes to include errors related to userspace.

| Value | Error Code  | Meaning                                                                                 |
|-------|-------------|-----------------------------------------------------------------------------------------|
| 1     | FAIL        | General failure condition: no further information available.                            |
| 2     | BUSY        | The driver or kernel is busy: retry later.                                              |
| 3     | ALREADY     | This operation is already ongoing can cannot be executed more times in parallel.        |
| 4     | OFF         | This subsystem is powered off and must be turned on before issuing operations.          |
| 5     | RESERVE     | Making this call requires some form of prior reservation, which has not been performed. |
| 6     | INVALID     | One of more of the parameters passed to the operation was invalid.                      |
| 7     | SIZE        | The size specified is too large or too small.                                           |
| 8     | CANCEL      | The operation was actively cancelled by a call to a cancel() method or function.        |
| 9     | NOMEM       | The operation required memory that was not available (e.g. a grant region or a buffer). |
| 10    | NOSUPPORT   | The system call is not available to or not supported for the calling process.           |
| 11    | NODEVICE    | The driver specified by the driver number is not available to the calling process.      |
| 12    | UNINSTALLED | The resource was removed or uninstalled (e.g., an SD card).                             |
| 13    | NOACK       | The packet transmission was sent but not acknowledged.                                  |
| 1024  | BADRVAL     | The variant of the return value did not match what the system call should return.       |

Values in the range 1-1023 reflect kernel return value error
codes. Kernel error codes not specified above are reserved. TRDs MAY
specify additional kernel error codes from these reserved values, but
MUST NOT specify kernel error codes greater than 1023.  The Tock
kernel MUST NOT return an error code unless the error code is
specified in a TRD.

Values greater than 1023 are reserved for userspace library use. Value 1024
(BADRVAL) is for when a system call returns a different failure or success
variant than the userspace library expects.

3.5 Returning To Userspace
---------------------------------

When the kernel returns to userspace, it only gets to set registers for
one stack frame. In practice, we have two cases:

### Direct Resume

Userspace resumes execution directly after the `svc` invocation, so the
assembly that follows the `svc` command can use the values in r0-r3
as-set by the kernel.

### Pushed Callback

Userspace resumes execution at the start of the callback function.

The values in r0-r3 are consumed by the callback. When the callback
finishes, it will `pop {lr}` (or similar), where the link register in
the callback stack frame has been set by the kernel to the instruction
after the `svc` that relinquished control to the kernel.

The assembly that invoked the syscall now gets to run. At this point
r0-r3 are unknown as those are caller-save registers (which means the
Upcall callback can clobber them freely). The assembly that invoked the
`svc` cannot make any assumptions about the values in r0-r3, nor can the
kernel use them to pass things "to" the calling assembly. Thus, the
`PushedCallback` case has to use a pointer-based approach for the kernel
to communicate with the assembly that invokes the `svc` (e.g.
`yield-param-A` in `Yield-NoWait`).


4 System Call API
=================================

Tock has 7 classes or types of system calls. When a system call is
invoked, the class is encoded as the Syscall Class Number. Some system
call classes are implemented by the core kernel and so the supported
calls are the same across kernels. Others are implemented by system
call drivers, which can be added and removed in different kernel
builds. The full set of valid system calls a kernel supports therefore
depends on what system call drivers it has installed.

The 7 classes are:

| Syscall Class    | Syscall Class Number |
|------------------|----------------------|
| Yield            |           0          |
| Subscribe        |           1          |
| Command          |           2          |
| Read-Write Allow |           3          |
| Read-Only Allow  |           4          |
| Memop            |           5          |
| Exit             |           6          |

All of the system call classes except Yield and Exit are
non-blocking. When a userspace process calls a Subscribe, Command,
Read-Write Allow, Read-Only Allow, or Memop syscall, the kernel will not put the process
on a wait queue while handling the syscall. Instead, the kernel will complete the
syscall and prepare the return value for the syscall immediately.
The kernel scheduler may not, however, run the process immediately after
handling the syscall, and may instead decide to suspend the process due
to a timeslice expiration or the kernel thread being runnable.
If an operation is long-running
(e.g., I/O), its completion is signaled by an upcall (see the
Subscribe call in 4.2).

Successful calls to Exit system calls do not return (the process exits).

System calls implemented by system call drivers (Subscribe, Command,
Read-Write Allow, Read-Only Allow) all include two arguments, a driver number
and a syscall number. The driver number specifies which system
call driver to invoke. The syscall number (which is different than
the Syscall Class Number in the table above) specifies which instance of
that system call on that driver to invoke. Both arguments are unsigned
32-bit integers. For example, by convention the Console system call driver
has driver number `0x1` and a Command to the console driver with
syscall number `0x2` starts receiving console data into a buffer.

If userspace invokes a system call on a peripheral driver that is not
installed in the kernel, the kernel MUST return a Failure result with
an error of `NODEVICE`. If userspace invokes an unrecognized system call
on a peripheral driver, the peripheral driver MUST return a Failure
result with an error of `NOSUPPORT`.

4.1 Yield (Class ID: 0)
--------------------------------

The Yield system call class is how a userspace process handles
upcalls, relinquishes the processor to other processes, or waits for
one of its long-running calls to complete.  The Yield system call
class implements the only blocking system calls in Tock that return:
`Yield-Wait` and `Yield-WaitFor`.
The kernel invokes upcalls only in response to Yield
system calls.

There are three Yield system call variants:
  - Yield-Wait
  - Yield-NoWait
  - Yield-WaitFor

The register arguments for Yield system calls are as follows. The registers
r0-r3 correspond to r0-r3 on CortexM and a0-a3 on RISC-V.

| Argument               | Register | Type     |
|------------------------|----------|----------|
| Yield number           | r0       | `u32`    |
| yield-param-A          | r1       | _varies_ |
| yield-param-B          | r2       | _varies_ |
| yield-param-C          | r3       | _varies_ |

The Yield number (in r0) specifies which call is invoked:

| System call     | Yield number value |
|-----------------|--------------------|
| yield-no-wait   |                  0 |
| yield-wait      |                  1 |
| yield-wait-for  |                  2 |

All other yield number values are reserved. If an invalid yield number
is passed the kernel MUST return immediately and MUST NOT
use `yield-param-A`, `yield-param-B`, or `yield-param-C`.

The meaning of `yield-param-X` is specific to the yield type.


### 4.1.1 Yield-NoWait

| Argument               | Register | Type           | Value                                            |
|------------------------|----------|----------------|--------------------------------------------------|
| Yield number           | r0       | `u32`          | `0`                                              |
| yield-param-A          | r1       | `*mut [u8; 1]` | Pointer to one byte of userspace memory or `0x0` |

Yield number 0, Yield-NoWait, executes a single upcall if any is
pending.  If no upcalls are pending it returns immediately.
There are no return values from Yield-NoWait. This is because if an upcall was
invoked, the kernel pushes that function call onto the stack, such that the
return value may be the return value of the upcall.

Yield-NoWait will use
`yield-param-A` as the memory address of an 8-bit byte to
write to indicate whether an upcall was invoked. If invoking Yield-NoWait
resulted in an upcall executing, Yield-NoWait writes 1 to the field address. If
invoking Yield-NoWait resulted in no upcall executing, Yield-NoWait writes 0 to the
field address. Userspace SHOULD ensure that `yield-param-A` points to a valid address
in the current process. If userspace does not wish to receive the Yield-NoWait
result, it SHOULD set `yield-param-A` to `0x0`. The kernel SHALL write the Yield-NoWait result
if `yield-param-A` points to any valid process memory and SHALL NOT write the Yield-NoWait
result if it points to an address not in the memory allocated to the
calling process.

Yield-NoWait can use the Yield-NoWait result to allow userspace loops that want
to flush the upcall queue to execute Yield-NoWait until the queue is
empty.

`yield-param-B` and `yield-param-C` are unused and reserved.


### 4.1.2 Yield-Wait

| Argument               | Register | Type           | Value |
|------------------------|----------|----------------|-------|
| Yield number           | r0       | `u32`          | `1`   |

Yield number 1, Yield-Wait, blocks until an upcall executes. It is
commonly used when applications have no other work to do and are waiting
for an event (upcall) to occur to do more work.

This call will deliver events to the userspace application in the order
they occurred in time in the kernel. If an application has multiple
subscriptions, the userspace upcall handler is responsible for in some way
noting which callback occurred if necessary.

_Note:_ This will _only_ return after an upcall _executes_. If an event
occurs which would normally generate an upcall, but that upcall is
currently assigned to the Null Upcall, no upcall executes and thus this
syscall will not return.

Yield-Wait has no return value. This is because
invoking an upcall pushes that function call onto the stack, such
that the return value of a call to yield system call may be the
return value of the upcall.

`yield-param-A`, `yield-param-B`, and `yield-param-C` are unused and reserved.


### 4.1.3 Yield-WaitFor

| Argument               | Register | Type  | Value            |
|------------------------|----------|-------|------------------|
| Yield number           | r0       | `u32` | `2`              |
| yield-param-A          | r1       | `u32` | Driver number    |
| yield-param-B          | r2       | `u32` | Subscribe number |

The third call, Yield-WaitFor, blocks until one
specific upcall is ready to execute. If
other events arrive that would invoke an upcall on this process, they
are queued by the kernel, and will be delivered
in response to subsequent Yield calls.
Event order in this queue is maintained.

The specific upcall is identified by a Driver number and a Subscribe number
(which together form an UpcalId).

- Driver number: `yield-param-A`
- Subscribe number: `yield-param-B`

This process will resume execution when an event in the kernel generates
an upcall that matches the specified upcall. No userspace callback
function will invoked by the kernel. Instead, the contents of r0-r2 will
be set to the Upcall Arguments provided by the driver when the upcall is
scheduled.

`yield-param-C` is unused and reserved.


4.2 Subscribe (Class ID: 1)
--------------------------------

The Subscribe system call class is how a userspace process registers
upcalls with the kernel. Subscribe system calls are implemented by
peripheral syscall drivers, so the set of valid Subscribe calls
depends on the platform and what drivers were compiled into the
kernel.

The register arguments for Subscribe system calls are as follows. The
registers r0-r3 correspond to r0-r3 on CortexM and a0-a3 on RISC-V.

| Argument            | Register | Type               |
|---------------------|----------|--------------------|
| Driver number       | r0       | `u32`              |
| Subscribe number    | r1       | `u32`              |
| Upcall pointer      | r2       | C_FUNCTION_POINTER |
| Application data    | r3       | OPAQUE_GENERIC     |


The `upcall pointer` is the address of the first instruction of
the upcall function. The `application data` argument is a parameter
that an application passes in and the kernel passes back in upcalls
unmodified.

The `upcall pointer` SHOULD be a valid upcall, i.e., either a
`SubscribeUpcall` or the Null Upcall, as defined in the next section.

If the passed upcall is not valid (is outside process executable
memory and is not the Null Upcall described below), the kernel MUST
NOT invoke the requested driver and MUST immediately return a failure
with a error code of `INVALID`. The currently registered upcall
remains registered and the kernel does not cancel any pending invocations
of the existing upcall.

Any upcall passed from a process MUST remain valid until the next
successful invocation of `subscribe` by that process with the same
syscall and driver number. When a process makes a successful
subscribe system call (one which results in the `Success with 2 u32`
return variant), the kernel MUST cancel all pending upcalls on that
process for that driver and subscribe number: it MUST NOT invoke
the previous upcall after the call to `subscribe`, and MUST NOT
invoke the new upcall for events that the kernel handled before the
call to `subscribe`.

Note that these semantics create a period over which upcalls might
be lost: any upcalls that were pending when `subscribe` was called
will not be invoked. On one hand, losing upcalls can create strange
behavior in userspace.  On the other, ensuring correctness is
difficult. If the pending upcalls are invoked on the old function,
there is a safety/liveness issue; this means that an upcall function
must exist after it has been removed, and so for safety may need to be
static (exist for the lifetime of the process). Therefore, to allow
dynamic upcalls, an upcall can't be invoked after it's
unregistered.

Invoking the new upcall in response to prior
events has its own correctness issues. For example, suppose that
userspace registers an upcall for receiving a certain type of
event (e.g., a rising edge on a GPIO pin). It then changes the
type of event (to falling edge) and registers a new upcall.
Invoking the new upcall on the previous events will be
incorrect.

If userspace requires that it not lose any upcalls, it should
not re-subscribe and instead use some form of userspace dispatch.

The return variants for Subscribe system calls are `Failure with 2 u32`
and `Success with 2 u32`. For success, the first `u32` is the upcall
pointer passed in the previous call to Subscribe (the existing
upcall) and the second `u32` is the application data parameter passed
in the previous call to Subscribe (the existing application data). For
failure, the first `u32` is the passed upcall pointer and the second
`u32` is the passed application data parameter. For the first successful
call to Subscribe for a given upcall, the upcall pointer and
application data parameter returned MUST be the Null Upcall (described
below).

4.2.1 The Null Upcall
---------------------------------

The Tock kernel defines an upcall pointer as the Null Upcall.
The Null Upcall denotes an upcall that the kernel will never invoke.
The Null Upcall is used for two reasons. First, a userspace process
passing the Null Upcall as the upcall pointer for Subscribe
indicates that there should be no more upcalls. Second, the first
time a userspace process calls Subscribe for a particular upcall,
the kernel needs to return an upcall pointer and application data parameter indicating
the current configuration; in this case, the kernel returns the Null
Upcall. The Tock kernel MUST NOT invoke the Null Upcall.

The Null Upcall upcall pointer MUST be 0x0. This means it is not possible
for userspace to pass address 0x0 as a valid code entry point. Unlike
systems with virtual memory, where 0x0 can be reserved a special meaning, in
microcontrollers with only physical memory 0x0 is a valid memory location.
It is possible that a Tock kernel is configured so its applications
start at address 0x0. However, even if they do begin at 0x0, the
Tock Binary Format for application images mean that the first address
will not be executable code and so 0x0 will not be a valid function.
In the case that 0x0 is valid application code and where the
linker places an upcall function, the first instruction of the function
should be a no-op and the address of the second instruction passed
instead.

If a userspace process invokes subscribe on a driver ID that is not
installed in the kernel, the kernel MUST return a failure with an
error code of `NODEVICE` and an upcall of the Null Upcall.

4.3 Command (Class ID: 2)
---------------------------------

The Command system call class is how a userspace process calls a
function in the kernel, either to return an immediate result or start
a long-running operation. Command system calls are implemented by
syscall drivers, so the set of valid Command calls depends on the
platform and what drivers were compiled into the kernel.

The register arguments for Command system calls are as follows. The registers
r0-r3 correspond to r0-r3 on CortexM and a0-a3 on RISC-V.

| Argument          | Register | Type                         |
|-------------------|----------|------------------------------|
| Driver number     | r0       | `u32`                        |
| Command number    | r1       | `u32`                        |
| Argument 0        | r2       | `{u32, i32, u64_lo, i64_lo}` |
| Argument 1        | r3       | `{u32, i32, u64_hi, i64_hi}` |

When passing values with types whose representation is smaller than 32 bits
(e.g. `u8` or `i16`), they must be explicitly cast to a `u32` (if unsigned) or
`i32` (if signed).

The return variants of Command are instance-specific. Each specific
Command instance (combination of Driver and Command number) specifies
its failure variant and success variant. If userspace invokes a
command on a peripheral that is not installed, the kernel returns a
failure variant of `Failure`, with an associated error code of
`NODEVICE`. Therefore, command invocations that need to
handle userspace/kernel mismatches should be able to handle `Failure` in
addition to the expected failure variant (if different than `Failure`).


4.3.1 Command Identifier 0
--------------------------

Command Identifier 0 provides an existence check for drivers. Command
Identifier 0 MUST return either `Success` or `Failure` with `ENODEVICE`.
`Success` indicates that the driver is present and the userspace process can
issue system calls to it. If the driver is not accessible, Command Identifier 0
returns `Failure` with an error code of `ENODEVICE`. A driver may be not
accessible because the kernel does not have it, the process does not have the
required permissions to use it, or other reasons.

4.4 Read-Write Allow (Class ID: 3)
---------------------------------

The Read-Write Allow system call class is how a userspace process
shares a buffer with the kernel that the kernel can read and write.

The register arguments for Read-Write Allow system calls are as
follows. The registers r0-r3 correspond to r0-r3 on CortexM and a0-a3
on RISC-V.

| Argument         | Register | Type              | Additional Restrictions                                                                 |
|------------------|----------|-------------------|-----------------------------------------------------------------------------------------|
| Driver number    | r0       | `u32`             |                                                                                         |
| Allow number     | r1       | `u32`             |                                                                                         |
| Address          | r2       | `POINTER_OR_ZERO` | Pointers must refer to a contiguous array of writable userspace memory of length `{r3}` |
| Size             | r3       | `SIZE`            |                                                                                         |

The *allow number* argument is an ordinal number (index) of the buffer.
When Read-Write Allow is called, the provided buffer
SHALL get assigned to the provided *allow number*,
replacing the previous buffer assigned to that *allow number*,
if there was one.
The supported *allow number*s are defined by the driver.

The Tock kernel MUST check that the passed buffer is contained within
the calling process's writeable address space. Every byte of a passed
buffer must be readable and writeable by the process. Zero-length
buffers may therefore have arbitrary addresses. If the passed buffer is
not complete within the calling process's writeable address space, the
kernel MUST return a failure result with an error code of `INVALID`.

The return variants for Read-Write Allow system calls are `Failure
with 2 u32` and `Success with 2 u32`.  In both cases, `Argument 0`
contains an address and `Argument 1` contains a length. When a driver
implementing the Read-Write Allow system call returns a failure
result, it MUST return the same address and length as those that were passed 
in the call. When a driver implementing the Read-Write Allow system call
returns a success result, the returned address and length MUST be those
that were passed in the previous call, unless this is the first call.
On the first successful invocation of a particular Read-Write Allow system 
call, an driver implementation MUST return address 0 and size 0.

If the kernel cannot access the grant region for this process, `NOMEM`
will be returned. This can be caused by either running out a space in
the grant region of RAM for the process, or the grant was never registered
with the kernel during capsule creation at board startup. If the specified
allow number is not supported by the driver, the kernel will return `INVALID`.

The standard access model for allowed buffers is that userspace does
not read or write a buffer that has been allowed: access to the memory
is intended to be exclusive either to userspace or to the kernel. To
regain access to a passed buffer B, the process calls the same
Read-Write Allow system call again. If this call returns a success
result, the result contains buffer B. The process can call with a zero-length 
buffer if it wishes to pass no memory to the kernel. Once a buffer has been returned to userspace as
part of a Read-Write Allow system call, it is guaranteed for the
kernel to no longer have access to the described memory region, unless
it is currently shared with the kernel as part of the passed in buffer
or another Allow mechanism.

Note that buffers held by the kernel are still considered part of a
process address space, even if conceptually the process should not
access that memory. This means, for example, that userspace may extend
a buffer by calling allow with the same pointer and a longer length
and such a call is not required to return an error code of `INVALID`.
Similarly, it is possible for userspace to allow the same buffer
multiple times to the kernel. This means, in practice, that the kernel
may have multiple writeable references to the same memory and MUST 
take precautions to ensure this does not violate safety within the
kernel.

Finally, because a process conceptually relinquishes access to a
buffer when it makes a Read-Write Allow call with it, a userspace API
MUST NOT assume or rely on a process accessing an allowed buffer. If
userspace needs to read or write to a buffer held by the kernel, it
MUST first regain access to it by calling the corresponding Read-Write
Allow.

4.4.1 Buffers Can Change
---------------------------------
The standard use of Read-Write Allow requires that userspace does not
access a buffer once it has been allowed. However, the kernel MUST NOT
assume that an allowed buffer does not change: there could be a bug,
compromise, or other error in the userspace code. The fact that the
kernel thread always preempts any user thread in Tock allows capsules
to assume that a series of accesses to an allowed buffer is
atomic. However, if the capsule relinquishes execution (e.g., returns
from a method called on it), it may be that userspace runs in the
meantime and modifies the buffer. Note that userspace could also, in
this time, issue another allow call to revoke the buffer, or crash,
such that the buffer is no longer valid.

The canonical case of incorrectly assuming a buffer does not change
involves the length of a buffer. In this example, taken from the SPI
controller capsule, userspace allows a buffer, then a command
specifies a length (`arg1`) of how many bytes of the buffer to read or write.
The variable `mlen` is the length of the buffer.

```rust
if mlen >= arg1 && arg1 > 0 {
    app.len = arg1;
    app.index = 0;
    self.busy.set(true);
    self.do_next_read_write(app);
    CommandReturn::success()
}
```

Checking that the length fits within the allowed buffer when the
command is issued is insufficient, as it could be that the buffer
changes during the underlying hardware I/O operation.  If the buffer
is replaced with one that is much smaller, the length passed in the
command may now be too large. The `index` variable keeps track of
where in the buffer the next write should occur: the capsule breaks up
long writes into multiple, smaller writes to bound the size of its
static kernel buffer. If capsule code blindly copies the number of
bytes specified in the command, without re-checking buffer length,
then it can cause the kernel to panic for an out-of-bounds error.

Therefore, in the `read_write_done` callback, the capsule checks the
length of the buffer that userspace wants to read data into. The third
line checks that the end of the just completed operation isn't past
the end of the current userspace buffer (which could happen if the
userspace buffer became shorter).

```rust
let end = index;
let start = index - length;
let end = cmp::min(end, dest.len());
let start = cmp::min(start, end);

let real_len = cmp::min(end - start, src.len());
let dest_area = &mut dest[start..end];

for (i, c) in src[0..real_len].iter().enumerate() {
    dest_area[i] = *c;
}
```

For similar reasons, a capsule should not cache computations on values
from an allowed buffer. If the buffer changes, then those computations
may no longer be correct (e.g., computing a length based on fields in
the buffer).

4.5 Read-Only Allow (Class ID: 4)
---------------------------------

| Argument         | Register | Type              | Additional Restrictions                                                                 |
|------------------|----------|-------------------|-----------------------------------------------------------------------------------------|
| Driver number    | r0       | `u32`             |                                                                                         |
| Allow number     | r1       | `u32`             |                                                                                         |
| Address          | r2       | `POINTER_OR_ZERO` | Pointers must refer to a contiguous array of readable userspace memory of length `{r3}` |
| Size             | r3       | `SIZE`            |                                                                                         |

The Read-Only Allow class is very similar to the Read-Write Allow
class. It differs in some ways:

1. The buffer it passes to the kernel is read-only, and the process MAY
   freely read the buffer.
2. The kernel MUST NOT write to a buffer shared with a Read-Only Allow.
3. The *allow number*s in the Read-Only Allow
   are independent from those in the Read-Write Allow.

The semantics and calling conventions of Read-Only Allow are otherwise
identical to Read-Write Allow: a userspace API MUST NOT depend on
writing to a shared buffer and the kernel MUST NOT assume the buffer
does not change.

This restriction on writing to buffers is to limit the complexity of
code review in the kernel. If a userspace library relies on writes to
shared buffers, then kernel code correspondingly relies on them. This
sort of concurrent access can have unforeseen edge cases which cause
the kernel to panic, e.g., because values changed between method
calls.

The Read-Only Allow class exists so that userspace can pass references
to constant data to the kernel. This is useful, for example, when a
process prints a constant string to the console; it wants to allow the
constant string to the kernel as an application slice, then call a
command that transmits the allowed slice. Constant strings are usually
stored in flash, rather than RAM, which Tock's memory protection marks
as read-only memory. Therefore, if a process tries to pass a constant
string stored in flash through a Read-Write Allow, the allow will fail
because the kernel detects that the passed slice is not writeable.

Another common use case for Read-Only allow is passing test or
diagnostic data. A U2F authentication key, for example, will often
run some [cryptographic tests at boot](https://github.com/google/tock-on-titan/blob/master/userspace/u2f_app/fips_crypto_tests.c) to ensure correct
operation. These tests store input data, keys, and expected output data
as constants in flash. An encrypt operation, for example, wants to be
able to pass a read-only input and read-only key to obtain a
ciphertext. Without a read-only allow, all of this read-only data
has to be copied into RAM, and for software engineering reasons
these RAM buffers may be difficult to reuse.

Having a Read-Only Allow allows a system call driver to clearly
specify whether data is read-only or read-write and also saves
processes the RAM overhead of having to copy read-only data into
RAM so it can be passed with a Read-Write Allow.

The Tock kernel MUST check that the passed buffer is contained within
the calling process's readable address space. Every byte of the passed
buffer must be readable by the process. Zero-length buffers may
therefore have arbitrary addresses. If the passed buffer is not
complete within the calling process's readable address space, the
kernel MUST return a failure result with an error code of `INVALID`.

4.6 Memop (Class ID: 5)
---------------------------------

The Memop class is how a userspace process requests and provides
information about its address space.  The register arguments for
Memop system calls are as follows. The registers r0-r3 correspond
to r0-r3 on CortexM and a0-a3 on RISC-V.

| Argument               | Register | Type     |
|------------------------|----------|----------|
| Operation              | r0       | `u32`    |
| Operation argument     | r1       | _varies_ |
| unused                 | r2       |          |
| unused                 | r3       |          |

The operation argument specifies which memory operation to perform. There
are 12:

| Memop Operation | Operation                                               | Success          |
|-----------------|---------------------------------------------------------|------------------|
| 0               | Break                                                   | Success          |
| 1               | SBreak                                                  | Success with u32 |
| 2               | Get process RAM start address                           | Success with u32 |
| 3               | Get address immediately after process RAM allocation    | Success with u32 |
| 4               | Get process flash start address                         | Success with u32 |
| 5               | Get address immediately after process flash region      | Success with u32 |
| 6               | Get lowest address (end) of the grant region            | Success with u32 |
| 7               | Get number of writeable flash regions in process header | Success with u32 |
| 8               | Get start address of a writeable flash region           | Success with u32 |
| 9               | Get end address of a writeable flash region             | Success with u32 |
| 10              | Set the start of the process stack                      | Success          |
| 11              | Set the start of the process heap                       | Success          |

The success return variant is Memop class system call specific and
specified in the table above. All Memop class system calls have a
`Failure` failure type.

4.7 Exit (Class ID: 6)
--------------------------------

The Exit system call class is how a userspace process terminates.
Successful calls to Exit system calls do not return.

There are two Exit system calls:
  - `exit-terminate`
  - `exit-restart`

The first call, `exit-terminate`, terminates the process and tells the
kernel that it may reclaim and reallocate the process as well as all of its
resources. Usually this indicates that the process has completed
its work.

The second call, `exit-restart`, terminates the process and tells the kernel
that the application would like to restart if possible. If the kernel
restarts the application, it MUST assign it a new process identifier. The
kernel MAY reuse existing process resources (e.g., RAM regions) or MAY
allocate new ones.

The register arguments for Exit system calls are as follows. The registers
r0-r3 correspond to r0-r3 on CortexM and a0-a3 on RISC-V.

| Argument         | Register | Type  |
|------------------|----------|-------|
| Exit number      | r0       | `u32` |
| Completion code  | r1       | `u32` |

The exit number specifies which call is invoked.

| System call     | Exit number value |
|-----------------|-------------------|
| exit-terminate  |                 0 |
| exit-restart    |                 1 |

The difference between `exit-terminate` and `exit-restart` is what behavior
the application asks from the kernel. With `exit-terminate`, the application
tells the kernel that it considers itself completed and does not need to run
again. With `exit-restart`, it tells the kernel that it would like to be
rebooted and run again. For example, `exit-terminate` might be used by a
process that stores some one-time data on flash, while `exit-restart` might
be used if the process runs out of memory.

The completion code is an unsigned 32-bit number which indicates status. This
information can be stored in the kernel and used in management or policy decisions.
The definition of these status codes is outside the scope of this document.

If an exit syscall is successful, it does not return. Therefore, the return
value of an exit syscall is always `Failure`. `exit-restart` and
`exit-terminate` MUST always succeed and so never return.


5 libtock-c Userspace Library Methods
=================================

This section describes the method signatures for system calls and upcalls in C, as an example
of how they appear to application/userspace code.

Because C allows a single return value but Tock system calls can return multiple values,
they do not easily map to idiomatic C. These low-level APIs are translated into standard C
code by the userspace library. The general calling convention is that the complex return types
are returned as structs. Since these structs are composite types larger than a single word, the
ARM and RISC-V calling conventions pass them on the stack.

The system calls are implemented as inline assembly. This assembly moves arguments into the correct
registers and invokes the system call, and on return copies the returned data into the return type
on the stack.


5.1 Yield
---------------------------------

The Yield system calls have these function prototypes:

```c
int yield_no_wait(void);
void yield(void);
```

`yield_no_wait` returns 1 if an upcall was invoked and 0 if one was not invoked.

5.2 Subscribe
---------------------------------

The subscribe system call has this function prototype:

```c
typedef void (subscribe_upcall)(int, int, int, void*);

typedef struct {
  bool success;
  subscribe_upcall* upcall;
  void* userdata;
  tock_error_t error;
} subscribe_return_t;

subscribe_return_t subscribe(uint32_t driver, uint32_t subscribe,
                             subscribe_upcall uc, void* userdata);
```

The `success` field indicates whether the call to subscribe succeeded.
If it failed, the error code is stored in `error`. If it succeeded,
the value in `error` is undefined.

5.3 Command
-----------------------------------

The subscribe system call has this function prototype:

```c
typedef struct {
  syscall_rtype_t type;
  uint32_t data[3];
} syscall_return_t;

syscall_return_t command(uint32_t driver, uint32_t command, int data, int arg2);
```

Because a command can return any failure or success variant, it returns a direct
mapping of the return registers. `rtype` contains the value of `r0`, while
`data[0]` contains what was passed in `r1`, `data[1]` contains was passed in `r2`,
and `data[2]` contains what was passed in `r3`.

5.4 Read-Write Allow
---------------------------------

The read-write allow system call has this function prototype:

```c
typedef struct {
  bool success;
  void* ptr;
  size_t size;
  tock_error_t error;
} allow_rw_return_t;

allow_rw_return_t allow_readwrite(uint32_t driver, uint32_t allow, void* ptr, size_t size);
```

The `success` field indicates whether the call succeeded.
If it failed, the error code is stored in `error`. If it succeeded,
the value in `error` is undefined. `ptr` and `size` contain the pointer
and size of the passed buffer.


5.5 Read-Only Allow
---------------------------------

The read-only allow system call has this function prototype:

```c
typedef struct {
  bool success;
  const void* ptr;
  size_t size;
  tock_error_t error;
} allow_ro_return_t;

allow_ro_return_t allow_readonly(uint32_t driver, uint32_t allow, const void* ptr, size_t size);
```

The `success` field indicates whether the call succeeded.
If it failed, the error code is stored in `error`. If it succeeded,
the value in `error` is undefined. `ptr` and `size` contain the pointer
and size of the passed buffer.

5.6 Memop
---------------------------------

Because the Memop system calls are defined by the kernel and not extensible, they are
directly defined by libtock-c as library functions:

```c
void* tock_app_memory_begins_at(void);
void* tock_app_memory_ends_at(void);
void* tock_app_flash_begins_at(void);
void* tock_app_flash_ends_at(void);
void* tock_app_grant_begins_at(void);
int tock_app_number_writeable_flash_regions(void);
void* tock_app_writeable_flash_region_begins_at(int region_index);
void* tock_app_writeable_flash_region_ends_at(int region_index);
```

They wrap around an underlying function which uses inline assembly:

```c
void* memop(uint32_t op_type, int arg1);
```

5.7 Exit
---------------------------------

The Exit system calls have these function prototypes:

```c
void tock_exit(uint32_t completion_code);
void tock_restart(uint32_t completion_code);
```

Since these two variants of Exit never return, they have
no return value.


6 Authors' Address
=================================
```
Guillaume Endignoux <guillaumee@google.com>

Jon Flatley <jflat@google.com>

Philip Levis
414 Gates Hall
Stanford University
Stanford, CA 94305

Phone: +1 650 725 9046
Email: pal@cs.stanford.edu


Amit Levy <aalevy@cs.princeton.edu>

Pat Pannuto <ppannuto@ucsd.edu>

Leon Schuermann <leon@is.currently.online>

Johnathan Van Why <jrvanwhy@google.com>
```

7 References and Additional Information
=======================================

- [Design RFC for Command 0 Semantics](../rfcs/2023-08-18--CommandZeroSemantics.md).
