# Tock Network WG Meeting Notes

- **Date:** July 14, 2025
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Vishwajith Govinda Rajan
    - Gongqi Huang
- **Agenda:**
    1. Updates
    2. IPC in Tock
    3. IPC in Other Systems
    4. IPC Use Cases (didn't get to it today)
- **References:**
    - [IPC in Tock](https://docs.google.com/presentation/d/1hkgztMHbiTuLGDVfxZur3i1Fl0qFpw1cWMsT_he-VNk/edit?slide=id.p#slide=id.p)
    - [IPC in Other Systems](https://docs.google.com/presentation/d/17hkSP4a3oA_gIjWUJSy2NCa4WzzGc9HbghoIrSgWuUk/edit?slide=id.p#slide=id.p)
    - [IPC Use Cases](https://docs.google.com/document/d/1iL_DPMygbB4XAEZMSESP8Q-xA7Kq04C-B1lLUfB114E/edit?tab=t.0#heading=h.5hhktnom69b3)


## Updates
- Leon: Fixed some long-standing QEMU issues (which is where we test Ethernet). There are a set of versions that work for dev again


## IPC in Tock
* https://docs.google.com/presentation/d/1hkgztMHbiTuLGDVfxZur3i1Fl0qFpw1cWMsT_he-VNk/edit?slide=id.p#slide=id.p
* Tyler: Userspace API. Kernel side is very small right now.
* Tyler: Client/Service model. IPC service and client of that service
* Tyler: Clients discover a service by a string identifier for app. Gets App ID
* Tyler: Client registers a callback function
* Tyler: Client can share a chunk of memory with the service. With implicit restrictions on memory alignment and length. Maybe checked in the kernel?
* Tyler: Key design points: 1) shared memory for all communication and 2) application name for communication means one service per application
* Tyler: Service side. First, register service, with string identifier and callback function
* Tyler: Other action is notify client, which triggers client callback
* Branden: No data transfer on notification?
* Vish: Right. All data sharing goes over shared memory.
* Tyler: Client callback might also get ID for server
* Tyler: On kernel implementation, just uses upcalls and MPU reconfiguration. Buffer is stored as a read-write process slice in Grant. MPU region is added to safely access that.
* Tyler: Seems that you can only use the shared buffer during the upcall right now. I don't think the MPU region added persists right now.
* Tyler: No mechanism for unallowing memory right now
* Tyler: Generally pretty messy and underdeveloped
* Leon: Client can only share one buffer. But if the server has N clients, it needs to support N buffers, which hits MPU hardware limits. Often, we can only have one or two buffers max, but it's not predictable when you're going to hit that limit and not clear that the implementation handles it gracefully.
* Tyler: With the RISC-V PMP, what are the limits there?
* Leon: Depends on each board. In the most constrained case it's also a single buffer. But it could be more than that.
* Branden: This is a good level of depth for Tock side of things
* Branden: The context: IPC was very early work in Tock -- even before we published the SOSP paper, we've had it for doing basic tests. Thrown together in an afternoon (or something like that). It's been touched only lightly, a couple of times.


## IPC in Other Systems 
* https://docs.google.com/presentation/d/17hkSP4a3oA_gIjWUJSy2NCa4WzzGc9HbghoIrSgWuUk/edit?slide=id.p#slide=id.p
* Branden: Surveyed a couple of interesting operating systems:
  * Hubris
  * ThreadX
  * Rust Message Passing primitives
  * Redox (high-level)
  * For each: how does it work on a high-level, some interface details
* Branden: Hubris is an embedded Rust OS, good docs. Two mechanisms: synchronous message send/receive & notifications (async, single-bit)
  * Synchronous Message Passing
    * 1-to-1 communication, synchronous & blocking interface
    * Receiver blocks until message is available
    * Sender creates message, sends, blocks until response
    * Messages are copied, only copied once
    * Open / Closed receiver: open to any message, closed to accepting from a specific task
    * Kernel can also be a receiver
  * Notifications
    * When you call receive, you could receive a message, but you could also just receive a 32-bit value. That's a collection of 32-bits that [...]
    * This is asynchronous, it will then set these bits to one at the receiver
    * Seems to primarily intended for interrupt delivery in the kernel
    * Not quite asynchronous in the kernel, need to poll for it
    * Can be sent to normal tasks. What they call out in their documentation is that this is a way for a server to inform a client of some information without blocking. You don't want to block a client, instead you notify the client, which can then call back into the server
    * Vish: don't you then block anyways?
    * Branden: Blocking on sending is dangerous as a server, so you block on receive
    * Leon: Threat-model here. Clients could be denied service by the server. Server wants to not be denied service by a client. If I recall, this is exactly the threat model Microsoft would be happy with
    * Branden: And you could add a timeout mechanism easily
  * Interfaces:
    * If you want to send a message, you send it to a specific TaskId. Operation mode (16-bit value), send a buffer of bytes, receive bytes, mechanism to share memory.
    * In addition to sending a chunk of memory, you can send shared pointers to resources. You can have an array to shared resources. Those only exist during synchronous communication. This allows you to send a more complicated thing than a series of bytes.
    * Leon: do you have insights into why one would use one over the other?
      * Branden: leases allow for zero-copy.
    * Leon: do leases actually share memory? Hubris does use an MPU, so they'd run into alignment issues?
      * Branden: don't know, they allow for a many leases (256), we should look into that.
    * Leon: nice high-level design to have two different mechanisms, for different types of operations. But it's weird that they're lumping in both types of sharing memory into the same op.
    * Branden: Many mechanisms with generic send function
    * Leon: Confusing because you are lumping messaging with sharing memory, which means everything suffers from the zero-copy constraints. Cool to have both, but mixing them sounds like the worst of both worlds. Synchronous blocking is necessary to make zero-copy work
  * Receive interface:
    * Branden: receiving blocks until a message comes in. If you do not reply, the sender will never wake up.
  * Task Death:
    * Sending and receiving is all by task IDs. Includes both a fixed ID and a "generation". Each time a task restarts, generation monotonically increases. If you try to send to the server but the server's restarted, you get an error indicating that this is the right task, but it's the wrong generation. On the server side, if you reply and the client restarted, then the reply is just dropped.
    * Leon: This is extremely close to ProcessID. ProcessID isn't strictly a tuple, but it is an instance
    * Branden: The nice thing in Hubris is having knowledge of this being the correct Task ID, but the wrong generation.
    * Vish: AppID is fixed and can do this
    * Leon: If we just wanted to use ProcessID, that would be tricky, as the kernel would need to remember the history of ProcessIDs, which would require linear space in the number of process instances.
  * Tyler: overall opinion on Hubris' approach?
    * Branden: discovery of TaskIDs is completely missing, this is something that we need for Tock.
    * Branden: generally, it seemed reasonable. Liked the flexibility of message passing and sharing memory.

* Branden: ThreadX, embedded RTOS in C, long history.
  * Reason it came to mind is a conversation with a medical equipment company that has been using ThreadX.
  * In ThreadX, you can create arbitrary message queues that can be sent or received on.
  * Because it's in C, fewer questions about sharing, ownership, etc.
  * Message queue: resource created from a chunk of memory. All messages are fixed-size, so a message queue has a fixed number of elements. You can just use pointer+length and then pass around buffers that way.
  * Usually FIFO, but can re-sort the queue based on priority. Function you can call to move the element to the linked list.
  * Queues are just like global variables; discovery is unsolved.
  * When interacting with queues, you can send or receive from queue, no restrictions. You can get a notification when a queue-send occurs.
  * Send interface: Send with a queue a pointer to your message, and an option for waiting. No wait, wait forever, timeout. Only wait if the queue is full.
  * Receive interface: receive into a destination pointer, option for waiting.
  * Leon: your thoughts on this?
    * Branden: most basic message passing mechanism. Fixed size is nice, nice to have as many queues as you want.
    * Branden: questions about trust. Any process could call receive.
    * Branden: timeouts are nice.
    * Leon: variable-length buffers and pointers seems in conflict with Tock's memory protection mechanism.

* Branden: Rust message passing primitives. `mpsc` queue primitive in std lib. Sync and async flavors. Nightly also has `mpmc` variant.
  * Create a channel giving two objects, sender & receiver. Sync is bounded, async is unbounded (unlimited number of messages).
  * Once you have a sender channel, you can call `send` on it.
  * On the receiver `receive` receives elements from the queue. Multiple variants: nonblocking, with timeout, etc.
  * Leon: You give up ownership of elements you put in there. Move operation on the object. Huge objects should be boxed on the heap first.
  * Leon: Queue here has memory that's neither the sender's or the receiver's. Not likely something Tock would want
  * Branden: We could do that by having boards create the queues if we wanted

* Branden: Redox has IPC, everything is a file. Not very instructive for how Tock would do things. Open a file for shared memory, cast into slices of objects, etc.

## IPC Use Cases
 * https://docs.google.com/document/d/1iL_DPMygbB4XAEZMSESP8Q-xA7Kq04C-B1lLUfB114E/edit?tab=t.0#heading=h.5hhktnom69b3
 * Didn't get to this today. We'll cover it next meeting

