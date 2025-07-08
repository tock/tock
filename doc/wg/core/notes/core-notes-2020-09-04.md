# Tock Core Notes 09/04/2020

Attending
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Samuel Jero
- Amit Levy
- Pat Pannuto
- Leon Schuermann
- Jonathan van Why
- Alistair

# Updates

## Arduino Nano BLE support

- Brad: Student has been working on the new Arudino Nano BLE which has
  a suite of sensors. He's been working on a driver for the proximity
  sensor, for which a PR was opened.

  There are a lot of students learning to work with Tock. There are
  some rough edges which can be improved by feedback.

  Reason for picking this chip: Adafruit Feather (similar, same MCU)
  has the same sensor, hence seemed like a good one to start with.

- Amit: Is this just Adafruit rebranding the same board design?

- Brad: Not quite. Adafruit one has multiple buttons, uses a different
  nRF module, all other sensors are different. However, the proximity
  sensor is the same. The Adafruit Feather board is arguably better,
  but just was not available.

- Amit: What kind of sensors does it have?

- Brad: A lot of different sensors. Temperature, Acceleration,
  Microphone, Proximity, ...

  The major issue is that tockloader does not work with it, since
  there is a unidirectional connection to the bootloader on the
  Nano33. Not clear as to why this is the case, since nobody wants to
  talk about the bootloader. Appears as if it just doesn't implement
  the read functionality.

- Branden: No reason to believe this is a hardware restriction -
  probably just not implemented.

- Amit: Is there a JTAG available?

- Brad: Yes, but they require soldering wires to small pads on the
  back.

- Branden: Only I currently have JTAG access.

- Brad: In theory it should be possible to use the existing bootloader
  to flash a new bootloader, to then replace the existing bootloader.

- Branden: Probably not much missing, just engineering work to make
  that work.

- Brad: Yes, the Tock bootloader must implement the UART bus on top of
  the USB stack for that. Also, the reset functionality must be
  implemented so that tockloader can get the board into the
  bootloader. That should be doable since we can parse the baudrate
  configuration.

## Board-based instantiation of chip drivers

- Hudson: Received reviews of Alistair and Guillaume. The main concern
  seems to be that it is currently implemented as a large macro that
  lives in the respective chip folder and is called from the board's
  main.

  There is no fundamental reason that it has to be a macro, but could
  instead be a struct which can be imported from a board main if all
  default peripherals should be used.

  If a board does not want to use the default chip peripheral
  selection, a board would instantiate its own struct (which would
  need to be done anyways if the default macro is not to be used).

  Motivation for making it a macro and calling that from the board
  itself enforces that all peripherals are visible to the board crate.

  If the struct would actually reside in the chip crate, the
  peripherals would not necessarily be visible to the board crate for
  the default instantiation to work, however custom structs could not
  create instances anymore if the peripheral's constructors were not
  visible. New PRs could add peripherals with insufficient visibility
  and still work in the default case, but for custom peripheral
  selections PRs to change the visibiliy would then be required.

  On the other hand, large macros are bad. Added peripherals with
  insufficient visibility & occasional PRs to fix that might be a good
  trade off.

- Brad: Will the `new`-constructor take arguments?

- Hudson: Currently not.

- Brad: So adding a new peripheral does not require updating `n` boards.

- Hudson: Yes. If the peripherals need to be connected to a capsule,
  then the boards need to be updated to perform that mapping.

  Same as before: adding a peripheral as a global `static mut` did not
  require updating boards to do the interrupt mapping, but did require
  updating board to actually instantiate a capsule using that
  peripheral.

- Amit: Sounds like: if the interface changes, you have to change how
  to use it. As long as it stays fixed, you do not need to change
  anything.

- Brad: The simple case should always stay simple.

- Hudson: That will remain true.

# Current state of the USB stack

- Amit: Hoping to address what the general state of the USB stack is.
  There have been a number of significant contributions in the
  previous months. Is someone able to summarize the state of it?

- Brad: There is a HIL. It has been revised once (between the SAM4L
  implementation and the NRF52 implementation) and now seems to
  encompass sufficient functionality to be compatible with 3 different
  platforms: SAM4L, nRF52 & OpenTitan.

  The HIL takes care of low-level HW description. Support for all
  specific USB profiles, descriptors and drivers live in
  capsules.

  Drivers currently exist for CDC-ACM, and some HID support (in OpenSK
  and an open PR by Alistair).

  There is a fair bit of shared code, but also some code duplication.

