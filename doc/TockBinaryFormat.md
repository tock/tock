# Tock Binary Format

<!-- npm i -g markdown-toc; markdown-toc -i Compilation.md -->

<!-- toc -->

- [TBF Header](#tbf-header)
  * [TLV Types](#tlv-types)
    + [`1` Main](#1-main)
  * [`2` Writeable Flash Region](#2-writeable-flash-region)
  * [`3` Package Name](#3-package-name)
- [Code](#code)

<!-- tocstop -->

Tock processes are represented using the Tock Binary Format (TBF). A TBF
includes a header portion, which encodes meta-data about the process, followed
by a binary blob which is executed directly.

## TBF Header 

The TBF header contains a base header, followed by a sequence of
type-length-value encoded elements. The base header 16 bytes, and has 5 fields:

```
 0         2             4            8       12         16
+---------+-------------+------------+-------+----------+
| Version | Header Size | Total Size | Flags | Checksum |
+---------+-------------+------------+-------+----------+
```

    * `Version` a 16-bit unsigned integer specifying the TBF header version.
      Always `2`.
    * `Header Size` a 16-bit unsigned integer specifying the length of the TBF
      header in bytes.
    * `Total Size` a 32-bit unsigned integer specifying the total size of the TBF
      in bytes (including the header).
    * `Flags` each bit indicates whether a flag is enabled (1) or disabled (0).
      - Bit 0 marks the process enabled. Disabed processes will not be launched
        at startup.
    * `Checksum` the result of XORing each 4-byte word in the header, excluding
      the word containing the checksum field itself.

The header is followed immediately by a sequence of TLV elememnts.  Each
element begins with a 16-bit type and 16-bit length followed by the element
data:

```
 0      2        4
+------+--------+-----...---+
| Type | Length | Data      |
+------+--------+-----...---+
```

  * `Type` is a 16-bit unsigned integer specifying the element type.
  * `Length` is a 16-bit unsigned integer specifying the size of the data field
    in bytes
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

  * `init_offset` is the offset in the binary that contains the `_start` symbol
    (i.e. the first instruction to execute).
  * `protected_size` the amount of flash, from the beginning of the header, to
    prevent the process from writing to.
  * `minimum_ram_size` the minium amount of memory the process needs.

### `2` Writeable Flash Region

`Writeable flash regions` indicate portions of the binary that the process
intends to mutate in flash.


```
 0      2        4        8      12
+------+--------+---------------+
| Type | Length |     Data      |
|======+========+========+======+
|  1   |   12   | offset | size |
+------+--------+--------+------+
```

  * `offset` the offset from the beginning of the binary of the writeable region.

  * `size` the size of the writeable region.


### `3` Package Name

The `Package name` specifies a unique name for the binary. It's only field is
an ASCII encoded package name.

```
 0      2           4
+------+-----------+-----------...--+
| Type |   Length  | Data           |
|======+===========+===========...==|
|  3   | len(name) | package name   |
+------+-----------+-----------...--+
```

  * `package name` is an ASCII encoded package name

## Code

The process code itself has no particular format. It will reside in flash,
but the specific address is determined by the platform. Code in the binary
should be able to execute successfully at any address, e.g. using position
independent code.

