# What uncertainty means here

Uncertainty is a field in this register rather than a comment, and that commits
the project to saying what the field means. The literature it reads from does
not agree on an answer.

What publications actually print. A value with a plus or minus and no statement
of what the interval covers. A percentage accuracy in the caption. Error bars on
a figure and nothing in the text. A number quoted to four significant figures,
where the last digit is the only claim being made. A recommended value in a
review with a footnote giving the spread among the sources it drew on. And most
often, nothing at all.

## The vocabulary is borrowed, not invented

The terms come from the Guide to the expression of uncertainty in measurement,
JCGM 100:2008, and its first supplement on propagation of distributions by a
Monte Carlo method, JCGM 101:2008. Where a term appears in this project it means
what it means there, and no document here defines it a second time.

The terms this project uses are standard uncertainty, combined standard
uncertainty, expanded uncertainty, coverage factor, coverage interval, and
evaluation of Type A and of Type B. The one word that is this project's own is
the interpretation rule below, which names a reading the register applied and
which has no counterpart in the published vocabulary because the published
vocabulary assumes the measurer is the one writing it down.

The version of each document the project was written against is pinned where the
external specifications are pinned, beside the unit grammar, so a revision is a
change to the tree rather than something that happens underneath a run.

## What a record stores

Three things, and dropping any one of them loses something the other two cannot
carry.

The printed statement, exactly as the publication gave it. Held as digits and a
form rather than as a sentence: the number or numbers, their unit where the
publication gave one, and which of the shapes below it was. It is stored as
digits under the string rule in `means-format-and-language.md`, so a stated
`0.020` keeps the claim its trailing zero makes.

The normalised standard uncertainty. One number in the quantity's own unit,
which is what anything computing over the register reads. This is the useful
field and it is the one that hides an interpretation, which is why it never
appears without the third.

The interpretation rule, a tag from a closed vocabulary naming how the second
was produced from the first. This is what makes the reading auditable, reversible
in bulk when it turns out to be wrong, and countable. A report can say how many
rows in a comparison rest on an assumed coverage factor rather than a stated
one, and that count is a real measure of how much of an answer is this project's
reading rather than the field's statement.

Storing only the printed statement was rejected. Every consumer would then decide
what a bare plus or minus meant, which is the same guess made many times instead
of once and recorded.

Storing only the normalised value was rejected. Somebody decided that a stated
interval was two standard uncertainties, and that decision would have left no
trace anywhere.

## The interpretation rules

Each rule is a tag, and each one names what it assumes. A rule that assumes
nothing says so.

`as-stated-standard` applies where the publication says the number is a standard
uncertainty, a standard deviation, or gives a coverage factor of one. The
normalised value is the printed one. It assumes nothing.

`stated-coverage-factor` applies where the publication states a coverage factor.
The normalised value is the printed one divided by that factor. It assumes that
the publication's coverage factor means what the published vocabulary means by
it, which is usually safe and occasionally is not, in older papers that use the
word for a confidence level.

`assumed-standard` applies to a bare plus or minus with no statement of what it
covers, in a paper reporting its own measurement. The normalised value is the
printed one. It assumes the field's most common convention, that a bare plus or
minus in a primary measurement paper is one standard deviation.

That default is a real assumption and its cost is stated rather than buried. If a
source meant an expanded uncertainty at a coverage factor of two, this register
understates that row by a factor of two. The tag is what makes the repair
possible without re-reading every source: the rows carrying it can be listed,
counted, and moved to `stated-coverage-factor` when somebody establishes what a
particular journal or a particular group meant. A curator who already knows
better does not use this rule.

`half-width-rectangular` applies to an interval given as bounds with no
distribution behind it, which is what an accuracy specification is. The
normalised value is the half width divided by the square root of three. It
assumes a rectangular distribution over the interval, which is the Type B
treatment the published vocabulary gives for exactly this case.

What would show that assumption wrong for a particular source is the source
describing its own interval as something else: bounds given as a coverage
interval at a stated level of confidence, or bounds a publication says came from
repeated observation, are not the rectangular case and are read under the rule
that fits them. An accuracy specification that names no distribution and no level
stays here.

`percent-of-value` is not a rule on its own and never appears alone. A
percentage produces an interval from the value, and one of the rules above then
turns the interval into a standard uncertainty. The tag is recorded as the pair,
so a row reads as a percentage that was treated as a rectangular half width
rather than as a percentage that was treated as a standard deviation, and those
two readings differ by the square root of three.

`digitised-bar` applies to an uncertainty that exists only as the length of a
plotted error bar. The bar is read off the figure, the caption is what says what
the bar means, and the rule the caption implies is applied to the length. Where
the caption says nothing, the half length is treated under
`half-width-rectangular`, and the tag records both that reading and the fact
that the caption was silent. The digitisation uncertainty is combined with the
result in quadrature and is never omitted, under the rule
`provenance-and-the-citation-chain.md` already sets for a digitised value.

`last-digit` applies to a value with no uncertainty statement at all, quoted to a
number of significant figures. The half width is half a unit in the last place
and the normalised value is that divided by the square root of three. It assumes
the publication rounded rather than truncated, and it assumes the last digit was
meant as a claim rather than as an artefact of somebody's calculator.

That second assumption is weak often enough that this rule is never applied
automatically. A curator applies it deliberately, per row, and the tag records
that a curator did. Applying it across the register by default would manufacture
an uncertainty for every value in it and make the field with no uncertainty
disappear, which is the failure the next section exists to prevent.

`none` applies where the publication states nothing and nothing is inferred. The
normalised standard uncertainty is absent.

