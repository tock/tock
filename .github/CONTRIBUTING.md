# Contributing to Tock

Thank you for your interest in contributing to Tock! There are several ways to
contribute, and we appreciate all of them.

* [What Goes Where?](#what-goes-where)
* [Feature Requests](#feature-requests)
* [Bug Reports](#bug-reports)
* [Pull Requests](#pull-requests)
* [Issue Triage](#issue-triage)

If you have questions, please make a post on the [mailing list][listserv] or
join us on [Slack][slack].

As a reminder, all contributors are expected to follow the Rust [Code of
Conduct][coc].

[slack]: https://join.slack.com/t/tockos/shared_invite/enQtNDE5ODQyNDU4NTE1LTg4YzE1MTkwYzI0YjhjNjA0YWExOGY2ZGYwNjQ2YmFiZjdhOTdlMzY0YTBiYTA2YTRlYzMyZTI1MDdmMTgwMzc
[listserv]: https://groups.google.com/forum/#!forum/tock-dev
[coc]: https://www.rust-lang.org/conduct.html

## What Goes Where?

This repository contains the core Tock kernel and the officially supported
hardware platforms. Drivers useful for these platforms and improvements to the
core kernel live here.

If you are porting Tock to a different hardware platform or building an
application for an existing platform, that code should go in a [separate
repository][out-of-tree]. We still encourage you to join the [mailing
list][listserv] and [Slack][slack] and ask questions there. Of course, if
building your application or port requires in changes in this code base, we
encourage you to contribute them here.

## Feature Requests

To request a change to the way that Tock works, please post an RFC to the
[mailing list][listserv] rather than an issue in this repository.

## Bug Reports

While Tock is designed to be a safe OS, it is not bug free, and we can't fix
what we don't know, so please report liberally. If you're not sure if something
is a bug or not, feel free to file a bug anyway.

If you have the chance, before reporting a bug, please [search existing
issues](https://github.com/tock/tock/search?q=&type=Issues),
as it's possible that someone else has already reported your error. This doesn't
always work, and sometimes it's hard to know what to search for, so consider this
extra credit. We won't mind if you accidentally file a duplicate report.

Opening an issue is as easy as following [this
link](https://github.com/tock/tock/issues/new) and filling out the
fields.  Here's a template that you can use to file a bug, though it's not
necessary to use it exactly:

    <short summary of the bug>

    I tried this code:

    <code sample that causes the bug>

    I expected to see this happen: <explanation>

    Instead, this happened: <explanation>

    ## Meta

    Hardware platform:

All three components are important: what you did, what you expected, what
happened instead. Please include the hardware platform you tested on as well as
any steps required to reproduce the error.

If an application faults or the kernel panics, Tock should output a detailed
error message. Including this error message in its entirety can be very helpful.

## Pull Requests

### Step 1: Fork

Fork the project [on GitHub](https://github.com/tock/tock) and check
out your copy locally.

```text
$ git clone git@github.com:username/tock.git
$ cd tock
$ git remote add upstream git://github.com/tock/tock.git
```

### Step 2: Branch

Create a feature branch and start hacking:

```text
$ git checkout -b my-branch -t origin/master
```

You should always start new feature branches from `upstream/master`. Try not to
stack up changes on local branches. We try to resolve pull requests in a timely
manner so you shouldn't have too many outstanding changes in flight.

### Step 3: Commit

Make sure git knows your name and email address:

```text
$ git config --global user.name "J. Random User"
$ git config --global user.email "j.random.user@example.com"
```

Add and commit:

```text
$ git add my/changed/files
$ git commit
```

Writing good commit logs is important. A commit log should describe what
changed and why. Follow these guidelines when writing one:

1. The first line should be 50 characters or less and contain a short
   description of the change. All words in the description should be in
   lowercase with the exception of proper nouns, acronyms, and the ones that
   refer to code, like function/variable names. The description can
   be prefixed with the name of the changed platform, chip, driver or subsystem and should start with an
   imperative verb. Example: "sam4l: use DMA for USART transfers"
2. Keep the second line blank.
3. Wrap all other lines at 72 columns.
4. Any quoted text or code should be indented four spaces

A good commit log can look something like this:

```txt
subsystem: explain the commit in one line

Body of commit message is a few lines of text, explaining things
in more detail, possibly giving some background about the issue
being fixed, etc.

If you quote a discussion or some code it should be intended:

    Should we have an example of indentation? Yes I think so.

The body of the commit message can be several paragraphs, and
please do proper word-wrap and keep columns shorter than about
72 characters or so. That way, `git log` will show things
nicely even when it is indented.
```

The header line should be meaningful; it is what other people see when they
run `git shortlog` or `git log --oneline`.

Check the output of `git log --oneline files_that_you_changed` to find out
what subsystem (or subsystems) your changes touch.

### Step 4: Rebase

Use `git rebase` (not `git merge`) to sync your work from time to time.

```text
$ git fetch upstream
$ git rebase upstream/master
```

### Step 5: Push

```text
$ git push origin my-branch
```

Go to https://github.com/<yourusername>/tock and select your branch.
Click the 'Pull Request' button and fill out the form.

### Step 6: Discuss and update

You will probably get feedback or requests for changes to your Pull Request.
This is a big part of the submission process so don't be disheartened!

Tock has a [standardized process for reviewing pull requests](../doc/CodeReview.md).
In general the relevant team members will review a pull request, ask for any
changes, and get support from the larger team before merging a PR. If you are
curious about the specifics, feel free to read through the specific process.

To make changes to an existing Pull Request, make the changes to your branch.
When you push that branch to your fork, GitHub will automatically update the
Pull Request.

You can push more commits to your branch:

```text
$ git add my/changed/files
$ git commit
$ git push origin my-branch
```

Feel free to post a comment in the Pull Request to ping reviewers if you are
awaiting an answer on something.

Before its ready to merge, your Pull Request should contain a minimal number of
commits (see notes about [rewriting-history](#rewriting-history)).

### Step 7: Style

Mainline Tock uses [rustfmt](https://github.com/rust-lang-nursery/rustfmt) to
format code, using the default style options. As rustfmt is included as a component
in nightly Rust releases, the version of rustfmt that Tock uses is directly tied
to the nightly version Tock is currently using. The build system will
automatically use (and install if needed) the correct rustfmt version when you
invoke `make format`.

The target `make formatall` in the root will automatically run all style checks
and make any required changes. PRs must pass the formatting checks before landing.

### Step 8: Landing

In order to land, a Pull Request needs to be reviewed and
[approved](#getting-approvals-for-your-pull-request) by at least one person with
commit access to the Tock repository and pass the continuous integration tests.
After that, as long as there are no objections, the Pull Request can be merged.

We use the bors-ng bot to merge PRs. In short, when someone replies `bors r+`,
your PR has been scheduled for final tests and will be automatically merged. If
a maintainer replies `bors delegate+`, then you have been granted the authority
to merge your own PR (usually this will happen if there are some trivial
changes required). For more on bors,
[see the bors documentation](https://bors.tech/documentation/).


## Issue Triage

Sometimes, an issue will stay open, even though the bug has been fixed. And
sometimes, the original bug may go stale because something has changed in the
meantime.

It can be helpful to go through older bug reports and make sure that they are
still valid. Load up an older issue, double check that it's still true, and
leave a comment letting us know if it is or is not. The [least recently
updated sort][lru] is good for finding issues like this.

[lru]: https://github.com/tock/tock/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-asc

## Notes

### Rewriting History

Once the reviewer approves your Pull Request, they might ask you to clean up the
commits. There are a lot of reasons for this. If you have a lot of fixup
commits, and you merge all of them directly, the git history will be bloated.
Or, if your recent commit fixes your previous commit in the same PR, then you
could simply rebase it.

To achieve this, you can use the `git rebase -i` command.

Run `git rebase -i`, which will bring up your default text editor with a
content like:

```
pick 7de252c X
pick 02e5bd1 Y

# Rebase 170afb6..02e5bd1 onto 170afb6 (2 command(s))
#
# Commands:
# p, pick = use commit
# r, reword = use commit, but edit the commit message
# e, edit = use commit, but stop for amending
# s, squash = use commit, but meld into previous commit
# f, fixup = like "squash", but discard this commit's log message
# x, exec = run command (the rest of the line) using shell
# d, drop = remove commit
#
# These lines can be re-ordered; they are executed from top to bottom.
#
# If you remove a line here THAT COMMIT WILL BE LOST.
#
# However, if you remove everything, the rebase will be aborted.
#
# Note that empty commits are commented out
```

Keep the first commit as pick, and change any commits you'd like to collapse in
the previous commit with either `squash` (or s for short) or `fixup` (or f for
short).

```
pick 7de252c sam4l: use DMA for USART transfers
squash 02e5bd1 run rustfmt

# Rebase 170afb6..02e5bd1 onto 170afb6 (2 command(s))
...
```

Now save and quit the text editor, the rebase will run until the end.

After the rebase is finished, the editor will pop-up again, now you can write
the commit message for the new commit.

`git push -f` to push the squashed commit to GitHub (and update the PR).

