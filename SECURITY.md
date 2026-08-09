# Security policy

This project is a register of material parameters. Most of what can go wrong
with it is a wrong number rather than a broken program, so this file has to
cover both, and the two are reported differently.

## Reporting privately

Use GitHub's private vulnerability reporting on this repository. It is on:

    gh api repos/iderex/stoffbuch/private-vulnerability-reporting
    {"enabled":true}

read on 2026-08-08. From the Security tab of this repository, the report opens a
private advisory that only the maintainer can read. Nothing about it is public
until it is published, and a report that turns out to be nothing is closed
without ever being published.

Do not open a public issue for anything listed under what is in scope below, and
do not open one for a value you believe was made up. Everything else, including
a value you believe is simply wrong, belongs on the public tracker, because a
correction argued in the open is a better correction.

No address other than the GitHub route is given here, because a second channel
that nobody watches is worse than one.

## What a reporter can expect

An acknowledgement that the report was read. Then one of two answers: what is
being changed and where, or why nothing is being changed. You will be told which
of the two it is rather than left to infer it from silence.

No response time is promised. A deadline stated here and not held would be worth
less than this sentence, so none is stated. A report sitting without an answer
means nobody has picked it up yet; it does not mean it was judged and dismissed.

If a fix lands, the advisory is published with the report credited, unless the
reporter asks not to be.

## What is in scope

A failure in the code that reads the register. A crash, a hang, memory that
grows without bound, or a stack that runs out, reached by a record file, a source
file or a tabulated block that a reader was pointed at. Anyone can propose a
record here, so a record file is untrusted input to every consumer downstream,
and that is the reason this class matters more than the size of the code
suggests.

Anything a record can do beyond carrying its value. A record that makes a reader
write outside where it was told to write, read a path it was not given, reach the
network, run anything, or change what a different record means. The register is
data and it is meant to stay data. This includes any expression a record carries
for a value that was computed rather than transcribed: an expression is evaluated
by the library, and an evaluator that can be made to do more than arithmetic is a
defect of this kind.

The supply chain of the released artefacts. What goes into a build, what a
release contains, and anything that would let a released artefact differ from the
tree it names as its source. A register that changed between the tree and the
download is indistinguishable from a wrong register, and it is the failure the
citation form exists to make impossible.

Anything in the register that reaches a reader's machine. The project's position
on what leaves a host is in
[docs/decisions/data-protection-and-what-leaves-the-host.md](docs/decisions/data-protection-and-what-leaves-the-host.md).
A way to make a run send something anywhere is in scope here even where the
sending looks harmless.

## A value suspected of being deliberately wrong

This is the failure with no equivalent in ordinary software, and it is the one
this register would be used to launder. A plausible number with a plausible
citation, contributed on purpose, propagates into published results and is very
hard to find afterwards. It is closer to research misconduct than to a
vulnerability, and it is reported through the same private route as everything
else above.

What happens is fixed here rather than decided under pressure:

Nothing is deleted. Not the row, not the version, not the source entry, not the
contribution. A row that should never have existed gains a final version marked
withdrawn, and a row whose number was wrong gains a new version carrying the
correction. The rules for both, including what a correction has to say and who
can still resolve the old citation, are in
[docs/decisions/versioning-and-citation.md](docs/decisions/versioning-and-citation.md)
and are not restated here.

The correction is recorded with its reason. A reader who cited the old number
follows their citation, finds the version they cited, and is told what replaced
it and why. That is the whole point of not deleting: a reader who used a
fabricated value learns it from the register rather than from nowhere.

Every other row from the same hand is re-checked against its source, not only the
one that was reported. A single deliberate error is evidence about a body of
work, and re-reading one row and leaving the rest is the cheap answer that would
leave the register worse than it looks.

The reason travels in the row rather than only in the repository's history. A
consumer who received files and has no clone still reads why the value changed,
because that is what the correction block on the new version carries.

Whether anything is said to a contributor's employer, funder or publisher is not
decided in this file and is not promised either way.

Suspicion is not a finding. A value that is merely surprising, or that disagrees
with another publication, is what the disagreement machinery is for and is
handled as ordinary work in the open.

## Which releases receive corrections

The newest release. There is no older line that keeps receiving fixes, and this
file will say so until there is.

A published release is never altered. A citation names a release and has to
resolve to the same bytes years later, so a correction appears in the next
release and never in one that already exists. That follows from the immutability
rule in
[docs/decisions/versioning-and-citation.md](docs/decisions/versioning-and-citation.md)
rather than being chosen here.

There are no releases yet:

    gh api repos/iderex/stoffbuch/releases --jq 'length'
    0
    gh api repos/iderex/stoffbuch/tags --jq 'length'
    0

read on 2026-08-08. So the paragraph above is a policy that nothing has yet been
measured against. The release route and how a version is numbered are settled in
issue #78, and if that produces more than one supported line, this section is
changed with it rather than left to drift.

## What this file does not cover

It does not list the checks this repository runs. Those are printed by the gate
command:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

This sentence said the command did not exist until it landed.

It does not decide the terms a contribution is accepted under. The repository is
licensed under AGPL-3.0, and the text is in [LICENSE](LICENSE); this paragraph
said there was no licence at all until that file landed. What the rows
themselves carry, and whether a change from outside is taken at all, is the rest
of the first entry of issue #1 and is the maintainer's to settle. Reporting a
problem does not depend on it; contributing a fix may.
