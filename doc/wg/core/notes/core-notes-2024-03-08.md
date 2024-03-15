# Tock Meeting Notes 03/08/2024

## Attendees

- Alyssa Haroldsen
- Amit Levy
- Andrew Imwalle
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Pat Pannuto
- Tyler Potyondy

## Updates

- Branden: Networking WG update. The OpenThread stack works right now. Tyler is
  using a C library as the OpenThread stack, which communicates with a capsule
  to drive the radio. It is capable of joining a Thread network, remaining
  attached, responding to requests, etc. Did not encounter timing issues -- the
  most timing-sensitive parts are handled within the driver, the app only
  handles non-timing-sensitive parts. What's next is a more flushed-out story
  for switching channels, a better RSSI story, and a flash implementation (I
  think a PR or issue was posted today).
- Amit: What does flash implementation mean?
- Branden: The OpenThread stack wants to save state to help re-join the network.
- Tyler: That's a blocker because e.g. OpenThread won't commit network keys
  until it believe it has been saved to flash. We're currently faking that
  storage in RAM. Working on putting it into flash to be spec-compliant.
- Branden: Notably, none of them are blockers. It's pretty much working.
- Tyler: The biggest blocker is potentially the ring buffer, which we've
  discussed. Have had some issues with that and timing sensitivity. Will
  continue discussion on the PR. May need a new construct for this.
- Branden: For background, the capsule is trying to share packets with apps fast
  enough.
- Johnathan: Reminder there's already a TRD for a shared kernel-userspace buffer.
- Tyler: Having some issues around upcall semantics.
- Brad: Maybe need to look into it more.
- Amit: I know the goal at one time was to get a Thread sleepy-edge-node (or
  something) device running. Since the path has been to support OpenThread in
  userspace
- Branden [in chat]: Sleepy End Device (SED)
- Amit: is the path still in reach, or are there differences.
- Tyler: No, they're not. I haven't tested it, but theoretically they can talk.
  Previously I was having to implement the logic in the capsule, which was
  simple for SED, but a router is more complicated. We're in reach of having
  general OpenThread functionality. Being able to have a router device should be
  in reach.
- Amit: So overall, Thread is a generally full-featured protocol and it is an
  existence proof of taking a nontrivial full-stack C library and running it in
  a Tock process.
- Tyler: Yes.
- Alyssa [in chat]: Question about the userspace readable allow: why not instead
  pass the buffer up to user space in a syscall, with the driver indicating to
  the kernel that the memory is readable by a specific userspace app? That could
  also allow sharing buffers between apps.
- Branden: Networking WG has also been working on a better strategy for sharing
  buffers. Leon and one of Alex's students (Amalia?) has been working on a
  method that attaches process IDs to buffers.
- Brad: Documentation working group update: we have a bunch of READMEs in the
  repo, and scripts to keep them up to date. I'm working to run them, creating
  PRs, and setting up a CI thing to keep them up to date. We also merged the
  capsule test trait, and Branden and I are working on a PR for the book to
  point users to the capsule test trait.
- Johnathan: OT working group discussed the safety of RISC-V CSR accesses, I'll
  take a look at the PR and comment on it.
- Brad: Have been working with a LoRa chip, have the LoRa stack running. It
  joins networks and transmits packets. Think we can merge library support to
  libtock-c by the end of the semester.
- Amit: Update on mutable statics. Leon and I have prototyped a design that we
  think can replace `static mut` in a way that is both sound and zero-cost in
  the current Tock case. Incidentally it would allow support for multi-core in
  principle with some cost, including potentially being suitable for host
  environments (as Alyssa suggested). It's along the lines of what Alyssa has
  sketched, looks like `thread_local!` with a different implementation. There's
  probably a diff that I could pull up but we'll probably have a PR in the next
  few days.
- Tyler [in chat]: @Alyssa Seems like the biggest downside to this is that the
  buffer size will be static since it comes from the kernel. By allocating in
  userspace, we are able to provide a buffer of larger or smaller size (this is
  important for if the userprocess wants to provide a larger buffer to not drop
  packets if the userprocess expects to infrequently yield and handle upcalls)
- Alyssa: The main issue I've found is you can't get a `&mut` reference, so we
  have to modify types to take a `Deref<Target = &mut ...>` type instead.
- Amit: With `thread_local!` you get a closure where you operate on a
  non-`'static` version. Could maybe do a guard type.
- Alyssa: What I have in our current codebase returns a `RefMut` from a
  thread_local that unsafely escapes the thread_local. It returns a wrapper type
  that is zero-cost on chip but on host it is a `Cell<RefMut<>>`. The `RefCell`
  remains locked, but because it is not `Send` or `Sync`, you can't have
  unsoundness due to sharing it across threads. I think it would be more
  ergonomic, as nested closures get sucky.
