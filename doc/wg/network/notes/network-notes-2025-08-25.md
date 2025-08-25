# Tock Network WG Meeting Notes

- **Date:** August 25, 2025
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. IPC Discussion
- **References:**
    - [IPC Design Slides](https://docs.google.com/presentation/d/13mYERv0iKvBPOsu52jsd4jjZalbd_KXHbwDGdt8qgRw/edit?usp=sharing)


## Updates
- Tyler: Comment in Slack about profiling Thread performance in Tock. I need to go through that and consider.


## IPC Discussion
### Updates
 * Branden: I spent some time thinking through issues we discussed last time and considering application scenarios to see how the mechanisms would work for them.
 * Branden: I'm still tracking stuff in the slides here, which evolve as I go: https://docs.google.com/presentation/d/13mYERv0iKvBPOsu52jsd4jjZalbd_KXHbwDGdt8qgRw/edit?usp=sharing
### "Process Descriptors" Mechanism
 * Branden: One consideration was how to validate clients. I'm considering a mechanism where the IPC Manager capsule would do so, via some specification within the Board file (possibly a service-specific specification). The client would be validated upon discovery, rather than require validation at each time-of-use.
 * Branden: A separate option would be to keep the kernel out of it and pass off some information to the service itself to validate? Not sure it would have the right information or capabilities to do so.
 * Tyler: Interesting idea to have the board file explain the functionality you expect. Not necessarily a problem, but different.
 * Branden: I think right now if you're validating processes for loading, the board file has a specified mechanism for that based on the TBF header. So I could imagine this being similar. I'm not 100% sure how that works, but I suspect a conversation with Brad would show the required mechanisms to be like 90%+ existent already.
 * Tyler: Why do we need client validation/authentication? Is that something Microsoft wanted?
 * Branden: Good question. I think so. It seems like a good idea to be able to have. But maybe process loading authentication is sufficient?
 * Tyler: Have we seen authentication in other OSes? Maybe we shouldn't twist ourselves trying to design something super secure
 * Branden: I'll have to think about that some more and look into other OSes. I don't think I've seen it...
 * Tyler: You could imagine some higher-level system where you ignore messages from apps unless they have some pre-shared key. That would keep the kernel out of it.
 * Branden: Great point. Want to not overly complicate things, although want to make sure that it's possible.
 * Branden: So, let's set aside the how for now. There's a secondary problem of how to we track process-to-process communication validation. The way I'm considering is "process descriptors", like "file descriptors". So the app would be given an opaque number that maps to some data structure stored with the process by the kernel with "open processes". Validation would occur before opening them, just like with files. Once attached, the app could just use a single number to refer to some more complicated process concept. This, importantly, would stop applications from just crafting process IDs by guessing, which would break validation. A downside is that this means we need dynamic space attached to each process to track this list of process descriptors.
 * Tyler: I think we can get away with using the grant for that. We could create dynamically sized memory in the process's grant space to hold the process descriptors. I think that mechanism maybe exists, or the idea for it anyways, but also maybe that no one is actually using it at all.
 * Branden: I agree. A conversation with Amit about that would be useful, although we should perhaps focus first on your question of whether we need client authentication at the IPC level anyways. And related is whether we need Service authentication either...


### Application Scenarios
 * Branden: A second thing to talk about are applications scenarios. I have just one right now, a Thread service and client example.
 * Branden: In this scenario, a Thread Service exists using OpenThread and IPC. Clients can send packets to the service to be sent over Thread. They can also register with the service to open a port and get all messages sent to that port. I think this maps to the existing Thread tutorial example?
 * Tyler: Yes. Right now kernel receives a packet, bubbles to userspace receive callback which goes to OpenThread. Then the app can register a callback from OpenThread which gets called with the packet. Importantly, you pass in the function to call, so we have a lot of control there.
 * Branden: I also mapped out which system calls each thing does. They use the IPC Manager for registration/discovery, then use the one-copy Mailbox mechanism, which does allow-to-allow copies. Each would have an allow-read-only for outgoing messages, and an allow-read-write for incoming messages.
 * Branden: For the service, it would make sense for it to have a FIFO of incoming packets which should be sent over IPC. Each Thread packet arriving goes in the FIFO. Each time a client reads their message, the next message from the FIFO is sent out. If the FIFO ever fills, the Service can drop the oldest message, which is the one it's currently attempting to send over IPC but which the client hasn't read, and can move to the next. So the Service would never block forever on clients, although it could be delayed based on them (there could be a timeout in addition). However, if a client isn't fast enough, it could lose packets meant for it.
 * Branden: Annoyingly, the sizing of this FIFO in the service is based on incoming packet rates and number of running processes, as the client will have to wait to service the message until it gets to run again.
 * Tyler: I don't think it's a huge issue for thread, but you might have different message priorities. You could have a priority queue.
 * Branden: Agreed
 * Branden: For complicated clients, they might also have a FIFO for outgoing packets in case it wants multiple in flight, even though the mailbox only allows one at a time.
 * Branden: Overall, I think this application scenario totally works with the mailbox capsule design.
 * Tyler: I'm skittish about overhead for all of these things. Timing sensitivities are a concern for networking, and I'm unsure of how this compares to what we already have.
 * Branden: I think not much worse? There's definitely a copy going on which takes time. But otherwise the notification callbacks should be on the same order for timing as existing IPC via shared memory. That's just a guess though.
 * Tyler: With thread, we de-coupled timing sensitivities for things that need to be sent/received to keep thread running, from random data transfer packets. 
 * Tyler: What about shared memory?
 * Branden: That would be a separate capsule from the Mailbox. This application scenario didn't need it. However, you could imagine a more complicated and capable Thread service which did use shared memory to allow it to access packet queues (incoming/outgoing) in the application itself. That would allow more packets to get through without dropping any, and for the application to size the queue based on its needs. Definitely more complicated than just sending with a Mailbox though.
 * Branden: I'm annoyed that with Mailboxes we could have one client affected by delays in another client. Especially because it means packets could be dropped because there are too many other processes that are taking full timeslices to run.
 * Tyler: We could have one FIFO per client to solve the client issue. Only drop from slow clients, but other clients don't affect it.
 * Branden: Great idea! Something I'm appreciating about the IPC designs I'm playing with is that it feels like they give services and clients the capability to build endless complexity on top of them as desired. Which feels good.
 * Branden: Finally, we also have to have error callbacks. As it's possible to enqueue a message in your Mailbox for a process that's valid at command-time, but then later crashes before reading the message. So you need an error to know to cancel that message.
 * Tyler: We also have to check process ID validity at command time. That the process still exists.
 * Branden: Yes, definitely.
 * Tyler: When processes crash, that would cost extra kernel cycles now in iterating mailboxes. So restarting processes would waste time in the kernel. Way around that is to not restart processes.
 * Tyler: So performance is still not a big deal here. I think we're okay.
 * Branden: For implementation, we would also need some mechanism in the kernel for the Mailbox capsule to get a callback when any process changes. That way it could determine that there is an error and send a message. That mechanism feels a bit odd, so it'll need some thought. 
 * Tyler: One more thought here, my group implemented something for RedoxOS with multiple threads and async/await and futures recently, similar to some of the proposals for libtock-rs. Something to consider here is how these mechanisms for work with that. We should make sure that there's the right functionality to support Rust mechanisms (or other languages).
 * Branden: Yeah, definitely. Worth talking to Johnathan/Alexandru to see if these mechanisms should have extra functionality to better support userland designs.
 * Branden: My goal from here: I'm going to put together a syscall document for each of the mechanisms I'm proposing. Maybe a draft PR even. And then I'm also going to keep working on application scenarios to validate them. I figure a bunch of design documentation seems like something I could get people to look at with particular insights about authentication, grant space, userland implementations, etc. Then we can argue about those for a while, and later actually implement them.
