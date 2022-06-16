Public and Private Encryption Keys
========================================

**TRD:** 1<br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Authors:** Alistair Francis <br/>
**Draft-Created:** 11 Oct, 2021<br/>
**Draft-Modified:** 11 Oct, 2021<br/>
**Draft-Version:** 1<br/>

Abstract
-------------------------------

This document describes the Tock Public Private Key implementation.
This documents the design process and final outcome. This focuses
on the original RSA key support, but applies to all public/private
keys.

1 Introduction
====================================================================

The goal of pub/priv keys in Tock is to allow the kernel and apps to use
pub/priv crypto operations. It is expected that these are used before loading
applications, to check signatures as well as by the kernel and/or apps during
runtime.

The goal is to support 3 main use cases, for key storage:

 1. Keys stored on flash. The keys are stored at some address in read only
    flash and we want to "import" them and use them in the kernel.
 2. The app specifies a key. A userspace application obtains a key and
    passes it to the kernel to use for crypto operations
 3. We generate a key pair while running

2 Design Considerations
====================================================================

The design needs to integrate well with the rest of the Tock kernel and
capsule design. As well as that we want to ensure

2.1 Low memory overhead
--------------------------------------------------------------------

Pub/priv keys can be very large. For example a 4096-bit RSA key is 512
byes long. That means to store a pub/priv key pair in RAM we need at least
1024 bytes (1K) of memory, just for one key pair. That doesn't take into
account potential post quantum algorithms that can have even larger keys.

Due to this the design should avoid copying keys into memory where not
required. For example generating a new key pair will need to use memory,
but reading existing keys from flash should avoid copying keys to memory.

2.2 Mutable and immutable buffers
--------------------------------------------------------------------

As the implementation should support importing existing keys from flash or
from userspace the design must allow for both mutable and immutable buffers.


3 Possible key structure implementations
====================================================================

Below is a list of possible implementations, as well as outcomes of that
design. For consistency all designs below are for a 2048-bit RSA key/pair,
but the designs could apply for any pub/priv operations

3.1 In memory buffers
--------------------------------------------------------------------

Keys would be stored in a memory sturcture, similar to:

```rust
pub struct RSA2048Keys<'a> {
    modulus: [u8; 256],          // Also called n
    public_exponent: u32,        // Also called e
    private_exponent: [u8; 256], // Also called d
...
}
```

As mentioned in section 2.1 this requires large in memory buffers, even when
using an existing key on flash. Due to that this method will not be used.

3.2 TakeCell buffers
--------------------------------------------------------------------

In order to avoid storing the keys in memory, the design can instead use
`TakeCell`. This way existing keys can pass in a buffer to the key, while
new keys can use a buffer created with `static_init!()`

```rust
pub struct RSA2048Keys<'a> {
    public_key: TakeCell'static, u8>,
    private_key: TakeCell'static, u8>,
...
}
```

For example, importing a key would look like this:

```rust
fn import_public_key(&mut self,
    public_key: &'static mut [u8],
) -> Result<(), (ErrorCode, &'static mut [u8])>
```

The problem with using `TakeCell` is that then the buffer must be mutable.
This won't work with a read-only buffer stored in flash.

The design also can't use `Cell` and immutable buffers instead, as then the
design doesn't work with mutable buffers, required for genearating keys or
interacting with userspace.

3.3 Mutable and Immutable buffers
--------------------------------------------------------------------

Similar to above, this design uses interior mutability, but adds this enum

```rust
pub enum MutImutBuffer<'a, T> {
    Mutable(&'a mut [T]),
    Immutable(&'a [T]),
}
```

Then the key structure will look like

```rust
pub struct RSA2048Keys<'a> {
    public_key: OptionalCell<MutImutBuffer<'static, u8>>,
    private_key: OptionalCell<MutImutBuffer<'static, u8>>,
...
}
```

This is similar to 3.2, but allows either a mutable or immutable buffer.

For example to import a key the function would look like:

```rust
fn import_public_key(
    &'a self,
    public_key: MutImutBuffer<'static, u8>,
) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)>;
```

This allows the design to use either a mutable or immutable buffer and doesn't
have a high memory overhead.

3.4 Read and Read/Write keys
--------------------------------------------------------------------

Similar to 3.3 the other option is to have a read only key and a read/write
key and move the enum a level higher.

For example

```rust
pub struct RSA2048ReadOnlyKeys<'a> {
    public_key: OptionalCell<&'static [u8]>,
    private_key: OptionalCell<&'static [u8]>,
...
}

pub struct RSA2048ReadWriteKeys<'a> {
    public_key: TakeCell'static, u8>,
    private_key: TakeCell'static, u8>,
...
}

pub enum RSA2048Keys<'a> {
    Mutable(RSA2048ReadWriteKeys<'a>),
    Immutable(RSA2048ReadOnlyKeys<'a>),
}
```