- Alyssa: Rust will soon be stabilizing exclusive range patterns. I know there
  was some unpopularity with that in this group -- there's a lint being landed
  at the same time.
- Amit: Why didn't we like it?
- Alyssa: The argument was you can accidentally create a wildcard branch that
  includes more than you expected, e.g. `(0..10, 10..20, 20..30)` misses 10, 20,
  and 30.
- Alyssa [in chat]: @Tyler why would the buffer size be static? The kernel can
  send a pointer and size, even coming from within the userspace grant
- Tyler [in chat]: That could work. I'll need to think more about this but this
  is an interesting alternative

## Kernel Testing (#3873)

- Brad: We discussed this last week. I made a comment summing up some of the
  discussion. One question was the name, then there hasn't been other
  discussion. I dunno what that means -- everyone happy? More time to think?
  Just need to argue about the name?
- Amit: I was under the impression it would be merged.
- Brad: There are still concerns about the name.
- Amit: Yes.
- Branden: I'm good with the name.
- Amit: The name being boards/configurations
- Alyssa: I can't help but bring up config as a shorter version.
- Discussed "config" vs "configuration", some jokes were passed about file size.
- Amit: I was one of the people unhappy with the name, but in lieu of better
  suggestions, bikeshedding the name is not a reason to hold this up. There is
  some confusion about the name "boards" to begin with that may be a reason to
  rename them in general.
- Alyssa: Yeah, different teams use "board" very differently.
- Amit: So this won't change the imperfectness of the concept-we-call-boards.
- Brad: I like the comment of putting "test" in the crate name, so there's no
  ambiguity that this is not a "real" board.
- Amit: Is "test" not a valid crate name?
- Brad: Then you have a folder called "test", and it'll crate ambiguity in what
  "test" means.
- Alyssa: Could call it a "development" board.
- Brad: That has the same problem.
- Alyssa: "Simulated"
- Brad: Well, it's not simulated.
- Amit: Often `test` folders are for things like unit tests, and more
  integration-y tests go elsewhere.
- Alyssa: They generally go in a folder called "tests" too, as a sub-crate if
  they're complicated.
- Amit: This would be a folder of crates, because each test case is its own
  crate. Each case deployed to a different device.
- Brad: But "test" is ambiguous.
- Amit: We don't really have anything called "test"
- Alyssa: which is not great
- Amit: I don't think this will be overloaded with something else.
- Alyssa: As someone who's not very familiar with hardware, if I open a project
  and see "test" it's not clear if it's on-device tests or on-host tests.
- Brad: This is about verbal communication. If I say "you need to run the test",
  it's unclear what that means.
- Alyssa: We'd call that "dev board tests"
- Brad: I'm proposing "configuration board"
- Alyssa: That sounds like something else
- Brad: At some point, it's a Tock-ism.
- Amit: I don't share the same fear. If we have a top-level folder called
  "tests", with a "nrf52840dk_test_id_sha256", and you told me "run the
  nrf52840dk test ID sha256 test", it's a natural place to look.
- Brad: But what if I told you to run the kernel test called "sha256", you'd
  have trouble finding it. You have to go to capsules/extra/src/test to find
  that, don't you know?
- Amit: I would have Alyssa's intuition that unit tests are in-situ with the
  code. If you told me to run "sha256 unit tests in capsules" I would go look
  for the implementation for tests.
- Brad: These things sound like they would seem so obvious once you know the
  repository, but we're creating more things all called the same thing.
- Amit: Can you point to the file or folder that you think is overloaded?
- Brad: These board are not just meant to be running tests. They're really
  supposed to be variants or configurations -- they may look like normal boards
  but setup with code that's not commonly used. Won't necessarily look like a
  test.
- Amit: I still suggest that we avoid further bikeshedding. We can rename a
  folder in the future. To ask for one thing: change the PR title, then I'll
  approve it.
- Brad: Can do.

## Flash HILs (#2248, 2993)

- Amit: There were efforts two years ago to revise the flash HILs (or add an
  additional one) to adapt storage that is different in nature to the ones the
  current flash HIL is designed for and maybe address other issues. Those PRs
  are very stale. One sticking point is they got a lot of feedback and
  enthusiasm, then things fell silent. Is this something we're ready to engage
  in, or should we close them? It doesn't seem super urgent for the rest of us,
  but I don't feel the flash HIL is perfect. I suspect Phil -- who's not here --
  would be the most vocal.
- Brad: Which PR exactly
- Amit: #2248
- Branden: It's in the 2000s
- Brad: I think that's a sign. Adding synchronous HILs is a bad idea -- we don't
  need it.
