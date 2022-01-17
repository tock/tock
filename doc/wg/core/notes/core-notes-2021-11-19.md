# Attendees
* Hudson Ayers
* Brad Campbell
* Philip Levis
* Alexandru Radovici
* Brian Granaghan
* Jett Rink
* Johnathan Van Why
* Vadim Sukhomlinov
* Pat Pannuto


# Updates
* Brad: From OT call -* one of the challenges with RSA is that if you want to
  preserve the mutability of your RAM buffer, it is challenging to do in an
  ergonomic way. One decent solution seems to be that externally, you have two
  traits, which can represent mutable or immutable approaches. Internally,
  however, you have a single type -* MutImutBuffer -* so that you don't have
  much code duplication at a low level. This seems like a good tradeoff between
  size and usability for the "user-level".
* Phil: The key thing is that the MutImutBuffer allows an internal
  implementation to take in a mut buffer and return an imut one, which
  is more or less impossible to handle. Internal errors should be handled by
  the implementation, but not the application.
* Leon: So there are two different clients?
* Phil: Yep, that is exactly right. Now three things must align: which client
  you have, and the mutability of the two buffers. You pass two buffers often
  -* what if one is mutable and one is immutable? Do we need 4 traits for all
  possible combinations? My sense is no..but stay tuned. Hard to imagine mutable
  pub key but immutable private key.
* Alex: I have been working on WiFi. Managed to list WiFi networks on the
  Arduino with the RPi processor, so you will see some action on that draft
  PR. What is really cool is that we can list networks, hopefully can connect
  soon.
* Phil: WiFi has a lot of modes. How are you planning to encapsulate that?
  Where do you see this going? Do you want a very generic WiFI abstraction,
  or a minimal working abstraction for the ESP chip?
* Alex: Figure we can have an AP or a station, and we only have HILs for
  connecting to a WiFi network as a station, and a HIL for becoming an AP.
  These HILs are only high level at this stage, which gives more flexibility
  to the implementation. We might need a configuration trait soon.
* Leon: Yeah, I don't think this would work with internal WiFi hardware. And
  we should note that the HIL is not final.
* Alex: My expectation is that a high level trait should support most actions,
  but additional chips might need additional traits for configuration, etc.
* Leon: Sure, if you want to meet to talk about implementing the data channels
  I would love to meet, I have been working on this for Ethernet.
* Alex: Would love to. Let's connect.
* Phil: A couple things. You will need to consider frequency band, channel
  width.
* Alex: Many ESP chips cannot configure these things
* Phil: But when you scan you will need to know the properties of these
  networks
* Alex: Ahh..can you add those comments to the PR?
* Phil: Sure
* Alex: Should I start with a WiFi TRD?
* Phil: I am not sure.
* Hudson: I am a fan of draft HIL over TRD when we only have one chip.
* Phil: Agreed, just make sure we understand what the standard says can happen
* Phil: Lets name the HIL after the chip it is implemented for, for now.
* Johnathan: I submitted a working, sound PR for subscribe to libtock-rs. Am
  working on similar ones for allow -* not sure if I will be able to submit
  those today.
* Jett: I am working on in-kernel allow handling. Johnathan, if that landed
  upstream, would that make things easier on libtock-rs?
* Johnathan: Yeah that simplifies a lot of the logic that I am working on
  now.
* Jett: Cool, I hope to submit a draft PR today. Most of the remaining work is
  porting capsules, assuming people like the design/implementation.

# Flash discussion
* Brian: Yeah, I believe when we left off we were talking about access
  control. I added some stuff to the issue about that.
* https://github.com/tock/tock/issues/2901
* Brian: What I came up with so far for access control is a trait where you
  can create an object by passing in an array of page regions along with a
  permission. Do we think that the permissions of
  Read/ReadWrite/ReadWriteErase are enough,
  or do we need more permissions?
* Phil: In the linux case there is Read-Write-Execute, each is a separate bit
* Jett: I can't imagine write only, or erase but no write.
* Phil: If we use trait objects, could we restrict access by just controlling
  what traits are exposed to different clients?
* Brian: The weakness there is when it comes to these permissions, now we have
  to have an instance of the permissions object stored by each one of those
  traits, which is some overhead.
* Leon: Where is an object implementing the access control stored?
* Brian: Based on the conversations from last week, whoever creates this is
  implicitly trusted (boot),  my thought was pass this into the HIL at
  instantiation and not let it be modified later
* Leon: I think a solution to that is make this an unsafe trait, so that it
  cannot be implemented except by code that can't use unsafe. Then we solve the
  issue of storing multiple of these by passing a reference to the implementer
  of the trait (aka make it a capability).
* Phil: Yeah, making this a capability makes a lot of sense to me. Read or
  Write capabilities. It is not 0-size -* it has a range, so values that are
  read at runtime.
* Brian: So the suggestion is to pass it with every call?
* Leon: Yeah that is a downside, and checks always being a runtime check.
* Leon: I talked with Hudson about this last week, in particular about access
  control. I think there are a lot of approaches with varying runtime impact.
  I think one important question is whether this should provide raw access with
  no windows.
