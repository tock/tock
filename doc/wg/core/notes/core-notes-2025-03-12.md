# Tock Meeting Notes 2025-03-12

## Attendees
 - Branden Ghena
 - Amit Levy
 - Alexandru Radovici
 - Viswajith Govinda Rajan
 - Brad Campbell
 - Johnathan Van Why
 - Leon Schuermann
 - Pat Pannuto
 - Kat Fox
 - Hudson Ayers


## Updates
 * Branden: Not Network working group meeting this week, but Tock Ethernet stuff is progressing and we're excited to move it into the master branch soon
 * Amit: UCSD verification people found a potential bug in the ARM MPU implementation. https://github.com/tock/tock/issues/4366 It looks pretty solvable, but very exciting that they found it
 * Amit: I'm working with Lawrence Esswood at Google on the CHERI changes and upstreaming them to Tock. We have an open CHERI PR now: https://github.com/tock/tock/pull/4365 We're still working on some other changes that could be valuable upstream
 * Leon: What's the strategy for getting these to land in Tock? Super cool, but a lot of separate changes that maybe should be split into PRs
 * Amit: They are all interrelated, I think. Because of toolchain issues and differences between 32 and 64 bit. And CHERI-specific stuff. So I think they are all related. It's a bunch of changes we pulled out from a bigger changeset, and we think it's the minimal thing that doesn't include changes that wouldn't be used anywhere
 * Amit: Could have been split into unused building blocks, but then they initially wouldn't be used.
 * Leon: It does make sense. Easier to see in context, but harder to check that we aren't breaking something else. Maybe we should have a big PR and split out tiny PRs that advance parts of it. We have different standards of review for different parts of this, for example
 * Amit: Yeah. It might be easier to see if you look commit-by-commit. I do hear you though. A lot of changes in RISC-V is predominantly moving files too. I will admit that the RISC-V changes got the least attention from me yesterday, so maybe those need more scrutiny
 * Viswajith: There was some discussion on the dynamic process loading TRD. Pat created an issue out of the things he found about general process loading, which are separate from normal process loading. Implementation PR is ready to review: https://github.com/tock/tock/pull/3941
 * Pat: Actually two things. There's one line of discussion where it looks like an issue with the threat model, which I split into the issue. https://github.com/tock/tock/issues/4364
 * Pat: There are also process-loading design issues, which notably don't have a document defining them. So at the moment we have an implementation without a design document. So I think we could merge the implementation of dynamic process loading right now, but we should probably expand the design doc to also include all process loading. That will take a lot longer though, so the implementation should move forward and we could update the implementation later
 * Amit: Okay, so the TRD is still in discussion, but the implementation is ready for review

     
## Type Summary for Syscalls
 * https://github.com/tock/tock/pull/4228
 * Amit: This is now approved with a long enough wait period to merge. I figured we could confirm on this call though
 * Johnathan: There are still comments on the PR from Brad and I that I think haven't been responded to? Maybe not looked at
 * Pat: I haven't looked at this in quite a bit. So I think I need to handle those. Hopefully some quick fixes though?
 * Johnathan: What to do with integer types that are ambiguously encoded is a longer issue unfortunately
 * Pat: Okay, this will go back on my queue for effort
 * Amit: I will say that I'm not sure what's unresolved
 * Brad: My comment is about upcalls, which is useful, but not blocking to this PR
 * Johnathan: We had the discussion last time and I left some comments after our last meeting. I don't have any big concerns anymore, but I do have some small things
 * Amit: Okay, so I'm unclear on what the actual outstanding stuff is
 * Pat: I think it was just changing the uint pointer to a void pointer, which I just pushed
 * Johnathan: I will link you to the others
 * Pat: I'll handle those
 * Amit: Meta question: I almost merged this last night since it was last-call and approved. How do we better understand if there are issues moving forward?
 * Pat: I always look for unresolved comments on the PR
 * Amit: On Github, I tend to think of Approved as approving, and Request Changes as blocking merges. But just a Comment doesn't block anything. So we should probably be marking as Request Changes when we mean that
 * Leon: There is an option to disable merging for unresolved conversations. It only counts review comments though, not other comments.
 * Johnathan: The annoying thing about Requested Changes is that you can't indicate which comments are dealbreakers and which are not. It also relies on people updating their status later after things are resolved
 * Pat: You can submit some comments as a batch review as Comments, and then you can review again and Request Changes with dealbreakers


