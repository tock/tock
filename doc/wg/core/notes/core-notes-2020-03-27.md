# Tock Core Notes 3/27/2020

Attending:
  - Samuel Jero
  - Jean-Luc Watson
  - Amit Levy
  - Phil Levis
  - Pat Pannuto
  - Brad Campbell
  - Johnathan Van Why
  - Alistair
  - Garret Kelly
  - Guillaume
  - Hudson Ayers
  - Branden Ghena

## Updates!
 * Jonathan: libtock-rs core, is progressing.
 * Amit: started prototyping parts of the new time HIL, not yet in a state for other eyes on it, but also making some progress.
 * Hudson: now have 3 different working schedulers with different collections to manage processes. Hoping to float a PR to get some feedback within the next couple of days.
 * Amit: his student is starting to build parallel implementations of these algorithms (and using unsafe as liberally as possible to see if there's a performance benefit)
 * Phil: TRC on the Time HIL
 * Guillaume: working on fully-reproducible builds for talk - almost there, can recreate same SHA256 across different folders and machines. Looks promising. Before, Cargo would include different library identifiers based on your directory structure -- with a Cargo workspace, everything is a relative path so the overall build is reproducible
 * Amit: might solve some problems folks were having with VSCode

## Logistics

 * Zoom vs. Uberconference?
 * Amit: would folks be open to switching to Zoom? Those at universities have free pro accounts through their institutions, and already using them already. Comes with phone numbers you can call into. Can create a recurring meeting with a persistent Zoom address
 * Phil: strongly recommended that you don't start the meeting until the host joins, because scanners are trying to find idle rooms and displaying "unrelated" content
 * Phil: how does the audio transcript look like? Having them is incredibly valuable.
 * Amit: haven't actually tested the transcript in Zoom beyond a hello world
 * Branden: Zoom requires you to install something
 * Pat: actually it doesn't, but you need to find the option to enable it
 * Jonathan: won't attend meetings if requires installing an application
 * Brad: not having different tools for different meetings is nice. One thing that's bad about uberconference is the app is terrible compared to Zoom.
 * Amit: if we stuck with uberconference, do we have a reasonable way to host the other calls?
 * Brad: if we want to keep the transcription, requires that anyone hosting needs to pay for it, and needs to attend the meeting to turn it on.
 * Garret: Could just share the credentials between chairs so that at least one person makes the meeting every time.
 * Amit: It's worth it, if it's better.
 * Phil: can we table this for a month or so, let the privacy implications surface?
 * Amit: in the mean time, will share credentials with Brad!
  
 * Amit: will come back to asynchrony (more people, shifting timezones) at a future meeting where we have more time.

 * Working groups/taskforces
 * Amit: have already formed two different task forces (+ RISC-V working group): time interfaces, preemption in userspace. What do people think about formalizing what these mean, and using them as a tool we use more often?
 * Phil: don't think we need to add any formality to these small groups. Structure necessary when conflict over domain, or when there are failure cases. Should promote two values: seeking technical excellence and inclusion of participation (we'll listen to what you have to say), we'll be in good shape, certainly at our scale. Both of the existing task forces seem functional.
 * Brad: helpful to know ahead of time where projects stand. E.g. network stack, which is very fast and iterative, vs. HILs where we need to take our time. More clarity on how we view different WIPs, whether it's ok to have a partially implemented feature.
 * Phil: is there a structure you could add that would resolve that? Other than keeping an eye on things generally.
 * Brad: should at least have some understanding of what's going on, down in writing.
 * Phil: By that do you mean an understanding based on what directories the code is touching = bar that the code must meet (e.g. capsules/hil vs. capsules/...other)
 * Amit: Do you also mean that when beginning something new, there should be an email to the mailing list explaining the path forward.
 * Brad: Yes - a pointer in the PR to that email would be good as a reference to what had already been agreed on
 * Amit: for the emergent task forces, happy to prototype this process. Only other point of friction noticed is that it's not clear where communication happens: on the Slack? on tock-dev? a whole new mailing list?
 * Phil: let people do what works. If we want some representation of on-going work, the Github Projects are a good way to start. As opposed to using issues or tags, very difficult to find all ongoing work in one place. Happy to write up a Time HIL project/summary as a prototype of this.