* Brian: Yeah this hits one of my open questions -* do we default to allow all
  or allow none?
* Phil: Gotta be default off. There could be an extra trait that does not
  require any capabilities. Similar thing here is ProcessManagementCapability
  for external implementations of the Process trait.
* Leon: I still think runtime checks are a big downside
* Phil: How could you avoid that?
* Leon: Controller has accessible region set at boot, and always provides
  access to that entire region to all clients. One solution would be the
  exact same HIL copied twice -* one safe, requires access control struct,
  other unsafe, and gives unlimited access. This allows flash access with
  no performance hit.
* Brian: That seems similar to Phil's suggestion to me.
* Phil: We could also go to virtual pages and virtual addresses for clients.
  Within a region, each client has virtual addresses starting at 0. I am not
  sure that is a good idea or not.
* Leon: I like that idea. Capsules usually won't care about absolute
  addresses.
* Phil: trick with different capabilities within ranges, you need access
  control on top of this virtualization.
* Phil: If I can read the first half of the region and write the second half
  of the region I still need ranges and capabilities associated with those
  ranges.
* Leon: I imagined access would be on the granularity of pages
* Phil: Yes that makes some things easier, but I am just trying to say that
  virtualizing page addresses can complicate reasoning about access ranges,
  since those access ranges will need to correspond to absolute addresses
  somehow.
* Brian: Sounds like we have a low-level, just expose what the chip does as an
  unsafe interface. Then above that have a more abstract component that has
  access control and can be implemented across multiple chips.
* Leon: So remaining question is mostly should we use virtual vs physical
  addresses throughout the entire stack.
* Brian: HIL will require a page_from_address and page_to_address function.
  Not sure if we have to add it to a HIL. Could be out of band.
* Brian: I did have a couple of questions about invariants requirements. I
  have on Read, it should return an error if a Read would cross a page
  boundary. Write returns an error if the size is not an integer multiple of
  WRITE_SIZE. Error if write would cross a page boundary. see PR for a couple
  more.
* Jett: Note that multiple writes being undefined is at the hardware level.
  The bits that will be there are undefined, but this is not UB in Rust.
* Jett: So verify after a write is not guaranteed to have committed.
* Phil: I think that is a good framing. Basically, you are assured that a
  single write, barring hardware failure, is assured to verify, but further
  writes after that are not.
* Brian: ok, I will make that update. zeroize has basically the same
  requirements as read.
* Brian: verify has the same requirement around page boundaries.
* Jett: Are these all top level traits?
* Brian: Right now these are all broken out as separate traits
* Jett: Should we combine zeroize and erase, for example? What about verify
  and read?
* Brian: Verify and read could go together. Zeroize is just a special form of
  write. so could combine.
* Phil: All could be individual traits, and you can also have automatically
  implemented composite traits.
* Jett: Why separate them at all though? We don't do that with most traits,
  i.e. you don't usually see one trait per function.
* Phil: Major reason would be for dead code elimination. If I only erase and
  never zeroize, separate traits can help if I am using trait objects.
* Phil: Also, more generally, principle of least privilege
* Jett: Don't we discourage trait objects?
* Phil: Yeah, but sometimes you have to use them.
* Jett: this is nitty-gritty, can defer this.
* Phil: I have a question about access controls. There seems to be the
  possibility for arbitrary logic about whether something is allowed. Do you
  imagine this needing to be a dynamic decision?
* Leon: I agree -* so maybe using a struct is a better option here?
* Phil: Brian, what is the justification for the method based approach?
* Brian: For our current app, since it is all in hardware, we would not end up
  using this on the Google side because we have hardware protection, this
  approach was just something that is more flexible in terms of having multiple
  regions. If that is not a large concern we can certainly simplify.
* Leon: I imagined a virtual device, one per flash region
* Jett: I like the high level idea of a virtual flash region. Nice mental
  model.
* Phil: I think you could just do that with this API bc of the page
  abstraction.
* Jett: So is consensus to remove trait based approach for access control and
  just use structs, for the better immutability guarantees?
* Phil: That is more in line with what other interfaces in the kernel do
* Phil: network stack permissions are the best analog here, shared some links.

# RSA Concern
* Phil: There was a concern about the degree of typing RSA keys -* should the
  types specify key lengths?
* Phil: We talked about this and one of the challenges is accelerators, which
  dynamically load instructions.
* Phil: If a userspace program can load instructions into an accelerator,
  being able to statically check what keys it can accept is tricky. Because
  the code could change! So we are trying to work through that.
* Hudson: I see, so the fact that an accelerator is underneath the HIL limits
  how strongly you can type the HIL.

# Porting
* Leon: A friend gave me a cheap RISC-V board. I want to port it to Rust. It
  is neat hardware -* lots of flash and SRAM, but no PMP at all. Is it worth
  the effort to try to port this further? I have a blinking LED. What is our
  policy on chips without memory protection?
* Hudson: We used to have support for the nrf51, I think it not having an MPU
  is part of why it was removed from the main tree.
* Phil: Have fun, but yeah idk whether we would want it. Could say it is
  libtock-rs only.
