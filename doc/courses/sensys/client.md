Part 3: Keep the client happy
=============================

---

## Course Agenda

- [Introduction](README.md)
- Part 1: [Getting started with Tock](environment.md)
- Part 2: [Application Basics](application.md)
- **Part 3: Client Delivery**
- Part 4: [Free-form Play](freeform.md)

---

You, an engineer newly added to a top-secret project in your organization,
have been directed to commission a new imix node for your most important client.
The directions you receive are terse, but helpful:

```
On Sunday, Nov 4, 2018, Director Hines wrote:

Welcome to the team, need you to get started right away. The client needs
an imix setup with their two apps -- ASAP. Make sure it is working,
we need to keep this client happy.

- DH
```

Hmm, ok, not a lot to go on, but luckily in orientation you learned how
to flash a kernel and apps on to the imix board, so you are all set for
your first assignment.

There were some other apps in `libtock-c/examples` so you go looking there first.
And sure enough there is a folder called "important-client"! And even better,
it has two apps inside of it! "Alright!" you are thinking, "My first day
is shaping up to go pretty smoothly."

After installing those two apps, which are a little mysterious still, you
decide that it would also be a good idea to install an app you are more
familiar with: the "blink" app. After doing all of that, you run `tockloader
list` and see the following:

```
$ tockloader list

No device name specified. Using default "tock"
Using "/dev/ttyUSB1 - imix IoT Module - TockOS"

[App 0]
  Name:                  app2
  Enabled:               True
  Sticky:                False
  Total Size in Flash:   16384 bytes


[App 1]
  Name:                  app1
  Enabled:               True
  Sticky:                False
  Total Size in Flash:   8192 bytes


[App 2]
  Name:                  blink
  Enabled:               True
  Sticky:                False
  Total Size in Flash:   2048 bytes


Finished in 1.959 seconds
```

---
> ### Checkpoint
>
> Make sure you have these apps installed correctly and `tockloader list`
> produces similar output as shown here.
---

Great! Now you check that the LED is blinking, and sure enough, no problems
there. The blink app was just for testing, so you `tockloader uninstall blink`
to remove that. So far, so good, Tock!
But, before you prepare to head home after a
successful day, you start to wonder if maybe this was a little too easy. Also,
if you get this wrong, it's not going to look good as the new person on the team.

Looking in the folders for the two applications, you notice a brief description
of the apps, and a URL. Ok, maybe you can check if everything is working.
After trying things for a little bit, everything seems to be in order. You
tell the director the board is ready and head home a little early&mdash;you did
just successfully complete your first project for a major client after all.


## Back at Work the Next Day

Expecting a more challenging project after how well things went yesterday, you are
instead greeted by this email:

```
On Monday, Nov 5, 2018, Director Hines wrote:

I know you are new, but what did you do?? I've been getting calls all morning
from the client, the imix board you gave them ran out battery already!! Are you
sure you set up the board correctly? Fix it, and get it back to me later today.

- DH
```

Well, that's not good. You already removed the blink app, so it can't be that.
What you need is some way to inspect the board and see if something looks like
it is going awry. You first try:

```
$ tockloader listen
```

to see if any debugging information is being printed. A little, but nothing
helpful. Before trying to look around the code, you decided to try sending the
board a plea for help:

```
help
```

and, surprisingly, it responded!

```
Welcome to the process console.
Valid commands are: help status list stop start
```

Ok! Maybe the process console can help. Try the `status` command:

```
Total processes: 2
Active processes: 2
Timeslice expirations: 4277
```

It seems this tool is actually able to inspect the current system and the active
processes! But hmmm, it seems there are a lot of "timeslice expirations". From
orientation, you remember that processes are allocated only a certain quantum
of time to execute, and if they exceed that the kernel forces a context switch
back to the kernel. If that is happening a lot, then the board is likely unable
to go to sleep! That could explain why the battery is draining so fast!

But which process is at fault? Perhaps we should try another command.
Maybe `list`:

```
 PID    Name                Quanta  Syscalls  Dropped Callbacks    State
  00	app2                     0       336                  0  Yielded
  01	app1                  8556   1439951                  0  Running


```

Ok! Now we have the status of individual applications. And aha! We can clearly
see the faulty application. From our testing we know that one app detects
button presses and one app is transmitting sensor data. Let's see if we can
disable the faulty app somehow and see which data packets we are still getting.
Going back to the help command, the `stop` command seems promising:

```
stop <app name>
```

Then checking back on the status webpage we now know what functionality we
have to fix.


## Time to Fix the App

After debugging, we now know three things about the issue:

- The name of the faulty app.
- What that app is supposed to do.
- That it is functionally correct but is for some reason consuming excess CPU cycles.

Using this information, dig into the the faulty app.

### A Quick Fix

To get the director off your back, you should be able to introduce a simple fix
that will reduce wakeups by
[waiting a bit](https://github.com/tock/libtock-c/blob/21234c671eee0ae491faa5d23f35f3762b25c522/libtock/timer.h#L76)
between samples.

### A Better Way

While the quick fix will slow the number of wakeups, you know that you can do
better than polling for something like a button press! Tock supports
asynchronous operations allowing user processes to _subscribe_ to interrupts.

Looking at the [button interface](https://github.com/tock/libtock-c/blob/master/libtock/button.h),
it looks like we'll first have to enable interrupts and then sign up to listen to them.

Once this energy-optimal patch is in place, it'll be time to kick off a
triumphant e-mail to the director, and then off to celebrate with some
[baijiu](https://en.wikipedia.org/wiki/Baijiu)!

