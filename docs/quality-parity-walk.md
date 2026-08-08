# The quality target, walked check by check

The quality target for this repository is the merge gate on a public sibling
project, and the target is read as a command rather than remembered:

    gh api repos/Flowfin/jellyfin-plugin-sso/rulesets --jq '.[] | select(.name | startswith("Protect")) | .id'
    18802863

    gh api repos/Flowfin/jellyfin-plugin-sso/rulesets/18802863 --jq '[.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context]'
    ["build","ABI floor build","Package (JPRM) / Build package","Package (JPRM) / Generate SBOM","CodeQL","Analyze (csharp)","DCO sign-off","Deterministic PR-hygiene checks","Enforce greppable invariants","Reject Trojan Source Unicode","Audit workflows (zizmor)","prettier","dependency-review"]

Both read on 2026-08-08. Thirteen names, and a later reader can run the same two
commands to see whether the target has moved.

Here, on the same day, nothing is required at all:

    gh api repos/iderex/stoffbuch/rulesets/20523097 --jq '[.rules[].type]'
    ["deletion","non_fast_forward","pull_request"]

The gap between those two outputs is what this milestone closes, and requiring
names is the last issue of it rather than the first, because a name is only worth
requiring once the check behind it is worth passing.

This document holds the names from the target board and the outcome for each. It
is also the one place a name is allowed to be restated, so that a rename shows as
a change to this file.

## The outcomes

Each name below is adopted, adapted or declined.

Adopted means the check transfers as it stands and only has to be reached from
this project's gate. Adapted means the property transfers and the mechanism does
not, so what runs here is a different thing serving the same end. Declined means
the check protects something that does not exist in this project, and where that
is the outcome the property it was protecting is named along with whatever holds
it here instead.

The reasons matter more than the outcomes. A check copied without its reason is a
check nobody can maintain, and the first person to find it inconvenient removes
it.

### build

Adapted. There a build proves the code compiles, and here compilation is the
smaller half: a run that builds cleanly over a register with a broken row proves
nothing about the product, so what corresponds to it is the single gate command
that covers the code and the register together. Issue #15 builds that command and
issue #16 gives the workflow that runs it a name a rule can require.

### ABI floor build

Declined as written and replaced. It exists there because a plugin has to load
against an older host, and nothing loads this project. The property underneath it
survives the move and gets stronger: a consumer holding an older reader has to be
able to read a register written after their reader was built, or a citation
followed years later stops resolving. Issue #89 holds the replacement, and it was
opened by this walk because nothing else on the board held it.

### Package (JPRM) / Build package

Adapted. What is packaged there is a plugin and what is packaged here is a binary
together with a snapshot of the register, so the artefact is different while the
requirement that the gate produce it rather than a person assembling it at
release time is the same. Issue #77 holds it.

### Package (JPRM) / Generate SBOM

Adopted, unchanged in purpose. A released artefact says what went into it, and
that is the same sentence in both projects. Issue #70 holds it.

### CodeQL

Adopted in purpose, and its tooling has to be chosen again for this language.
This is the item most likely to change on contact with the facts, because the
choice that is obvious there does not transfer, and the interesting half is not
the tool at all but naming the surface that decides refusals so that a higher
standard can be pointed at it. Issue #69 holds both halves.

### Analyze (csharp)

Declined as written, since there is no C# here and the name is bound to a
language rather than to a property. What it protects is the same thing the entry
above protects, so it is covered by issue #69 and by that entry's reasoning
rather than by a second mechanism.

### DCO sign-off

Adopted, unchanged. A contributor asserts the origin of what they submit, and
that is language independent and project independent. Issue #71 brings it into
the one gate command, and issue #22 holds the defect found alongside it, that the
text a contributor is told they are asserting is not in the tree.

### Deterministic PR-hygiene checks

Adapted, to what an issue and a change look like here. The rules being held are
this project's own, that no work happens without an issue and that a change stays
inside the scope its issue declares, and today nothing reads either of them.
Issue #90 holds it, opened by this walk because nothing else on the board did.

### Enforce greppable invariants

Adopted. It is the cheapest check on the target board and it is the natural home
for the rules this project states that no schema can express, of which several
are already Done-when conditions on other issues with nothing behind them. Issue
#91 holds it, opened by this walk.

The gate part is named `invariants`. That is the name a refusal prints and the
name this document is holding stable, and it is not yet a name a protection rule
can require: a rule requires the context of a check that runs on the server, and
no workflow here runs the gate. #16 is what creates one, and the name it gives
that leg is what a rule would name. Which invariants the part holds is printed by
the run and is not restated here.

### Reject Trojan Source Unicode

Adopted, with one adaptation that decides whether the register can do its job.
This register will hold author names, journal titles and material descriptions
that legitimately carry characters outside ASCII, in exactly the fields a
citation depends on, so the guard has to refuse bidirectional control characters
and other invisible reordering marks specifically rather than refusing everything
that is not ASCII. Issue #71 holds the adaptation and requires the distinction to
be a fixture rather than a claim.

### Audit workflows (zizmor)

Adopted, unchanged. What it reads is workflow files, which this project has for
the same reasons and with the same failure modes. Issue #71 holds it.

### prettier

Adapted, and the adaptation is where the weight is. Formatting here means two
things: the language formatter over the code, which is the straightforward half,
and the canonical form of the register, which is the half that matters, because a
diff that shows the field a curator changed and nothing else
is what makes a curation review possible at all. Issue #34 holds the canonical
form and the refusal of a file that is not in it.

### dependency-review

Adopted, unchanged. It reads what a change adds to the dependency graph, which is
the same question here. Issue #70 holds it together with the locked restore and
the bill of materials.

## Beyond the thirteen

The target board also runs checks that are not required for a merge and are worth
copying anyway. Coverage on the surface that decides refusals is issue #72,
mutation testing on that same surface is issue #73, and fuzzing the register
parser is issue #74. The supply-chain score sits with issue #71, which is where
the guards already in this tree are brought into one procedure.

## What this document does not hold

It does not list this repository's own checks. What runs here is printed by the
gate command, and that command does not exist yet: issue #15 builds it, and until
it does there is nothing to point a reader at that would print anything. That is
the one condition of issue #68 this document cannot meet, and it is recorded here
rather than left for a reader to notice.

It does not change the protection rule. Requiring these names is issue #75, and
it comes last on purpose, because a required name that no job produces blocks
every merge with no repair except changing the rule back.
