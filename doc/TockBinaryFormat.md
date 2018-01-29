# Tock Binary Format

<!-- npm i -g markdown-toc; markdown-toc -i Compilation.md -->

<!-- toc -->

- [TBF Header](#tbf-header)
  * [TLV Elements](#tlv-elements)
  * [TLV Types](#tlv-types)
    + [`1` Main](#1-main)
    + [`2` Writeable Flash Region](#2-writeable-flash-region)
    + [`3` Package Name](#3-package-name)
- [Code](#code)

<!-- tocstop -->

Tock processes are represented using the Tock Binary Format (TBF). A TBF
includes a header portion, which encodes meta-data about the process, followed
by a binary blob which is executed directly. All fields in the header are
little-endian.

## TBF Header

The TBF header contains a base header, followed by a sequence of
type-length-value encoded elements. All fields in both the base header and TLV
elements are little-endian. The base header 16 bytes, and has 5 fields:

```
 0         2             4            8       12         16
+---------+-------------+------------+-------+----------+
| Version | Header Size | Total Size | Flags | Checksum |
+---------+-------------+------------+-------+----------+
```

  * `Version` a 16-bit unsigned integer specifying the TBF header version.
    Always `2`.
  * `Header Size` a 16-bit unsigned integer specifying the length of the
    entire TBF header in bytes (including the base header and all TLV
    elements).
  * `Total Size` a 32-bit unsigned integer specifying the total size of the
    TBF in bytes (including the header).
  * `Flags` specifies properties of the process.
    - Bit 0 marks the process enabled. A `1` indicates the process is
      enabled. Disabled processes will not be launched at startup.
    - Bit 1 marks the process as sticky. A `1` indicates the process is
      sticky. Sticky processes require additional confirmation to be erased.
      For example, `tockloader` requires the `--force` flag erase them.  This
      is useful for services running as processes that should always be
      available.
    - Bits 2-31 are reserved and should be set to 0.
  * `Checksum` the result of XORing each 4-byte word in the header, excluding
    the word containing the checksum field itself.

### TLV Elements

The header is followed immediately by a sequence of TLV elements. TLV
elements are aligned to 4 bytes. If a TLV element size is not 4-byte aligned it
will be padded with up to 3 bytes. Each element begins with a 16-bit type and
16-bit length followed by the element data:

```
 0      2        4
+------+--------+-----...---+
| Type | Length | Data      |
+------+--------+-----...---+
```

  * `Type` is a 16-bit unsigned integer specifying the element type.
  * `Length` is a 16-bit unsigned integer specifying the size of the data field
    in bytes.
  * `Data` is the element specific data. The format for the `data` field is
    determined by its `type`.

### TLV Types

TBF may contain arbitrary element types. A standard set of element types are
standardized.

#### `1` Main

The `Main` element has three 32-bit fields:

```
 0      2        4             8                12             16
+------+--------+---------------------------------------------+
| Type | Length |                  Data                       |
|======+========+=============+================+==============|
|  1   |   12   | init_offset | protected_size | min_ram_size |
+------+--------+-------------+----------------+--------------+
```

  * `init_offset` is the offset in bytes from the beginning of binary payload
    (i.e. the actual application binary) that contains the first instruction to
    execute (typically the `_start` symbol).
  * `protected_size` the amount of flash, in bytes, after the header, to
    prevent the process from writing to.
  * `minimum_ram_size` the minimum amount of memory, in bytes, the process
    needs.

If the Main TLV header is not present, these values all default to `0`.

#### `2` Writeable Flash Region

`Writeable flash regions` indicate portions of the binary that the process
intends to mutate in flash.


```
 0      2        4        8      12
+------+--------+---------------+
| Type | Length |     Data      |
|======+========+========+======+
|  1   |    8   | offset | size |
+------+--------+--------+------+
```

  * `offset` the offset from the beginning of the binary of the writeable
    region.
  * `size` the size of the writeable region.


#### `3` Package Name

The `Package name` specifies a unique name for the binary. Its only field is
an UTF-8 encoded package name.

```
 0      2           4
+------+-----------+-----------...--+
| Type |   Length  | Data           |
|======+===========+===========...==|
|  3   | len(name) | package name   |
+------+-----------+-----------...--+
```

  * `package name` is an UTF-8 encoded package name

## Code

The process code itself has no particular format. It will reside in flash,
but the specific address is determined by the platform. Code in the binary
should be able to execute successfully at any address, e.g. using position
independent code.

