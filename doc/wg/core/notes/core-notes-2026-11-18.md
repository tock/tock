# Tock Core Call Notes 2026-11-18

Attending:
- Brad Campbell
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why
- Pat Pannuto
- Branden Ghena

## Updates

### Tock Registers 2.0
 - Brad: Johnathan and I have merged the changes to the Tock repo that gets us ready for the 2.0 switch
 - Brad: Everything is version tock-registers v0, and we can update chips boards to what will be the 2.0 format, i.e., updating to `register_map!`
 - Johnathan: And I'll be working on the mass-migration of code
 - Johnathan: The other thing is that we realized that the `register_bitmaps!` side could use some love before 2.0 release; will likely discuss as agenda item today
 - Brad: We'd like to convert esp c3, qemu, nrf52, and x86 to 2.0 to have more hands-on experience to make sure things are how we want before greenlighting full conversion and formal 2.0 announcement
 - Brad: If we can get that set of chips set, then we'd feel confident in 2.0, mass migration, etc

### CI / Treadmill
 - Brad: We merged the first 'integration harness' PR
 - Brad: Should be getting us on the path for getting Treadmill running reliably
 - Leon: Have a series of PRs almost ready locally; PhD pre-defense on Monday [ed: Good luck!]

## Tock Registers "the other half"
 - Brad: In the work to tock-registers 2.0 so far, we have resolved the soundness issues
 - Brad: However, we haven't really scrutinized the user-facing API
 - Brad: We've iterated that over time as we've found issues, before we had any stability guarantees
 - Brad: But if we're going to release 2.0, now's the time to revisit this and solidify the interfaces
 - Brad: Review "Register Operation Summary" section of tock-register readme...
 - Brad: Having sat with this for a few days, there's some parts that are awkward, but none so bad that I feel strongly I don't think
 - Branden: What does `.debug` do?
 - Leon: It gives a printable version of the register, but in a way that lazily evaluates so it doesn't trigger incidental MMIO accesses
 - Leon: It gives a fairly nice overview of the register for human consumption
 - Branden: To my memory, this list was never reasoned through, it was pretty organic
 - Branden: But it's also proven sufficient for writing an embedded OS...
 - Johnathan: The one I noted (issue open) is that `read_as_enum` doesn't do any type checking; the only thing it 
 - Brad: What `read_as_enum` does require is that the enum implements `TryFromValue`, which is a tock-registers trait; it's a little bit awkward to come up with your own enum in say a chip driver that's sufficient; you'd have to implement this internal trait to tock-registers, which feels odd
 - Brad: The problem is that the `R` (`RegisterLongName`) only binds to the register type, not the field type, so it works for any field in the register
 - Brad: In a lot of cases this isn't really an issue probably, but it could be an issue if you have a register that has essentially the same field over and over, like a GPIO mode field and there's 8 of those in one register; a copied field with different bit offsets will generate different enums internally, and it kind of works, but it's not great
 - Brad: I think one kind of awkwardness with tock-registers is that this enum is created internally, and there's a reason the signature is like this, in that there's no ergonomic way to tell a user what `E` (the enum type) is; it happens implicitly when you define field values
 - Brad: To me, it seems like if you've bothered to create these field values, you should be using them; right now there's places in our codebase where we end up just doing a `.read` and then converting again manually to field values, which isn't great
 - Brad: Part of me wishes that the `.read` that you get depends on whether or not you specified field values
 - Branden: I get what you're saying
 - Pat: I wonder if read should just always return an enum (single variant), even if there are no field values, since you're getting back not just a `u32` but a type with some semantics from hardware about what that `u32` is
 - Brad: I fear that makes code awkward, since there's enums everywhere where you don't necessarily care that much about the conversion
 - Alexandru: But I think that's really not bad as long as they support `.into`
 - Pat: Yeah, there's a real tradeoff here, and I'm not sure which is better. But I think capturing types and type conversions explicitly better matches the spirit of Rust, and the mechanics aren't that burdensome since it's just `.into`; on the other hand, users might be annoyed / wonder "why do I have to type `.into` all the time??"; but I do feel like "lots of `.into` is pretty canonical Rust...
 - Johnathan: One other thing, if we are going to put effort into redesigning this; ferrous systems has a nice comparison blog post of the four major mmio libraries; maybe we should be more similar to other systems
    - https://ferrous-systems.com/blog/hardware-access-rust/
 - Alexandru: We might need to write assembly for tock-register for ARM 64
 - Johnathan: Upstream decided to fix the language
 - Johnathan: On ARM 64, if you're running in a confidential VM environment, the code that the compiler emits for MMIO isn't always valid; the new decision is to only ever emit safe instructions for volatile accesses
 - Brad: We have these two types in the signature most places, `<T, R>`; it's not clear why, might just be an artifact in how is was implemented
 - Brad: `T` is essentially the underlying type, e.g., `u32`, and `R` is the register's name
 - Brad: Internally, in 2.0, Johnathan already created a new type (`DataType`, `DT`), and I wonder if we shouldn't just have this single type
 - Johnathan: The reason I created this type was primarily to limit the scope of how much I was redesigning
 - Brad: If this was just one type (I'd probably just call it `R`), it makes it simpler
 - Pat: As a rough experiment, I threw Claude at the question, and it said it's a pure mechanical deletion of the type everywhere except LiteX and RiscV CSRs
 - Leon: Both of these can be fixed in isolation; LiteX just throw an agent at it should be fine; I think we need a full redesign of the RiscV CSRs anyway; none of these should be blockers
 - Brad: So you're supportive of simplifying the parameters, there should be one?
 - Leon: Yes.
 - Leon: The only argument that could be going against this is churn
 - Pat: This is the moment for churn (the 2.0 migration)
 - Brad: You want to guess how many `matches_any` are in our repo?
 - Brad: One. There's one.
 - Pat: I feel like I remember adding that when there was a bug with plain `matches`, which lead to the `_any|_all` split, but I don't remember the context; happy to remove the `_any` case though
 - Brad: There's a comment in our code in one of STM chips that says "should there be a `modify_[or|and]` API?"
 - Brad: What's happening in the driver is that there's a 'read; or of a flag; modify'
 - Alexandru: There's some chips that have funny things; e.g. RPi has a single register that has multiple addresses where different views write different parts
 - Pat: This is the opposite case of alias I think (alias is two different registers at same address, e.g., reads hit one and writes hit the other)
 - Pat: You're describing multiple memory addresses that point to the same underlying storage
 - Brad: Let me make sure I fully understand, there's a 
 - Alexandru: RPi has GPIO Value, GPIO Set, and GPIO Clear; these are all the same storage; Value is the read/write semantics, but Set and Clear only affect bits which are "1" on the write
 - Alexandru: How we do this right now is there's just a different register in the register map that's defined, and the driver has to pick the right thing
 - Alexandru: But it'd be nice if a `modify_or` could use the right 
 - Alexandru: A bunch of register in the Cortex R52 are not memory mapped, they are registers; we did macros for them
 - Johnathan: Are the addresses dynamic?
 - Alexandru: They're MRS/MSR
 - Pat: e.g., `MSR r1, CONTROL // Move the contents of the "special register" CONTROL into r1` that's the read of the CONTROL register
 - Pat: They aren't addresses, they are fixed encodings in the instruction itself
 - Leon: That's exactly what RISC-V CSRs are
 - Leon: Don't think we need another full proc_macro for this, can do with declarative rules
 - Brad: New thing. `write` is I want to write this field and clear everything else; `modify` does the read and write
 - Johnathan: I don't like this, should just have one
 - Pat: Need both though, because `modify` does two memory transactions and reads can have side-effects
 - Johnathan: But `set` exists
 - Leon: `write` is a commonly used method, because of MMIO semantics, but the naming is confusing
 - Brad: So what would happen if we just got rid of `write`
 - Leon: It's kind of sad that we'd have to have callers manually convert the FieldValue to the register type
 - Leon: Really, `write` should take a named register type
 - Pat: This is this mirror of what we discussed with `.read` earlier; the same 'always a named type'?
 - Leon: Yes; then you can have something that can more explicitly convert the FieldValue into the named register type
 - Leon: This is a fairly common pattern in Rust, it's sort of the builder pattern
 - Brad: What I'm taking away from this, is that with a little crowdsourcing on one call we've come up with real and good ideas, thus it is worth it to revisit this API for 2.0
 - Brad: Hopefully we can move forward in parallel, however, i.e., this should not block switching to `register_map`
 - Brad: And we have a target: We'd like to get tock-registers 2.0 done by the end of the calendar year.

## USB CDC Changes
 - https://github.com/tock/tock/pull/5304
 - https://github.com/tock/tock/pull/5356
 - Leon: The reason I held off on reviewing these is that I just don't know anything about that subsystem
 - Brad: These have been so tricky because they've been contributed, but not quite in ways that align with how we'd want them done, but that's hard to express sometimes, so I've just been recreating them
 - Brad: But these are clearly real pain points for downstream users, so we should get the fixes in
 - Brad: Now we're in the case where I've opened PRs, but I don't want to merge my PR
 - Leon: I did a quick look of 5304; approved
 - Pat: I've got a few minutes, and I understand USB a bit, I can look at these after the call

## Register Struct PRs
 - https://github.com/tock/tock/pull/5281
 - https://github.com/tock/tock/pull/5315
 - (etc?)
 - Brad: I'd appreciate if someone would merge all of these
 - Leon: Do they have two approvals?
 - Brad: No.
 - Leon: I think I've mechanically reviewed all of them.
 - Johnathan: I've reviewed Amit's _tool_, and identified a few tiny bugs; was hoping he'd address those and would re-run and verify output is the same
