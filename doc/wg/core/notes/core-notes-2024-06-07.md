# Tock Meeting Notes 2024-06-07

## Attendees
- Alex Radovici
- Brad Campbell
- Amit Levy
- Pat Pannuto
- Leon Schuermann
- Johnathan Van Why

## Updates
- None

## Buffer Swapping PR https://github.com/tock/tock/pull/4023
- Alex: Problem with buffer used for streaming/networking. Application uses two buffers that are swapped and driver does not know when the buffer is swapped. This is important for indexing. If we use a command to signal when this swap has occurred this becomes a race condition. 
- Alex: We have two PRs for this. Neither of these PRs solve the generic problem.
- Alex: I've tried to create a more generalized solution for this.
- Alex: To answer Brad's question from the PR, I called this a ring buffer because Leon/Amit called this a ring buffer on the last call (when it is really just a buffer).
- Leon: One design is to just swap buffers, the other is a ring buffer that wraps. The ring buffer seems like a more general design. 
- Alex: How do you imagine a ring buffer that is not contiguous in memory? How would the application consume the buffer?
- Alex: Would there be mutability/immutability unsoundness with this?
- Leon: There is not unsoundness since we are switching between user/kernel space and only one is running at any given time.
- Leon: With the ring buffer design, we would never switch buffers.
- Leon: I think your proposed design is more useful.
- Amit: What role does the checksum provide?
- Alex: To avoid accidental/invalid values. This is just an XOR between the first three bytes.
- Amit Which value would be invalid?
- Alex: This is to check that the application placed a zero on the first four bytes.
- Amit: What specifically though does the checksum provide?
- Alex: This avoids not being able to use a buffer that is shared without being properly initialized. If the checksum fails and the buffer was previously used for something else, the capsule would reset the buffer.
- Leon: I am skittish of having this check since this is not something we check against in other implementations.
- Leon: The kernel assumes the app only allows memory that is not being used by other apps. I do not think it is the kernel's job to manage this.
- Amit: We are not trusting the application, we are just allowing it to shoot itself in the foot.
- Amit: Hypothetically if we do not have the checksum, what is the worst thing that can happen?
- Alex: The capsule would not be able to write to the buffer for incorrectly formatted buffers.
- Alex: With the checksum, the kernel can detect misformed buffers (not set to zero) and can reset this from the kernel.
- Alex: I do not feel strongly about including the checksum.
- Amit: What questions remain with this?
- Brad: My question is do we want a ring buffer? 15.4's ring buffer that Tyler implemented overwrites old packets if it fills.
- Leon: Alex's proposed design is simple. It is not clear to me if we should have a ring buffer overwrite data or simply block until data is read from it.
- Brad: The benefit of this design is the simplicity. The challenge for this with a ring buffer is the size is not necessarily known, whereas in 15.4 the packet size is known.
- Brad: It would be hard to implement a ring buffer that wraps "mid packet".
- Leon: Assume we get a packet from 15.4 and want to send this over to ethernet. Alex's proposed design would be easier to pass this to other subsystems. A ring buffer would be more challenging to slice and take ownership.
- Leon: Specifically, I am making an argument for this design being better than nothing. 
- Amit: I do not think there is a debate here.
- Alex: I will rename the PR to not be named ring buffer.
- Leon: To be clear, a ring buffer would be useful elsewhere, but for Alex's ADC streaming case, a ring buffer is overcomplicated.
- Amit: The answer seems to be we do not need a ring buffer for this and should rename this PR to something more specific.
- Amit: Would this work for 15.4?
- Brad: This seems like it would work for 15.4, 
- Tyler: I agree that this seems like it will work for 15.4. In 15.4 we currently use two buffers that are swapped by application. Each buffer is a ring buffer that overwrites the oldest data. This buffer is of size n * 15.4 packet size. 
- Tyler: It would not be much of a change to switch to Alex's proposed design.
- Brad: ADC driver has multiple allow buffers and iterates through them. There is only one allow buffer for this PR.
- Alex: This PR does the same thing but reduces the number of system calls. 
- Leon: How would this be used for a network layer?
- Alex: The idea would be to have multiple packets since the app is slower than the kernel.
- Alex: From the point we notify the app, new packets are received and we are losing buffers.
- Leon: So you are swapping buffers still.
- Alex: My opinion is if we need to read app/kernel 
- Brad: I still think this is a good solution.
- Brad: Summarizing, the kernel gets data from source, if the data comes slowly, most drivers are fine. The challenge is if data is received quickly. The kernel always needs a buffer that it can place data into. To do this, we need multiple buffers that are always available. This approach uses one allow buffer that we can place multiple things into. The upcall will notify the app there is data. The app allows new buffer (which is essentially atomic operation). 
- Amit: Without this, we have the ability to swap buffers atomically. What is missing from that?
- Brad: The application may not do the swap quickly enough. 
- Amit: This is just an accounting mechanism to say that a buffer will contain one or more things.
- Leon: One other side to this is the accounting parameter can tell the kernel in a unified manner where it should start writing again. 
- Alex: One thing is that this design loses new packets. Tyler mentioned 15.4 loses old packets.
- Leon: Having non fixed size packets is important too. 
- Amit: Won't the packets do this themselves?
- Amit: We could also get around this with prepending this in the driver semantics.
- Alex: Should I implement the ring buffer functionality as well? With fixed or variable packet size?
- Amit/Brad/Leon: I vote no.
- Leon: It seems the main benefit of this is the streaming aspect of this. Presumably the name should reflect this.
- Amit: It seems this is a network queue.
- Tyler: I will look this over in more detail after the call to confirm this will work with 15.4, but it does not seem like it will have issues.