- Amit: and #2993. It was making progress, got blocked by a release, then
  interest waned, and yeah. Not really changing paradigms but expanding the
  abstraction to cover more kinds of devices.
- Brad: We would run into the problem of "do we have code that would use it"?
- Amit: I have this board. Development on that project stagnated. To be clear,
  my question is not "is this a PR we should merge now?", it is "this is a stale
  PR... are we going to revive interest enough?"
- Brad: If we don't have a use case, we probably won't have the interest. If we
  have that, then maybe. I would be more interested in thinking about it. Does
  that make sense?
- Amit: Not entirely. This was blocking for upstreaming the board.
- Brad: Is there a pull request?
- Amit: No, it's sitting in a different repository.
- Brad: We have a policy that if we add code, we have to use it.
- Amit: I would upstream that board. It is straightforward except for the
  storage.
- Brad: Isn't there another layer like a capsule or something?
- Amit: Yes. It's a watch that works that I wore for 6 months. I think it's an
  nrf52840, most hardware is supported.
- Brad: Is this the SMAQ3?
- Amit: Yes.
- Brad: That's already upstreamed.
- Amit: I can look at the details again. This would not be orphaned changes. I'm
  hearing that the answer is #2993 maybe, #2248 no.
- Brad: Yeah
- Amit: I'll either take on #2993 or close the PR.

## TBF header parsing (#3902)
- Amit: I think they're fairly non-controversial. Brad, could you describe a bit
  of both? I think the TBF parsing is more relevant to folks, but.
- Brad: #3902 changes how TBF headers are parsed. Currently we have
  fully-expanded objects the size of the TLV stored in flash. A lot of legacy
  reasons why it's written this way, but it means the object is large. Avoided
  passing it on the stack, but my other PR does that, which led to some issues.
  Are storing it in process memory for every process, currently 368 bytes (or
  similar). This PR would parse things used a lot into full objects, then
  lesser-used would be stored as a slice and parsed upon use. This makes the
  object shrink to 120 bytes.
- Amit: And the cost is it may be slower to access later.
- Brad: Right
- Branden: Is it slower? Don't we have to parse it at some point?
- Brad: Were parsing it at boot time, moving to whenever we call "get me the
  storage permissions". Could happen many times.
- Amit: Another side effect is it seems to save a bunch of code size on many
  boards. Don't know exactly why.
- Brad: That makes sense if you never call the function to get a particular
  item, the parsing code for that item is removed.
- Alyssa: We currently have totally-custom parsing code; I'll see if the
  techniques applied here apply there.
- Amit: Seems fairly uncontroversial to me. I think the main drawback may be in
  power-constrained settings.
- Brad: The other benefit is it removes a statically-defined length. No longer
  limited to 8 permissions.
- Amit: Lets discuss the other PR.
- Brad: I have a board with a monochromatic screen, I've ported a library to
  libtock-c and we can draw on and use the screen.
- Amit: And I believe it gets the screen size dynamically from the kernel,
  right? Can apps take a variable amount of the screen?
- Brad: In principle, you can create windows on the screen. Can assign those
  windows to processes by name.
- Tyler: I was able to test it. This will hopefully be used for CPSIOT
  tutorials. Worked on a PCB to make it plug into the nrf board we're using.
- Amit: We need approvals, particularly for the PR against libtock-c. People
  should look at it.
- Tyler: I have the ability to approve, but does that account for anything? It's
  been unclear to me if it's helpful.
- Amit: Since you do not have approval permissions, it doesn't count as one of
  the two necessary, but seeing other approvals is a signal.

## TockWorld
- Johnathan: We're ready to make travel plans, correct?
- Brad: Yes
- Amit: Yes

## libtock-c redesign (tock/libtock-c#370)
- Brad: It's a lot of work, could use help. Touches every file.
- Branden: I thought we were going to merge the PR then fix, I didn't realize
  the PR would fix everything.
- Brad: I think we'd end up with half-and-half and never finish, so I wanted to
  do it all. I was somewhat concerned we'd finish it then someone would decide
  they don't like it. If anyone has any interest in what they look like, now is
  the time to comment.
- Amit: I'm interested, I'd like to take a look, you know my reliability.
- Tyler: I can devote some time to that.
- Brad: I think if we're going to do tutorials, lets get the better version out.
  Do it now, I don't see any reason to wait.
- Tyler: I agree.
- Brad: I'm really unhappy with the formatting, I wish C formatting tools were
  easier to use.
- Tyler: Could continue to iterate on the design.
- Brad: No way do we want to do this again. Don't think if it as iterative,
  think of this as one and done.
