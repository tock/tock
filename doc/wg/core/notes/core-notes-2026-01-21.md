# Tock Meeting Notes 2026-01-21

## Attendees
 - Brad Campbell
 - Branden Ghena
 - Hudson Ayers
 - Johnathan Van Why

## Updates
 * Branden: Tockbot has been broken for two weeks and has not been assigning PRs
   to people. Opened PR #4727 to fix. Several recent PRs have no responsible
   party.

## PanicWriter PR #4684
 * Brad: I couldn't find my Arty 100T to test one of the boards, but I did test
   the nrf52840dk and I tested the QEMU boards. So I think this is in a pretty
   good place. Stuck in the same place as last time, which is that it is a lot
   of work.
 * Branden: Can you upgrade the nrf without changing anything else?
 * Brad: This changes the existing panic functions to \_old and does some other
   renaming to help us track which have been updated.
 * Branden: Is anything still in debate for this PR?
 * Brad: The only thing that comes to mind is "how strict do we want to be on
   the implementation resetting the world when a panic happens?". Is it a
   best-effort thing (ideally it works if the kernel never set up the UART)?
 * Branden: I think perfect is really hard here. Really resetting the world
   sounds really challenging to me.
 * Hudson: I agree, if you are in a panic state and try to reset you are more
   likely to hit some other thing that's broken in a panic. So try to reset
   things immediately for the UART, so best effort. The worst case is you get no
   debug effort.
 * Branden: Johnathan, how bad can it be if the panic handler does something
   bad?
 * Johnathan: If it's undefined behavior, then it can result in garbage debug
   output and time-travel backwards.
 * Hudson: Back when Rust generated UB for infinite loops, when I set the panic
   handler to `loop {}` it caused panics to be optimized away.
 * Brad: Leon has more thoughts on this. I think Leon would agree that this new
   structure makes it easier to handle these things.
 * Branden: It makes it easier for a group to really handle this right.
 * Brad: The x86 port does something really similar. There's a separate struct
   in the chip crate to handle these synchronous writes.
 * Branden: I think this is an improvement on the state of the world and we
   should merge it. Brad, what's your bar? Is this above upkeep? How many
   reviewers?
 * Brad: An approval is helpful.
 * Hudson: It looks like the change is we're storing a buffer in
   SingleThreadValue rather than a static mut. Everything else is safe, right?
 * Brad: I think so.
 * Hudson: Did you test panic output that was using RTT?
 * Brad: Yes. I found we had inadvertently broken tockloader, I fixed that too.
 * Hudson: I haven't fully looked through so I won't approve but I'm okay with
   you merging based on Branden's approval.
 * Brad: It's still kinda in Amit's court because this was originally his
   design. More eyes is helpful.
 * Hudson: This is the perfect use case for having an AI tool port additional
   boards.

## Userspace libraries
 * Brad: It's been a while since we've really looked at our userspace
   libraries. Last merge 2 months ago. We've lost track of yield-wait-for. Are
   we in the phase where we need to just iterate through, or are we waiting for
   something like a compiler feature?
 * Branden: There seem to be two yield-wait-for PRs. #547 and #553. Discussion
   between the two of you, doesn't look like something I should be spending time
   on.
 * Brad: Right. Did we switch to this deferred thing, is that what we ended up
   doing?
 * Branden: I thought we did merge the deferred thing. Why can't I find it
   anywhere?
 * Branden: It's #541, which is merged.
 * Brad: libtock-rs too. Someone from Alex's group implemented yield-wait-for
   but the PR is sitting.
 * Johnathan: I intend to eventually get back to libtock-rs. Tyler sent a PR
   that failed CI but it wasn't the PR's fault. I figured Tyler would fix it.
   I'll eventually go and look back at libtock-rs but I want to make progress on
   tock-registers first.
 * Brad: yield-wait-for looks reasonable. I don't really see anything else that
   is hanging.
 * Johnathan: There's also the Pin-based Allow and fixing CI, which I'll
   eventually do.

