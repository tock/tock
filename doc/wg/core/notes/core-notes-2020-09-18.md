# Tock Core Notes 09/18/2020

## Attending
 * Alistair
 * Amit Levy
 * Brad Campbell
 * Branden Ghena
 * Hudson Ayers
 * Johnathan Van Why
 * Leon Schuermann
 * Pat Pannuto
 * Phil Levis
 * Samuel Jero
 * Vadim Sukhomlinov
 
## Updates
 * Leon: I've been working porting Tock to the LiteX. If you're not familiar
   with LiteX, it is a sort of SoC generator which spits out Verilog which you
   can then put on FPGAs. You can configure different types of CPUs with
   different peripherals by command-line parameters. I'm using this for a
   research project for university, where I need to write a driver for the Mac
   ethernet hardware and experiment with changes to that.
 * Amit: What kind of hardware does it run on?
 * Leon: I have an Arty A7 board here, my main target currently, but essentially
   it runs on any Xilinx or Lattice FPGA with sufficient specs. You can say you
   just want a CPU with e.g. no compressed instructions or leave other features
   out so it fits on your FPGA.

## 1.6 Release
 * Amit: Today is a last call for blockers for a 1.6 release. Does anyone have
   anything to add to the 1.6 release? I think we're really only waiting on
   finishing up the Time HIL redesign.
 * Brad: The time HIL is the major update. There's a minor change to the
   scheduler timer that I would like to have part of this release. That should
   be a good milestone for the scheduler and the scheduler timer.
 * Amit: Would you mind explaining the scheduler timer change.
 * Brad: The scheduler timer is what allows for process timeslice keeping. Due
   to the ARM SysTick implementation -- where once a timer expires you can't
   trust the values coming from the hardware at that point -- the API is
   designed so you can implement it on hardware that loses state when an event
   happers. So the API is a bit weird. A user could call one function in the API
   then another function and think they're doing everything right but there's a
   race condition with the timer that could lead to issues.
 * Phil: What's the PR number?
 * Brad: PR number is 2107. Instead of having two related but separate functions
   where we can do things wrong, we concatenated them into one operation.
   Implementation now better represents that you cannot get the time value if
   the timer is expired. It'd be good to get that in for 1.6.
 * Phil: Given all I'm doing on timers I'm happy to take a look.
 * Brad: Sure. Hopefully it is more of a stylistic thing than anything, but it
   doesn't hurt to have more eyes. I haven't heard any other board-specific
   things that we want in 1.6. Right now the plan would be to start testing once
   the timer and one or two other PRs are merged.
 * Alistair: I'd like to get the CTAP and OpenTitan USB disable in. Then we
   would have a release with USB disabled as it doesn't work in upstream
   OpenTitan.
 * Brad: I agree.
 * Amit: I didn't quite follow, disabling USB and implementing CTAP?
 * Brad: This would include the CTAP communication bus in Tock, but not the
   application logic as that would be handled in userspace.
 * Alistair: It is already in libtock-rs, so it would be nice to have a release
   with both of them. We want to disable USB due to hardware bugs.
 * Brad: And that just basically removes it from the `main.rs`, because there's
   still some issues with the underlying hardware.
 * Hudson: Brad identified a bug yesterday. It is really a longer-standing bug
   that was exposed by my scheduler work. Previously, if the kernel work count
   was wrong the board would never go to sleep because the scheduler would
   always think there was still work to be done and it would spin. I wrote the
   round-robin scheduler under the assumption the kernel work accounting would
   always be correct, and now it ends up in an endless loop if the process
   faults while it is running. That seems like probably something we should fix.
 * Brad: Yes, we should get that in. We're catch it during testing, but we
   should fix it before testing.
 * Phil: I would like to see 2047 -- virtual ADC support -- in 1.6, but if it is
   a big debate then we can skip it. It allows capsules to use the ADC while
   userspace uses the ADC, with everything being multiplexed correctly.
 * Brad: Do you have a sense for what the lead time would be on that?
 * Phil: I personally think the code is ready. My one requested changes is,
   because there are now two versions of the driver. The old one allows
   userspace applications and capsules to continuously sample and lock down the
   ADC forever. The new one fully virtualizes access but doesn't allow
   continuous sampling. Because there are now two implementations of that
   syscall driver, we need them named carefully and better documented.
 * Brad: Yeah
 * Phil: And if push comes to shove, I can write that comment. We can have the
   discussion on the PR.
 * Pat?: Should I put a release-blocker tag on that PR?
 * Brad: I think so. We can undo if need be.
 * Phil: I can be the critical path. This author is generally good but if we're
   blocked, we can unblock us.
 * Amit: I agree as well.
 * Pat: Do you have thoughts on whether 2052 (unsoundness and dynamic grant
   allocation) should be pushed through?
 * Amit: Potentially a better thing for 1.6 is to remove the allocator -- which
   is unused -- now and re-introduce it in 2.0. Can consider design carefully
   then. I am fairly confident the fix is correct, but I was fairly confident in
   the original interface.
 * Phil: Are you confident the new interface is better than the previous
   interface?
 * Amit: Yes.
 * Phil: It might not be perfect bt it is a step forward. Something doesn't need
   to be perfect to be wonderful.
 * Amit: That's true. Lets make it a release blocker and I'll look at it after
   the meeting. Should not have any impact on anything, as it is unused except
   for David's BLE porting.
 * Amit: Anything else?
 * Phil: For the alarm, we are blocked on Apollo3. Alistair, do you have time to
   work? Please let me know if I can help.
 * Alistair: I think I'll look at it today.
 * Amit: Phil, can you confirm what the numbers mean in the output of the test?
   Should we see a close match between the expected value and actual value?
 * Phil: Not necessarily. Wakeups can be delayed due to hardware limitations and
   busyness. For example, if you requested "I want an alarm at 0 in the future",
   it will be pushed something like 8-10 ticks in the future to make sure you
   don't miss it. As the tick frequency goes down, the diff magnitude will go
   down. If you saw lots of diff = 0, that would be amazing, and you could do
   that if you have a 1 Hz underlying clock, then...
 * Amit: But if we're seeing things like diff = `a very very very large number`,
   is that correct?
 * Phil: Where do you see a very very very large number?
 * Amit: HiFive. Emulated in QEMU with a high clock rate. Running out of buffer
   space because it is practically outputting timers instantly. Trying to sanity
   check that numbers are due to fast clock rate not an implementation issue. It
   is the same timer as the Arty board.
 * Phil: Send me the output and I'll take a look. Could be the test is not fully
   hardware-independent. I would be concerned if I saw really wild swings in the
   diffs. It's a percentage variance -- swinging between 15M and 16M is fine,
   swinging between 15M and 100M is not okay. That could indicate a lot of other
   CPU processing is going on, but that would be weird.
 * Amit: It is occasionally basically negative.
 * Phil: Oooh okay, that is interesting.
 * Pat: I also have a block of code consistency changes that I would like to get
   in for 1.6. #1973

