Syscalls: Command 0 Semantics
==============================

- Initial Proposal: 2023-08-18
- Disposition: Under Review
- RFC PR: https://github.com/tock/tock/pull/3626

Summary
-------

This document captures the proposal(s) brought up by @alevy / @bradjc / others
in https://github.com/tock/tock/issues/3375#issuecomment-1681535604 and
on the August 18, 2023 core team call.


Motivation
----------

The utility of an "exists" syscall is laid out well in the current TRD104 text:

> If userspace tries to invoke a system call that the kernel does not
> support, the system call will return a Failure result with an error
> code of `NODEVICE` or `NOSUPPORT` (Section 4). As the Allow and
> Subscribe system call classes have defined Failure types, the kernel can
> produce the expected type with known failure variants. Command, however,
> can return any variant. This means that Commands can appear to have
> two failure variants: the one expected (e.g., Failure with u32) as well
> as Failure. To avoid this ambiguity for `NODEVICE`, userspace can use the
> reserved "exists" command (Command Identifier 0), described in Section
> 4.3.1. If this command returns Success, the driver is installed and will
> not return a Failure with `NODEVICE` for Commands. The driver may still
> return `NOSUPPORT`, however. Because this implies a misunderstanding of
> the system call API by userspace (it is invoking system calls that do not
> exist), userspace is responsible for handling this case.

However, there is nothing in this rationale that would require _only_ allowing
`Success` as-opposed to any of the success variants.


Initial Discussion
------------------

