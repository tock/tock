# Tock Core Notes 2024-01-05

Attending:
- Alyssa Haroldsen
- Andrew Imwalle
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Leon Schuermann

# Updates

- Branden: Two Agenda Items:

  - Amit made a post on Slack on code size in Tock. May want to talk
    about it today. Although not dialed in today...

  - If Andrew wants to talk about it -- state of TicKV.

- Brad: Playing around with my old PR on doing RSA signatures for app
  signing. Trying to lay some groundwork for doing that. Things have
  gotten slightly better with regards to external library
  support. Rust crypto RSA library uses heap-allocated numbers. There
  is currently work underway to switch out the bigint library that
  this crate is using. I'm told the next iteration will allow for
  fixed-sized values.

  - Branden: Timeline uncertain?

  - Brad: Yes, although a lot of progress has been made on the library
    side.

- Leon: Have been able to test compiling libtock-c's with the
  precompiled newlib, seems to work. It's great to see this being
  portable, even works on NixOS!

  - Brad: Its a little unclear what the order of operations is here?
    We should merge and then replicate those precompiled packages to
    more places. Shouldn't need to change anything, given the SHA
    doesn't change.

  - Branden: Previously we were told to hold off merging. Is this
    ready now?

  - Brad: Yes, it's ready!

# TicKV Update

- Andrew: Posted an issue a couple of weeks ago concerning
  fragmentation. Alistair seems to like one of the solutions I
  mentioned. Given that this is blocking for me, we'll be moving
  forward to implementing it. This seems to be a good replacement for
  garbage collection. Functionally, it fixes the case where you could
  be told that the flash is full, when there's actually still space
  left (just fragmented).

- Brad: Good to hear that you don't feel stuck on that.

  In general, Alistair is the original creator of the library, but
  it's also under the Tock umbrella. It's not finished, and I'm not
  too worried about preserving the original model. If this is strictly
  better, we should go ahead with it.

- Andrew: Alistair is slightly concerned with maintaining useful
  properties, such as consistency on power loss, etc. Of course want
  to preserve those.

# Conferences & Tutorials

- Brad: Had a couple discussions about different tutorials we may want
  to do. We're committed to doing one at CPS-IoTWorld. There's another
  potentially interesting one more focus on hardware security. Are
  there others that people might be interested in?

- Leon: When we talk about security, I'm curious what the focus of our
  tutorial should be on. E.g., I'm still working on a system for
  safely executing C code in the kernel, which may be fitting (e.g.,
  to drive trusted hardware). But without that, what should the focus
  be on?

- Brad: Good question. We as developers are thinking about the most
  recent changes. I think we should separate those concerns --
  tutorials don't need to focus on only that. We should try not to
  focus on these developments, because it's hard to make them into a
  tutorial. For someone who's new to Tock, everything's new -- so we
  can present established tutorials like the Security Key example we
  presented at TockWorld.

- Leon: This makes sense to me. It takes a significant amout of time
  and effort when using these tutorials as driving force to polish and
  present on new subsystems.

- Brad: Yes, it's hard to develop something, get it working reliably,
  and teach people about it!

- Leon: I'd be happy to see the USB security key tutorial be reused
  and continued. I can only speak to the reception of it at the
  company I was doing my internship, but at least there is was very
  well received. It helped people understand general Tock kernel
  concepts.

- Branden: It is a lot easier to iterate on something, than making
  something entirely new. We should be able to improve on our security
  key tutorial much more easily.

- Leon: Regarding a HOST'24 tutorial, I'm perhaps slightly worried
  about the target audience. We're not necessarily focused on
  considerations around timing side channels, etc.

- Brad: Yes, it's such a range of topics, so it's hard to know what an
  audience is actually interested in.

  Comment on doing an internal variant of the tutorial is good, it'd
  be great to do some more trial runs, e.g., at universities?

- Leon: How should we be moving forward on this?

- Brad: There's a few of us who are motivated generally. If a specific
  opportunity comes up for anyone, we're happy to support it!

  Hard part seems to be finding opportunities in the first place
  (interested parties, content). We have people interested in figuring
  out logistics.

# Tock CI Architecture & Demo