## Tock 2.0 Syscall API
 * Amit: Lets discuss combining capsule numbers and syscall IDs in a single
   register.
 * Phil: Johnathan, Amit, Vadim, and I met yesterday and discussed this. Look at
   section 3.1 and 4.3 in the TRD. The first thing that came up was shifting the
   system call ID from `a0` to `a4`. What this means is the syscall arguments
   and return values are in `a0` to `a3`, the C ABI calling convention
   registers. When the syscall class ID was in `a0`, the first argument was in
   `a1`. When you call a syscall, the kernel needs to shift the argument from
   `a1` to `a0`, and `a2` to `a1`, etc, which takes code and time. It's not
   clear necessarily that moves wouldn't need to happen with the suggested
   design, but this way moves aren't forced to happen -- it is possible to write
   a faster path. The question that came up is if we
   jump down to section 4.2, the current approach is `r0` is the driver ID and `r1`
   is syscall ID, then parameters. This mostly matches the function signatures within
   the kernel (allow is the exception). `r0` is used to dispatch on driver ID and
   replaced with the capsule's `self`, then `r1`, `r2`, `r3`, `r4` can be passed
   through unchanged. Vadim pointed out we don't have 2^32
   commands or drivers, so why don't we compress them into a single register
   (e.g. driver in the top 16 bits, the command ID is in the bottom 16 bits).
   This would give us another register. E.g. in command, we could pass more
   arguments. Downside is it's unclear which register it should be. If it is r0
   it will be used by `self` within the kernel. Would somehow need to move
   things around, and may make the kernel use the stack internally. I hope that
   describes the tradeoff and I want to test the wind about peoples' thoughts.
   Is it worth having potentially another argument into command and the
   flexibility that gives worth it?
 * Leon: I wouldn't have a problem with it. I've seen occasions where we are
   assigning semantic meaning to command/allow numbers and we wouldn't have much
   space left for semantic meaning. E.g. using a particular bit to indicate a
   driver is out-of-tree.
 * Phil: We'll definitely have to be more careful about namespace management.
 * Leon: A while back I proposed a driver registry. At driver construction time,
   the driver could look up a dynamic driver number, then use that for calls.
   That would work well with the compression.
 * Johnathan: I think if adding an extra argument to command causes the kernel
   to internally pass things by the stack because it ran out of ABI argument
   call registers, that sounds pretty expensive. I don't know how many system
   calls would benefit from a third argument from command. So far as we can
   combine and make userspace set one fewer registers, that seems beneficial to
   me, because that could eliminate assembly instructions. It's not a huge deal,
   because at least on ARM if you set a register to a small value it's a 32-bit
   Thumb instruction, whereas setting a register to a large value is two 32-bit
   instructions anyways. I don't think we should make the kernel pass arguments
   via the stack into capsules.
 * Amit: Why do you imagine that happening?
 * Johnathan: Phil brought it up. If you make `command` take 3 arguments, then
   within the kernel, the capsules would take `(self, command_id, arg1, arg2,
   arg3)` which is 5 arguments, and runs the risk of spelling to the stack. The
   Rust ABI is undefined, but my understanding is it doesn't have more
   argument registers than the C ABI, and I don't think those calls will be
   reliably inlined. Some of them are virtual calls.
 * Amit: I think if the arguments are 16 bits then they will be packed into a
   single 32-bit register.
 * Phil: It wouldn't be, though. You strip out the major, making `(self, minor,
   arg1, arg2, arg3)`.
 * Amit: You're right, because `self` is a pointer.
 * Vadim: Can we avoid passing minor # into capsules, have the kernel dispatch
   call the correct function?
 * Phil: So if we have an array of commands, and it uses an index?
 * Vadim: Kind of a table, yes. You will skip the match in the syscall handler,
   and prepare a table of handler functions with all the same signature.
 * Amit: That would be effectively re-implementing a vtable.
 * Vadim: That's correct.
 * Phil: Right now you have effectively a switch statement, so it is built in
   code, which is different from putting it in data (an array) because of bounds
   checking. For example, when you index into the array, you have no assurance
   that the userspace value is in-bounds.
 * Amit: Bounds checks and capsules will need a way to register minor numbers.
 * Phil: Would need to have a method like `get_commands` in the `Driver` trait
   which returns a slice.
 * Amit: Would also lose fidelity there. A reasonable thing is to have similar
   implementations for different sub-system calls that do different things
   depending on the value. That is tricky if we're not passing in the minor
   number. Would be restrictive.
 * Phil: This has come up in bus functions, as they use minor number bits to
   select the buses.
 * Vadim: That means you need a third parameter. Because you don't have that,
   you have to pack it in the syscall number.
 * Amit: That is in a sense the definition of the minor number: to specify to
   the driver the particular sub-meaning of the command.
 * Vadim: I see.
 * Phil: These are hypotheticals, we're guessing whath the compiler will do. We
   want to write the code and check assembly. If it's zero-cost -- if we aren't
   spilling onto the stack -- then it seems like a clear win. What is the cost
   we are willing to pay?
 * Amit: There is a potentially useful saving in userspace by combining the
   major and minor numbers into a single register and a potentially separate
   benefit of adding a register to command or other system calls.
 * Vadim: That's a good point. Stack allocation might happen in the syscall
   handler. If the parameter is unused then driver code that doesn't use it
   wouldn't look any different. Thus the cost might be localized to a single
   place.
 * Johnathan: The minimum cost to adding another command argument is userspace
   has to populate the register. If it's used it's beneficial but if it's unused
   it costs one instruction.
 * Vadim: That's what I was thinking: if you put the command and device
   identifier in a register that isn't used by the ABI at all, e.g. `t0` in
   RISC-V or `r12` in ARM, then that register would be caller-saved anyway and
   the fact you put the syscall number in it will not hurt.
 * Phil: What I'm hearing is that if doing this would necessitate causing all
   calls to spill to the stack that would be too high a cost. If the cost would
   only be born by commands that use that argument, then that's okay. Does that
   sound right?
 * Amit: Sounds like to me.
 * Pat: I've been catching up to this. I'm reading more in real time about what
   was going on, but I don't understand the optimization suggested in 3.1 in the first place.
   In ARM, the CPU pushes everything to the stack anyways and the kernel has to
   read it off the stack anyways.
 * Phil: Can the kernel read it from the registers anyway?
 * Pat: They're banked registers. They can be wiped out when crossing
   userspace\<-\>kernel boundary.
 * Phil: That's not the ARM architecture. If you go to ARMv6, the registers are
   the same.
 * Pat: They're guaranteed to be the physical registers? I don't think they
   guarantee that.
 * Phil: They don't push them on the stack in ARMv6. You have to be super
   careful.
 * Pat: Yeah they do, they push `r0` through [unintelligible]
 * Brad: So this is a different discussion which is not related to the sixteen
   bit.
 * Phil: Yeah, thank you Brad.
 * Brad: The sort of long story short is yes I agree with Pat, but it's more
   about maybe somebody could optimize it for one platform and one case. I feel
   like we've talked about the sixteen bit issue every couple of years. Do we
   have institutional memory on why we never chose to compress the bits?
 * Amit: We never went through a major backwards-incompatible system call
   overhaul, that's certainly part of it. Always teetering on the question of
   whether the cognitive load balances potential efficiency is a thin line.
   Didn't have evidence to suggest changing the status quo.
 * Brad: Okay.
 * Phil: Sounds like we should write the code and see what the assembly does.
 * Amit: Doing this in an evidence-based way makes sense.
 * Brad: Too bad we can't include that system call number in there too [ed: this
   was said in a sarcastic tone].

## Libtock-core dev move to 2.0
 * Amit: Question -- if I interpret correctly -- should libtock-core be tracking
   the 2.0 system call interface now or still on 1.x? Is that the question,
   Johnathan?
 * Johnathan: I am essentially rewriting `libtock-rs` independent of the current
   implementation that is exposed publicly. I think it's probably time I
   switched that rewrite to use the Tock 2.0 system calls. In part, because I
   was following the Tock 1.x system call interface and due to the lack of
   unallow I had to write some difficult code to handle allow buffers in a
   memory-safe manner. I realize now I can't use those buffers from an
   application -- the API is bad -- so I either need to go back and change those
   significantly or I need to go to Tock 2.0 where I can pass native Rust types.
   What I'm not sure is how stable people think it is -- little things like how
   you pack things in registers don't matter here, but at the high level it
   seems to be stabilizing. An additional benefit is I would be more likely to
   catch memory safety issues posed by the syscall API design.
 * Amit: It seems it would be better for the project to track 2.0 as it would
   catch issues and help design 2.0. Ultimately your decision -- potentially
   more work in the short term but less in the long term, because that API is
   subject to change and not fully implemented yet.
 * Johnathan: The only big downside is I wouldn't be able to test the apps on
   the Tock kernel, unless I had a kernel that implemented the system call ABI,
   but I can do it using unit testing which seems reasonable.
 * Amit: Anyone else have thoughts, particularly reasons to stick to 1.x.
 * Alistair: I like the 2.0 idea.
 * Leon: A proof of concept of the 2.0 interface should come out in the next few
   weeks, I guess, depending on what our schedule is.
 * Phil: I'm in favor of 2.0 because you'll tell me what we're doing wrong. I
   want to make sure it will work for you and `libtock-rs`.
 * Amit: We haven't given you as much of a headache as normal so this seems like
   a way to bring it back.
