# Data protection, and what leaves the host

At first reading this project has no personal data in it. It computes on numbers
taken from published literature, and a published measurement is nobody's personal
data. That reading is too quick, and the places it is wrong are the places a
promise made carelessly gets broken later.

The position, stated before the reasoning: nothing an operator does with these
tools leaves their machine unless the operator deliberately switches on a
federation feature that names exactly what it would send. No telemetry, no crash
reporting, no update check, no licence check, no font or schema fetched at run
time, no remote lookup that happens without being asked. Every read of an outside
source is an explicit action with an explicit command.

That sentence carries no qualifier because the code needs none. Where a later
version does need one, the qualifier goes in and this record changes with it, and
the direction that is refused is the other one: a sentence saying nothing leaves
the host never survives a change that makes something leave it. The rule behind
the whole record is that a sentence in the documentation is written from what the
code does and not from what it intends, so a statement is a report and never a
promise about future work.

## The four routes personal data arrives by

Personal data can reach this project by four routes, and none of them is the
register itself.

### A curator's name and contact address

These are personal data, they are in the version control history whatever else
is decided, and they may or may not also be in the rows. The history is not
something this project can make anonymous without giving up the thing history is
for, so the answer there is disclosure rather than avoidance: a contributor is
told, before they contribute, that their name and the address they commit with
become a permanent public record. Whether the curator is also recorded inside a
row is reserved for the maintainer and is not decided here. What is decided is
that the answer changes the size of this route and not its treatment: a curator
field in a row is personal data published deliberately, it is disclosed in the
same place as the history, and it is subject to the rule below about what a
released row can no longer do.

The rule that matters if a curator field ever exists: a released row is immutable
and cited by version, so a name written into a row cannot be taken out of the
released register afterwards. That is a property of the versioning decision rather
than of this one, and it is the reason a curator field is a decision worth making
deliberately rather than by default. `versioning-and-citation.md` is where the
immutability comes from.

### The operator's own input files

An operator runs these tools over files describing a device, a process or a
stack, and such a file is somebody's unpublished work even when it names no
person. It is frequently the most sensitive thing on the machine, and it is not
this project's to hold, index, cache outside the run, or copy anywhere. What the
tools do with an input file is read it, and what they write is what the operator
asked for, where the operator asked for it. Nothing is written to a shared
location, a user-level cache directory or a temporary file that outlives the run
without being named on the command that created it.

### Federation between registers

An institution running its own register with its own internal measurements in
it, publishing some subset upstream, sends whatever that institution put in its
rows. This project does not control that content and cannot promise anything
about it, so the promise it makes instead is about the mechanism: the mechanism
is off unless switched on, and it discloses what it would send before it sends
it. The section below says what such a feature would have to carry before it
could exist. Whether it may exist at all is reserved for the maintainer.

### A file path in diagnostic output

On most systems a path under the user's home directory contains their account
name, and an account name is frequently a real name. A diagnostic that quotes
the path it failed on therefore contains personal data, and a diagnostic is
exactly the thing an operator is asked to paste into a public issue. This is the
route that leaks by accident rather than by design, and it is the one this
record spends the most words on, because it is the only one of the four where
the leak happens without anybody deciding anything.

## What diagnostic output may contain

Two audiences, two forms, and the difference is decided by the code rather than
by the operator remembering.

What the tools write for the operator to read on their own machine may contain
any path it needs to, in full. An operator debugging their own run needs to know
which file failed, and reducing that to something unrecognisable would be a worse
outcome than the one this rule guards against.

What the tools write for the operator to share may not contain an absolute path.
That covers anything the project asks an operator to send, attach or paste,
including a run manifest, a report intended to accompany a bug report, and any
future diagnostic bundle. In that form a path is reduced to one of two things: a
path relative to the register root, or the base name of the file. Neither carries
a home directory and neither carries an account name.

The environment is not sanitised, it is not collected. A shareable diagnostic
carries no environment variables, no user name, no host name, no command line as
the operator typed it, and no directory listing. What it carries is the version of
the register, the version of the tool, what was asked for, and what refused it.

