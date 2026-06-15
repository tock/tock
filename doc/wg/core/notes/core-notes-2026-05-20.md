# Tock Meeting Notes 2026-05-20

## Attendees
 - Branden Ghena
 - Johnathan Van Why
 - Amit Levy


## Tock Registers Update
 * Johnathan: Work on Tock registers PR has reached point where I think this is the correct design moving forward. Thought about typestates and DMA deeply for a bit, and found that they shouldn't require any changes to the design. Can be developed outside of tock-registers and then later move into it if we want.
 * Johnathan: I'm going to reach out to the dev email list next for thoughts. Then start making PRs to merge
 * Johnathan: I'm also going to reach out to the Rust embedded community about it asking for feedback. I'm gonna make a matrix chat room so people can live-chat there about it.
 * Amit: Awesome!!
 * Branden: Yeah, super big deal. It'll be great to have this merged into Tock
 * Johnathan: Lots of small nit-picky things we should decide on. Some today.

### Version Numbering
 * Johnathan: Should this be 1.0, or 2.0, or a new minor version? Or what?
 * Branden: My thoughts are that we should go straight to 2.0. This is a new thing, separate from the prior thing. So 1.0 might feel like just building on top of the prior thing.
 * Johnathan: We should release a version with both the old and new stuff. Otherwise you'd have to update everything if you jumped to the new version.
 * Amit: Couldn't you have two dependencies and rename one of them
 * Branden: My concern is that having new stuff and old stuff together in a release is a bit of a mess
 * Johnathan: Okay, so we'd have no transitional release. Just go straight to 2.0.0 with no 1.0.0 release ever existing. Or we could do a transitional release with both the old stuff and new stuff together as a 0.something release.
 * Amit: Does crates.io support beta releases? We could have 2.0-beta, then 2.0 which supersedes it? Is there a way for us to non-committally try the new thing. That way we can see how hard it is to do the transition. That would help us decide whether to briefly have both available.
 * Johnathan: We have about 7000 registers in Tock. Not sure how painful that will be. Cargo has some reference to alpha releases, so that might be an option.
 * Amit: Might be good for soliciting feedback
 * Johnathan: So far all of my changes have been backwards-compatible. Deleting the old thing would let us make other breaking changes though.

### Merge Strategy
 * Johnathan: Planning to merge into master directly, get it all in, then do a release. But should we instead make a feature branch, merge into feature branch, then merge that into master
 * Johnathan: Is there any reason to use a master branch?
 * Branden: Will master end up broken if you merge some PRs but not others?
 * Johnathan: No.
 * Branden: Then I don't mind merging directly into master

### Punctuation Bike-shedding
 * Johnathan: https://github.com/tock/tock-registers/pull/11#discussion_r3274627305
 * Branden: I vote Idea A: commas everywhere. Everything else has rules about when and where to do things which I'm going to forget, and commas everywhere is straightforward
 * Amit: No opinion here, seems fine

### Renaming Macro
 * Johnathan: Should it be `register_layouts!` or something else? Want this before sharing PR broadly
 * Branden: Brad thought this should be `register_layout!` singular. I suspect no one actually cares
 * Amit: Why not `registers!`?
 * Johnathan: Overloaded with other traits and structs. Registers plural appears 900 times in the PR. Worried about word losing meaning.
 * Amit: But doesn't it mean the same thing? Or we could use `regs!` maybe?
 * Amit: Why this matters: this is the main thing that people are typing and reading. 99% of the time people will use the public interface in their own code. The thing in their own code being useful seems important.
 * Branden: I thought it was nice to have register_layouts compared to register_bitfields. Both are part of registers
 * Amit: It does take in a single register map. One or more maps actually
 * Johnathan: Yes. Could be as small as a single register.
 * Branden: So that lands on `register_layout!` to me. It's a set of registers that exists as a single layout
 * Johnathan: Hmm. Inside of the macro, it provides zero or more things. Those things are single registers or register blocks. Each of those things internally is a "layout". But also, a chip has a register layout. So the collection of things you accumulate is a layout. Means two different things.
 * Amit: To me, a layout is a set of registers with positions relative to each other.
 * Johnathan: That is a register block.
 * Amit: So what is a register_layout?
 * Johnathan: It's an AST
 * Amit: So registers from `registers!` means something. But the "layout" portion of `register_layouts!` doesn't mean anything useful. We wouldn't want to name it `register_ast_node!`
 * Branden: So I think a register block is what Brad and I thought a register_layout was. Maybe we were just confused
 * Amit: I think a block sounds like a meaningful term inside of tock registers. And a register map is a term of the same thing
 * Amit: A UART has a register map, registers that are near each other. A chip has a register map which is many register blocks that aren't near each other.
 * Branden: I don't understand what a `register_layout!` that isn't "a set of registers with positions relative to each other"
 * Johnathan: They're definitely different. You can make a layout that has a single register, or multiple which aren't positioned relative to each other.
 * Branden: I don't understand what that would even mean
 * Amit: So the things inside a `layouts!` is a register, a block, a whatever. Each of those single elements, has one or more registers within it with positions relative to each other. Each block is also named independently. Blocks do not have relative relationships to each other. For example, if I wanted to define the entire set of registers for a chip, I would not do that with a single register_layouts! because that wouldn't let me specify positions relative to each other.
 * Amit: When I define more than one thing in a single `register_layouts!`, I have a name for each of these things, which is semantically identical to defining them in separate macro invocations.
 * Johnathan: You have to copy the inner attributes, but yes
 * Branden: So then, why does `register_layouts!` allow you to put multiple things inside of it?
 * Johnathan: It allows you to have inner attributes that apply to all of the things inside that register invocation. It's not a huge deal at the moment, because the only attributes are bus, buses, and doc comments
 * Branden: If we only held a single thing inside the macro, then it would make sense to me. Right now it doesn't make sense to me because it's an abstract syntax tree
 * Branden: We could make it so it only holds one thing
 * Johnathan: That would probably be fine
 * Amit: So we could have it hold one thing, and have it called `register_map!` or `register_layout!`. Or hold multiple things and it should be plural and something different from those two. The name has to be explainable.
 * Johnathan: I'm also fine with `regs!`
 * Amit: `regs!` makes sense as a connotation of "I'm entering the DSL". But the length of the name doesn't matter so much
 * Amit: How often do we expect reusing inner attributes to be useful?
 * Johnathan: Probably not often. I'm just worried that I'm wrong and would have to re-add stuff later.
 * Amit: Yeah. Or is there another way to do it.
 * Johnathan: We could allow multiple things, but sort-of hide it and make that not the default usage
 * Amit: Is it possible to share inner attributes via some external thing?
 * Johnathan: That would be hard. We'd have to invoke one macro from another macro. Gets messy fast
 * Amit: For what it's worth, I feel fine with something like `register_layouts!` be an experimental or not highly surfaced option that's still "there". And having `register_map!` be only a single register block and be the main thing that's advertised and documented. And if use cases come up, then we can do stuff.
 * Branden: I like that
 * Johnathan: So you're proposing two macros, one with multiple things internally and one with only a single.
 * Amit: Yes. Just call register_layouts! from register_map!
 * Johnathan: Okay. That would be several macros, but doable. Feels a little duplicative. I think we just have it accept one thing by default, and we have it possibly accept two things but not be obviously presented
 * Amit: Could have them be aliased and identical, but documented differently
 * Johnathan: I think there shouldn't be more names. Confusing.
 * Amit: We just wouldn't want it to be part of the stable API, that way we could remove it later if we wanted
 * Johnathan: Then remove it and make it only accept one. It'll be a pain to add back in later

