Kernel block storage HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Dorota Czaplejewicz<br/>
**Draft-Created:** 2021/08/24 <br/>
**Draft-Modified:** 2021/08/24 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** https://github.com/tock/tock/pull/2993<br/>

Abstract
-------------------------------

In order to build file systems, modern operating systems typically rely on the block device abstraction. In Tock kernel, this abstraction is provided by the block_storage HIL, a minimal layer exposing the necessary functions needed to drive devices with separate erase and write functionality.

1 Introduction
-------------------------------

Block-based storage devices are the abstraction on top of which file systems are traditionally written. For the Tock ecosystem to be able to take advantage of existing and future file systems without the need to rewrite them, the kernel needs to provide a similar interface.

### 1.1 Leaky abstraction

The block device abstraction covers devices with a wide range of characteristics, like floppy drives, hard drives, SMR hard drives, raw Flash devices, Flash devices with Flash Translation Layers, as well as emulated devices built up of random access memory (XPoint).

The characteristics of the underlying device determine what kinds of file systems are compatible with it, making the abstraction permeable.

For example, blocks in raw Flash devices may only be rewritten a limited number of times (10-100000) before they become unuseable. File systems rewriting data in place (like FAT) would cause quick degradation and failure of such devices.

Some file systems are tuned specifically for a particular kind of storage characteristics. [SPIFFS](https://github.com/pellepl/spiffs) requires that "An erase will reset all bits in block to ones" and "Writing pulls one to zeroes".

To mitigate problems stemming from differing characteristics of the storage, the block_device HIL exposes a "least-common-denominator" interface corresponding to that of raw Flash storage.

### 1.2 State tracking

Raw Flash-based storage, even within the same broad group, can have differing restrictions on the number of allowed writes after an erase: between 1 and an arbitrary number. That introduces a layer of state embedded in the storage device: "how many writes are remaining until an erase is required?". This state cannot be queried from the device directly, and so, it must be tracked by the software stack in order to achieve the best longevity and utilization of the storage device.

Because this state is nonvolatile, any mechanism to track it also needs to store this information in a nonvolatile way. Because the only guaranteed nonvolatile storage is the device itself, tracking it efficiently implies creating a layer of indirection over the device, that is, creating a Flash Translation Layer.

The block_device HIL is intended to be a minimal layer over the storage device, taking no opinion about which data is stored on it. Therefore, it  *does not* make an attempt to track any nonvolatile state, and instead leaves that problem to solve to the HIL clients.

As a consequence, while block_device mandates certain patterns between the erase, read, and write operations, it **cannot** enforce them by design, and it's possible to use the block_device HIL **incorrectly, causing damage** to the storage device, without any compile-time or runtime errors.

2 Implementation
-------------------------------

### 2.1 Structure

The device is formed from equally-sized storage blocks,
which are arranged one after another, without gaps or overlaps,
to form a linear storage of bytes.

The device is split into blocks in two ways, into:

- discard blocks, which are the smallest unit of space that can be discarded
- write blocks, which are the smallest unit of space that can be written

Every byte on the device belongs to exactly one discard block,
and to exactly one write block at the same time.

### 2.2 Operations

The block device in Tock is composed of three interdependent fundamental operations: "read", "write a block", and "discard a block".

#### 2.2.1 `WriteableStorage::discard`

The discard operation affects the erase block under the provided index.
It corresponds roughly to the erase operation on raw flash devices.

A successful `discard` leaves bytes in the selected block undefined.
The user of this API MUST NOT assume any property of the underlying bytes after the operation completes. The operation that makes the bytes useable again is the write operation.

#### 2.2.2 `WriteableStorage::write`

The write operation affects the write block under the provided index.
The implementer SHALL NOT discard the block first.
The user of this operation MUST ensure that the relevant block has been successfully discarded first.

Once a byte has been written as part of a write block,
it MUST NOT be written again until it's discarded
as part of a discard block.
Multiple consecutive writes to the same block
are forbidden by this trait, but the restriction is not enforced.
    
#### 2.2.3 Read

`ReadableStorage::read` returns the data of the write block under the provided index.

`ReadRange::read_range` returns the data stored under the provided byte range.

The read functionality discloses the contents of the storage to the user.

Reading from a byte that has been discarded before returns undefined bytes.

3 Authors' Address
------------------------

```
Dorota Czaplejewicz <totrad.dcz@porcupinefactory.org>
https://dorotac.eu
```