- Alistair: Working on HID support currently, did not know about the
  efforts in OpenSK.

  CDC mostly working on OpenTitan as well.

- Brad: High level learnings are

  - Currently, setting up a new device (board) with USB support:
    pretty rigid, descriptors to be included are somewhat hard-coded

	Probably required some amounts work to get things up and running.
	Host device enumeration structure is relatively hard coded.

  - USB stack does not quite follow the conventions of other protocol
    stacks under Tock.

    Function calls that go both up & down in the same layer can be
    very confusing. There are good reasons to do that, based on the
    USB protocol & hardware implementations. On the other hand, much
    was driven by the SAM4L's USB implementation which does not
    necessarily match up with other chips.

- Branden: USB is not virtualized yet. Using CDC or HID works, but not
  both at the same time.

- Brad: Yes. Grouping together different descriptors from different
  users of USB and being able to fan out messages to those is not yet
  supported.

- Branden: Conceptually, USB appears easy to virtualize. All
  descriptors need to be combined and everyone would get different
  endpoints.

- Amit: In what sense is the stack different in Tock compared to other
  stacks?

- Brad: Need to focus on it to give a precise answer, in general:
  thinking of layers, calls typically go down to a specific layer,
  then up again through interfaces. Whereas in USB, calls will go to a
  much higher layer and the same interface will have callbacks in both
  directions.

  It is difficult to keep track of the exact relationships and tasks
  of the individual layers.

- Alistair: Second that, it is extremely confusing.

- Brad: Not entirely sure whether it is provably wrong, but very
  difficult to follow.

- Amit: Sounds like the interface was hastily put together to get it
  working.

- Brad: Tricky that there is an inherent state machine and different
  layers must interact with different stages of that state machine.

  The interfaces are general enough that different components can hook
  into various stages, which is important.

- Alistair: Each layer ends up with its own state machine as well.


# Discussion of the TRD Tock 2.0 system call interface document

