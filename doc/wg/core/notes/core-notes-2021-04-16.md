# Tock Core Notes 2021-04-16

## Attending
  - Leon Schuermann
  - Hudson Ayers
  - Amit Levy
  - Vadim Sukhomlinov
  - Jonathan Van Why
  - Philip Levis
  - Pat Pannuto
  - Branden Ghena

## Updates 

 - Phil: Been looking into TRDs, finalizing them, updating them.
 - Amit: I'm been looking into the nano33ble sense. There was a bug in one
of the sensors, I started looking at libtock-c. First of all, the whole
thing is wonderful, the build and program process that Brad did is amazing,
but there are some warts, transitioning to 2.0 interface, introducing
some bugs.

## Updates 

- Amit: Agenda items today are things remaining before 2.0. Phil had pointed out 5 things. Which are to discuss?
- Phil: Mostly just updates on where they are, not discussion/argument.
- Leon: Update. Hudson and I met to figure out how to do this so capsules can be non-virtualized yet store state within the main capsule struct.  In the new approach we can't. Brad also came up with a solution which we did eventually go with. We're reasonably confident on how to transition the capsules to the proper interface. We just need to come up with the list and figure out who will do which. Then we can make all those changes and get the branch to be mergeable.
- Brad: Is there a tracking issue? It would be good. It can be hard to find them.
- Leon: Sure, I will put checkboxes in just the original PR.
- Phil: Can you explain Brad's approach and what we are doing?
- Leon: We are putting the process' own resources in the grant regions.  We use the enter to figure out if a process is still alive. With non-virtualized processes, the first process to call it will get ownership. But we want, if that process dies, for someone to be able to regain control. For example, we can do this on AppId. We don't want to expose this to processes directly. But this way, if the process is truly dead then we can allow someone else to claim the capsule.
- Leon: The current PR to resolve Guillaume's issue is 2642. That requires that we always have the upcall and appslices within a grant region. This allows us to create process-bound, driver-bound, and identifier-bound upcalls. 
- Amit: So 2642 completely resolves 1905...
- Brad: Only resolves the subscribe, not the allow.
- Amit: So without it, do we at least resolve the issue, if I get something back, if I get an allowed buffer back from a capsule, I know that it doesn't have acccess to it anymore. Or if I get all references?
- Leon: For an allowed buffer, this should already be the case, because an AppSlice isn't copyable. But we do need to check aliasing, for Rust to be sound. So we will want to use a separate mechanism, to check, because of the lifetime.
- Leon: We are going to want to use many of the mechanisms for subscribe in 2642 also for AppSlices for allow.
- Amit: That's if we resolve the harder problem, of enforcing that a capsule has to release these. There's two levels of mistrust. One level, which I believe we already do. A capsule can tell you that they are no longer using an appslice, but they still might.
- Leon: Already resolved.
- Amit: The same could be for subscribe, but that's not currently resolved.  Under that model, a capsule can just not release something. It can't lie about it, but it can not do it. I think that's what 2642 resolves.
- Leon: A capsule can always refuse an operation. But there's an additional issue, that only lives in allow, which is because they are buffers, which is what if you have two mutable references to the same area. We have to make sure that userspace doesn't allow two overlapping areas. Which requires that we keep track of allows in a table. I think this is a big
- Amit: Maybe I'm bringing up something that's already been exlpained.  For AppSlices, can we not view them as a slice of Cells.
- Leon: Yes, so this resolves one issue. We don't view them as being persistent mutable references that persist across system calls. We could store them as pointers, and make them into mutable references during a system call. So this solves the user/kernel problem, but not if userspace tries passing two appslices which overlap.
- Jonathan: If an app shares overlapping buffers with the kernel, and the kernel doesn't have any way to tell, does that create a memmory safety issue in the behavior?
- Amit: Yes. Because it's undefined behavior. But if we have it as a volatile Cells, then I think this handles it.
- Amit: In practice, it probably doesn't make a practice in terms of what gets compiled right now. But in principle, by promising no mutable references, the compiler has assurances about what it can do.
- Jonathan: Yeah, it's the reference types that make the difference, but what's actually behind them.
- Leon: In many cases we expect we can get a regular mutable reference of an appslice, since this is what we use internally.
- Amit: So what is blocking for 2.0? Is 2642 or some weaker, that solves the appslice sharing issue, what's blocking for 2.0?
- Leon: I think by our discussions, 2642 is a viable option. It might be ugly, but it's the best option we have. It makes a superset of the guarantees for what we'd like for 2.0. For callbacks of course, appslice is another issue.
- Hudson: My understanding that 2642, or some other version that provides the same guarantees, is a blocker for 2.0.
- Amit: Is it fair to say that the... this is a tough one, it'a a design decision. Very different worlds that we are describing. It would be great to resolve the safety issue before 2.0, ideally to not regret whatever decision we chose.
- Leon: This is only callbacks currently. I think 2642 is a safe bet to include. There is not current apparent safety issue. The only thing we would be introducing additional constraints on capsules interact with processes. It's a separate beast from AppSlices. There aren't really any other proposals out there.
- Amit: OK, so next up is the system call TRD in 2431.
- Amit: OK, 2508 and 2511. 2508 is merged.
- Hudson: Yeah, 2511 is just blocked on libtock-c. Brad's PR for libtock-c also fixes the final bullet point.
- Amit: 2511 is blocked because libtock-c just won't work?
- Hudson: Yes. And book and stuff say use master.
- Amit: OK, then we have renaming AppId to ProcessID, 2184.
- Brad: Blocked on 2510. Blocked on process re-org, which I finally think is time. And whatever pull request. It's just a hard one, it touches every file.
- Amit: It's a straightforward rename?
- Brad: Mostly.
- Leon: There's also documentation.
- Brad: Have to settle for best effort, fix as we go.
- Amit: As soon as a window opens up, do a best effort, renames everything, then we can do documentation piecemeal since that won't conflict.
- Brad: Business as usual, until I can fix the PR. It shouldn't be too problematic.
- Leon: We should just get it across. If everything breaks, that how it works.
- Amit: Brad's libtock-c PR for the return types in libtock-c. Is it blocked on me? OK, it'll happen sometime between this hour and 20:32.  I would love to us this time to do this. 
- Hudson: Through the agenda items!
- Amit: Going once, twice. I will do the libtock-c PR right now. 
