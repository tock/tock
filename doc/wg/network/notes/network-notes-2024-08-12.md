# Tock Network WG Meeting Notes

- **Date:** August 12, 2024
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda**
    1. Updates
    2. PacketBuffer
- **References:**
    - None


## Updates
### 15.4 Testing
- Tyler: Spent some time on 15.4 test infrastructure. Writing an automated test, which was failing due to a bug somewhere. Going to need to debug with hands on the hardware soon.
- Branden: Why do you need to have hands-on to debug this?
- Tyler: Don't have a wireshark trace, which I need for ground truth. It's also a lot more effort to debug in-person, plus I don't really trust the test infrastructure much yet.
- Branden: Okay, this makes sense. I expect we'll be using CI with tests we trust more, and sometimes we'll end up debugging those with our own boards on our desk.
- Branden: Networks like this are also very vulnerable to rot over time, so testing is great.
- Tyler: Next target is an OpenThread specific test too
### CI Infrastructure
- Leon: We have a CLI now for interacting with boards more directly, which could hopefully help for these manual debugging exercises
- Leon: Working with Max and Ben on building CI platform. Making great progress and almost ready to have alpha testers. While doing this, we're starting with nRFs, so we will probably translate some tests first.
- Leon: Also wired up some Ethernet boards with VLANs so we could test the Ethernet stack as we develop it.
- Leon: Before Network WG writes tests, we'll focus on release tests first.
- Branden: What has the work focused on?
- Leon: Recreating the Tockworld demo, but with reliable and maintainable code now. Rewrite now made it easier to get multiple people involved right away.


## PacketBuffer
- Leon: Need to follow-up with Alex and Amalia
- Leon: Plan will probably PacketBuffer as a standalone PR, followed by a draft PR of console showing the use. We do want to separate concerns, but need the use case as an example.
- Branden: Unrelated dumb question, could we use Macros to rewrite the generic parameters to hide them and maybe do math with them?
- Leon: When we're writing drivers, they still need to be extended with type parameters
- Leon: Inside board main files hiding stuff with a macro is totally doable
- Branden: Agreed, I knew that
- Leon: Prototype https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=78df75cd3f848be51795678cdf2982d9
- Leon: We could actually just do math with constants, and then place those into the right places
- Branden: In a component or something (yes)
- Branden: What I was thinking was whether we could hide stuff in the drivers themselves
- Leon: The types do still need to be fully qualified in their original locations
- Branden: I guess I was thinking:

```Rust
packetbufferize!(lower-layer-driver-name, upper-layer-driver-name, impl blah for blah { 

});
```

- Leon: Which would expand to?:

```Rust
impl<const HEAD: usize, const TAIL: usize> blah<HEAD, TAIL> for blah<HEAD, TAIL> {

}
```

- Leon: ~~That maybe works.~~ I just don't know if it's more confusing than the original.
- Branden: I'm not recommending it, just curious
- Branden: So, what I wanted was to do was to hide all the parameters somehow so they didn't appear when reading the driver code. And we could figure out what parameters we need by looking at the other driver files in some way?
- Leon: Okay, that won't work. Rewriting the code has to be separate from instantiation because they happen at different times
- Branden: Right. Yes.
- Leon: Also, I do want the draft PR for Console to have all the parameters, so we can see how bad they are to start with. We don't need to get rid of things until we actually see how bad they are
- Leon: I do definitely want to clean up board files with symbolic parameters rather than magic numbers. That should hopefully help quite a lot
- Branden: We will want to see what it would have looked like without the effort
- Leon: Okay, well I'll sync up with Amalia and figure out what to do from there