## An absent uncertainty is absent

A value with no uncertainty statement has no uncertainty in this register. It
does not have an uncertainty of zero.

No route in this project substitutes zero for an absent uncertainty. Not the
library, not the evaluator, not a propagation, not an export, and not a default
in a schema. The distinction is the difference between a number nobody has
characterised and a number known exactly, and collapsing them turns the least
trustworthy rows in the register into the most trustworthy ones.

What follows from that, concretely. A normalised standard uncertainty of zero is
refused by the gate, because no measurement has one and a zero in that field is
always either a substitution or a transcription error. A calculation over an
input whose uncertainty is absent produces a result whose uncertainty is absent,
and a warning saying which input carried none, rather than a result whose
uncertainty is understated by everything the missing input would have
contributed. An export to a format with no way to express absence refuses rather
than writing zero, and the refusal names the rows it could not represent, so the
absence is what stops the export rather than something the export invents a
value for.

An absent uncertainty is not a defect in the row and is not reported as an
error. It is the ordinary state of a great deal of the literature this register
is about, and the coverage report counts it rather than complaining about it.

## The five cases worked

A bare plus or minus, no statement. The publication prints a value and `0.020`
after it. The printed statement holds `0.020` and the shape symmetric. The rule
is `assumed-standard`, the normalised value is `0.020`, and the row is counted
in the report of rows resting on an assumed reading.

A percentage accuracy in the caption. The publication prints `3.42` and a caption
saying the values are accurate to two percent. The printed statement holds two
percent and the shape relative. The rule is the pair `percent-of-value` and
`half-width-rectangular`. The half width is `0.0684`, and the normalised standard
uncertainty is that divided by the square root of three:

    0.0684 / 1.7320508075688772 = 0.03949075841257041

recorded to the digits the value's own precision justifies rather than to all of
them.

An error bar on a figure and nothing in the text. The row is digitised, so it
already owes a digitisation uncertainty and a method. The bar half length reads
as `0.02` in the quantity's unit. The caption is silent, so the rule is
`digitised-bar` with `half-width-rectangular`, the bar contributes `0.02`
divided by the square root of three, and the digitisation uncertainty is
combined with it in quadrature. Two numbers, both recorded, one normalised
result.

Four significant figures and no uncertainty. The publication prints `1.12`. The
printed statement holds the fact that nothing was stated and the digits as
printed. Where a curator chooses to read the last digit as a claim, the rule is
`last-digit`, the half width is `0.005`, and:

    0.005 / 1.7320508075688772 = 0.002886751345948129

Where the curator does not, the rule is `none` and the normalised value is
absent. Both are complete rows and the tag is what tells them apart.

Nothing at all. The rule is `none`, the normalised standard uncertainty is
absent, and no route anywhere fills it in.

## Asymmetric uncertainty

A publication that prints different numbers upward and downward is making a
claim about the shape of what it knows, and averaging the two into one number
destroys exactly that claim.

The printed statement holds both, the shape is recorded as asymmetric, and the
normalisation produces a pair rather than a single number. Every interpretation
rule above applies to each side independently.

A consumer asking for one number is given both and decides for itself. A
propagation over an asymmetric input uses the Monte Carlo method below, because
the linear method has nowhere to put the asymmetry and would silently symmetrise
it.

## The uncertainty of a derived value

Both methods, with one of them the default and the other required in named
cases.

The default is the linear method of JCGM 100:2008: the combined standard
uncertainty from the first order partial derivatives of the recipe with respect
to its inputs. It is cheap, it is analytic, and its result is reproducible
without a random source.

The Monte Carlo method of JCGM 101:2008 is required, rather than merely
available, in three cases. Where any input is asymmetric, for the reason above.
Where the recipe is strongly nonlinear over the range the input uncertainties
cover, since that is the condition under which the first order expansion stops
describing the answer. And where the quantity is bounded and the result sits
near a bound, where the linear method produces an interval that crosses into
values the quantity cannot take.

Every derived value records which method produced its uncertainty. A Monte Carlo
result additionally records the number of draws and the seed, so the number is
reproducible by anybody holding the same inputs, which is the property this
whole project is about and which a Monte Carlo result loses by default.

Correlation between inputs is what the linear method needs and is not settled
here. It is settled where the uncertainty block in the record is built, together
with the covariance a fitted coefficient set carries. Until a coefficient set can
carry a covariance, a propagation over more than one coefficient of one set
assumes the coefficients are independent, that assumption is wrong for almost
every fit, and it travels as a warning on the result rather than as a silent
choice. The assumption is named here so that the warning exists before the
machinery does.

## What a check can refuse

- a normalised standard uncertainty of zero
- a normalised standard uncertainty that is negative
- an interpretation rule outside the vocabulary
- a normalised value present with no interpretation rule, or a rule present with
  no printed statement to have applied it to
- a rule of `none` with a normalised value present
- a percentage rule recorded without the second rule it composes with
- an asymmetric printed statement normalised to a single number
- a digitised row whose uncertainty carries no digitisation contribution
- a derived value with an uncertainty and no statement of which method produced
  it, and a Monte Carlo one with no draw count and no seed

## What this record does not decide

It does not decide the field names or the layout of the uncertainty block, which
belong with the schema. It does not decide the covariance a coefficient set
carries, nor how correlation is expressed, which belong with the uncertainty
block in the record. It does not decide the number of draws a Monte Carlo
propagation uses by default, nor the test that says whether a recipe is strongly
nonlinear, both of which belong with the evaluator. It does not decide how the
spread between publications is computed or reported, which is a different thing
from uncertainty and is settled in `disagreement.md`.