## Tock book PRs
 * Brad: Looking at the book, I haven't gotten back to Johnathan. Mostly looks
   good, but there are a few odd things where it's not clear how things connect
   back to design requirements.
 * Johnathan: Are you referring to your existing comments or new things?
 * Brad: Both my existing comments, and kindof in general.
 * Brad: There's also Paul's addition.
 * Branden: It seems reasonable.
 * Brad: That doesn't pass CI, does it?
 * Johnathan: No CI in that repo.
 * Brad: No CI in that repo?
 * Johnathan: There's a Makefile, but it's a bit of a bear to set up.
 * Brad: There's definitely CI, right?
 * Johnathan: I wished, when I opened the PR I was hoping I wouldn't have to set
   up things locally.
 * Brad: There should be, only the last 2 PRs are missing check marks.
 * Brad: It's probably best we just merge his PR and if anyone wants to mess
   with it we can.
 * Branden: It definitely needs a prettying pass.
 * Brad: You know it's always the naming that gets us. It's called architecture.
   Is that accurate? This is really a course, isn't it?
 * Branden: Yes
 * Brad: Shouldn't this be under tock courses then?
 * Branden: I told him to make it a new chapter because it didn't really fit in.
 * Brad: Okay.

## elf2tab PRs
 * Brad: https://github.com/tock/elf2tab/pull/103. elf2tab was rounding to power
   of two size to make things Just Work at different locations, but this causes
   space wastage which Tyler was hitting. How about we do power-of-two for small
   apps but not for larger apps? That could bite if someone is building a larger
   app and concatenating it without paying attention to alignment.
 * Branden: Does anyone concatenate multiple apps?
 * Brad: I think not because I've found it broken multiple times. It's kindof a
   breaking change, but we've never really advertised that people should be
   concatenating TBFs.
 * Branden: We've also never promised that they'd be power-of-two.
 * Brad: So I think this is reasonable. Gives you more flexibility in using
   multiple apps at the expense of flexibility of avoiding tockloader.
 * Branden: We don't have tockloader for every cortex-m platform.
 * Brad: Yes. We should. The alternative that we have upstream is the objcopy
   hack.
 * Branden: Will that still work with this?
 * Brad: Yes, assuming you weren't building two apps, manually concatenating
   them, then pointing the kernel to the manually-concatenated app.
 * Branden: Is that the only solution for running multiple apps without
   tockloader?
 * Brad: Yes
 * Branden: Do any tutorials tell users to do that?
 * Brad: No, I would never let that happen. There is another way to do it -- use
   tockloader to assemble the binary images, then add that to the kernel. That
   didn't exist a while ago.
 * Branden: That's a good solution. My only concern is that fighting alignment
   on these limited systems feels like a fool's errand. I just want to be on
   platforms with more memory.
 * Brad: All things considered it's not too bad, because we still have PIC and
   it shouldn't be too hard to implement a better algorithm in tockloader.
 * Branden: He has a PR that does that too? #128?
 * Brad: I haven't looked at it yet.
 * Branden: I support this, I am conceptually on board. I won't click approve
   without looking at the code, but this has my support.

## Blog
 * Brad: Has anyone merged my blog post?
 * Branden: No
 * Brad: It's from September
 * Hudson: Why don't we merge it.
 * Brad: We should. Well, Leon is not here.
 * Hudson: I think I recall there being a deadline for him to look at it. Am I
   misremembering?
 * Brad: No, I remember that.
 * Branden: Should I approve and merge?
 * Brad: Yes
 * Branden: Alright, it's done.

## Bootloader
 * Branden: Someone asked if tock-bootloader was dead, and I replied that it
   just didn't need much work.
 * Brad: Writing bootloaders is not fun. Ours is also not good.
 * Brad: Someone was working on another.
 * Branden: I don't think we really need it.
 * Brad: Well, secure boot is useful. Shows up in requirements. It's difficult
   to even learn about other secure bootloaders. Feels important for real-world
   adoption.

## tock-registers
 * Branden: Johnathan already working on it.
 * Johnathan: At some point I'll add CI to it.
