# Tock Network WG Meeting Notes

- **Date:** September 15, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. WiFi Status
    3. IPC Status
- **References:**
    - [External Dependency PR](https://github.com/tock/tock/pull/4589)
    - [Wifi PR](https://github.com/tock/tock/pull/4529)
    - [Tockworld IPC Presentation](https://docs.google.com/presentation/d/1QbFCSCuWroqlnQAfLszsRlrByxtKrS76jwCTYrLWaHE/edit?slide=id.g3886d17a55c_0_105#slide=id.g3886d17a55c_0_105)


## Updates
- Leon: Working on external dependency stuff. https://github.com/tock/tock/pull/4589 Lots of debate going on here. Agreement that handling dependencies isn't working right now. No assurance that dependencies don't move from under us. Could use some fresh thoughts on this.


## WiFi Status
 * https://github.com/tock/tock/pull/4529
 * Alex: Darius is working on the SDIO interface. Irina was decoupling media for communication from WiFi functionality
 * Alex: I'll have an update in the next couple of weeks
 * Branden: Cool. No rush. Just want to make sure that if we can help, we know to.
 * Branden: Separating out the transport layer from the WiFi layer sounds like the most important part.
 * Alex: x86 VirtIO stuff will let us use a network card. So we'd like to combine those networking efforts with the WiFi networking efforts.
 * Leon: Existing ethernet stack should "just work" on x86.
 * Alex: Need to improve it and stabilize it. IPC with a dedicated Service application sounds like the first step for it


## IPC
 * Leon: Amit and I talking about research in IPC stuff. Discussing tradeoffs and that there is no one universal IPC interface. Potentially a research paper there. Super preliminary for now, I'll update you all later.
 * Branden: Presented IPC at Tockworld core team meeting. https://docs.google.com/presentation/d/1QbFCSCuWroqlnQAfLszsRlrByxtKrS76jwCTYrLWaHE/edit?slide=id.g3886d17a55c_0_105#slide=id.g3886d17a55c_0_105
 * Branden: So we discussed IPC designs at Tockworld to get feedback. Positives: general agreement that the design goals make sense. Particularly moving to an "IPC ecosystem" where we have multiple IPC capsules rather than one IPC system within the kernel. 
 * Leon: Still privileged capsules. They'd have some capability to call kernel functionality to make the IPC happen.
 * Branden: Something great there is that everyone could make their own IPC capsule for their specific needs, such as Alex's automotive "send a number" requirement.
 * Branden: Problems from Tockworld: particularly focused on mailbox design. The concern was that if you have one allow each direction and a client talking to multiple services, then a service could try to write it a message and find that it is full currently. Then the service has to decide what to do: wait for a while (delaying other clients) or drop the message (potentially breaking this client). So we want something more like a guaranteed response from the Service without delay. Lots of possible designs were discussed.
 * Branden: Still considering how best to achieve this. Something I was thinking of separating the mailbox into client and server sides. The client would dedicate a request and response buffer simultaneously, and they would be busy until the server responds. Then there would be a dedicated buffer, because the client can't talk to another service until the first response comes.
 * Leon: Problem here is that you could have async responses from services, like IP packets arriving. You could have a notification to send a request, but then the server would still have to hold onto the packet for some duration of time. If you have the model where you only have one request outstanding at a time, it's unreasonable to assume that the service has a buffer. If there's a dedicated buffer, then there's a guarantee that you can always copy the first packet at least.
 * Branden: But queueing the second-through-third packet is identical to queueing the first-through-third packet, right?
 * Leon: You could have StreamingProcessSlice, which the Service appends packets into. The Client is informed that packets are written and handles the chunk. The Server could drop packets if the StreamingProcessSlice fills
 * Leon: My point here is that this only works when the client has a long-standing request to the service
 * Branden: What about a sync/async split? A sync request-response set of buffers, and an async StreamingProcessSlice mechanism
 * Leon: We could have one StreamingProcessSlice that all Services share. Services would still allow a buffer to the kernel, and the kernel does the modification of the StreamingProcessSlice. That would return "fit or not fit".
 * Alex: We should allow the client to ask for messages from a Service. The client should only be able to get messages if it requests them in some way. Otherwise a malicious Service could block up a process
 * Leon: What StreamingProcessSlice avoids is having in-kernel storage which is sized as clients-times-services. If we have any allocation in the kernel that's linear in clients or servers it could go in one of their grants. In clients-times-services, that's variable and tricky.
 * Alex: We could track "is the client allowed to talk to client or not"
 * Leon: Where would we store that a given service is allowed to write to a given service?
 * Alex: Bitmap. Impose a limit of 32 services?
 * Leon: Also need to map bit index into service ID
 * Leon: I'm just saying that shared StreamingProcessSlice avoids grant allocations
 * Branden: Services would need an ID to send a message back. So they can't send messages to a client until the client sent them a message first.
 * Leon: That's a capability-based system. Where discovery gives you a capability.
 * Leon: If a client talks to a service and the service gets the client ID, at some point the service will stash that ID for later, and then later it goes to the kernel and sends an async message to the client. We can't know that the service hasn't forged this client ID. We can't authenticate this per-call
 * Branden: You can solve that with kernel storage. Storage allocated in the service as a function of the number of clients.
 * Leon: Then the service needs to allocate enough memory for however many clients it might possibly service
 * Alex: So if we stored it in the client instead, that would avoid that. Then the kernel would track the ID number for each Service. That could be fixed N services total.
 * Leon: Problem you run into there is if a service crashes and reboots. Instance IDs are hard here
 * Leon: We could have an identifier for a service that's a 64-bit number. Then a client could have an allow with a list of service IDs which are allowed to write to it. That would be entirely in the client's domain. Not efficient, but possible. Variable-sized, but in the client's memory. If the client dies, that allow would reset anyways and block the message.
 * Branden: Summarizing: async/sync split is possibly interesting. Need to look at OS research there. Then Alex's point is that possibly malicious clients and services shouldn't be able to break each other or fill each other's buffers. Need some access control on both sides.