## Yield Wait

- Brad: The todos are to review what is implemented, see if it needs to be updated, and perform more testing.
- Alex: I can work on this and take ownership of this.
- Amit: This is very high impact and important.
- Leon: The need for this came up when preparing for the tutorial and created a lot of issues. Having yield wait will allow for more easily creating reliable apps.
- Alex: I have had issues with this as well.   

## Storage Permission TRD https://github.com/tock/tock/pull/4021

- Brad: The existing implementation stems from TBF header storage permissions. This is very similar, but has some minor changes. This goes with AppID to identify who is the owner of anything that is stored. The storage permissions header did not use AppID since AppID did not exist at the time.
- Amit: Is the intention for this TRD to dictate the default policy if otherwise unspecified? Is the implication of this TRD that there cannot be a storage driver with a different policy?
- Brad: How the policy is set is a separate discussion. Different implementations of different drivers can have different policies.
- Brad: The intent is that if there is nothing that disqualifies an app from accessing it's own state, it should be able to do this. This design makes it more clear that this needs to be specifically excluded.
- Amit: Rephrasing the question, if some other stakeholder does not agree with the specified policy here, does this TRD rigidly enforce this? Would they need to fork?
- Brad: If you implement something that stores state, you need to have an API that gets permissions for applications. You can't implement your own permission system (have to use the kernel permission). You also have to use AppID to identify state attached to an application. Using a different permission system would require forking.
- Johnathan: This was stabilized four years ago with the threat model being stabilized. This TRD seems to just make the storage TRD compliant with the threat model we've already stabilized.
- Brad: This is not meant to codify how you implement this trait, it allows for others to implement the trait as desired.
- Leon: One question, does this relate in any way to richer abstractions (file systems) we wish to implement in the future. How would or does this relate at all?
- Johnathan: The analogous unix abstraction for AppID here is user.
- Leon: I want to ensure that this TRD would not restrict us from implementing future implementations.
- Amit: You could imagine attaching a permission implementation to each storage system object.
- Leon: It may be beneficial to add a sentence stating and clarifying this for other uses and that this is just the canonical case.
- Alex: I am planning to implement FAT this summer.
