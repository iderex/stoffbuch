# Model forms: identity, domain of validity and the evaluation contract

Most of what this register is about is not a number. It is a function: a band gap
against temperature, a refractive index against wavelength, a mobility against
doping and field. A publication reports a named form and a set of coefficients; a
consumer needs a value at a condition. The gap between those two is where tools
disagree.

The failure is worth stating plainly, because a register that stores a form name
and a coefficient set and stops there has reproduced it rather than fixed it.
Two implementations of the same named model return different numbers because the
printed equation had a typo, or because one uses the lattice temperature where
the other uses the carrier temperature, or because a coefficient was defined
with the opposite sign.

## What a form is

A form in this register is three things together, and it is not a form until all
three exist.

The printed equation, as the publication wrote it. Held as text, and as a
reference to an image of the printed equation where the typography carries
meaning that text does not, which is common in older reports.

The evaluation contract. A statement precise enough that two independent
implementations written from it agree. It names every variable and every
parameter, and for each one it gives the physical meaning without abbreviation,
the dimension, and the unit the evaluator expects. It writes the equation with
every sign explicit. It says what the expression does at the edges of its own
definition.

The test vectors. Inputs and expected outputs that any implementation must
reproduce. A form with no test vectors cannot be referenced by anything: a
coefficient set naming it is refused, and the form is not a form, it is a note.
This is the part that makes the contract testable rather than persuasive.

A coefficient set may only name a form that exists, at a version that exists.

## Domain of validity, and it is both

Two domains, and they answer different questions.

The form declares its structural domain, which is where the expression is
defined at all. A denominator that reaches zero, a logarithm of a
non-positive number, a temperature below absolute zero. Outside it there is no
number, and the evaluator refuses. Returning something there would be inventing
a value, which is the failure this register is against.

Each coefficient set declares its fitted domain, which is the range the
publication states its coefficients hold over. Outside it the evaluator returns
a number and attaches a warning to the result, because the arithmetic is
well defined and the claim is not. A coefficient set fitted between 100 and 400
kelvin says nothing at 700 kelvin, and every simulator in the field will
evaluate it there without comment.

A publication that states no range is recorded as stating none, in the same way
an unstated condition is recorded. Evaluating such a set anywhere attaches a
warning saying the domain is unstated, so the silence travels with the answer
instead of being lost at the moment it matters.

The warning attaches to the result rather than to a log, so a result file read
later still carries it. The three classes are set out in
`error-and-failure-policy.md`.

## Versioning

A form is versioned and never edited in place. A change to the printed equation,
the parameter list, the evaluation contract or the test vectors produces a new
version.

That includes a change that only corrects this project's transcription of the
equation. A machine cannot tell a correction that changes no value from one that
changes every value, and asking a curator to make that call at the moment of the
edit puts the judgement where nobody will see it again. Every change is a
version, and the cheap case costs one file.

An existing coefficient set goes on naming the version it was entered against.
Nothing is migrated. The reason is that a coefficient set is a claim about what a
publication reported under a particular reading of the equation, and a new
reading does not retroactively become what the publication meant. Moving a
coefficient set to a new form version is a curator action: it needs the
publication in hand, and it produces a new version of the coefficient set row
naming the new form version, with the old version still readable for anything
that cited it.

A form version that nothing references is refused by the gate. The practical
consequence is that a form lands together with at least one coefficient set that
uses it, rather than ahead of its first use, and a form version that every
coefficient set has moved off is removed as part of the change that moved the
last one.

## The conventions a paper leaves implicit

Three of these cause the disagreements, and all three are recorded in the
evaluation contract rather than in a note beside it.

A temperature convention is recorded by naming which temperature the variable
is, in full, at the point where the variable is declared. Lattice temperature,
carrier temperature and ambient temperature are three different variables and
the contract never writes an unqualified temperature.

A unit convention is recorded per variable and per parameter, as a dimension and
a unit, in the same vocabulary the quantities use. This is what turns a
convention into something a check can refuse, because a coefficient carrying the
wrong dimension is a refusal rather than a discrepancy discovered later.

A sign convention is recorded by writing the equation with the signs explicit,
and then by at least one test vector that gives a different answer under the
opposite convention. The contract alone is not enough here: a contract can be
correct and still be read wrongly, and a vector that would pass under either
reading discharges nothing. The obligation is on the vector set, not on the
prose.

## A worked example of an ambiguous published equation

The Sellmeier form for refractive index against wavelength is written in the
literature as a sum of terms of the shape `B * lambda^2 / (lambda^2 - C)`.

The ambiguity is what the tabulated `C` is. In one common convention it is the
quantity subtracted from `lambda^2`, so it has the dimension of a length
squared. In another the table prints the resonance wavelength itself, so it has
the dimension of a length and the evaluator must square it. A second ambiguity
rides along with the first: the wavelength unit, which is micrometres in most
optical tables and nanometres in some, and which the table often does not state
because within its own field it is obvious.

Neither wrong reading produces an absurd number. Both produce a refractive index
in the range a reader expects, which is exactly why this ambiguity survives into
released software.

The contract resolves it in three moves, and none of them is a comment. The
parameter declaration fixes the dimension of `C`, so a coefficient set whose
values are resonance wavelengths is refused by the dimension check rather than
evaluated. The variable declaration fixes the wavelength unit, and a coefficient
set entered from a nanometre table is converted on the way in only if the
publication printed it that way, which it did not, so instead the two conventions
are two form versions and a coefficient set names the one its publication used.
And the test vectors pin the reading numerically, so an implementation that
squares `C` when it should not fails before it is used on anything.

## What a check can refuse

- a coefficient set naming a form that does not exist
- a coefficient set naming a form version that does not exist
- a coefficient set carrying a coefficient name outside the form's parameter
  list, and a coefficient set missing one the form requires
- a coefficient whose unit has a dimension other than the one the form declares
  for that parameter
- a form with no test vectors, and any reference to such a form
- a form whose own test vectors its evaluator does not reproduce
- a form version that nothing references
- an evaluation requested outside the structural domain

## What this record does not decide

It does not decide which forms exist, which belongs with the form register, nor
the evaluator's numerical tolerances, which belong with the evaluator and its
test vectors. It does not decide the covariance a coefficient set carries, which
belongs with uncertainty. It does not decide how a form version appears in a
citation, which belongs with versioning.
