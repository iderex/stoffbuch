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

`.github/workflows/gate.yml`, on every pull request and on every push to the
default branch. It runs the command at the top of this document and nothing
else, so what the server judged and what you judged are the same thing rather
than two lists that agree until one of them is edited.

The check it produces is called

    The gate

and this is the only document that writes that name down. A protection rule can
only name a check as a literal string, so the name is an interface: rename the
job and every rule pointing at the old name is pointing at a check nothing
produces, which is a rule that passes everything. Writing it here makes a rename
a change to two files a reviewer sees rather than one line inside a workflow.

The rule on the default branch does not require it:

    gh api repos/iderex/stoffbuch/rulesets/20523097 \
      --jq '[.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context]'
    ["DCO sign-off","dependency-review","Reject Trojan Source Unicode","Audit workflows (zizmor)"]

read on 2026-08-09. Four names are conditions of a merge and this is not one of
them, so the check runs, it goes red where the gate refuses, and a merge is
still possible over the top of it. Adding the name to that list is
[#75](https://github.com/iderex/stoffbuch/issues/75), and it comes after the
check exists on purpose, because a required name that no job produces blocks
every merge with no repair except changing the rule back.
