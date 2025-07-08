# Tock Core Notes 2021-09-03

## Attending

- Hudson Ayers
- Brad Campbell
- Arjun Deopujari
- Branden Ghena
- Philip Levis
- Amit Levy
- Gabe Marcano
- Pat Pannuto
- Jett Rink
- Leon Schuermann
- Johnathan Van Why

## Updates

### Tock release 2.0

- Amit: Tock 2.0 has been released.

*people celebrating*

- Branden: So much work went into what we were thinking about months
  and months ago. And then finally the push to get it out the door,
  especially by Brad, was great.

- Phil: Brad, thank you for the lead on.

- Brad: Glad its done.

- Jett: Also still released in the same month that was targeted.

### Process console

- Amit: On the note of the release, does it seem like we're converging
  on something for the process console. Alexandru is not here, but
  there was some back and forth. There appears to a high-level
  agreement on how the console should behave.

- Brad: I think the issue is that because we have a virtualized UART,
  we cannot really guarantee ordering. We might have the shell prompt
  and then the "initialization complete" message.

- Phil: Is there not a separate issue where with USB CDC, there is a
  state machine which need to run through until the character device
  is ready in the first place?

- Brad: We want to be able to enable this type of support, i.e. if we
  can detect when a user connect we should show the prompt then. The
  API I'm pushing forward would support that, but it would be wired up
  later. The ProcessConsole should not need to change then.

- Phil: Right now everything is done in main. It seems one would like
  to know when the system is fully booted and the kernel main loop is
  running. A standard deferred call for that could work?

  It is fundamentally an asynchronous operation, so one would need an
  asynchronous callback.

- Brad: Seems like a good idea, but we don't have that currently.

- Phil: It would be a callback which is invoked once every dependency
  of the target operation is up.

### Application ID

- Phil: Have started working on AppID. Going through all of the past
  discussion. Writing it up as a TRD so that hopefully we can talk
  about it on the next call.

## Asymmetric / RSA cryptography HIL

