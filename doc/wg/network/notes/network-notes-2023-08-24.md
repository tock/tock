# Tock Network WG Meeting Notes 2023-08-10

- **Date:** August 24th, 2023
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Amit Levy
- **Agenda**
    1. Updates
    2. Alexandru: Solution for drawing during meetings
    3. Tyler: Interfaces for different layers used for Thread networking
    4. Leon: Tock-ethernet integration strategy
- **References:**
    - [NRF52840 802.15.4 PR](https://github.com/tock/tock/pull/3615)
    - [STM32F429 Ethernet PR](https://github.com/tock/tock/pull/3523)

## Updates
- Branden: There's a label for Network WG related PRs on the Tock github repo
- Tyler: Working on 15.4 PR. Need some closure on behavior for transmit
- Branden: We had a discussion about handling on/off states for the 15.4
  radio. Other things in Tock like GPIO just "do the right thing" and set
  themselves in the state they need to be. The 15.4 radio HIL expects callbacks
  when changing power states though, which likely makes sense for external radio
  chips and radios that take some real-world time to start up.

  Long story short: in Tyler's PR, calling `transmit()` also turns the radio
  on. While this might be fine, we may want to back away from that in the
  future.
- Tyler: Previous PR did not even have a check for that before. Add this
  functionality after some discussion with Amit.

  Alternative would be to return an error, if the radio is off.
- Amit: Takeaway is - what Tyler implemented is fine, what was there before was
  probably also fine. Figure out what the right behavior is, not clear if the
  old was _wrong_.

  The existing clients (like XMAC?) won't work before your PR. This would need
  to be fixed, but not on this PR.
- Branden, Amit: Action for here - Tyler should leave the PR as is, it already
  has one approval.
- Amit: Finished EEM (Ethernet over USB driver). It'll be nice that's available
  on anything with USB. Also, it's a very simple hardware implementation;
  sidesteps a bunch of the low-level implementation details at is relies on USB
  for the transport. Good simple simulation of Ethernet hardware (maybe too
  simple) but great for experimenting.
- Leon: Especially good for working on higher layers without an Ethernet board


## Solution for drawing during meetings

- Alex: There's nothing comparable to draw.io. Draw.io allows us to log in with GitHub. As long as we can save files in a GitHub repo, everybody should have access to drawings.
- Branden: Do we need to have a new repo to contain the drawings?
- Alex: A branch might be sufficient. We could select the branch which will also contain the meeting notes (create it ahead of a given meeting).

  Drawings are portable, so you can open them in your browser with the draw.io website.

- Branden: Is there a good way to make a shared channel or group for it?
- Alex: Draw.io is working in your browser, and it plugs into a backend to save files.
- Branden: We wouldn't be able to see things as people draw in real time?
- Leon: Maybe just use a screenshare? (yes)
- Leon: We can embed drawings into PDF files themselves.

## Interfaces for different layers used for Thread networking

- Tyler: Was quickly sketching up a design of what Pat and I came up with yesterday. ([sharing draw.io diagram](./2023-08-24/2023-08-24_thread_stack_tyler.drawio.pdf))
    - Original thought was to put Thread on top of UDP.
    - It may be best instead to just give Thread access to each of these layers.
    - Userspace driver would issue system calls to the Thread control-layer to the Thread component itself (control path).
    - Actual communication payload would go into the UDP layer directly through something like a socket-interface (data path).
    - Don't know whether it'd be an issue where Thread would sidestep the typical layering.
    - When you specify a socket that you bind to, you can specify the interface you want to bind to.
- Leon: One option is to move the UDP/TCP boxes into the Thread component here, so the socket is accessed through the Thread interface. So when a socket is specific for a given underlying interface, it might make sense to make the UDP component instantiated within the Thread capsule. Then Ethernet could conceptually instantiate its own UDP component
- Amit: Right now, in the 6LoWPAN stack there is a UDP component. If I'm not mistaken, it's not explicitly a system call driver (rather a library for constructing UDP packets). What's the imagined purposes of having UDP and (in particular) TCP in the kernel at all?
- Tyler: You're proposing UDP and TCP would be moved outside of the kernel?
- Amit: TCP seems hard to implement in the kernel. In userspace, there is dynamic memory allocation and libraries, etc.

  Is the purpose of having them in the kernel purely for abstraction purposes? Or perhaps also for ACL reasons?
- Alex: It seems there is another reason - some hardware peripherals expose UDP and CTP sockets directly. So having support in the kernel would let us work with those or with other boards.
- Leon: Apart from these external hardware devices, I think UDP/TCP could definitely be in userspace. So we could construct things assuming that we accept arbitrary IP interfaces. Then it would be easy to move the interface to kernel for some things. The userspace application could be the same.
- Alex: Another note is that having TCP/UDP in userspace would mean if multiple apps need to communicate they would both need complete copies of the stack since we can't share libraries. We could use IPC for this though.
- Amit: For code size?
- Alex: TCP is stateful.
- Amit: Per-stream stateful.
- Amit: UDP is super simple. Not stateful, couple of headers.
- Amit: For Thread, it might not be required to have the control and data plane in the same capsule.
- Tyler: Does Rust have already existing libraries 
- Leon: We have SmolTCP running in userspace. (not merged or in a public branch yet)
- Amit: Where does Thread come into play in terms of managing the link? It sends broadcast packets to join a network, etc. Presumably it manages when the radio wakes up and when to listen for incoming packets. Is it also interposing between layers of the data plane?
- Tyler: Short answer: no. he packets are just 15.4 / UDP. Caveat: for the mesh-link establishment, it uses the same encryption of the link layer. UDP needs to have knowledge of the encryption state of the Thread link layer. Other state is also used as part of the encryption, such as the frame counter.
- Amit: One thing that seems nice about the presented design is that it seems like there is the ability to update the control-interface of the network protocol can be updated independent of the data path (e.g., when a new version of Thread comes around). Using that, could we, for example, switch between an XMAC and Thread network by just swapping out one capsule?
- Branden: In the diagram -- UDP still connects down to 6LoWPAN, and that still connects to 15.4? So this means that the 6LoWPAN capsule needs to be quasi-virtualized, with two clients?
- Tyler: There are some degrees of virtualization at each of these layers. By the next call, I can have a more extensive update on that. We will need virtualization. I have not drawn the upcall paths; that may become tricky.
- Leon: By _virtualization_, do we mean packets routed to either the "Thread" or "UDP" components? If so, does Thread have a way to determine whether a given packet goes to the data- or control plane, like IP with the nextheader field?
- Branden: Yes, it would need to have that. Thread indicates this using TLVs.
- Amit: Is data-plane traffic wrapped in the payload following these TLVs?
- Tyler: I believe this is handled by means of a specific port (`19788`).
- Amit: If applications are sending payload data, the 15.4 frame will not have any Thread-specific TLVs, right?
- Tyler: Yes. Once the network is established, everything is sent with link-layer encryption as well. Once you formed your network, the only control that is occurring are heartbeat messages. The UDP port is what differentiates control from data messages. Thread does not intend to replace or wrap UDP / 15.4, but simply acts as a control layer on top of it.
- Branden: [*shares different draw.io image which adds an "encryption" box between 6LoWPAN and 15.4*](./2023-08-24/2023-08-24_thread_stack_branden.drawio.pdf)
- Tyler: This seems accurate.
- Amit: Perhaps the encryption is not a dedicated component between layers, as it might be implemented in hardware or implicitly as part of one of the other components (such as the 15.4 implementation).
- Tyler: It's very near to where we're able to join a sleepy-end device. Hoping to get a PR soon which may not be a finalized version of this, but gets us closer to a fully-working implementation.


> Notes from Amit from dialpad chat:
> Like, basically, the thing that's titles "Thread" in the kernel in Tyler's diagram > needs to be in the kernel in order to have control over a fixed set of things:
>
> - It needs to be able to control the power/read-state of the radio itself
> - Maybe set MAC and/or IP addresses
> - Plug in some crypto stuff (maybe set the key, add some information, etc)
>
> That all seems _pretty_ generic, beyond thread, such that all the thread specific stuff might fit in an app
> It sounds like, from a reception perspective, the thread application receives UDP packets on port 12345 (whatever.. > I keep forgetting) and that's it. So it's _basically_ like a normal application on the reception side


## Tock Ethernet
- Leon: Workshopping Ethernet support in Tock over the next two weeks with Amit. Hoping to talk about that and SK_BUFF stuff next meeting

