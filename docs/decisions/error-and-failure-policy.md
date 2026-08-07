# The error and failure policy

A register built up slowly is incomplete at every moment of its life. The tools
have to behave sensibly against an incomplete register without teaching anybody
to ignore their output, and that is a policy rather than a property of any
individual check. Writing it before the first check exists is cheaper than
retrofitting it onto twenty of them.

## Three classes

### Refusal

Something that makes the register wrong, or makes the request meaningless. It
stops whatever asked, and nothing partial is produced.

The rule for placing something here: an answer built on top of this would be
false rather than weak, and no caller could act correctly on it whatever they
were told.

Examples from this plan:

- a row that does not satisfy the schema
- a unit whose dimension is not the dimension its quantity declares
- a coefficient set naming a form, or a form version, that does not exist
- a chain link naming a source that is not in the source register
- an evaluation requested outside a form's structural domain, where the
  expression is not defined at all
- a derived value whose recipe, recomputed over the versions it names, does not
  reproduce the number it holds

### Warning

Something that makes an answer less trustworthy while leaving it an answer. The
run continues and the warning travels with the result.

The rule for placing something here: the arithmetic is right and a claim
underneath it is weaker than the answer looks. A caller who knows this can still
use the number; a caller who does not know it will misread it.

Examples from this plan:

- a coefficient set evaluated outside the fitted domain its publication stated
- a coefficient set evaluated at all when its publication stated no domain
- a comparison in which some rows rest on an interpreted coverage factor rather
  than a published one
- a row admitted into a comparable set under a condition tolerance rather than at
  the condition asked for
- an export that dropped rows the target format cannot represent
- a derived value that is stale, meaning every version it names still reproduces
  its number but one of those versions has been superseded

### Absence

Something that is simply not there. It is not an error and is never reported as
one.

The rule for placing something here: nothing in the tree is wrong, the register
does not hold what was asked for, and the honest answer is empty.

Examples from this plan:

- a quantity nobody has curated yet
- a material with no rows for the quantity asked about
- a comparable set that is empty because the tolerance excluded every row
- a source with no digital identifier, which is an ordinary source and not an
  incomplete one

A register whose normal state produces errors trains its users to skip them, and
after that the refusals are unread too. That is the failure this class exists to
prevent, and it is why an empty result is an empty result rather than a
diagnostic.

## Warnings attach to the result

A warning is a field of the result, not a line in a log. A result file read a
year later by somebody who never watched the run carries the same warnings the
operator saw.

Every result the tools write carries a `warnings` list. Each entry has a code
from a closed vocabulary, the subject it is about named by identifier, and a
sentence for a person. The code is what makes a warning something a consumer can
act on without parsing prose, and the closed vocabulary is what stops the set
growing into a category nobody can enumerate.

An empty `warnings` list is written explicitly. A result with no such field is a
result from a version of the tools that predates this policy, and it must not be
readable as a clean one.

## No defaults, anywhere

The tools never supply a condition the register does not hold. There is no
default temperature, no assumed room temperature, and no convenience helper that
fills one in.

That sentence has no exception clause and does not acquire one. Filling in a
plausible condition is how the untraceable numbers this project is a reaction to
were made, and a helper that does it quietly is worse than a tool that does it
loudly, because nobody reads the helper.

## Exit codes

Three, and they are part of the interface rather than an implementation detail.
A script can only act on what it can distinguish, so the set is small and each
value means one thing.

- `0` the run completed and nothing was refused. A run that completed with
  warnings exits `0`, because the warnings are in the result and the result is
  usable.
- `1` something was refused. The register or the request is wrong, the refusal
  names the subject and the property, and no result was produced.
- `2` the command could not judge. It could not start, could not read the
  register, was given arguments it does not understand, or failed inside itself.
  This is deliberately not `1`: a script that cannot tell a refusal from a broken
  invocation will eventually treat a broken invocation as a clean refusal, and
  that is the direction in which the mistake is silent.

These values are covered by tests, one per code, and a change to them is a
change to the interface.

## A warning and a usable result at once

This is the ordinary case, not an edge. The result is written, the warnings are
in it, and the exit code is `0`.

A caller who needs warnings to be fatal says so explicitly, and then the run
refuses and exits `1`. That is opt-in strictness for a script that would rather
stop than carry a weak number into a published figure.

A caller may suppress warnings from the terminal. A caller may not suppress them
from the result. The quiet form changes what a person watching sees and changes
nothing in the file, so a result that was produced quietly is byte for byte the
result that was produced loudly. Suppression is a property of the display, never
of the artefact, and there is no flag that removes a warning from a result file.

## What this record does not decide

It does not decide the warning vocabulary itself, which grows with the checks
that raise the entries. It does not decide the output format of a result. It
does not decide what the gate prints, beyond that the gate places its own
findings in these three classes like everything else.
