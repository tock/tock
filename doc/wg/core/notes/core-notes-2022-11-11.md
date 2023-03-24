# Tock Core Notes 2022-11-11

Attendees:
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Chris Frantz
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Vadim Sukhomlinov

# Updates
 * Hudson: One updated from Pat, who is not on the call, is he submitted a
   [PR](https://github.com/tock/tock/pull/3318) for a license policy for Tock,
   but that's the first item on the agenda so I'll wait on it.
 * Alyssa: There's some unsoundness in `MapCell`. Discussed internally, I'll see
   if we can share it with the Tock Slack.
 * Johnathan: He sent [the PR](https://github.com/tock/tock/pull/3325) already,
   it's public now.
 * Hudson: That seems like a big problem, thanks.

# FlashController HIL (#3319)
 * Alexandru: The problem we have is in the flash HIL.
 * [Alexandru, in chat:
   https://github.com/tock/tock/issues/3319#issuecomment-1311203404]
 * Alexandru: The flash HIL in tock defines the associated type `Page` which
   needs to be `AsMut<[u8]> + Default`. In TickFs, Alistair assumed page sizes
   are 2KiB. His flash controller for TicKV uses our flash controller from Tock.
   The flash page doesn't have a fixed size, but his flash controller needs a
   size, so it can compute addresses and erase pages. Basically, there is no
   checking here.
 * [Alexandru, in chat:
   https://github.com/tock/tock/blob/0f7fe7d9355002e4e7065cc3fb940a0d121e8d21/capsules/src/tickv.rs#L74
   https://github.com/tock/tock/blob/0f7fe7d9355002e4e7065cc3fb940a0d121e8d21/kernel/src/hil/flash.rs
   https://github.com/tock/tock/blob/0f7fe7d9355002e4e7065cc3fb940a0d121e8d21/kernel/src/hil/flash.rs#L114]
 * Alexandru: The page size in TicKV needs to be the same size as the associated
   type `Page` from our flash controller. Our suggestion was to implement it as
   in the issue, but then we realized that `Page` could be anything that gives
   us a slice, which does not have a size at compile time.
 * Hudson: I'm not super familiar with TicKV, but it seems like the real fix is
   for TicKV to also be generic in the same way as the flash controller with
   regards to buffer sizing.
 * Alexandru: It needs to know how to erase pages, so it needs to know the size
   of the page. TicKV is generic on two sizes -- the write size, the minimal
   item we can write, and the erase size, the minimal item we can erase. It has
   a W and an E size. But the flash controller it Tock only allows reading one
   page at a time, writing one at a time, or erasing one at a time. This might
   be fine with controllers with a 2 KiB page, but the STM we use has a 16 KiB
   page. The TickFs implementation Alistair did was coded to a 2 KiB page. We
   can fix that, but I don't know how to connect the length to Tock's flash HIL,
   because it does not have a constant on the page size. That may be a problem
   because the flash HIL has erase page with a page number, but all the other
   functions -- read and write -- can take an arbitrary-length buffer.
 * Leon: That's an interesting observation given the associated type requires
   every implementer to have an `AsMut<[u8]>` implementation. It allows every
   consumer to potentially reference the flash page to a mutable slice. I'm
   wondering whether that's a wrong level of abstraction we're using here
   because when we want to have something dereferenceable to a mutable slice of
   bytes, we force the implementation to provide an in-memory view anyways. I'm
   wondering whether it would hurt any of our use cases to make the associated
   type an associated constant over the size of the flash page instead.
 * Alexandru: The idea would be that somebody would need to provide a buffer of
   that size when writing or reading?
 * Leon: It's a good question. I mean, potentially we could also add an
   associated constant that says the flash page has to be a particular size but
   use an unsized slice. It would cause some inconsistencies in the API. I'm not
   sure I have a good solution.
 * Alexandru: Reading and writing is fine, erasing is a problem because it needs
   to happen at a page level. Reading and writing is a strong limitation. We
   have 16 KiB pages but we never read or write 16 KiB.
 * Chris: That's exactly correct. Typically, reads and writes have fewer
   constraints. Erases have to be aligned and be a full sector size. It's not
   uncommon for flash implementations to even allow you to re-write the same
   word multiple times -- you can start with the word full of ones and clear a
   bit at the time, but that's typically dangerous. For some flash
   implementations, you can damage the flash if you re-write the same word too
   many times without an intervening erase. I think maybe we should look at how
   the flash control driver is modelling the flash and have the requirements
   flow outward from there.
 * Alexandru: I think TicKV is relying on the fact it can write multiple times.
   I think it needs to be able to write at least twice to mark
   garbage-collectable keys.
 * Alexandru: Right now I have a chip that has some 16 KiB pages and other pages
   of 128 KiB. We're currently limiting ourselves to the 16 KiB pages. With the
   HIL we need to read and write 16 KiB at a time. TicKV doesn't require it, the
   flash HIL today does.
 * Alyssa: Ti50 has its own flash HIL for that reason.
 * Alexandru: We could do that but we would like to have something consistent
   with upstream Tock.
 * Alyssa: I think that an associated constant for the flash size would be good,
   but maybe we could have attributes that can be set on the flash that enable
   or disable single-word writes or reads. Something that can describe the
   properties of the flash and its restrictions.
 * Alexandru: I'm not sure how. Something that could be read by an upper driver
   layer?
 * Alyssa: You could have some functions that are only enabled on flash chips
   with certain properties. You could define a "can write single word" property
   as an associated constant and only enable the write-single-word function if
   the constant is true.
 * [Hudson in chat: https://github.com/tock/tock/pull/2248
   https://github.com/tock/tock/pull/2993]
 * Alexandru: PR 2248 looks interesting but it's very old and not merged.
 * Hudson: The description of that PR sounds very similar to the issues that
   you're raising, so I think it's worth looking through the discussion on that
   PR and figuring out why it was not merged.
 * Alexandru: I would be happy to continue the discussion there and be able to
   merge it. We could write our own HIL but want to do something aligned with
   upstream Tock.
 * Alyssa: Tock basically needs to enable more configuration to be able to
   handle these diverse systems.
 * Hudson: I think that's what PR #2248 does.
 * Hudson: My takeaway is that we've known for a long time that the existing
   flash HIL is bad, people keep running into some of the same issues. #2248
   seems to be the minimal PR that gets close to addressing these. The reason it
   hasn't made it past the finish line is because everytime somebody puts in
   effort to rewrite the HIL, others come up with cases where the new HIL
   doesn't fully meet their needs, a debate happens, and people lose interest in
   pushing the PR forwards. If #2248 looks like it would largely address your
   concerns, Alex, it would be great if you were willing to take a look at the
   discussion on that PR and try to push it across the finish line. I get the
   feeling it had gotten close when Alistair was still working on it.
 * Alexandru: We'll do that, maybe continue the discussion next week. I can't
   push directly to that branch. I could open a pull request to his branch, or
   open a draft PR and Alistair can pull the changes.
 * Hudson: That would be fine. Alistair may start looking at it again.
 * Alexandru: Some boards don't have equally-sized pages. We have a board with a
   few pages of 16K, one page of 64K, and a few more of 128K. This is the reason
   tockloader doesn't work on STMs, as tockloader assumes a 4K page, which is
   not the case. Using tockloader on STMs breaks immediately.
 * Hudson: That's not related to this flash HIL.
 * Alexandru: The only way of flashing apps on these boards is re-bundling
   completely.
 * Hudson: Your observation is that this assumption we've made that flash pages
   are a fixed size is not only wrong here but also in tockloader.
 * Alexandru: I don't know other boards that well to be able to generalize. NRFs
   have equally-sized pages.
 * Hudson: Right, as does SAM4L. SAM4L and NRF are the main chips that inspired
   a lot of Tock HILs. It's not surprising we got that wrong.
 * Alexandru: I don't know if this is the exception or the common case.
 * Hudson: Do you feel like you have a path forward on this?
 * Alexandru: Yes. Looking at #2248 and coming up with feedback and hopefully we
   can push it forward into merging it.

# License and Copyright Policy (#3318)
 * Hudson: Pat is not here, but I still want to at least bring it up because it
   seems like something we don't want to leave outstanding. Amit has a PR
   implementing this policy. I want to get a feel where everyone stands on this,
   especially Johnathan and Amit.
 * Amit: I can get mine out of the way. I think I have agreed with most of what
   Johnathan suggested. Otherwise I don't have a particularly strong opinion on
   it. I would love to just see this get merged as soon as we feel comfortable.
   I think Johnathan is thinking a bit more like a lawyer and I don't know. What
   Pat wrote seems like it's in the spirit of being right to me, I don't have a
   strong opinion about the specific text, except for agreeing with Johnathan's
   points.
 * Johnathan: I left some comments there that haven't been addressed yet; I
   assume the status is we're waiting for Pat to address them. The biggest one
   is my comment about the format section. When we get really picky about
   license headers, that makes it difficult or impossible for us to use code
   from other projects. If their license headers don't follow our format or are
   in the middle of the file, then that doesn't meet our requirements. If I
   codify that into a CI tool that will fail in any PR that pulls that in. In
   one of the files, Amit pointed out we could move the license header and that
   is true but that is not true in every case. For the most part, we cannot mess
   with other peoples' headers. In that case, we could move the notice to the
   top of the file because we can converted it from C to Rust and we put the
   lines above the license header. It would be nice to be open enough that we
   can grab files from other Apache 2.0 OR MIT projects and use them in our
   codebase.
 * Amit: Broadly, if I am reading the tone of your comments correctly, it is
   that the first draft was basically overspecified...
 * Johnathan: Yes
 * Amit: and that we should say less if we don't absolutely need to be specific
   about it. I assume Pat's perspective was that he was like "oh I'm going to
   sit down and write this" and he just wrote something up and got carried away
   with details. Again I'm projecting -- I'm assuming he doesn't disagree with
   you. We should just prune it of details that are not necessary. We have
   nothing right now, so anything we do will be more specific than what we have.
   Can leave ourselves more freedom.
 * Johnathan: The other thing I'll state is I want to see this policy converge
   and get more agreement, then I can reach out to the OpenTitan legal
   committee. I'll see if they approve of the header from the perspective of the
   OpenTitan project contributing code to Tock and also using Tock.
 * Amit: Do you think we should do that before we hit merge?
 * Johnathan: Yes. But I do think it should be somewhat stabilized.
 * Hudson: Johnathan, do you have a concrete recommendation for what we should
   do with regards to specifying where a copyright should go or how it should be
   formatted? It seems your recommendation is to not specify where it should go
   or how it should be formatted, but we do need to specify that it exists. How
   are you going to write a tool that works around such a loos specification?
 * Johnathan: Yeah, I think the answer is we can specify things about projects
   made by Tock contributors. I don't want us to be too exact about all license
   headers in the code. We kind of need to divide -- if we pull code from
   another project, like the sip hash, I think we should be more open about what
   those license headers look like -- and we can have a tool that only checks
   for the Tock project's license header.
 * Alyssa: Why should we be more open instead of reformatting the copyright when
   we do the port.
 * Johnathan: Because we have a limited ability to reformat a copyright. In many
   cases we just can't touch it and we have to leave it in place.
 * Alyssa: An Apache/MIT one? Are there any cases where you can't put at the top
   "hey this is licensed under Apache/MIT and here are the original copyright
   declarations"?
 * Johnathan: That's a question for a lawyer not me.
 * Leon: There might still be a case where we import software that is e.g.
   released to the public domain, or just Apache or just MIT, which should be
   fine given our terms. I think it would make the most sense is for us to keep
   the description in the TRD vague but have a tool that may be overly picky,
   and we can maintain and extend it over time.
 * Johnathan: I don't think it's necessarily correct that we can take code that
   is only licensed for one of the licenses out of another project and put in
   ours and distribute it under both licenses.
 * Leon: I should've prefaced this with the disclaimer that it gets fuzzy and
   requires a separate discussion. I'm saying it's not always as easy as taking
   code from another project and reformatting the header because it's the same
   license.
 * Alyssa: For the limited cases that we do have, if we see that it says MIT
   license and Apache 2.0 license, I don't think there's any harm if we don't
   change what's there, we can always add a license declaration to the top as
   long as it matches the license that was listed previously. It's only adding
   the same information, it's not changing the semantics if it matches.
 * Amit: I agree that we can do that, and we probably should when we can. Do we
   need to specify that in the TRD? Suppose we're wrong, or it causes some
   conflict. Do we really care about there being a file with a license in a
   different place.
 * Johnathan: Let me give a concrete example. Lines [100 to
   10](https://github.com/tock/tock/blob/e2f9b0bf902e80d895b750bd29080a7438f566b7/doc/reference/trd-legal.md?plain=1#L100)
   state license information MUST come before copyright information. If you look
   at the OpenTitan project, we put our copyright line above the license
   information. And so that alone, if Tock were to try and take a file intact
   from the OpenTitan project -- which it'd already have to remain under the
   Apache 2.0 license -- but say we wanted to do that, the Tock project can't
   then change the license header on the OpenTitan file.
 * Alyssa: I think this TRD is overspecified. I think the only thing we should
   absolutely require is the SPDX license identifier because that is designed to
   be programmatic.
 * Johnathan: I agree with SPDX. I think we'll have to see what the OpenTitan
   legal committee if it's okay to omit the copyright statement.
 * Alyssa: As long as have some knowledge of the source, it's always fine to
   add.
 * Chris: Would it be acceptable to allow files in that have the SPDX header and
   maintain a list of exceptions? So, if you want to import something that
   doesn't have that and for whatever reason you can't change the sources to
   include it we can have a file that says "these files are an exception to the
   rule".
 * Johnathan: I was anticipating that when I build the CI tool I'd have to
   develop that.
 * Leon: Definitely seems feasible, especially for putting in external
   dependencies. We may want to add a check to make sure that the source doesn't
   change without noticing that and change the license terms, so something like
   including the hash may be a good idea.
 * Alyssa: That sounds like it is pretty fragile, would change very option, I
   guess by intention.
 * Chris: There's a tool within Google that does something like this. It tracks
   the licenses of third-party opensource libraries. It's more policy-based.
   It's not "did it change", but "whatever the license is, is it acceptable". If
   you build something that uses these opensource third-party libraries, you can
   run a check over your dependencies and make sure that everything you're
   pulling in fits some constraint. I don't think we need to build something
   quite that involved but if we have a list of acceptable licenses and make
   sure everything we have falls in that last, I would suggest it doesn't matter
   if it changes. If we have something under one acceptable license and it
   changes to a different acceptable licenes, that's fine, but if it changes to
   an unrecognized license then there's an issue.
 * Alyssa: I think that mechanism should be SPDX.
 * Chris: I agree
 * Johnathan: There's a distinction between tools that check that your
   dependencies have compatible license -- I think `cargo audit` might do this.
   There are a different set of tools that make sure your project's license
   headers match your project's standards. When Google releases an open source
   project and make it public for the first time, we make sure our own license
   headers are correct using a tool. I think Chris was talking about one type of
   tool and I'm talking about the other type. I'm talking about one that just
   checks the format of our license headers rather than checking the licenses of
   our dependencies.
 * Leon: Presumably, given our current policy on external dependencies, we won't
   have a flurry of external dependencies to track. It seems fine for that to be
   a manual process.
 * Alyssa: Why couldn't we edit or require all third-party dependencies to
   include an SPDX? It's a really reasonable request.
 * Leon: Because fundamentally we might want to pull the dependencies into a git
   subtree or submodule, and it creates an insane overhead to maintain a
   sightly-deviating version of the dependency.
 * Alyssa: I'm thinking of the current third-party dependency policy.
 * Johnathan: Well, consider `libtock-rs` which is not under the Tock kernel's
   third-party dependency policy and pulls in a lot more external crates.
 * Alyssa: In that case, either SPDX or cargo license metadata.
 * Amit: Can I step back, I got a little bit lost. Are we discussing things that
   should be included in the TRD as, like, MUST; things that should be included
   in the TRD as SHOULD; or things that shouldn't necessarily be included in the
   TRD but should do as a matter of practice.
 * Johnathan: I think it is a separate topic.
 * Amit: Okay, so for the TRD how about we either catch Pat on Slack and he has
   some cycles or I will try and take a pass later today to basically tone down
   the requirements.
 * Amit: How about we try to get thumbs-up approvals from people, then we can
   get feedback from external folks, then we can go from there.
 * Johnathan: Sounds good to me.
 * Hudson: That sounds good.
 * Alexandru: Considering there's an agreement that copyright information could
   be in the file, is there any way we can merge the CAN PR in the meantime?
 * Hudson: My feelings is it probably doesn't make sense to merge anything until
   we actually finalize this.
 * Amit: In that case, I think the ball is in my court right now, but can we
   kind of step on trying to merge this. Johnathan, how long do you think you'll
   need to get feedback from people once we approve?
 * Johnathan: I don't know for sure. I think under a week, but I can't make a
   promise.
 * Alexandru: That's fine, then I have no problem.
 * Amit: I'll try to get it ready for approval soon, and then lets try and get
   feedback and hand it over to Johnathan relatively quickly.
 * Hudson: I think this is something we should try to get through as quickly as
   possible.

# RasPi Pico USB
 * Alexandru: We have a working USB stack for the Raspberry Pi Pico. I think the
   PR is mostly done. Can someone with more experience look at it and maybe we
   can merge it? It's critical for the Pico because right now the rest of my
   Pico cannot be used without an exterier seriel converter. That would let us
   use it in the classroom.
 * Amit: I'm interested. What's the PR?
 * Alexandru: https://github.com/tock/tock/pull/3310
 * Alexandru: We just duplicated whatever was on the NRF. It works in and out --
   you can read and write data, and the process console seems to work.
 * Amit: This seems limited to boards and chips, seems like it should be easy to
   merge fairly liberally in my opinion.
 * Alexandru: It should be limited to Pico and the RP40 chip. As soon as this
   gets merged we will add it to some other boards with the same chip.
 * Amit: Great
 * Hudson: One thing I noticed when looking at that PR is you have this empty
   implementation for reset bootloader enter function
 * Alexandru: I have no idea how to do that yet, we're still looking into it.
 * Amit: Is that similar on the NRF?
 * Alexandru: No, the NRF reset. I was able to load apps on the clue. On the
   RasPi we have an idea but it's still under exploration. We'll probably follow
   up w/ another PR.
 * Hudson: The problem is not that you can't reset the chip, it's that you need
   a way to call the function?
 * Alexandru: No, the problem is we need to keep something in a register the
   bootloader reads. I have a pretty good idea how to reset the Pi but I don't
   have an idea how to keep data in a register while it restarts.
 * Amit: Yeah, that's exactly right. The whole implementation is a hack for the
   bootloader.
 * Alexandru: It has to do something. We can delete that function, it is still a
   TODO.
 * Hudson: I think it is fine, just something I noticed. Overall this looks
   pretty inoffensive and is confined to the board and chips so.
 * Alexandru: The parts I am particularly interested in are in the USB stack.
   The NRF have some comments with TODOs. We do this on DMA, but to do this is
   not something we can do, and these are the parts where another set of eyes
   that has done a USB stack before would be super useful.
 * Hudson: I think I'm going to go ahead and stamp this.