During the call, a few design constraints were discussed:

 - **Existence checks should be idempotent and side-effect free.**
    - This is not actually enforced by the current TRD text. However, the
       motivation is somewhat clear — a generic component polling for
       hardware existence / platform capabilities should not trigger
       hardware events.
    - The secondary implication here is that this implicitly forbids Command
       0 from triggering asynchronous events. Currently, a driver _could_
       generate an Upcall in response to a Command 0 event, which, given
       that 'anything' might check existence, probably isn't desirable.
    - We could codify these limitations without the more permissive change
       here though.

 - **Why not pass this information in a second Command 1 or equivalent for
     drivers?**
    - Something that cares about hardware count could just call Command 1
       and skip the existence check as [the kernel will automatically return
       `NODEVICE` if it's missing
       already](https://github.com/tock/tock/blob/aa33bf1bad61ff8f3ba99a36a0760368bc4e6c3f/kernel/src/kernel.rs#L1038).
    - However, now drivers have to implement two syscalls (0: exists, 1:
       info-about-existence) where one could easily be collapsed into the
       other, reducing code overhead.

 - **Having every driver implement an empty Command 0 is silly, we might as
     well let them do something with it.**
    - Yes.., but really that's an artifact of the legacy drivers that do
       something with command 0. Once all of those are gone, the kernel can
       take over management of Command 0.


Proposals and Prototypes
------------------------

The two main approaches are shown here:
 - Proposal A: Modify Command 0 to be more permissive.
 - Proposal B: Lock down Command 0 and give the kernel clear ownership.

I included some rough-sketch code of what the two variations might look like
to help discussion. It obviously will not compile as the surface area for
the complete change is huge.

```diff
diff --git a/doc/reference/trd104-syscalls.md b/doc/reference/trd104-syscalls.md
index b1b4de38e..8f78712f2 100644
--- a/doc/reference/trd104-syscalls.md
+++ b/doc/reference/trd104-syscalls.md
@@ -496,14 +496,32 @@ failure variant of `Failure`, with an associated error code of
 handle userspace/kernel mismatches should be able to handle `Failure` in
 addition to the expected failure variant (if different than `Failure`).
 
-4.3.1 Command Identifier 0
+4.3.1 Command Identifier 0 [PROPOSAL A]
 --------------------------------
 
-Every device driver MUST implement command number 0 as the
-"exists" command.  This command always returns `Success`. This command
-allows userspace to determine if a particular system call driver is
-installed; if it is, the command returns `Success`. If it is not, the
-kernel returns `Failure` with an error code of `NODEVICE`.
+Every device driver MUST implement command number 0 as the "exists"
+command.  If the driver is not installed, the kernel will return
+`Failure` with an error code of `NODEVICE`.
+
+Device drivers MUST return `Success` OR a success variant for command
+number 0. Drivers MAY use success variants to convey additional
+information, e.g. an LED driver might return the number of LEDs
+physically present on the board via `Success with u32`. Command 0
+implementation MUST NOT have ANY runtime effects. Any code present in
+the `0` match arm MUST be suitable for [constant evaluation by the
+Rust compiler](https://doc.rust-lang.org/reference/const_eval.html).
+
+
+4.3.1 Command Identifier 0 [PROPOSAL B]
+-------------------------------
+
+Command Identifier 0 is implemented by the core kernel and provides an
+existence check for drivers. If a driver is installed, the kernel will
+return `Success` for Command 0. If a driver is not installed, the kernel
+will return `Failure` with and error code of `NODEVICE`.
+
+Device drivers CANNOT modify the behavior of Command Identifier 0.
+
 
 4.4 Read-Write Allow (Class ID: 3)
 ---------------------------------
diff --git a/capsules/core/src/led.rs b/capsules/core/src/led.rs
index c87cd5586..db70042f1 100644
--- a/capsules/core/src/led.rs
+++ b/capsules/core/src/led.rs
@@ -81,6 +81,11 @@ impl<'a, L: led::Led, const NUM_LEDS: usize> LedDriver<'a, L, NUM_LEDS> {
 }
 
 impl<L: led::Led, const NUM_LEDS: usize> SyscallDriver for LedDriver<'_, L, NUM_LEDS> {
+    #[cfg(AAAAAAAAAAAAAAAA)]
+    const fn commandZero(&self) -> CommandReturn {
+        0 => CommandReturn::success_u32(NUM_LEDS as u32),
+    }
+
     /// Control the LEDs.
     ///
     /// ### `command_num`
@@ -93,14 +98,8 @@ impl<L: led::Led, const NUM_LEDS: usize> SyscallDriver for LedDriver<'_, L, NUM_
     ///        if the LED index is not valid.
     /// - `3`: Toggle the LED at index specified by `data` on or off. Returns
     ///        `INVAL` if the LED index is not valid.
-    fn command(&self, command_num: usize, data: usize, _: usize, _: ProcessId) -> CommandReturn {
+    fn command(&self, command_num: core::num::NonZeroUsize, data: usize, _: usize, _: ProcessId) -> CommandReturn {
         match command_num {
-            // get number of LEDs
-            // TODO(Tock 3.0): TRD104 specifies that Command 0 should return Success, not SuccessU32,
-            // but this driver is unchanged since it has been stabilized. It will be brought into
-            // compliance as part of the next major release of Tock.
-            0 => CommandReturn::success_u32(NUM_LEDS as u32),
-
             // on
             1 => {
                 if data >= NUM_LEDS {
@@ -131,6 +130,12 @@ impl<L: led::Led, const NUM_LEDS: usize> SyscallDriver for LedDriver<'_, L, NUM_
                 }
             }
 
+            #[cfg(BBBBBBBBBBBBBBBBBBBBBBBBBBB)]
+            // LED count
+            4 => {
+                CommandReturn::success_u32(NUM_LEDS as u32)
+            }
+
             // default
             _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
         }
diff --git a/kernel/src/kernel.rs b/kernel/src/kernel.rs
index d5121144b..811f8db36 100644
--- a/kernel/src/kernel.rs
+++ b/kernel/src/kernel.rs
@@ -1034,7 +1034,19 @@ impl Kernel {
                         arg1,
                     } => {
                         let cres = match driver {
-                            Some(d) => d.command(subdriver_number, arg0, arg1, process.processid()),
+                            Some(d) => {
+                                if subdriver_number == 0 {
+                                    #[cfg(AAAAAAAAAAAAAAAAAAAA)]
+                                    d.commandZero()
+                                    #[cfg(BBBBBBBBBBBBBBBBBBBB)]
+                                    CommandReturn::success()
+                                } else {
+                                    let sd_arg = unsafe {
+                                        core::num::NonZeroUsize::new_unchecked(subdriver_number)
+                                    };
+                                    d.command(sd_arg, arg0, arg1, process.processid())
+                                }
+                            },
                             None => CommandReturn::failure(ErrorCode::NODEVICE),
                         };
 
diff --git a/kernel/src/syscall_driver.rs b/kernel/src/syscall_driver.rs
index 562e48dc3..dbf5a8a9b 100644
--- a/kernel/src/syscall_driver.rs
+++ b/kernel/src/syscall_driver.rs
@@ -189,6 +189,13 @@ impl From<process::Error> for CommandReturn {
 /// corresponding function for capsules to implement.
 #[allow(unused_variables)]
 pub trait SyscallDriver {
+    // n.b. This requires new nightly features that are still a bit in flux:
+    // https://github.com/rust-lang/rust/issues/67792
+    #[cfg(AAAAAAAAAAAAAAAAA)]
+    const fn commandZero(&self) -> CommandReturn {
+        CommandReturn::failure(ErrorCode::NODEVICE)
+    }
+
     /// System call for a process to perform a short synchronous operation
     /// or start a long-running split-phase operation (whose completion
     /// is signaled with an upcall). Command 0 is a reserved command to
@@ -196,7 +203,7 @@ pub trait SyscallDriver {
     /// always return a CommandReturn::Success.
     fn command(
         &self,
-        command_num: usize,
+        command_num: core::num::NonZeroUsize,
         r2: usize,
         r3: usize,
         process_id: ProcessId,
```


Consensus
---------

Proposal B. It is easier to teach, can be centralized, is easier to implement
in userspace, and promotes better design patterns for drivers.

There _may_ be a rationale in the future to support a more flexible Command 0.
If we run into compelling use cases where changing command 0 provides
non-trivial code size or application complexity benefits, we can revisit
TRD 104, most likely in a highly backwards compatible way.