- Leon: Have been working on a prototype for a Tock CI system. This is
  motivated by me requiring access to a CW310 OpenTitan devboard, of
  which we only have one, and not being able to carry this with me
  while traveling.

  Sketched out and implemented a prototype system that we may be able
  to use more generally for Tock CI.

- Brad: Interested to hear about this, and what pieces we could re-use
  for this "cloud CI" idea that we're persuing?

- Leon: I could do a quick demo and screenshare?

  Disclaimer: the system that I hacked together is largely motivated
  by me trying out interesting technologies. Would not want to give
  the impression that this is an authoritative design in any way.

  [shares screen]

  Current system components:
  - Central web interface for management. Allows scheduling jobs on
    multiple boards. The web interface is running on a dedicated
    server in a datacenter, it is not physically connected to the
    boards.

  - The boards are connected to a different computer via USB, which
    runs a so-called "runner" software. You can select an
    "environment" to launch on a board runner, which governs the
    container environment provided to you. E.g., you can boot a given
    version of Ubuntu, which has a certain set of packages installed.

    This environment is then started in a container (system-nspawn),
    which provides access to only this board. Containers use a fresh,
    epehmeral root file system. Other boards are inaccessible.

    The container is then accessible via SSH.

- Brad: This looks cool. Where is the container running?

- Leon: This is running on the physical computer (e.g., Raspberry Pi
  or any other computer), connected directly via USB.

- Brad: Basically, you end up with a setup similar to if you just
  SSHed into a desktop sitting in your office, except that you can't
  see it.

- Leon: Exactly. Did think about adding a camera live stream, but not
  prototyped yet.

- Leon: Couple more technicalities:

  - This does not require any firewall exceptions / open ports. The
    "runner" computer next to the board connects to the central server
    and forwards all traffic through outgoing HTTPs connections
    (tunneling SSH through WebSockets).

- Brad: The local CI server basically needs:

  - some physical connection to the board

  - ability to run the containers

  - a little bit of software for management.

  Somehow you want to attach containers to a given board. How would
  you configure that?

  - Leon: The runner has a configuration file, where you can specify
    the exact set of devices passed through. Using udev rules to
    expose devices under well-known paths.

- Brad: The execution of a container is associated with a given user?

  - Leon: Yes. Containers run from images, which serve as
    templates. Two boards can share the same image, one board can have
    multiple images available.

    Idea behind this was to use it for both interactive development
    and for CI by just switching out the container image.

- Brad: Right. One model may be that we take this scheduler, and when
  a new CI workflow is started assign it to a board. Perhaps none is
  available, in which case we can either enqueue it, or return an
  error.

  - Leon: The system does support some very primitive queueing. If you
    attempt to start another job on a board that is currently in use,
    the job will be started when the other one is terminated.

- Brad: When we have a CI workload, do we want to have tests baked
  into the container, or do we load them in after the fact?

- Brad: Hopefully we can get someone to help with this. One request
  that I might have: the hardware platform will be physically
  distributed, and thus as pain to micro-manage / debug / etc. So it
  seems tricky to get right, or just specify to avoid dealing with a
  bunch of heterogeneity.

- Leon: Agreed! One really important thing to figure out there is how
  we integrate more buses and peripherals (i.e., GPIO
  connections). Can be a USB GPIO expander, FPGA, or something else.

  Deployment question is an interesting one, and a hard one to get
  right, but seems to be somewhat far out for now.

- Brad: The reason why I think that it is quite important to figure
  this out is that the system should really be distributed, and we
  want to able to tie in downstream users and contributors with their
  own platform. In theory things like OSes should provide the required
  abstractions, but in practice they're not sufficient. Trying to get
  this right the first time the best we can seems important.

- Leon: Good point. Have some knowledge to go by here, e.g.,
  netbooting a bunch of Raspberry Pis at a student ISP distributed
  over a large city. We should be able to take some inspiration from
  that.

  For tying in downstream users and deployment, there are more
  concerns, such as trust: are we going to manage these systems? Will
  we have a point of contact with downstream deployments? ...

- Brad: Hopefully what happens is that the value we get is so high
  that we have this built-in incentive for downstream users that may
  resolve some of these issues. People would be motivated to keep this
  going.

  We want to have a structure in place where we tell downstream users
  exactly what our expectations are (general architecture, OS,
  software components, availability, ...), that would set us up for
  success.
