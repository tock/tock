# Tock Core Notes 4/10/2020

Attending:
 - Amit Levy
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Jean-Luc Watson
 - Philip Levis
 - Alistair
 - Samuel Jero
 - Garret Kelly
 - Pat Pannuto
 - Hudson Ayers
 - Brad Campbell
 - Andrey Pronin
 - Vadim Sukhomlinov


## Updates!
 * Amit has an undergrad working on a simulator for Tock (that can run locally on Linux/Mac) who has made quite good progress. Would have mechanisms for running kernel on x86 and mechanisms for simulating hardware (like a temperature sensor).
 * Brad has a student working on composing applications out of multiple processes, pipes for communication between processes, could be useful for intermittent computing style programming. Processes run in order provided to kernel as a schedule.
 * Pat has a student working on getting a Lora implementation into Tock, and hopefully a PR will come this weekend!


## Tock 1.5 - Where are we?
 * Brad linked the tracking issue in the chat -- #1685
 * Since original requests, there have also been requests for fixing all timer related bugs and stabilize GPIO syscall interface
 * reminder that we can revise these if we think things aren't as important
 * Brad's opinion is we are really waiting on the updated time HIL, but that HiFive rev1b support is more optional (if it shows up great)
 * Phil: Complication for this is timer support on RISC-V. If we do an update for the HIL (and I think gendx is happy to do the nrf, and amit and I can do the sam4l), but who will do the RISC-V timer
 * Amit: My understanding was we didn't need the new time HIL for 1.5, we just wanted to patch fix three bugs (one of which already fixed)
 * Brad: Is it worth fixing it on the current interface if it is just going to be changed anyway?
 * Phil: I think the virtualiziation might end up being a pretty clean re-write but I can do that
 * Brad: Can probably do the RISC-V part of the timer rewrite
 * Discuss: Can we fix those bugs without the full timer HIL re-write
 * Phil: #1651/1513 are basically the same bug
 * Phil: Those bugs are mostly userspace exacerbating a fundamental problem with the API
 * Brad: If we fix the bug in the kernel but provide no guarantees a userspace process will handle it well, is that good enough of a fix for userspace
 * Pat: we could say we do not support short deadlines in userspace
 * Amit: We could also do a more comprehensive fix without actually waiting on the new time HIL
 * Amit: I would like to fix this, but lets put a < 1 week time limit on this because we need to do a release
 * Phil: hacky fix is hard because it is dependent on underlying clock speed
 * Pat: I am just looking for a bridge to 2.0. Lets pick a big conservative number and say no timers this close
 * Phil: Let's just say that something in the future in the MSB is assumed to be in the past (calls this the 2^31 fix)
 * Jonathan: Concern about how this will affect testing platforms that using very fast clocks
 * Brad: If we don't fix this timer bug before 1.5 we are failing to meet one of the major motivations for 1.5, so would we then need a 1.6
 * Amit: I think it is worth trying to fix the other bug but we have already fixed a major bug. I think a release is worthwhile whether the other one gets fixed or not given a time limit
 * Brad: My perspective is that not fixing 1651/1513 is inadequate, but at the same time we need to get this out
 * Brad: Lets get this done fixed out by the end of april (20 days)
 * Amit: I agree. One major value of a release is that a lot of stuff has changed in the kernel and we have not done extensive testing since the last release.
 * Phil: I can commit to doing the 2^31 fix, it is a simple thing that will fix 1651 and 1513.
 * Amit: I am still committed to testing HiFive rev1b in QEMU
 * Brad: I am still leaving that as optional
 * Amit: Limit for both of these things is next meeting - 1 week from now we start testing either way.
 * Hudson: Plan for Travis tool is to report for stuff from upstream Tock repository, and just print to build log for others. Will push reporting for PRs from forks past this release
 * Group: We still need to stabilize the GPIO system call ABI (be that saying what we have now is good or just pushing it past this release)
 * Last call for comments on 1.5 -- none
 * Summary: Out the door by end of april. 1 week merge window starting now. Major components are the time fixes and whatever other PRs are merged before then.
  
## Updated Component Interface Discussion
 * Pat dropped a link in the chat (PR #1618)
 * Summary: two not great options. Trait interface is not that meaningful and uses weird associated types to sort of enforce convention. Alternative is also kinda confusing and may have issues with circular references.
 * Brad: Check component changes in this PR. We are pretty happy with static\_init!() changes.
 * Brad: not sure about best way to explain the current status of this
 * Amit: Everyone take a look at the trait being proposed and the two instances of that trait.
 * Phil: I am having trouble wrapping my head around the problem
 * Brad: the current component is very verbose, usage is not that consistent, not clear what should go in new() vs finalize(), seems like not a lot of instruction for a component author
 * Phil: (took a look at before/after for alarm component) I buy this, it looks good
 * Brad: New issue is you have to use PhantomData, and that information about what a component requires is now captured via the associated types which is very Tock specific and not a typical way a Rust dev might expect to see that stuff
 * Amit: Ah, so this is also a problem for how docs.tockos.org will be able to show info about components
 * Pat: Yeah, and in general its just confusing to document/think about components where only field is nothing or phantomdata
 * Amit: Before the struct also had some fields the actual driver didn't need (like kernel reference which was just needed for the component to create the grant)
 * Amit: I don't totally buy that it is harder to document
 * Pat: gendx last comments about how this works for more complicated types are relevant
 * Brad: I don't think the new interface is all that clear and is in some ways might be more confusing
 * Leon: What if capsule could implement Component trait itself?
 * Amit: biggest problem is that capsules cannot use unsafe but create() requires unsafe
 * Amit: Would be great to get someone (gendx?) to summarize discussion and current state of things in a comment.
 * Brad: also tempted to try out current component interface with new static\_init interface and see how much better that is vs. what we have now
 * Amit: We clearly would have to actually try this for all the components to get a sense of how everything will work
 * Amit: I thought the original reason for splitting stuff between new() and finalize() was that the reset\_handler might have to do something between new() and finalize()
 * Leon: Yeah I think that is because of the possibility for circular dependencies.
 * Brad: I think I see what you are saying but also I think all component uses are currently one line.
 * Leon: I think that this is because all of the circular dependencies are now isolated to one component, so this works currently
 * Leon: If we had interdependencies between components that create circular references we could run into these issues again
 * Brad: I think that was really helpful even though we did not come to a conclusion.
 * Amit: Can gendx (or brad/pat) summarize this discussion on the PR, and then we can all take a look at it?
