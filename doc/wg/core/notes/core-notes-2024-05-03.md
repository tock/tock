# Tock Meeting Notes 2024-04-19

## Attendees
- Branden Ghena
- Amit Levy
- Alexandru Radovici
- Samir Rashid
- Leon Schuermann
- Anthony Tarbinian
- Amalia Simion
- Hudson Ayers
- Tyler Potyondy
- Alyssa Haroldsen
- Johnathan Van Why
- Andrew Imwalle


## Introductions
- Samir and Anthony are MS students at UCSD. Joining today to talk about Flash isolation
- Amalia is a student working with Alex and is working on the PacketBuffer application for console


## Updates
- Tyler: State of the world before the CPS Week Thread/Tock tutorial. We're mostly merged with the PRs and Todos. There are some alarm fixes needed still, but I think the smaller Alarm PR Leon has is likely to fix it. Also one Link-quality Indicator as well. The libtock-C port is also almost ready to merge.
- Tyler: In terms of writing, Leon and I have a vision for the applications and we'll hopefully finish the implementations today. We'll do the writing next week and we'll call out to others to check over that writeup


## PacketBuffer / Console Demo
- Hudson: Background is using PacketBuffers to separate kernel and process output
- Amalia: (sharing screen to demo things)
- Amalia: Developed a host-side application that can lead you interact with multiple Tock processes simultaneously. This app lets you choose between the applications on the board, keeps a list to let you select which one you're talking to over serial. Also lets you choose the kernel as one of the output/input sources.
- Amalia: This is the first version of the application. You can only see one process at a time. But there is also work in progress to have a layout which shows multiple process outputs simultaneously.
- Amalia: (demonstrates both of these working!!!)
- Hudson: This is really exciting (general agreement). A question, what happens to output from an application that's not in the foreground? Is it buffered for later?
- Amalia: There's a parser for each app. The messages sent when the process isn't in the foreground is saved. Separate buffers for each process
- Alex: It's a VT100 terminal too, so it renders with colors and stuff
- Amit: What about input? Can that be multiplexed too?
- Amalia: Not yet. Tock is also not able to multiplex input, so even typing in the app goes to the process console
- Amit: Can you explain details of how this works? How does the PacketBuffer come into it?
- Amalia: The buffer infrastructure Leon made allows for appending headers and footers to payloads throughout layers of a driver stack. In the uart stack, when a message is sent a process ID is appended, and also there's another flag for whether the source is a process or the kernel. With this info, the host application is able to distinguish the data sources. We also append a footer for the end of the packet. So the host-side application looks for the end of the packet, parses the header, then filters to the proper application console.
- Leon: I am also really excited about the state so far. I made the first version of the PacketBuffer, but Amalia has done a TON of work from there. Two things to mention about the PacketBuffer: 1) it completely avoids reallocation by communicating how much space is needed for lower layers and 2) it's guaranteed at compile time that there's actually enough space for the headers/footers. So a driver will never run out of space it needs to append stuff
- Alyssa: Do we use nightly generic const expressions?
- Leon: No. Unfortunately, we do everything manually for calculating sizes to avoid that. It would be super nice if we could add it, if it ever became stabilized
- Amalia: At each layer we have our own needed Headroom and Tailroom, and the lower layer's needed Headroom and Tailroom. So there are lot of const generic values passed through the interfaces of layers.
- Leon: Specifically, there's an operation that can convert between generic types with differing amount of headroom/tailroom by having a wrapper that's entirely inlined and optimized away
- Hudson: What's the raw serial output look like when this is running? Does it still make sense?
- Amalia: Sort of. There are non-printable characters that appear everywhere. Before and after each set of bytes sent.
- Hudson: It's not too bad, actually
- Alex: So, what would be the roadmap to implement this? Our hope is to integrate this host-side application into libtock-rs. How would this be merged into Tock side?
- Hudson: I think it depends a little bit. Do you expect libtock-rs to be at feature parity when you merge this?
- Alex: Probably not. We're hoping for usable this summer, but Tockloader has many features. We could add an interface for this to Tockloader, at least so it could remove the non-printable characters
- Hudson: We could totally have it as an option and explain how to enable and use it
- Leon: Also reasonable to me would to have it only be enabled after you send some magical character sequence. So any board would look fine until you started the host-side application, and it would switch at runtime
- Alex: I think that would work
- Hudson: Any idea what the code-size overhead of this would be?
- Leon: We haven't measured at all
- Branden: If it's a separate capsule in main.rs, we could have boards use it but people who care about code size can just not use it
- Leon: That could work. We would have to move the UART HIL to packetbuffers if we wanted this
- Leon: My vision for this now is to re-approach the PacketBuffer implementation and propose that to merge into Tock fist. Then later we would plan to merge the changes to UART. Then this bigger change
- Alex: So we should implement the PacketBuffer stuff in the UART HIL and console and whatnot, but without adding any extra data first. Then we could measure the size of that
- Tyler: So for the python Tockloader listen, we would just filter out the bytes here? Do we think other users might be using some other serial interface and this would get annoying? This would somewhat force Tockloader on users
- Hudson: Leon's suggestion of only enabling it after a magic control sequence from the host would fix this. So Tockloader implementations could send that sequence but other serial consoles would not
- Alex: Yeah, a process console command maybe. We might also have a flag that completely disables it.
- Alex: Amalia will be at Tockworld this year, so we'll plan a larger demo there


