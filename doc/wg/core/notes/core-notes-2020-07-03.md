# Tock Core Notes 07/03/2020

Attending
 - Branden Ghena
 - Brad Campbell
 - Amit Levy
 - Hudson Ayers
 - Johnathan Van Why
 - Pat Pannuto

## Updates
 * Brad: The IPC issue was actually a kernel stack overflow
 * Branden: Why does a bigger grant use more kernel stack
 * Brad: Because you have to come up with the initial version of the grant
   region that you are going to use and then write that to the grant region.

## Tock 2.0
 * Amit: No time to meet yet, hopefully meeting next tuesday, have
   asynchronously talked in the meantime about what is straightforward vs. what
is not and will require some design
 * Amit: Easy ones: read-only allow, exit syscall
 * Amit: Amit has implemented read-only allow, and that has been pretty
   straightforward.
 * Amit: Can view differences in PR on Amit's fork:
   https://github.com/alevy/tock/pull/1.
 * Branden: How is this implemented in Rust? Do you copy?
 * Amit: Nope, use a new syscall with a new type called SharedRO that does not
   let you call DerefMut (it doesn't implement those traits) so capsules
receiving those types cannot use the API that would allow for modification of
that buffer
 * Amit: Still not well tested
 * Amit: One question this raises is whether it gives us more free reign to
   modify system call numbers. Currently allow is 3 and memop is 4. So now
allowRO is 5, which is a little weird but probably doesn't matter that much,
but it might be nice to rearrange. If we are doing breaking changes anyway...
 * Brad: I think this is fine
 * Hudson: I think this is *good* because it ensures that any apps compiled for
   1.x will completely fail on 2.0, rather than working at first and maybe
failing weirdly.
 * Brad: What are the params to allow()? Why does this have to be a different
   syscall?
 * Amit: There are hacks, we could probably use the bottom or top few bits of
   the last argument to allow() but think its better to use a new number

## .app_hack section
 * Brad: Some boards have a section in the linker file that sits right after
   the kernel in flash and tells the kernel where apps should be in flash
 * Brad: One of the issues with doing this in linker files and that whole
   toolchain is you have to have content in this section to actually make it
   real. so .app_hack puts 0s in this section and then the binary is created as
   large enough to actually include this section, and then eventually we use
   objcopy to actually put apps in
 * Brad: Huge unfortunate side effect is that installing a kernel deletes all
   apps
 * Brad: This was fine at first, but now this has been copied like 3 or 4
   times, and I don't fully understand why it is still there and do we still
   need it and can we get rid of it
 * Amit: That is a good question. I wonder if maybe Johnathan has thoughts? I
   think titan is one of the use cases for this because you want a single
   binary for the whole image?
 * Johnathan: I had not looked into how this is done upstream. The way we did
   it in tock on titan is...let me look
 * Johnathan: I believe there was a seperate section in the kernel layout that
   apps were loaded into
 * Johnathan: The app memory is something defined in the build file, so sounds
   like maybe the same hack
 * Amit: So do apps disappear when you reflash the kernel?
 * Johnathan: Yeah, we flash a monolithic image so we can sign everything
 * Amit: The reason that things get erased is that in order for the section to
   be preserved the hack is to insert a word in the linker script?
 * Brad: Nope its in main.rs. 
 * Amit: Right...it seems it should be plausible to preserve that section
   without having to write to it, and we never really dug into how to do it
   better.
 * Brad: I guess one question is, is it important that objcopy be able to work
   with these elf files when creating a binary? Is it enough if tockloader can
   do it? Also, do we need to support concatenating binaries.
 * Amit: Obviously it needs to be supportable, but is it important for upstream
   Tock to do this explicitly?
 * Brad: There is nothing all that complicated happening, tockloader can do
   this easily, the only trick is it is hard to know where the start of apps is
   supposed to be (that is not necessarily encoded anywhere)
 * Brad: If we want objcopy to be able to do this we need a section. But
   obviously for many tock boards you can't do this.
 * Hudson: My thought is that most people do this because it saves them from
   having to submit a PR to tockloader to add support for apps. I think that
   once tockloader supports each board we should be able to remove this.
 * Hudson: Not sure about if we need this for QEMU/Verilator
 * Amit: Definitely don't need it for QEMU, you can place binaries at arbitrary
   locations in memory
 * Brad: Sounds like not strong feelings on this, and tockloader support
   probably good enough.
 * Brad: It makes sense to replace this mechanism with one where you can still
   get the concatenated version
 * Amit: The kernel starts with a vector table that indicates where code
   actually starts. We could have a header for the kernel that encodes certain
   things to make this easier.
 * Hudson: Could we have a solution where the elf has a section in it so
   objcopy still *just works*
 * Hudson: Like could we read the memory before we set it all to 0s and set it
   to what we read instead?
 * Brad: I don't think so, the section is part of the kernel so before we
   actually get to the point where we are setting it to 0s it has been
   overwritten as part of reflashing the kernel
 * Amit: Can we get a link to this?
 * Brad: Yeah look at an stm32 main.rs
 * Amit: Yeah I see...it would be great it we could just tell the linker to not
   garbage collect this section?
 * Hudson: Gonna show my ignorance here, are sapps and eapps fixed in flash
   even if the kernel grows?
 * Brad: Yes
 * Amit: Do we even use .eapps?
 * Brad: Yes because everything is a slice
 * Brad: I still owe Johnathan a PR to use slices everywhere we still use
   pointer + length
 * Amit: Oh, I see, .eapps is set to be the maximum possible size
 * Brad: Yeah
 * Amit: I agree that having this hack is gross and clearly unnecessary if we
   don't care to support this mode of creating a flat image, but it would be
   much nicer to preserve the section in the kernel elf otherwise if we can do so
   without overwriting it on flash. Which seems probably doable.

 ## SysTick
 * Hudson: Systick is the name of an (optional) ARM hardware timer peripheral
   which is designed to be used as a “System Timer” or “Scheduler Timer” in an
   embedded or real-time OS. It is intended to be used as a timer dedicated to
   providing periodic interrupts, where each interrupt will invoke the scheduler
   allowing the system to change between tasks etc. In Tock, we use this
   peripheral solely to preempt apps after they exceed some timeslice. This was
   fine when we only supported ARM chips, mostly because all of the boards we
   support used chips that included this optional peripheral
 * Hudson: However, there is no such thing as a Systick in RISC-V, it is an ARM
   specific name. Today, none of our RISC-V chips have support for preemption
   of apps, meaning that on RISC-V Tock cannot provide one of its core guarantees;
   that apps cannot starve one another.
 * Hudson: Looking into our currently supported RISC-V chips, 1 (maybe 0?) of
   our 3 supported chips have more than one hardware timer peripheral. So
   virtualization of this single hardware timer is necessary in order to provide
   alarms to capsules/userspace as well as to the scheduler. I decided to
   implement this for the hifive1 and opentitan (really e310 and earlgray) and in
   the process realized that the existing trait was pretty specific to the
   functions available to the ARM peripheral. For example there was an assumption
   that there would be a dedicated hardware interrupt for just that alarm
   (systick_handler in arch/cortex-m) which could be used to set a global static
   mut boolean to inform the scheduler when timeslice expirations happened. Also
   the existing trait was just kinda confusing -- its weird that enable() starts
   the timer and configures whether interrupts fire, and unclear whether calling
   it resets the timer (it doesnt). Assumes the timer counts down (mtimer counts
   up). Uses terms like overflow(), which assume that the timer wraps around,
   which isn’t the case for the RISC-V mtimer, etc.
 * Hudson: So the reason for 1985 is to Remove aspects of the trait that are
   really based on ARM assumptions (systick being the name, having a dedicated
   hardware interrupt, assumptions that it counts down etc.) Implement the trait
   for 2 risc-v chips using virtualization to show that it is possible to share
   the underlying hardware peripheral
 * Hudson: Rajiv’s PR was based on concern that the current design allows for
   timeslice expirations to be missed if they occur after a process returns to
   the kernel but before interrupts are disabled, but based on his reply last
   night he seems content that the new design fixes this issue without reliance on
   his changes.
 * Hudson: Scheduler PR just adds a way to get the time remaining so that
   timeslices can be fairly preserved across bottom half interrupt handling
   (there are probably other ways this could be done but this seemed the easiest
   way to start)
 * Brad: So to confirm, your PR does not change what time is attributed to
   processes?
 * Hudson: That is correct. And Rajiv's concerns on that point were more about
   the potential for race conditions than a concern about what time is
attributed to apps

 ## Closing Thoughts
 * Branden: Tock 5 year anniversary 1.5 months ago!!
 * Brad: Originally I was counting from when the README was created, but thats
   not until August
 * Brad: I think that having a bit of a retrospective paper could be
   fun/interesting
 * Brad: The one that stands out to me is we were all about capsules, but if
   you look very few of our commits go to capsules...maybe that will happen
   eventually
 * Pat: Its all drivers and chips
 * Brad: Its also that its easier to just not upstream capsules
 * Amit: Yeah it will be fun to go back and look
 * Pat: Will also be interesting to see where we diverged from what embedded-wg
   has chosen to do
 * Amit: Yeah that is a painful question that I have avoided
 * Brad: Painful?
 * Amit: We really tried to engage with each other at the start and it just
   kinda fell apart and diverged
 * Pat: There are some fundamental things that ought to be shared, like the
   register interface
 * Branden: They have a register interface that is macros on macros
 * Pat: Yeah theirs feels more like C code
 * Amit: I think their is a little of a "not written here philosophy" in both
   camps
 * Amit: They have been pretty wary of adopting stuff that we have done, though
   I do think their focus is different. They want to make it as easy as
   possible to do bare metal programming and we have more of a focus on security
   and robustness so we cannot put as much of an emphasis on ergonomics as they
   are.
 * Amit: But I am sure if we investigated we would find ways that we kinda
   messed up
 * Brad: A shame to have watered down effort across 2 projects
 * Branden: My surprise is how little effort has gone into the application side
   that wasn't a big makefile rewrite
 * Pat: Well, there is alexandr's PRs to libtock-c for the class he is teaching
   in Tock!
 * Pat: And it is a success story if apps dont have to be rewritten 
 * Brad: I attribute that to our wireless story being less robust than it could
   be, hard to build true IoT apps.
 * Hudson: And often its harder to write apps that are upstreamable than to
   write apps/capsules that are usable on a single board.