- Amit: Quickly skim over the document written up by Phil regarding
  the Tock 2.0 system call interface. Most important section appears
  to be [3.2 Return
  Values](https://github.com/tock/tock/blob/35d049a9208169efd2ed5d8dab941c47876d1129/doc/reference/trd-syscalls.md#32-return-values).

- Alistair: Why is success returning 128 instead of 0.

- Amit: It's an important choice to make it not 0, as both success and
  failure have multiple `r0` values  that they can return. Blanket `if
  (!result)  {` checking  as common  in C  would be  incorrect anyways
  since success would also include values != 0.

- Braden: I think there is an implication that for anything success,
  there is an error code which is 0.

- Alistair: Why do we have to return a different error code for a
  different number of return values. Wouldn't the caller know the
  number of return values? Very different to most syscall APIs.

- Leon: Agree. There should be a number of different wrappers in
  userspace, and you should call the respective wrapper for the
  syscall, which would be determined in advance.

- Alistair: Yes. If you're doing a specific `allow`, you know what it
  is going to return. If it succeeds you will know what the capsule
  placed in the registers, instead of checking what it returned or not
  returned.

- Braden: I think this is just for error checking. The section above
  does say that each syscall has only one return possibility.

- Amit: Every specific syscall (e.g. a command `1` to the console
  driver) should always return the same number of return
  values. However, having a different `r0`-value would allow us to
  generic wrappers in userland. For instance in Rust, write the
  low-level command wrapper as an enum of different return variants.

- Leon: Why would you want to do this? It adds runtime complexity. If
  we know what a specific syscall will return anyways, I would view is
  as part of the syscalls "type signature" and hence could immediately
  rely on this information. I would vouch for directly calling a
  respective syscall wrapper which has the return variant in its type
  signature which would allow us to directly unpack the values in
  userspace and not have runtime checks.

  I don't think most of these errors [unexpected return variant] could
  be sensibly recovered from.

- Branden: That's why the `BADRVAL` exists in section [3.2b Error
  Codes](https://github.com/tock/tock/blob/35d049a9208169efd2ed5d8dab941c47876d1129/doc/reference/trd-syscalls.md#32-error-codes). You'd
  put some checks in there to sanity-check your code, so when you
  first play around with some driver you don't risk your code
  malfunctioning.

- Jonathan: For `libtock-rs`, I'm not sure whether I'd return
  `BADRVAL` since `FAIL` will always be an option. I'm planning on
  return a specific enum for every system call having exactly the
  variant which can happen, as well as `FAIL` for unexpected
  scenarios. I would wrap an unexpected return value to `FAIL` since
  it is something that should not happen ever. Having two separate
  values translates to more runtime checks.

- Branden: If you get a failure the libtock-rs application might still
  be able to continue operating. However, if you get `BADRVAL` you will
  need to panic immediately.

- Jonathan: That's not true. If a capsule decides to return the wrong
  system call return value for a call there is no reason that should
  result in an application panic.

- Branden: That's not a runtime behavior though, so it means it must
  already be wrong at compile time?

- Amit: It might also be a runtime behavior.

- Leon: It most certainly will be. The `Driver`-trait and the
  respective `command` or `allow` functions will return an enum of all
  possible return type variants. Otherwise, multiple different
  `command` function with different return types would need to be
  provided, which would then need to be tested each.

- Brad: I agree with Branden. This would mean that there is either a
  bug in the capsule or a version mismatch between userspace and
  kernel. I don't see that capsule authors would be dynamically
  returning different variants.

- Branden: They are actually not allowed to return different
  things. Each individual syscall number must only return one variant.

- Amit: They must by convention.

- Alistair: But the application is already relying on the kernel. If
  the kernel is malicious, it can't do anything.

- Amit: I disagree. Trust in particular system call drivers is weaker
  than that. It should be reasonable for an application to want to
  detect that a capsule is not behaving as it would expect, so not use
  that capsule or recover for this.

- Alistair: If the capsule was malicious, it would rather return the
  correct variant and malicious data.

- Leon: Not necessarily. A failure case could be that we unpack values
  of a wrong format, and by relying on those cause behavior such as
  SEGFAULTing.

  Essentially `r0` is a kernel-provided value (not the capsule) on
  which we can rely on telling us what registers have been written or
  remain unchanged respectively.

- Jonathan: We probably mark every register as clobbered? I don't see
  it happening that we only mark some registers as clobbered based on
  what type of return value we are expecting.

- Alistair: If the kernel knows that only the first two arguments are
  being used, it should zero the other two.

- Amit: It is probably insufficient to think of capsules as being
  either malicious (in which case it could use malicious data in the
  right format), but it could just be a poorly written capsule.

- Leon: When we see a specific `r0`, e.g. `3` indicating all registers
  have been written, we can safely assume that there is data to be
  gathered from these registers.

  However, if we are in the case of `r0` being `0`, only one register
  has been written and reading the others (clobbered) could
  technically cause undefined behavior in userspace. It would work
  since they are just registers, but one cannot make any assumptions
  about the content.

- Amit: Yes. Unless we do enforce that unused registers are
  overwritten.

- Jonathan: Reading a register itself is not undefined behavior, but
  what you do with it could be. In the case of allow - where the
  return value would be a pointer - this could lead to undefined
  behavior. Only the core kernel should need to be trusted to prevent
  this from happening.

- Leon: Right. This would be possible if the `r0` value is included in
  the return type as it is currently documented.

- Amit: Another argument in favor of multiple return values in `r3`
  for different kinds of failure and success is "why not?".

  It does not cost us anymore whether we have just `0`/`1` or the
  current proposal. We would be using the register anyways. If we know
  the return type of a system call, then we can in userland branch on
  the success/failure determining bit.

- Brad: Why not have the error code overlap with the return type?

- Amit: Because we would lose a higher order bit.

- Brad: Are we going to have ~4 billion error codes?

- Leon: I think the concern is rather with the success values which
  would then also wrap in there. Covering all the return type variants
  (10) we would 4 bits for enumerating the cases alone, leaving us
  with 28 for the value.

- Amit: Presumably we would not need to encapsulate the variant
  information.

  We could shift the registers all over by one, have a case decision
  based on MSB of `r0`:
  - if `1`: the remaining 31 bits are the error code
  - if `0`: the remaining 31 bits are the return value

- Brad: We should not do that.

- Amit: I agree.

- Brad: If the MSB is `0`, the remaining bits indicate on how to parse
  `r1`, `r2` and `r3`.

- Amit: The distinction to what we have now would be swapping the
  success and error codes. Success would be `0` to `3` and failure
  would be `-1` to `-4`.

- Alistair: Definitely a lot better.

- Brad: Almost. However for rows 2, 3, 4 and 5, the `r1` column would
  be merged into the `r0` column.

- Leon: So in the success case, the `Return Value 0` should be merged
  together with the success type?

- Brad: That's already happening, so the success case does not change.

- Amit: No. In the success case, `r1` returns an actual usable
  value. What you had described is that `r1` goes away and is merged
  into `r0`, right?

- Brad: No, just for the failure cases.

- Amit: So the failure cases gain a register.

- Leon: Still need to cover all four variants of failure cases and
  those need an affiliated error code each, hence the variants would
  be encoded into the upper bits.

  Is this register gain worth it in the error case? I initially did
  not expect to have multiple variants in the error case at all.

- Amit, Brad: Don't really know.

- Alistair: I think it is worth gaining a register in the failure case
  not because you need the register, but because then it is the same
  as success.

- Amit: That is currently the case, except for the error / success bit
  to check.

- Leon: Semantically, the success value is stored in `r1` whereas `r0`
  would only encode the return type. Hence, in the current proposal,
  the success and error cases are already very close.

- Alistair: There are error codes, but no "success codes" which makes
  sense. Hence, error codes should be in `r0`.

- Amit: That does not mirror the success case. `r0` is defining the
  _PL type_ (shape, variant) of the return value. `r1` is separate
  from that.

  `r1` is the equivalent of C's errno. In the success case such a
  value would be meaningless.

- Leon: Also the success value might as well have different success
  cases, analog to the error cases. If we stored that information in
  `r1` as well, the semantics would be identical to the error case.

- Amit: From a performance perspective I don't think we would gain
  anything from combining these registers for the failure case.

  In either case, checking for success or failure will only require
  checking a single bit.

  If you already know the shape and trust the capsule to return the
  correct variant of value, the comparison would the same as in UNIX
  C.

  The only practical advantage would be to gain a register. We are
  unlikely to have 32bits of error codes or type information.

- Leon: If we decide to implement this, also the success case could
  also feature additional data. It would be the same thing
  semantically.

- Brad: I am not convinced that success and error are symmetric right
  now. There is a large difference between `r1` allowing one of 13
  values or anything.

  If success had different success types and those would always be in
  `r1` for every syscall, that would mean they are symmetrical.

  However, this would mean that we are throwing away 29 bits of `r0`
  which are completely unused.

  This is model after Cortex-M. It would look different if we had
  first implemented RISC-V and then Cortex-M, because we would have
  `a4` available to us.

- Alistair: Just because it could be better on RISC-V does not
  necessarily mean that it should be. This could also run on x86,
  where it is completely different.

- Amit: Right, on RISC-V we are using `a0` for the kind of system call
  (_allow_, _command_, etc.), but not using it for returning any
  useful value.

  Currently, the return registers mirror the arguments, hence the
  values returned on RISC-V are located in `a1` to `a4`.

- Alistair: This is a bad idea. Everyone else expects `a0` to contain
  the first argument.

- Leon: What do you recommend instead? We could use `a4` for the kind
  of system call (_allow_, _command_, etc.), as that would always be
  fixed per wrapper, using `a0` to `a3` for the arguments.

- Alistair: The calling semantics are fine as they are. However, upon
  returning, use the registers `a0` to `a3`, independent of where the
  system call kind was passed in.

- Amit: Is the argument that this is cleaner, or that RISC-V is used
  such that the first return value is always in `r0`.

- Alistair: It is cleaner. It follows what everybody else does,
  including the C ABI and the UNIX ABI.

- Brad: This does not seem particularly relevant. Implementation
  specifics are a different issue which should not be in this
  document.

- Amit: To summarize: the main remaining questions are

  - whether to combine `r0` and `r1` in the failure case
  - which values should be used to distinguish success and failure

- Brad: The success and failure values should be changed, e.g. success
  positive and failure negative (MSB set).

- Alistair, Leon, Brad, Amit: Agree

- Alistair: How to be included in the discussion?

- Leon: According to the document, discussion should be moved to the
  open mailing list (Google Group).

