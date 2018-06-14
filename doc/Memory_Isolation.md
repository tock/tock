Memory Isolation
=============

This document describes how memory is isolated in Tock, in terms of access permissions of the kernel and processes.
Before reading this, make sure you have a good understanding of the [design of Tock](Design.md) and the [Tock memory layout](Memory_Layout.md).
<!-- npm i -g markdown-toc; markdown-toc -i Memory_Layout.md -->

<!-- toc -->

- [Flash](#flash)
  * [Kernel code](#kernel-code)
  * [Process code](#process-code)
- [RAM](#ram)
  * [Kernel RAM](#kernel-ram)
  * [Process RAM](#process-ram)

<!-- tocstop -->


Memory isolation is a key property of Tock. Without it, processes could just access any part of memory and the security of the entire system would be compromised. Although Rust preserves memory-safety (e.g. no double frees or buffer overflows) and type-safety at compile-time, this doesn't prevent processes from accessing certain addresses which they should not have access to in memory. Some other component is necessary to prevent this from happening, or systems can not support untrusted processes.

Because of this, Tock assumes a memory protection unit (MPU). The MPU is a hardware component which can configure access permissions for certain memory regions. Three fundamental access types can be set for these memory regions: read (R), write (W) and execute (X). Full access implies all three access types are allowed in a certain memory region. 

Since processes are not allowed to access each others data, the MPU has to be configured differently for each process. Therefore, with each context switch, Tock reconfigures the MPU for that process. 

When the system is running in kernel mode, the MPU is disabled. This implies the kernel has the ability to access all parts of code: in practice, the Rust type system restricts what the kernel can do. For example, a capsule cannot access a process's memory. Processes on the other hand are enforced by the MPU, and therefore have specific access permissions to specific parts of memory.

We now proceed by going over memory isolation in flash and RAM, specifically zooming into access permissions for processes.

## Flash
### Kernel code

The kernel code in flash is not accessible by any process. 

### Process code

Processes have read-only access to their own memory in flash. Generally, processes can not write to their own flash. Using a capsule however, processes can write to a part of their own flash: the binary section. 

Processes aren't allowed to write to the other part of their flash, their Tock Binary Format (TBF) header. They also have no access to other processes' memory.

## RAM

### Kernel RAM

Kernel RAM is not accessible by any process.

### Process RAM

Process RAM is memory space divided between all running apps. The figure below shows the memory space of a process.

![Process' RAM](processram.png)

A process has full access to its own stack, data and heap and can use syscalls to move these around at will. However, it has no access to its grant region: this is only accessible by the kernel. 

Processes can communicate with each other through an [inter-process communication (IPC) mechanism](doc/tutorials/05_ipc.md). This basically entails a process setting up a part of its memory as a shared buffer, after which other processes can read/write this buffer. Otherwise, a process is never able to read/write other processes' RAM.