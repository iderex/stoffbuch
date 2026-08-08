# The gate

One command says whether the tree is in a good state:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

It runs its parts in a fixed order and stops at the first failure, so the output
ends at the thing to fix instead of burying it under the parts that came after.
A part that ran, a part that was skipped, a part that refused and a part the run
never reached are four different lines, and the last line counts all of them. A
run that examined less than the whole set therefore cannot be read as a run that
examined everything and found nothing.

What the parts are is printed by the command. This document does not list them
and neither does any other, because a list in a document drifts against the
thing it describes, and this set will grow for years. If you want to know what
the gate covers today, run it and read what it says; if you want to know why a
part is skipped, the line under it says what would make it run.

The exit codes are the ones in
[decisions/error-and-failure-policy.md](decisions/error-and-failure-policy.md),
which is where they are decided rather than here.

## The hook

    git config core.hooksPath .githooks

Once per clone. It makes a push run the gate first, which turns a mistake into a
minute rather than a round trip through a pull request.

It is a convenience and it is skippable. A fresh clone does not have it,
`--no-verify` walks past it, and nothing anywhere reads whether a clone set it.
So it is not the enforcement and this document does not call it one.

## What runs the gate on the server

Nothing. The workflows in this repository check other things and none of them
builds or tests anything:

    grep -rlE '\bcargo\b' .github/workflows/ ; echo "exit=$?"
    exit=1

So a run of the gate that never happened leaves the same trace as a green one,
and the only thing standing behind a merge today is that somebody ran the
command and said so. Making the gate a check a protection rule can require is
[#16](https://github.com/iderex/stoffbuch/issues/16) and
[#75](https://github.com/iderex/stoffbuch/issues/75); until they land, the
sentence above is the whole of it.