## Isolated Nonvolatile Storage Capsule
 * https://github.com/tock/tock/pull/4258
 * Amit: This is ready for review now. The thing Brad thought should be a todo item is no longer blocking us
 * Brad: Yeah, that was just for testing and is too hard to do right now
 * Brad: This is adding a new interface for userspace to store persistent state in storage where each application gets its own region of persistent storage space. The size is fixed by the kernel. It can be stored anywhere the platform wants, the example is on an external flash chip. It does support 64-bit address spaces
 * Leon: Does each application get the same size of storage? Could you choose not to give storage to some apps?
 * Brad: Every app gets the same size. And any app with "storage permissions access" will have access. Any app without that permission wouldn't need it
 * Leon: And it's on-demand allocated, so those wouldn't need storage


## Tock Planning Workshop
 * Amit: Topics and questions for discussion for the Tock planning workshop at the end of the month
 * Amit: So far we have one topic which is Rust userland support
 * Amit: Goal for now is to brainstorm other items that I'll use for planning
 * Branden: A priority list for networking interfaces and protocols sounds pretty valuable
 * Amit: Dynamic process loading and position-independent code for userspace
 * Alex: Revisiting the USB stack. We have a number of boards without debuggers and USB isn't working great for them
 * Leon: High-level CI testing plans now that we have some infrastructure
 * Branden: Non-execute-in-place, running apps in RAM
 * Branden: As a clarification, I think the goal here is to talk about priorities and focus for areas, not as much about the nitty-gritty low-level details
 * Brad: A question, when you said position-independent code in userspace what did that mean?
 * Amit: There's overlap with non-execute-in-place. There are other platforms where instruction and data RAM are separate
 * Brad: Auto-generation of system call interfaces
 * Pat: Opening the door to advanced proc-macros. We avoided it a long time ago, but now it's part of the Rust foundation and it's crazy powerful. Meta-conversation around procedural macros
 * Amit: That might feel like a relatively narrow topic of discussion. But maybe it's external dependencies in general and the appetite for them and how to deal with them
 * Leon: We did have extensive discussions about proc-macros on some core calls because of the tock registers redesign. I think we had a resolution to allow them for tock registers specifically
 * Amit: I think this could be a good chance for others to speak up about what their thoughts are on these kinds of issues with their own domain considerations. Like certification or company needs
 * Amit: Tock 3.0 system call ABI. Wants and desires here for the next version of the system call interface
 * Branden: Tock training materials, how to get started and videos
 * Amit: We won't possibly do all of these, but this will be a list I can pare down
 * Kat: The cryptography interface and what those capsules will look like
 * Kat: Also, testing strategy and fuzzing at the syscall level
 * Amit: And generally verification. Great

### Amit's summarized list from chat:
* Priorities for networking group, e.g. WiFi, Bluetooth, etc
* Dynamic process loading and PIC for userspace
* USB stack
* High-level CI testing plans
* Non-XIP
* libtock-rs
* Auto-generate system call interfaces
* Opening the door to advanced proc macros, plus also external dependencies, c-libs, what is the community appetite
* Towards Tock 3.0 system call ABI
* Tock training materials
* Cryptography interfaces
* Testing, fuzzing (e.g. at syscall level), verification


## Cross-Platform Testing
 * Leon: I'm trying to write unit tests for an old PR about PMP regions. RISC-V has different constraints around allowed PMP address ranges on different platforms. I'm trying to write unit tests for how we work with those. The functionality needs to change with the architecture. But if we run tests on a 64-bit x86 PC, then the architectural width doesn't align with our testing
 * Leon: So the question is how to do cross-platform testing
 * Johnathan: I wonder if Miri supports this as an option. It's not clear to me from the help page though
 * Leon: We should also add unit tests on different architectures to CI.
 * Johnathan: The other option is to pass details about the target as a generic parameter. It's annoying boilerplate though
 * Johnathan: Libtock-rs might need similar functionality for its test environment
 * Brad: We do (or could be) running unit tests for the nRF52. We have a configuration board that runs all the unit tests that are normally commented out in main.rs files
 * Leon: I'm talking about Rust unit tests sprinkled in the code. The nRF tests are really more integration tests
 * Leon: So, sounds like there's no obvious solution here so I'll think more on this