## Time HIL
 * Phil: New addition to the Time TRD (Section 8), a list of required modules that chips should provide if they expect to use the HIL correctly.
     * MUST provide 32 KHz Alarm (requirement for ultra-low power clock setting)
     * MUST provide 64 bit Time: not sure if Time or Counter, essentially goes on forever increasing monotonically over system lifetime
     * SHOULD provide 1 MHz Alarm: not required due to power implications, not free running; there are apps that would like more resolution than a 32 KHz clock.
 * Amit: why require a specific frequency rather than a min/max frequency?
 * Phil: first, doc says you can't fake a frequency (e.g. use a 32 KHz clock to pretend you have a 1MHz alarm). If you say it's a range, an alarm is going to have to be adaptive to adjust the actual frequency to that it expects, which pushes the logical complexity upwards.
 * Amit: what if you don't have a 32 KHz alarm?
 * Phil: if something is faster, you can just bring it down to 32 KHz.
 * Amit: The whole point of the frequency associated type is so that client code can be written independently of the frequency. If we're going to have implementations for very specific frequencies, maybe we should just have different traits.
 * Guillaume: most of the timing code is already doing some conversion based on frequency into ticks, so don't see the need necessarily for a specific requirement.
 * Phil: basically a code size complexity question. It's code that's going to be _everywhere_ - code that is completely general but already expects frequency range precludes us from writing something much simpler.
 * Guillaume: wait time is a compile-time known constant, we can optimize away a lot of the code before runtime.
 * Phil: I don't have as much faith in the compiler; wary of assuming that the compiler is correctly optimizing out code.
 * Guillaume: we could measure code size impact, could be premature optimization to special case this for timers.
 * Phil: So the alternative is that there are not such required modules. You say something like "a chip must provide an instance of Alarm with a frequency higher than _this_"
 * Guillaume: if you didn't have a 32KHz clock on your chip, you'd end up having extra code on the device to make the existing Alarm compatible.
 * Phil: So you're saying that: I have some things that rely on 32kHz, and because I use it, then the cost of doing the conversion once there, and providing it uniformly to everyone, might be a greater cost than having all of them adapting?
 * Guillaume: Yeah, you could want to wait for some number of ms, then have some conversion to 32kHz, then another conversion to the frequency of the chip...{unintelligible}.
 * Phil: Good discussion, will look into updating with feedback on ranges rather than fixed points
 * Amit: meta point -- would be a good discussion for the task force working on the time HIL.
  
## Threat Model
 * Johnathan: Proposals for how to prevent DoS from applications with malicious TBF headers. Example: what if an application needs/requests a whole lot of RAM and starves out the other applications. Proposals:
     * 1. Tock's DoS protections only apply when all applications are loaded. Punts the issue to the application loader developer, but allows us to ignore malformed headers that 
     * 2. Mimic the kernel memory allocation logic in tockloader.
     * 3. Build system requests an MPU region, loader gives it to the application or fails.
     * 4. Board gives app that MPU region or a smaller one, could fail.
 * Alistair: What stops the MPU region from being malicious as well, in the same header?
 * Jonathan: the key there is that the loader could refuse to load an app if it's going to starve an existing app of resources.
 * Alistair: Couldn't it just do that based on the size? Why need additional MPU information?
 * Jonathan: That's true, I've added location information for relocation reasons (two birds with one stone), allows statically relocated apps.
 * Alistair: then the MPU one seems difficult because it requires userspace to have some understanding of hardware below it and the MPU setup.
 * Jonathan: any of the proposals that give DoS guarantees before loading require the build system or the loader to understand the MPU layout.
 * Brad: what's the difference between an app that needs a lot of memory and a malicious DoS app? And how does tockloader figure that out?
 * Jonathan: Loader shouldn't load an application if loading that app prevents a different app already installed from executing, or the installed app would never execute.
 * Amit: I prefer option 1
 * Brad: same for RAM, but don't prefer that in the case of Flash
 * Amit: option 1 doesn't say the loader can't/shouldn't prioritize some applications, just that the threat model doesn't guarantee that they will.
 * Jonathan: basically says the loader can do it, but not a requirement.
 * Brad: where does the kernel's policy fall into this? The kernel could unilaterally decide not to load an app.
 * Amit: If I can rephrase, there's nothing in the threat model that says "if an app exists in flash, it will be loaded".
 * Amit: in Titan, the deployment model will be a single signed binary, where this issue is a non-issue in any case
 * Jonathan: yes, that falls into the "individual boards/chips/loaders pick their own policies"

 * Johnathan: Separating format for system trusted TBF headers vs. other custom headers. Should we keep one format?
 * Alistair: why can't we just sign the headers, as they are today?
 * Jonathan: From OT perspective, not sure that signer of headers is someone who can be trusted
 * Amit: from the design perspective, Laura? is working on redesigning headers. Probably makes sense to split it up into more trust-relevant headers vs. app's own metadata for configuration. Not sure how that factors into the overall threat model?
 * Jonathan: might be a moot point until I figure out the consequences of DoSing each other through the TBF headers
 * Amit: might be fine to give weaker guarantees (e.g. only a portion of it needs to be trusted)
 * Jonathan: let's push off the remaining questions to next week, since they matter the most.
 * Amit: people should go look at those comments on threat model definitions/terminology.

## v1.5
 * Brad: a milestone for 1.5 is desired documentation items (#1464) -- would be good to have volunteers to write parts of it.
 * Amit: was going to get to it today.
 * Pat and Hudson: will do some editing as well.
 * Amit: we can coordinate on Slack to ask folks to volunteer.