Two things follow that are worth saying because they are the ordinary way this
gets broken. A path that was assembled at run time is still an absolute path, so
the rule is about the bytes in the output rather than about which literal appeared
in the source. And a message that embeds an error from a lower layer inherits
whatever that layer put in it, which for a file system error is usually the full
path, so the reduction happens where the message is composed for sharing rather
than being hoped for at the site of each error.

## What a federation feature would have to disclose before it could exist

This is on the record before anybody builds one, so that the requirement is
something a later change is measured against rather than argued about at the time.
It is not a decision that such a feature may exist.

- It is off in every default configuration, and off means the code path that
  would send is not reachable without the operator having written something down.
  A prompt that defaults to yes is not off.
- It names every field it would send, by name, in a document in the tree, and
  the document is the specification rather than a description written afterwards.
- It can produce the exact bytes it would send, for the operator to read, through
  a command that sends nothing. An operator who cannot see the payload cannot
  consent to it.
- It names the recipient, and the recipient is a specific place rather than a
  configurable one with a default filled in.
- It sends nothing that its disclosure does not name, and that is refused by a
  check rather than reviewed by a person. A field reaching the sending path that
  the disclosure does not list is a failure of the build and not a note in a
  changelog.
- It records what was sent and when, on the operator's machine, in a form the
  operator can read.
- Its disclosure is written from what the code sends. A disclosure that describes
  an intention is the failure mode this whole record exists against, and it is
  worse than no disclosure, because it is believed.

The cost of the list is that it makes such a feature expensive to build, and that
is the intended effect rather than a side effect. The cost of not having the list
is that the feature arrives as a small convenient patch and the disclosure is
written from the patch author's intentions.

## What refuses any of this today

Nothing.

There is no check in this tree that would catch a new outbound network call, and
no check that would catch an absolute path reaching a shareable output. The
guards that exist read the workflow files and the tracked text and say nothing
about either subject:

    git ls-files .github/workflows/
    git grep -l -iE 'telemetry|outbound|egress|network' -- .github/ ; echo "exit=$?"
    exit=1

So every sentence above is a rule with nothing behind it, and a change adding a
network call would pass every route in this repository. That is the state, and it
is written here rather than left for a reader to infer from a green run.

Two checks are owed, and naming them is not the same as having them. One reads
the dependency graph and the source for the surfaces a network call goes through,
and refuses one that no document names. The other runs the shareable output of a
failing run through a pattern that matches an absolute path, in a fixture whose
temporary directory is known, and refuses a match. The second is the cheaper and
the more likely to bite, because it catches the route that leaks by accident.
The suite these would sit in requires that a test needs no network at all, which
is `headless-and-unprivileged.md`, and that rule is the reason a network call
introduced here would fail a run somewhere before it failed a check. Somewhere is
not a mechanism, and this paragraph is not claiming it is one.

## Why a project with no personal data in it says any of this

The audience includes institutions in Europe whose staff have to answer their own
organisation's question about what a tool sends before they may install it. For
that reader, a tool that processes no personal data still owes a statement,
because the operator is the one who has to give the answer, and a project that is
silent is read as a risk rather than as a project with nothing to declare. A page
they can point at is the deliverable, and it is worth more to them than the
absence it describes.

The second reason is internal. Writing the position down before the code exists
means the first change that would break it arrives as an argument against a
written rule rather than as a small patch nobody weighed.

## Where the reader-facing version lives

The operator-facing statement is short and is in the reader's language rather than
this one, and it points here rather than repeating the reasoning. This record is
the home: the four routes, the diagnostic rule and the federation requirements are
written once, here, and the reader-facing text carries the consequence and the
pointer. When the documentation set exists, that text moves into it and keeps the
same shape.

## What this record does not decide

It does not decide whether anything may ever leave the host by federation, nor
whether the curator of a row is recorded in the row. Both are reserved for the
maintainer, and this record is written so that either answer leaves it standing.

It does not decide the licence, which is what would let a contribution be accepted
on terms either side can point at, and which is where the disclosure to a
contributor about their name in the history has to appear. It does not decide the
schema, so it does not decide the name or the shape of a curator field if one is
ever added. It does not decide the run manifest, only that whatever the manifest
is, it is a shareable output and the path rule above applies to it. It does not
decide the harness for anything that genuinely needs a network, nor which of the
two owed checks is built first, nor whether they are one check or two.
