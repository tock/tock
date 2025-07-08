# Tock Network WG Meeting Notes

- **Date:** September 23, 2024
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Alex Radovici
- **Agenda**
    1. EWSN Tutorial Planning
    2. 15.4 Integration Testing
- **References:**
    - [4177](https://github.com/tock/tock/pull/4177)


## EWSN Tutorial Planning
 * Tyler: EWSN tutorial session is looking likely. Anthony (MS student at UCSD) will be helping support that. Reprise of the Thread tutorial. Drives the desire for Thread CI to make sure things don't break!
 * Branden: How are you going to do dev environments?
 * Tyler: Seems like the solution is just a little more polished, with better Windows support. Feels like our discussions on ideas are just running in place, no obvious ideas for how to do better.
 * Branden: Could have Docker images just to compile, and leave loading to local host. But maybe loading is actually the hard part...
 * Leon: Docker setup isn't exactly easy either. Could have an RDP / remote environment to some network server to  build code, then download from there and flash it.
 * Tyler: That feels like quite a bit of work
 * Leon: I saw someone do this with a thing called "Guacamole", which was basically a VNC viewer in the browser to an AWS VM.
 * Branden: A Docker version of that is called Github Codespaces. Lets you make an environment
 * Tyler: Fixing the compile-side issues is not nothing. But we really need to play more with loading on Windows.
 * Tyler: The Microsoft people, like Bobby, had some ideas and some stuff working there. May need to reach out to them
 * Leon: We should definitely remove the VM option. The tutorials still point to a really old VM image Brad is still hosting too.
 * Leon: Also, some attendees won't have admin rights to their machines?
 * Branden: What about RPis for everyone to use to program the boards? They connect to the RPi from their machines, and then they use the RPi for all development
 * Leon: Connecting to the RPis is non-trivial
 * Tyler: I am thinking maybe WiFi and VNC is okay?
 * Leon: Or maybe "Guacamole" works on the RPi, so people could just use a browser tab
 * Tyler: We might just warn people that they need a laptop that they have dev privilege on
 * Leon: We could buy precisely identical RPis for tutorials that we are using for CI anyways
 * Leon: Or for times when latency isn't an issue, we could literally use Treadmill
 * Tyler: Okay, action items are to circle back with Microsoft folks and concretely deciding what our plan to try out is for dev environment


## 15.4 Integration Testing
 * Tyler: What's the state of Treadmill? Where's it at and what are next steps for me to work on a 15.4 test?
 * Leon: First, there's a PR for integrating Treadmill into CI as of today! https://github.com/tock/tock/pull/4177
 * Leon: State now. First runs a Github actions job which spawns a variety of jobs for Treadmill to handle dynamically which actually run on different boards. Those jobs can be programmed with Github Actions. You write instructions like you do for any other Github workflow, but every shell command is running on the Treadmill RPis.
 * Leon: Right now, all it does is compile the board and does an lsusb to check that the board is connected.
 * Leon: In the next PR, we'll add a Python script with some basic integration tests.
 * Leon: So mapping to 15.4, we probably want to run sometimes, but not on every PR. Maybe nightly and let it run for an hour or two. And I think Tyler has scripts which already run stuff.
 * Leon: One thing that's missing right now: don't have a way to guarantee that when you launch a jobset of multiple jobs on multiple boards, you don't have an assurance that they'll run in parallel. So if we have three RPis, each of the three can have jobs, but the current form doesn't guarantee that they _will_ run in parallel.
 * Leon: The way to solve this now is that you can reserve all three of these, and only launch jobs on the reservation itself. It's possible that another job sneaks in though.
 * Leon: So, an integration test across distributed boards on distributed hosts is tough right now
 * Tyler: So reservations: we could have three RPis here at UCSD which are only for networks?
 * Leon: Well, we don't really have reservations, but we have permissions. So only one user could put stuff on some RPis. Which works about the same
 * Tyler: And my scripts just rely on the serial output to see if packets are received. There were some messages on the Matrix channel about how to do that. Are tests currently using serial output?
 * Leon: We don't have a working proposal or consensus right now. Ben is actively working on it. We really want to separate the testing workloads from the Treadmill platform. Treadmill can run arbitrary commands. We will want a standard way to do integration tests, but we haven't fully decided that. And 15.4 testing could have a different interface if it needs
 * Leon: I do think there will be at least two types of tests: short running integration tests and long running nightly/weekly tests. The long-running tests might look very different from the basic tests. They could need coordination between RPis, for instance.
 * Tyler: Does Treadmill have support for flashing boards?
 * Leon: Our script can flash boards via Tockloader. Really just invoking Tockloader
 * Tyler: I have written some scripts that do all the flashing. I'm hoping that will mostly work on Treadmill
 * Leon: Yes. We just give you a Linux shell. You can do pretty much anything inside that.
 * Tyler: Okay. So Treadmill is just the platform. And Treadmill has some way of doing actions that hook into the shell?
 * Leon: Demo time (sharing screen). Treadmill can enqueue some job with an image onto an RPi. Treadmill will schedule that on some hardware. It just opens an Ubuntu VM.
 * Leon: On Github, Github actions just runs commands to create the VM. Then it can farm out commands to the VM. Github Actions will abstract away the need to log in ourselves, it will run commands there and pipe the results back to Github and then can inspect the results.
 * Branden: How do you decide where to do interactions with boards? Is parsing done on Treadmill or on Github Actions?
 * Leon: On Treadmill. Github Actions only checks success/failure of the jobs
 * Branden: Back to 15.4 testing plan. Could we have one RPi with a bunch of boards attached that presents itself as one "15.4 testbench board"?
 * Leon: Yes, definitely. We could have a new board name and we can specify that a job requires a certain board
 * Leon: In the long term, we should also have a notion of a parallel jobset. Because the 15.4 boards could run normal things too when they're not in use
 * Branden: That's an optimization though. If we have enough hardware, we're okay
 * Tyler: What about timing of which board comes up when?
 * Leon: Yes, we would need some synchronization mechanism if we're across multiple RPis
 * Tyler: Right now, we program all boards, panic each as we go, then reset them in a particular order
 * Branden: And that should work fine if we have a single RPi with multiple boards
 * Leon: Agreed. I'm talking about big ideas for the system direction. But right now, one RPi with multiple boards is the perfect way to do this and should work with the system as-is. In the future, I'd like to have the idea of a jobset across multiple boards. That would be really interesting to have, but it's distant still
 * Tyler: Okay, so it sounds like we're at a place where Treadmill is just about stable enough to do some testing. My scripts aren't perfect, but if I polish them up a bit, we could drop them into Treadmill and kick them off on the RPi. And that should run on one RPi with four boards attached.
 * Leon: Yes. What we can do is hook up some nRFs to an RPi for you in the next few days. Or we could hook up an RPi for you at UCSD.
 * Branden: Okay, a plan is: 1) Tyler should get an RPi and attach four boards and polish his script so he can manually run it on the RPi and test boards. 2) Leon should set up a new RPi with four nRFs at Princeton and set it up as a new board in Treadmill. Then 3) Tyler and Leon can move the script over to Treadmill when they're confident in it.
 * Tyler: Cool. I will target some version of this working by late October
 * Leon: That's a reasonable timeline
 * Branden: And do we want to talk about Tyler's 15.4 script at all?
 * Tyler: It's not cleaned up yet, but what it's doing is building Tock and Libtock-c. Then it flashes with a modified Tockloader to specify the Jlink ID (I'll make a PR to Tockloader). So it uses Tockloader to flash the four boards. Then it can read the serial output with pyserial and sh to run shell commands like Tockloader. There are also YAML files with config information for the boards, like JLink serial numbers.
 * Branden: How do you control timing? Like waiting for a network to set up?
 * Tyler: I just had an OpenThread router running near these boards. So it's less of a timing constraint for the child. Rebooting the boards once they're all set up is sufficient
 * Branden: Is that OpenThread router separate?
 * Tyler: It is right now
 * Branden: That really needs to be part of the script, to make this portable to Treadmill/Princeton
 * Leon: Looking at Treadmill right now, we are making a general framework with pexpect.
 * Tyler: Mine is pretty non-general. But we could start with that, and I could migrate things later
 * Leon: One annoying thing, nrfjprog doesn't work on ARM64 architectures, so you'll have to use JLink directly
 * Leon: Adding this test as a nightly run, sounds like a very good plan. We'll be able to support that in weeks, not months
 * Tyler: Okay. I think this work needs to start on my end. I need to get the scripts working on my computer, and on an RPi locally. Then I'll go into Treadmill after that.
 
