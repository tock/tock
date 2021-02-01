# TicKV Technical Details

## How TicKV works

Unlike a regular File System (FS) TicKV is only designed to store Key/Value (KV)
pairs in flash. It does not support writing actual files, directories or other
complex objects. Although a traditional file system layer could be added on top
to add such features.

TicKV allows writing new key/value pairs (by appending them) and removing
old key/value pairs.

Similar to (Yaffs1)[https://yaffs.net/documents/how-yaffs-works] TicKV uses a
log structure and circles over the flash data. This means that the file
system is inherently wear leveling as we don't regularly write to the same
flash region.

## Storage Format

TicKV flash storage is formatted as a sorting of regions with each region
containing TicKV objects.

### TicKV Regions

A TicKV region is the smallest set of contiguous flash storage space that can
be erased with a single command.

For example if a flash controller allows erasing lengths of 1024 (0x400) bytes
then the length of a region will be 1024 (0x400) bytes.

The number of regions is determined by the capacity of the flash storage and
the size of regions.

The start and end address of flash used for TicKV must be region aligned.

### TicKV Objects

A TicKV object is the representation of a key/value pair in flash. An object
contains the value to be saved as well as useful header data.

TicKV saves and reads objects from flash. TicKV objects contain the value
the user wanted to store as well as extra header data.

A TicKV object consists of multiple parts:

```
|||||||||||||||||
|               |
| Object Header |
|               |
|||||||||||||||||
|               |
|     Value     |
|               |
|||||||||||||||||
|               |
|   Check Sum   |
|               |
|||||||||||||||||
```

#### Object Header

Currently ObjectHeader includes these fields:

```Rust
struct ObjectHeader {
    version: u8,
    flags: u4,
    len: u12,
    hashed_key: u64,
}
```

The `version` field is a byte containing the version of TicKV used when the
object was written.
This allows us to upgrade this library in the future, while still supporting
old data formats.

The `flags` field is a bitmap of at most 4 flags that can be OR-ed together to
describe an object state or features. The only flag defined is the `valid` flag
(bit 3), indicating that an object is valid.

It looks like this in flash:

```
|valid|Reserved|Reserved|Reserved|
|     |        |        |        |
|  1  |    0   |    0   |    0   |
```

Where `valid` indicates if an object is valid. A `1` indicates it is a valid
object, a `0` indicates that it has been marked as invalid (see below).

The `len` field is 12-bits long.
This field indicates the total length of the object, including the
header and check sum. The maximum length of the entire object is
4KiB (0xFFF) or the region size, whichever is smaller.

The `hashed_key` field stores the 64-bit (8 byte) output of the key hash.

ObjectHeader is internal to TicKV and users of TicKV do not need to
understand it.

#### Object Value

The Value component of the TicKV object is the value that the user wants to
store.

The values can be any length as long as they follow both:
 * Don't span multiple regions. That limits the maximum value length to
   `region_size - size_of::<ObjectHeader>()`
 * Don't have a maximum length greater then 4KiB (0xFFF).

#### Checksum

The checksum is a CRC-32 (polynomial 0x04c11db7) of the entire object (not including
the checksum).

### Object overhead

Currently the overhead of an TicKV object is 17 bytes. Most of this is the 8
bytes for the key hash and 4 bytes for a checksum.

### Location of objects

The region where a TicKV object is stored is dependent on the output of the
key hash and the number of regions.

TicKV determines the region an object will be stored or retrieved from using
region numbers, starting from 0. The region number of an object is equal to the
last two bytes of an object hashed-key modulo the number of regions.

This will produce a number that is between zero and the total number of
TicKV regions. This number is the regions number where the data is stored
or loaded from.

This allows us to quickly determine which flash region a key is stored in. This
should improve lookup time by reducing the number of reads required.

If a give region is full, we will start to look in neighboring regions. We first
perform the same search in the next (increment region number) region. If this
region doesn't exist (we are at the end of the regions) or if the region is full
we then search in the previous region (decrement region number).

When storing a object this process continues until we either:
 * Search all regions
 * Find a free space

When retrieving an object the process continues until we either:
 * Search all regions
 * Find the key we are looking for
 * Find a region that is empty

### Invalidating keys

Flash has the characteristic that although read/writes can happen at small
granularities an erase operation requires a entire block (specified
by the region size).

Due to this removing a key with the `invalidate_key()` function does NOT
remove anything from flash. Instead calling `invalidate_key()`
will mark the `valid` boolean for that object as `false` (0).

This is done because changing a `1` to a `0` in flash can be done with a
write to a single byte (where only 1 bit changes). While changing a `0`
to a `1` requires an erase of the entire region.

The key point here is that removing a key does NOT remove it or the object
from the flash storage. If the key contains sensitive information it will
still be in flash. Once invalidated though `get_key()` will not return the key
any more.

If all the objects in a region are no longer valid then that region will be
erased when `garbage_collect()` is called. Note that even if the flash is
full `garbage_collect()` will not be called automatically.

### Initialisation

When setting up a block of flash for the first time the entire size of flash
is erased. Then a super key called "tickv-super-key" is added with no
attached data.

On future initialisation the implementation will check for the
"tickv-super-key" key. If it exists no erase operations will occur. If it
doesn't exist the entire block of flash will be erased.

## What is looks like in flash

### Adding a key

This is an example of what `TicKV::new(..., 0xC00, 0x400)` will
look like in flash.

```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||                  |||||                  |||||                  |||||
--------------------------------------------------------------------------
```

All values in flash will be `0xFF` as the flash has just been erased.

When a key is added with a `[u8; 32]` `value` with `append_key()` it will
look like this:

```
Key:    "ONE"
Hash:   0xeda10078886193bb
Region: 1 (0x93bb % 3)
```

`Region` is the flash region where the object will be stored. In this case it's
region 1 of 3.

```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||                  |||||ONE               |||||                  |||||
--------------------------------------------------------------------------
```

Where the TicKV object ONE will look like this

```
0x400                                                                                              0x42C
--------------------------------------------------------------------------------------------------------
||||| version|len/flag|   len  |                              hashed_key                               |
|||||        |        |        |        |        |        |        |        |        |        |        |
|||||    0x00|10000000|    0x34|    0xed|    0xa1|    0x00|    0x78|    0x88|    0x61|    0x93|    0xbb|
-------------------------------------------------------------------------------------------------------|
```

```
0x42C                                                                                              0x52C
--------------------------------------------------------------------------------------------------------
|||||                                               value                                              |
|||||                                                                                                  |
|||||xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx|
-------------------------------------------------------------------------------------------------------|
```

```
0x52C                                   0x53C
-----------------------------------------
|||||              checksum             |
|||||        |        |        |        |
|||||    0x0a|    0x3c|    0xe1|    0x17|
----------------------------------------|
```

In this case the header information takes up 19 bytes of a total of 51 bytes,
which is around 37% of the space.

Note that if region 1 is full, we would then try to save the ONE object in
region 2, then try again in region 0.

### Adding a second key

When a new key TWO with a `[u8; 32]` `value` is added with `append_key()` the
flash might look like this:

```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||                  |||||ONE|TWO           |||||                  |||||
--------------------------------------------------------------------------
```

Where TWO will have the same structure as ONE, except with a different hash
value, different checksum and starts at address 0x53C. Note that depending
on the hash of TWO it could be added to any region.

### Adding a third key

When a new key THRID with a `[u8; 32]` `value` is added with `append_key()` the
flash might look like this

```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||THRID             |||||ONE|TWO           |||||                  |||||
--------------------------------------------------------------------------
```

Note that although region 1 isn't full, it is placed in region 0 based on the
region calculated from the lower byte of it's hash.

Objects will be placed in chronological order inside a region. Objects will
not be placed in order inside flash though. Instead they are placed at a
region depending on their hash. This is described in more detail above.

### Finding keys

Locating keys in flash follows a similar process to writing them.

First we calculate the hash and determine the region where we expect the
object to be stored. With the same key input and hash function we will always
get the same hash and region.

Using the same example as above, we will get this:

```
Key:    "ONE"
Hash:   0xeda10078886193bb
Region: 1 (0x93bb % 3)
```

Then we load the entire region from flash. In this example that would be
region 1. So we read the entire region 1 from flash.

We then iterate over the loaded region, starting with the first byte.

We check to make sure the version is supported and that the object isn't
marked as !`valid`.

We then check to see if the object hash matches the hash we are
looking for. If it doesn't we move forward in the loaded region by the
total length of the object we just checked and start the process again.

We continue this loop until we either find the key we are looking for or
find a version 0xFF, indicating the end of the blocks in that region.

This method allows a quick retrieval of data from a given key. We only
require a single read and store the entire region in memory. This should
only have a small memory foot print as regions are generally less then 4KiB.

### Adding keys to full regions

In the previous example we added the ONE object to an empty flash region. This
example shows what would happen if region 1 was full.

Image if the flash storage looked like this:

```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||                  |||||KEY|EXAMPLE|TEST| |||||                  |||||
--------------------------------------------------------------------------
```

and we wanted to add the same ONE key:

```
Key:    "ONE"
Hash:   0xeda10078886193bb
Region: 1 (0x93bb % 3)
```

We can't add the key to region 1 as it is full. Instead we fall back to trying
region 2. If region 2 was full we would then try region 0 and so on.

Adding key ONE would look like this:


```
0x000                  0x400                  0x800                 0xC00
--------------------------------------------------------------------------
|||||     Region 0     |||||     Region 1     |||||     Region 2     |||||
|||||                  |||||                  |||||                  |||||
|||||                  |||||KEY|EXAMPLE|TEST| |||||ONE               |||||
--------------------------------------------------------------------------
```

### Finding keys from full regions

Image the flash layout from above and then we wanted to find the non-existent
key TWO.

The hash of TWO indicates it should be in region 1. First we check region 1,
but we don't find TWO. Next we try to find TWO in region 2. We don't kind the
TWO object there, but as the region isn't empty we can't determine if it didn't
fit. Next we try region 0. As this region is empty we stop looking.

### Invalidating a key

When the key ONE is invalidated with `invalidate_key()`, the only change in
flash will be the `valid` flag. The object header for ONE will now look like:

```
0x400                                                                                              0x52C
--------------------------------------------------------------------------------------------------------
||||| version|len/flag|   len  |                              hashed_key                               |
|||||        |        |        |        |        |        |        |        |        |        |        |
|||||    0x00|00000000|    0x34|    0xed|    0xa1|    0x00|    0x78|    0x88|    0x61|    0x93|    0xbb|
--------------------------------------------------------------------------------------------------------
              ^
```

No changes will happen in flash until key TWO has also been invalidated.
At which point `garbage_collect()` can erase the region.

## Limitations of TicKV

### Fragmentation

Although TicKV has a `garbage_collect()` function, it makes no effort to
handle fragmentation.

That means that if you have the following objects in a region

```
0x000                  0x400
----------------------------
|||||     Region 0     |||||
|||||                  |||||
|||||ONE|TWO|THREE|FOUR|||||
----------------------------
```

Where ONE, TWO and FOUR have been marked as invalid the entire region will still
be classified as full. It will only not be full when all four objects are marked
for invalid.

It would be possible to have a reserved region and then free up the region with
the following steps:
 1. Move the region to the reserved region
 1. Erase the above region
 1. Move the valid objects from the reserved region to the original region,
    after being defraged in memory
 1. Erase the reserved region

The above steps would ensure we don't loose data on a power loss, as long as
we could ensure we could continue the operation after regaining power.

There are a few problems with this approach though.

In order to maintain "garbage collecting" state to restore after a power loss
we would need some sore of super block. This would then break the requirement
on wear levelling as whichever region that block is in will be written and
erased more often the others.

The other problem is the reserved region where we would move data to. This
would cost space as an entire region would be dedicated to garbage collecting.
The garbage collecting region would also have more erase and writes performed
on it breaking the wear levelling requirement.

### Somewhat high storage overhead

The storage overhead is somewhat high for TicKV. This is mostly due to the
two 64-bit hashes that are stored with every object.

The 64-bit value is determined by the output of the standard Rust
`core::hash::Hasher` trait.

For the hash of the key a 64-bit value can be justified by the lack of
collision avoidance in the implementation. If two keys have the same
hash the second key will be dropped. In this case a 64-bit hash should
hopefully make that occurrence very unlikely. Also by following the standard
Rust `core::hash::Hasher` trait the user is free to implement any standard
Hasher of their choosing. Some systems will even be able to offload the
hash to hardware.

### Memory usage

At a minimum TicKV requires a `PageSize` amount of free memory to store the
buffer obtained when reading flash. This could be reduced by instead performing
multiple reads of a smaller fixed size.

### Synchronous and Asynchronous Usage

TicKV supports both an synchronous and asynchronous usage. For developers
used to Tock's callback method the the async usage might seem strange and
sub-optimal. There are a few reasons that it is done this way.

One of the goals of TicKV is to allow other non Tock users to use it. The
current method should allow the library to be used to crates.io by other
developers.

Another reason for the synchronous support is that generally when there is
only a single flash bank all flash writes and erases must be done synchronously.
The synchronous API should allow these type of operations to occur.

On top of that it is much simpler to unit test a synchronous API. This means the
TicKV implementation can take advantage of a large range of unit tests to
ensure correctness in the design. This is more difficult and complex to do in
a async only operation.

The async functions (see the documentation) can be used to implement a fully
async operation. This is done using a "retry" method, such that special busy
error codes indicate operations should be retried. This ends up being easier to
implement as TicKV uses loops to find keys. This could be split out into
callbacks, but it is expected that the end result will look very similar as the
callbacks will maintain state and then restart the loop where it last left off.
