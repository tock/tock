# Tock Core Notes 07/31/2020

Attending
 - Johnathan Van Why
 - Brad Campbell
 - Leon Schuermann
 - Hudson Ayers
 - Phil Levis
 - Vadim
 - Andrey
 - Pat
 
 
## Updates
 * Phil: just got an opentitan board, should be able to test time stuff on risc-v
 * Johnathan: libtock-rs CI is broken and being difficult to fix, I am working on it, would apprecaite reviews despite the red X
 * Brad: That may be because hi-five is broken in current master bc of stack issues

## Tock 2.0
 * None
 
 ## Tock 1.6
  * Brad: The time stuff is clearly coming along, and the scheduler PR was merged.
  * Are there any concerns or requests for 1.6?
  
  ## Atomic context switching
  * Brad: we found that a particular combination of apps -- the catchall hail + whileone -- didnt work after the scheduler merged.
  * Brad: The reason for this was ultimately that an interrupt could arrive after the scheduler checks for interrupts but before the code switches to userspace, causing the interrupt to not be serviced until after whileone() returns.
  * Brad: For the nrf521822 serialization library, there is a strict state machine that relies on specific ordering of interrupt arrivals. Specifically, the USART TX interrupt needs to arrive before the DMA RX interrupt that will follow when the chip responds. But if the interrupt arrives in that gap, then the second interrupt arrival is what will cause the userspace app to be preempted, but the DMA interrupt is higher priority, so that callback is delivered first. Really the bug is in the nrf stack, but it would be good to make this not possible in the kernel.
  * Phil: So the issue is the non atomicity of the check + context switch. What is the fix
  * Brad: So on arm, you can disable interrupts, do the check, and have the context switch reenable interrupts (CPSR)
  * Brad: Yep, the reason we bring this up is to run that by everyone, and the tricky part is doing this for RISC-V, and that it makes it so schedulers cannot choose to run an app while interrupts are pending
  * Phil: We should not sacrifice correctness in mainline Tock when the benefit is an open research question
  * Brad: Well, we could use a diff to not have that be an issue
  * Phil: I think for now we should do the first one, and then Hudson as part of  his research can implement and test the more complicated one.
  * Hudson: I am happy with that
  * Brad: Sounds good. Also I have a patch for the nrf serialization library, but gonna wait to submit it until we fix this issue.
  * Johnathan: Maybe we need a random scheduler
  * Brad: I think the basic fix for this is on the map for 1.6
  * Brad: Any other thoughts/questions?
  * Pat: Do we still think our tests are enough? Should we write down what subsystems these tests tend to break, to track what our goals are in the long run for using these tests?
  * Brad: So that would allow us to better capture applicable tests when we run into issues
  * Phil: Is this anything more than for each test saying test x addresses y
  * Pat: I am pushing for more of a mental model of coverage in the kernel -- we want to know that our release testing covered all of our subsystems and peripherals
  * Hudson: Hopefully some of the host side testing stuff will help with that
  * Brad: We could also think of tracking syscall arguments and comparing it against the list of capsule assignments as one method toward determining coverage
  * Brad: Also, just as an FYI for everyone, the process console will now tell you how many grants a given capsule is using out of the total number available.
  
  ## Conclulsion
  * Brad: Sounds like we just need to keep pushing toward these 2 releases
  * Phil: The alarm stuff really is coming along
