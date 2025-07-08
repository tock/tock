# Tock Core Notes 2020-12-11

## Attending
 * Branden Ghena
 * Amit Levy
 * Leon Schuermann
 * Arjun Deopujari
 * Johnathan Van Why
 * Vadim Sukhomlinov
 * Pat Pannuto
 * Brad Campbell
 * Phil Levis
 * Hudson Ayers
 * Alistair


## Updates
- Phil: Progress on porting

- Arjun: Working on DALS over-the-air-updates implementation.

- Brad: Worked on Rubble, cleaning up the stack.
- Amit: Advertisement issue on Android?
- Brad: Still outstanding.
- Hudson: Issue with scan response.
- Brad: Not clear if Rubble API will let us call the function we need.

## System Call Porting
- Amit: Sign up here: https://github.com/tock/tock/issues/2235
- Phil: I grouped them, useful to get experience with porting them and learning
  new interface.
- Amit: Some are harder to test without hardware.
- Amit: I can take on the BLE advertisements capsule.
- Leon: We are just doing direct translation now, not trying to take advantage
  of the new capabilities.

## App Slice Swapping
- Leon: Should the kernel guarantee that capsules behave correctly with app
  slice management?
- Johnathan: Kernel must ensure capsules cannot duplicate app slices and pass
  them back multiple times. Not as much of an issue if the capsule switches them
  around.
- Johnathan: Should the kernel give capsules a method for accessing app slices
  they have access to rather than a reference to the app slice?
- Leon: The kernel could create app slices as needed, could lead to linear
  searches or a fair bit of complexity.
- Phil: End-to-end argument would say userspace has to check anyway, so is there
  a threat model we have to protect against?
- Johnathan: If the kernel provides guarantee, we do not need to do libtock-rs
  check in userspace.
- Leon: Could benefit application correctness.
- Johnathan: Might have to check for null anyway, not much overhead to check for
  buffer correctness as well.

- Phil: Idea: Capsule maintains table of appslices, and uses allow number to
  index the table and therefore has to swap the buffer (since that is all you
  could access).
- Leon: Tried this, couldn't get it to work in Tock context.
- Hudson: Might be some tricks you can do.

- Phil: Are we worried about capsules switching buffers between apps, or in the
  same app?
- Leon: Same app.

- Phil: Could this be chosen at compile time? Maybe only used for testing?
- Leon: Could be implemented that way. Could be a component compiled into the kernel.

- Brad: I'm supportive of the kernel enforcing this.

- Phil: We don't know what the overhead is. We can try it, and see if it
  matters, and then just have it all of the time if it is a small overhead. Or
  make it optional if it turns out to be problematic.

## Appslices Copy Trait
- Phil: Appslices do not implement Copy. Makes sense, there is only one.
- Problem is for zero length slices, need Default for creating grants.
- Capsules can then always generate zero-length slices and return those.
- Leon: Might be a way to help with this.
- Amit: I don't think there is a way to limit Default.

## #2252 Remove SuccessWithValue
- Phil: ReturnCode has two Success values. Convenient at the time, but difficult
  to map to the new syscall return types. HILs don't always specify exactly what
  they can return.
- Hudson: SuccessWithValue removed except for userspace Drivers. Eventually
  remove SuccessWithValue entirely. This allows us to add the `.into()`
  conversion.
- Leon: Makes it easier for the some command calls to return different success
  types.

## Tock 2.0 Discussions
- Phil: Useful to discuss 2.0 issues here?
- Brad: Good to bring up issues here, but details are overwhelming.
- Hudson: The Rust type discussion is difficult.