This has the advantage that it's more obvious if a key is mutable or immutable.
This has a large code duplication downside though. There will be two
implementations, one for `RSA2048ReadOnlyKeys` and one for `RSA2048ReadWriteKeys`
that are almost identical.

On top of that there also will need to be two HILS, for example:

```rust
pub trait PubKeyReadWrite<'a> {
    fn import_public_key(&self,
        public_key: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>
}

pub trait PubKeyReadOnly<'a> {
    fn import_public_key(&self,
        public_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])>
}
```

This has a complexity and code size downside compared to section 3.3, but can
avoid confusion where a mutable buffer is required but not supplied.

4 Possible low level interface APIs
====================================================================

On top of the key structure implementation, there will also be a HIL that
hardware implementations inside `chips` will implement.

This TRD is not trying to describe this API, so let's just assume this is one
of the functions are part of that HIL:

```rust
/// Calculate the exponent. That is calculate `message` ^ `exponent`
///
/// On completion the `exponent_done()` upcall will be scheduled.
fn exponent(
    &self,
    message: &'static mut [u8],
    exponent: T,
    result: &'static mut [u8],
) -> Result<
    (),
    (
        ErrorCode,
        &'static mut [u8],
        T,
        &'static mut [u8],
    ),
>;
```

This function takes the `message` buffer and calculates the exponent from the
public or private key of type `T` and stores it in `result`.

The below sections describe why type `T` should be.

4.1 Mutable and Immutable buffers
--------------------------------------------------------------------

See section 3.3 for the enum `MutImutBuffer`, which would be used like this:

```rust
/// Calculate the exponent. That is calculate `message` ^ `exponent`
///
/// On completion the `exponent_done()` upcall will be scheduled.
fn exponent(
    &self,
    message: &'static mut [u8],
    exponent: (MutImutBuffer<'static, u8>, Range<usize>),
    result: &'static mut [u8],
) -> Result<
    (),
    (
        ErrorCode,
        &'static mut [u8],
        MutImutBuffer<'static, u8>,
        &'static mut [u8],
    ),
>;
```

In this case the underlying API will take a `'static` buffer that is either
mutable or immutable. This is wraped in the `MutImutBuffer` enum. In this case
as well we specify a range of the buffer to be used.

This has the advantage that the hardware interfacing driver doesn't have to
manage keys, instead it is just passed a buffer (wrapped in an emum). This
is also similar to other Tock HILs.

The disadvantage is how to get the buffer before calling the above function.

This implementation requires that the above layer loose access to the buffer,
with something like:

```rust
fn private_exponent(&'a self) -> Option<(MutImutBuffer<'static, u8>, Range<usize>)> {
    if self.private_key.is_some() {
        let len = PubPrivKey::len(self);
        Some((self.private_key.take().unwrap(), 0..len))
    } else {
        None
    }
}
```

Which also requires a way to regain acceess to the buffer on the `exponent()` callback:

```rust
fn import_private_key(
    &self,
    private_key: MutImutBuffer<'static, u8>,
) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)> {
    if private_key.len() != 256 {
        return Err((ErrorCode::SIZE, private_key));
    }

    self.private_key.replace(private_key);

    Ok(())
}
```

This option also requires the `MutImutBuffer` enum to work

4.2 Keys
--------------------------------------------------------------------

The other option is to pass the entire key to the low level API, for example
something like:

```rust
/// Calculate the exponent. That is calculate message ^ exponent
///
/// On completion the `exponent_done()` upcall will be scheduled.
fn exponent(
    &self,
    message: &'static mut [u8],
    key: &'static mut dyn RsaPrivKey,
    result: &'static mut [u8],
) -> Result<
    (),
    (
        ErrorCode,
        &'static mut [u8],
        &'static mut dyn RsaPrivKey,
        &'static mut [u8],
    ),
>;
```

Using something like this in the HIL:

```rust
/// Returns the specified closure over the private exponent, if it exists
/// The exponent is returned MSB (big endian)
/// Returns `Some()` if the key exists and the closure was called,
/// otherwise returns `None`.
fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()>;
```

and an implementation similar to:

```rust
fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
    if let Some(private_key) = self.private_key.take() {
        match private_key {
            MutImutBuffer::Mutable(ref buf) => {
                let _ = closure(buf);
            }
            MutImutBuffer::Immutable(buf) => {
                let _ = closure(buf);
            }
        }
        self.private_key.replace(private_key);
        Some(())
    } else {
        None
    }
}
```

Then the final implementation can use `map()` with this code:

```rust
key.map_exponent(&|buf| {
    // Do operations of the `buf` array
});
```

This has the advantage that accessing information from keys is not distructive.
It does have the downside that hardware implementations in `chips` needs to
understand the key values to access.

5 Final implementation
====================================================================

TODO once agreed apon

6 Author's Address
====================================================================

    Alistair Francis
    alistair.francis@wdc.com
