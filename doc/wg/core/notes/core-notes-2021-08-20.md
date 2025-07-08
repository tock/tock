## Attending
- Hudson Ayers
- Amit Levy
- Alexandru Radovici
- Jett Rink
- Johnathan Van Why
- Branden Ghena
- Leon Schuermann
- Anthony Quiroga
- Brad Campbell

## Updates
- Alexandru: Still working on visa for presenting at embedded Linux conference
- Branden: Hudson – does Tock “just work” on your m1 mac?
- Hudson: yes, it compiles just fine, though it did require a system update to
  get there. I have not tried to flash any boards from it yet, still using my
  linux box most of the time

## Tock 2.0 Status
- Brad: We dropped a check in allow that would cause issues if your flash was
  (below/above) your RAM on your chip, so that broke a few chips
- Brad: Aside from that, no major bugs! Definitely a pleasant surprise. Instead
  just a lot of the small things that we might expect.
- Brad: Seems like a lot of boards have finished testing, I don’t have a great
  sense of which boards are left.
- Brad: On the libtock-c we are having some additional issues with newlib
  because we updated to a newer version. Alex and Hudson report that rebuilding
  newlib fixes it, but I have not observed that on my board. We need to track this
  down and get it fixed, quickly.
- Brad: I don’t think we need to retest all boards after this, but one board of
  each architecture probably
- Hudson: Ran through a list of all the boards, very little remaining.
- Hudson: We probably want to consider the msp-exp board deprecated, and remove
  it for 2.1 if we cannot get anyone to test it
- Brad: I can make the ACD testing more formal
- Hudson: We are still waiting on Pat for nrf52840_dongle
- Branden: we have tested all the other nrf52 boards
- Leon: I have seen board-specific bugs from the 2.0 release, so testing every
  board would be nice

## Process Console Prompt PR()
- Alexandru: Many of my students have not noticed that the process console is
  usable, so I added a prompt that can be generated if a board chooses to
  include it informing people how to use it
- Amit: I guess only downside is if you have processes that are printing
  regularly, but this is not a huge issue for now
- Leon: I imagine the long-term solution here would be a virtualized console
- Alexandru: Only requirement here is that you print the prompt after the
  “entering main loop” debug function.
- Amit: Does anyone have an issue with this?
- Brad: I don’t like adding the debug to start(), I think boards should be able
  to choose no prompt
- Alexandru: I could add a parameter to the start method
- Brad: I think a board can just..print it
- Hudson: I think that greet() or whatever should be a standalone function on
  the process\_console
- Amit: What about the tock$ prompt?
- Brad: I don’t feel strongly once a user has already decided to interact with
  the process console
- Amit: If we have two methods, should start() print by default and we also have
  a start\_silent() method?
- Amit: It does seem to me that there is a compelling argument that
  discoverability of the process console is a useful thing to have by default,
  and by having two separate methods to get that will more likely lead to some
  boards excluding the message on accident. If we want it by default, that should
  be the low-friction option. That is my pitch for keeping the default behavior in
  start as printing the message
- Phil: Is this just a documentation problem?
- Hudson: Most beginners don’t read much documentation
- Phil: Most shells do print a prompt of some sort
- Phil: Two methods or an argument seem reasonable to me
- Alex: I am going to add a start\_silent method that is called by start()
  internally after a message is printed.
- Hudson: Sounds good to me
- Amit: I think this is just one iteration on the console as we bring it closer
  to how you might interact with a regular Linux shell
- Amit: We probably should not worry too much about these particular details
  because a lot of this is likely to change again anyway
- Alex: I have a student working on a shell with backwards history
- Jett: I actually implemented a version of that downstream about a month ago
- Alex: Can you share your code? My student declared a matrix of u8s, but would
  be curious to see another approach
- Jett: I used a circular buffer of u8s, lets connect offline

## Clue bootloader
- Alex: Basically, the clue bootloader is identical to the arduino nano33
  bootloader with a different name. What I could not replicate on the clue is
  the trick with flashing two bootloaders and chaining them, I bricked my board.
  So for now I left my Adafruit bootloader, which occupies 150kb!! But that lets
  people easily switch back to micropython or something else.
- Alex: One problem with the bootloader is it does not compile with the current
  kernel, so I had to manually choose an older commit.
- Alex: So, I supplied a binary file, as well as a u2f file which you have to
  supply to the bootloader.
- Leon: We should avoid hacks to build the bootloaders which are not
  reproducible
- Leon: Maybe we should make it work to build with 2.0
- Hudson: I think we should make building work consistently by fixing a kernel
  dependency, but I do not think we should require anyone to update the
  bootloader to 2.0
- Amit: Looks like we use git paths for the cargo dependencies – could we not
  just point to a particular release tag?
- Alex: Yeah, I actually built it with the one without QEMU dependencies because
  otherwise it never finishes.
- Amit: I am saying that if we do this then anyone can download tock bootloader
  and get a deterministic build, rather than relying on some automated build
  process in CI
- Alex: Yeah, I'll send a PR to do that
- Leon: If we do that we should consider the bootloader a rust binary, rather
  than a rust library, and thus we should add a Cargo.lock file to prove it is
  reproducible and will always produce the same result.
- Amit: I agree with that
- Alex: Another point about using the tock bootloader with the clue, even though
  it is a bit slower than the Adafruit bootloader, is that the Adafruit
  bootloader fails about 30% of the time, while the tock bootloader never seems
  to.
- Amit: That's good to hear
- Amit: Is there still an open question about how to distribute the Adafruit
  bootloader?
- Amit: Other than questions about tock bootloader, the question was what is the
  best way to distribute it? Is that an open question still?
- Alex: I think we should put the u2f file in the repo
- Leon: Can we put the u2f file on a Github release rather than bloating the
  Tock/tock repository?
- Amit: Yeah I can upload it to the tock-bootloader repo
- Alex: Great, once that is done I will modify the Makefile and it will download
  from Github, then it will work like the Microbit