## Process Isolation for App Non-volatile Storage
- https://github.com/tock/tock/issues/3905
- Tyler: Some context here, OpenThread needs to write some persistent state to Flash. Particularly the network key. Currently we just dump that into shared Flash which any app can access, which isn't great. So Samir and Anthony have been working on the problem
- Anthony: The motivation is that OpenThread stores crypto keys, and those should be isolated and other processes shouldn't be able to read it. We started by looking into what Tock currently has for Flash interfaces, the app_flash_driver and nonvolatile_storage_driver.
- Anthony: App Flash does have isolation, but it forces us to write the entire Flash region every time it gets changed. And we don't want to write a full 4 kB every time we change anything.
- Anthony: Non-volatile storage does let you write on a page basis (smaller than 4 kB), but doesn't have isolation
- Anthony: We wrote some approaches in the issue. Option 1 was to have a main.rs file that allocates certain regions of flash to certain apps. The downside is that the kernel must be aware of the processes running on the board.
- Anthony: Option 2 is for processes to make requests of how much Flash they need, and the app tracks ownership of page allocations. So there would be some table (in non-volatile memory) with allocations. Certainly more complicated, and there's a question of how long does the process claim the page for. What ever releases pages?
- Anthony: Option 3 was designed by Alistair and suggests that instead of hard-coding regions, we make some generic per-app flash size. So every app would get one page or ten or something. And user processes could query to see how much space they have and interact with it. Still sort of has issue of ownership of process Flash regions and whether they're ever given up.
- Anthony: One more idea we brainstormed with Pat but haven't written yet. We could offload the complexity to Tockloader and have Tockloader read Flash regions and swap out the Flash storage with the application, so that state gets preserved. So if you re-flash the app, the Flash storage would come with it. Then you wouldn't lose any state when you add new apps.
- Anthony: We wanted to bring this up as a design overview to get some thoughts from the group.
- Tyler: We're hoping to learn what people have strong opinions on before starting work
- Branden: I'm not sure about the application needs. When does storage go away and when does it persist? If you reflash the kernel does it go away? What if you reflash an app?
- Tyler: Good question. Currently, I think as long as an application is on a board, it should keep the region.
- Branden: I think there are cases on both sides. Sometimes you want storage to stay as you make little changes and bugfixes. Sometimes you want the storage to disappear when you've changed how it works.
- Tyler: For Tockloader support, it would be about moving an app. We think that should also still have persistent memory, since the application is still installed.
- Tyler: I do think the ideas from Alistair and Brad on the issue about some linked-list structure for tracking the allocation of regions would be useful. Do other people on the call have differing opinions from them or pushback?
- Leon: This isn't the first time we've had this conversation. I think this was envisioned as part of App ID too. Maybe Phil should be part of the discussion
- Johnathan: It was certainly part of the original design
- Leon: Our workaround for the tutorial is that kernel can reserve flash storage in the kernel, and we're giving the app access to that. But reflashing the kernel erases that, which is unfortunate. The reason we're choosing this right now, instead of putting it somewhere else, is that we would need to have a new region that Tockloader is aware of so it doesn't overwrite it.
- Leon: We do have some changes in Tockloader where we place a footer in the kernel binary that has some details about application flash. So we could maybe have an extra field in there that tracks where bonus flash for applications resides
- Hudson: I would love to see any solution move forward and not let perfect get in the way of good enough. I think an initial design can ignore Tockloader adding/removing/moving applications and treat that as erasing Flash. So we promise storage across reboots, but not across loading applications
- Alex: One extra complication, this stuff works pretty well on the nRFs, because there are many flash regions. But STMs and NXP microcontrollers have much coarser flash regions and can't modify one without moving code to memory and executing from there. We already have some Tockloader issues on those platforms so be careful not to make that worse
- Tyler: Good to know
- Tyler: I know there was a proposal for app signing once upon a time. Is there something we should be careful about to not exclude using that stuff at some point?
- Johnathan: There's a very long thread on the Tock dev mailing list from 2020 that explores that in way too much detail. For the purpose of storage isolation, if two processes have the same application ID, they are the same app and should have access to the same storage. That's not just concurrently, that might be across reboots, reflashes, etc. So your storage driver can still identify it on reboot. What I don't know is if that property was maintained for short-IDs for applications. Overall, I would say that your identity should be aligned with application ID.
- Johnathan: There is a kernel question about when to deallocate memory. If a process isn't there at all, it may just temporarily be gone
- Johnathan: So maybe browse that email thread, but definitely read Phil's Application ID TRD
- Samir: Thanks for all of that! One question we are considering: should an app ever be able to ask for more flash, or is there some initial startup size specified during flash time, and that's it?
- Tyler: A parameter in the Makefile like STACK_SIZE for example
- Hudson: Okay, that's pretty different from Alistair's proposal then. Where apps could make dynamic requests at runtime.
- Tyler: Still a thought in progress. It seems like maybe a hardcoded design would be simpler than a dynamic design
- Hudson: I do see the advantages of both approaches, and haven't thought enough to support either right now