- Phil: [A pull request](https://github.com/tock/tock/pull/2693) from
  Alistair. It started out as a generic asymmetric cryptography HIL
  but has been made RSA specific. There have been some questions about
  how these traits work precisely. This requires some knowledge about
  public key crypto, its uses and its pitfalls.

*people take a look at the current state of the PR*

- Amit: is there reference hardware anywhere that one could look at?

- Phil: suppose there is OpenTitan, but that uses the OTBN
  accelerator. I believe the Teensys have hardware acceleration.

- Branden: in the trait `PubPrivKey`, there's the `pub_key` and
  `priv_key`. They return an `Option<()>`. What is going on?

- Amit: it's signaling success.

- Branden: shouldn't it return a `bool` or a Result?

- Phil: it could be that there is a good reason for it. Alistair may
  be able to explain it.

- Branden: why are we passing the buffers in the `RsaCryptoBuffers`
  struct. Can't we just pass them in via parameters? Seems kind of
  unwieldily.

- Leon: also uses the same struct for vastly different purposes. Seems
  like that could be confusing.

- Amit: in RSA, there fundamentally are only encryption and
  decryption. Signing and verification are just uses of the former.

- Phil: originally Alistair wrote this as a generic public-key crypto
  interface. I pushed back on that as there are public-key crypto
  schemes which only do sign-verify but not encrypt-decrypt. Not sure
  whether any to encrypt-decrypt but not sign-verify. It seems like
  one should decouple these things.

  Another thing: the client is parameterized on the buffer type. Thus
  it signals which operation succeeded. It's a very generic client
  trait.

- Leon: the `RsaKey` depends on the `PubPrivKey` trait. While that
  does not mandate that you have a cryptographically secure random
  number generator in your system, it does mandate that there is an
  implementation for key generation. I can imagine use cases where a
  key would be embedded in the hardware.

- Jett: related to that, you could just have the public key portion
  and verify based on that or encrypt to that key respectively. It's
  very possible to just have public key material.

- Phil: yes, for that one would use `import_public_key`.

- Amit: more generally, it's important that HILs should be -- to the
  extend possible -- as little abstraction as possible, while being
  portable across devices which implement the same functionality.

  It's hard to judge whether this HIL does that. It seems as if the
  hardware implementation for this HIL is expected to do a lot of
  nontrivial computation.

- Phil: we have HILs which do much more. They don't necessarily have
  to be very thin layers over hardware. It could be that the hardware
  does not do key generation at all, but there would still need to be
  a HIL.

- Amit: we would need a HIL if there is at least some hardware which
  would be able to do that, so we could take advantage of that
  functionality.

  However, if all hardware was just a random number generator and big
  number accelerators, then key generation should just be a library or
  capsule. The HIL would just be a portability layer on top of the
  hardware.

- Phil: one can have multiple key lengths for RSA like 2048, 4096. Not
  sure how this is reflected here? How do we make sure what operation
  to perform?

- Hudson: is the question how the HW can know what key length to use,
  or how can the caller know what key lengths the HW is expecting?

- Amit: both. Do drivers have to support all key length? According to
  the interface one would have to support all key lengths, even ones
  not supported by RSA. Also, as a user, how to know what key lengths
  the HW supports.

- Phil: or even request them, such as for key generation. I guess for
  `generate`, there is a length passed in.

- Amit: similarly, what about padding?

- Leon: not entirely sure why the public and private keys would be
  contained in the same structure. This makes it ambiguous as to
  whether the individual public/private key components relate to the
  same or different keys.

  We could potentially take more advantage of Rust's type safety if we
  expressed public and private keys as such, and in operations such as
  `encrypt` always pass in a public key.

- Phil: there are times when you would want to encrypt with the
  private key, not necessarily with the public key.

- Amit: one signs by doing a decrypt operation. One never encrypts
  with the private key. I believe RSA decrypt and encrypt are the same
  math, but using a different key. In general with public key
  cryptography, the fact that the operations are called encrypt and
  decrypt is incidental. There is a private key operation and a public
  key operation. Encrypt happens with the public, decrypt with the
  private key. Verify happens with the public, sign with the private
  key.

- Leon: then perhaps the inconsistency is what confuses me: we would
  either have only `encrypt` and `decrypt` operations, where we can
  pass in both key types, or we would have a separate `verify` and
  `sign` and allow only the respective key types to be passed in. Not
  both of those options combined as implemented in the HIL now. (Note
  while transcribing: it appears this latter approach is similar to
  what the interface would do now, i.e. just access the private or
  public key portions of the `RsaKey` depending on the operation, just
  without the type safety)

- Jett: it almost seems like the public key should be a supertrait of
  the private key. If you have the private key, you can always get the
  public key.

- Phil: having a structure representing keys seems to make sense, when
  one is performing key generation. However, for just using some key,
  it should be sufficient to just pass it in.

- Amit: so there should be some structure returned from key generation
  which holds the full key, but individual operations should just be
  passed a structure which contains the respective key components
  (e.g. the modulus and public exponent).

- Leon: should the key be a trait? Because, for keys in memory, they
  could always have a canonical representation.

- Amit: is that true, or should it also be possible for the keys to be
  stored in hardware?

- Jett: one can have the key material stored in hardware and never
  accessible to the operating system.

- Leon: that's fair, although then we'd need to change the HIL to
  return a handle, as currently there always are methods to retrieve
  the stored keys.

- Jett: yes. Having an interface to copy portions of a key in a buffer
  is a very dangerous feature, which can lead to not be certified. It
  should probably not be on the HIL.

- Leon: so we would want an interface which allows us to use and
  manipulate keys using a handle or keygrip.

- Phil: to me, this sounds like key generation should be generic over
  the structure which is returned with the generated key, and that
  structure can be used for subsequent operations. This structure can
  be a handle or reference.

- Leon: we can have optional interfaces for importing keys from RAM
  and exporting keys to RAM, but which does not need to be implemented
  by every piece of hardware.

- Jett: reaching out to some internal crypto experts about this
  interface. They have experience with designing cryptography API
  layers.

- Amit: this seems quite far from what an ideal interface for RSA or
  public key cryptography. We would not want to the crypto experts to
  review something that would change on the grounds of non-crypto
  concerns anyway.

- Jett: they are familiar with Tock as well. They could think about
  what a reasonable interface would look like.

- Amit: does anyone have a pointer to some RSA hardware module? Does
  OpenTitan have one?

- Johnathan: I've been assuming OpenTitan is going to go its own route
  for the design and would have a much more involved design
  process. It is going to get passed around to different stakeholders
  for many months. We might not be using the Tock upstream HIL.

- Amit: it matters if hardware is going to implement high-level RSA
  operations vs. a big number accelerator.

- Phil: for hardware, there is Tock on Titan (h1b). It has a big
  number accelerator.

- Amit: this seems like a useful data point. We know that AES is
  something which is accelerated in HW in a very common way. I have
  not seen RSA be accelerated in this way.

- Jett: for clarification, these HILs are only for cases where there
  is hardware acceleration? You wouldn't use these HILs for when the
  hardware does not support it?

- Amit: not exactly. We want HILs when we want to promise to higher
  levels of software that some subsystem exists and that it is
  commonly hardware accelerated. For software portability reasons, it
  would make sense to use that HIL but then implement the HIL in
  software.

- Jett: this seems like a weird use. For software, one would just use
  a capsule. So the decision of using the HIL vs a capsule would be
  based on whether one wants software vs. hardware crypto.

- Amit: if it were the case that 9 out of 10 microcontrollers had an
  RSA accelerator, it would make sense to build a HIL for that and
  build the cryptography stack to assume that part was accelerated in
  HW. For the one in 10 microcontrollers where this is not the case,
  it would make sense to emulate this HW in a capsule.

- Leon: for enabling usage of more complex HW, wouldn't we want to
  offer both a HIL for big number accelerators and then a more
  high-level HIL for entire RSA operations. This would allow to
  support more complex chips where the keys wouldn't be stored in RAM,
  but still be usable for chips which only offer primitive
  acceleration.

- Amit: yes, though this is all very hypothetical. This is a design
  process, so we must figure out what the right level of abstraction
  is.

- Phil: it seems every microcontroller vendor seems to have chips with
  public key accelerators. For instance, NXP for the Teensys, STM32
  (linked in chat), nRFs has one, etc.

- Leon: one must be careful with the vendor's documentation. For
  instance the ARM CryptoCell for the nRFs is hidden behind a
  SoftDevice, hence we can't really tell what parts of operations are
  properly done in hardware or software.

- Amit: correct. The only register available to us is _ENABLE_.

- Phil: that's true, but nRFs are the most pronounced on that. Other
  vendors support more control and direct access to the hardware.

## UART HIL

- Branden: can start with thoughts Jett has.

- Jett: there is a `Configuration` and a `Configure` trait, one allows
  to read and the other to set the configuration. It appears the
  `Configuration` should be a supertrait of `Configure`.

- Phil: went back and forth on that. Just because you can set it does
  not mean you need to be able to get it.

- Jett: but you should be able to.

- Phil: not necessarily, but I wouldn't be against making one a
  supertrait.

---

- Jett: general comment on HILs. When we switched from `ReturnCode` to
  Result with `ErrorCode`, I read somewhere that we want to encourage
  users to make their own error enumerations. Is that something which
  we're trying to do that for HILs?

- Phil: generally speaking, `ErrorCode` is a general thing to
  use. Systems can have their own error types if they want, notably
  I2C. This is the case when it does not map well to `ErrorCode`.

- Leon: there seem to be two good reasons to not use `ErrorCode`:
  either, when you want to describe error cases which don't map well
  to variants of `ErrorCode`, or if those error are critical and must
  not blindly be passed back to upper layers. If there is a custom
  type, you are essentially forcing users to properly unpack and
  handle it. The API your offering is likely to have other error
  enumerations, at which point one needs to translate to `ErrorCode`
  or some different type.

- Jett: so in the latter case one would not add an
  `impl<From<ErrType>> for ErrorCode`. That makes sense.

---

- Jett: in the TRD we talk about asynchronous and synchronous
  callbacks. Maybe TRDs should have a link back to the HIL TRD where
  these things are explained, for when one is reading TRDs in a vacuum
  without having read the other ones.

- Phil: might be something to go in the abstract. Something like "this
  TRD is in compliance with TRD 10x". It's a very common practice.

---

- Jett: regarding `abort`. The way `abort` works, it says it should
  always return an error if there is going to be a callback. One
  cannot tell if one could or couldn't cancel the callback.

  For example, when there is a pending transmit and it is aborted,
  there is going to be a callback either way. So one always gets an
  error and cannot tell whether it has been aborted.

- Phil: it gets tricky -- what if the operation is already succeeded
  and it is in a deferred call queue.

- Jett: it's worth to let callers know whether the abort was
  successful or unsuccessful.

- Phil: the `BUSY` vs. `FAIL` is going to tell you that?

- Jett: let's say an operation takes 100s. One second after starting
  it, abort is called. A callback might arrive in 99s or
  immediately. One still needs the callback to get the buffers back,
  but one cannot tell what will happen when `abort` is called.

- Phil: that information is conveyed in the error code from the
  regular operation callback. The error code in the callback indicates
  whether the operation was successfully canceled or not.

- Jett: there is no abort callback, right? It's a synchronous
  operation.

- Phil: no, the operation callback is used in any case.

  Example: call `transmit_buffer`, get `SUCCESS`. Then call
  `abort_transmit`, that returns `FAIL`. This means that the
  transmission will not be canceled and the callback will return
  `Ok`. If the callback returns `BUSY`, then the transmission will be
  aborted and one receives the callback.

- Jett: seems a little backwards, but makes sense.

- Phil: supposedly the `Ok` could get a bool, indicating whether the
  operation was aborted or not.
