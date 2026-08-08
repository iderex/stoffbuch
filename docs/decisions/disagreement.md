# How disagreement between publications is represented

The register will hold several values for the same quantity of the same material
under the same conditions, and they will not agree. That spread is a first class
output of this project rather than a defect to be resolved, and this record says
what it means concretely.

Disagreement is not uncertainty and the two are never added into one number
without saying so. A publication's uncertainty is what that publication claims
about its own measurement. The spread between publications is evidence about how
well the field knows the quantity, and when it is much larger than the individual
uncertainties, that fact is the most useful thing this register can tell a
reader.

## What a comparison is

A comparison is an answer to one question: what does the field say about this
quantity, for this material, under these conditions.

Its inputs are the quantity, a material identity, a condition window, and the
release of the register it was computed against. From those, the comparable set
is built by the rule in `material-identity.md`: the rows whose identity matches
on the parts the quantity declares relevant, and whose conditions match on the
conditions the quantity requires, within the tolerances the quantity declares.

Its output is the rows themselves, visible, together with the statistics below.
A comparison never replaces the rows with a summary. A reader who is shown five
numbers and told they disagree can act on it; a reader shown a mean and a spread
has been given somebody's reading of five numbers and cannot get back to them.

Measured rows and published computed rows are separated and counted separately,
never folded into one spread, which `out-of-scope.md` already fixes and which
this record does not weaken.

## Computed, not stored

The register stores no comparison. A comparison is computed from rows on demand.

The reason is that a stored comparison is a derived thing that goes stale the
moment a row is corrected or a new row lands, and a stale summary that still
reads as current is worse than no summary. The register already has a kind for
derived things, with an inconsistency rule and a staleness rule attached, in
`record-kinds.md`. So a comparison somebody needs to cite is written as a derived
row and inherits that machinery, and nothing is gained by inventing a second
mechanism for the same shape.

That leaves the ordinary case cheap and the citable case correct, which is the
right way round. Most comparisons are asked once by somebody deciding what
number to use.

## The two statistics, and both are always reported

Neither statistic is ever reported alone. One says how far apart the values are
and the other says whether that distance is surprising, and either one without
the other is misread.

### The dispersion

The observed dispersion is the sample standard deviation of the row values,
reported with the number of rows, the smallest and largest value, and the values
themselves.

It must not be read as an uncertainty. It is not the uncertainty of any row, it
is not the uncertainty of a value computed from the rows, and it is not
something to be quoted beside a value as though it were. It is a description of
a set of numbers.

With two or three rows a sample standard deviation carries almost no
information, so the count is printed beside it always and never as a footnote.
With one row there is no dispersion, and the report says there is one row rather
than printing a zero, because a zero there reads as perfect agreement.

### The consistency

The consistency measure is the chi squared statistic of the rows about their
uncertainty weighted mean, with one fewer degree of freedom than there are rows,
reported as the Birge ratio: the square root of chi squared divided by the
degrees of freedom.

A ratio near one means the scatter between publications is about what their
stated uncertainties would predict. A ratio much greater than one means the field
disagrees by more than it claims to. A ratio much less than one means the stated
uncertainties are larger than the scatter, which in this literature usually means
the rows are not independent rather than that everybody was cautious.

It must not be read as a hypothesis test whose outcome decides which value is
right, and it must not be used as a factor to inflate anybody's uncertainty.
Some fields multiply a stated uncertainty by the Birge ratio to force
consistency. This register does not, anywhere, because that operation writes a
disagreement between publications into a number attributed to one of them, and
the whole argument of this project is that those two things stay separable.

The weighted mean appears in this statistic only as the point the scatter is
measured about. It is not a recommended value, it is not published as one, and
nothing in this record turns it into one.

### What the statistics rest on, and the count that says so

A chi squared over rows assumes the rows are independent determinations. In this
literature they frequently are not: six rows can descend from one measurement
made in 1974, reported in a review, copied into a handbook, and transcribed from
there by four later papers.

This register can see that, because the provenance chain is a field rather than
a sentence. So every comparison reports how many distinct earliest sources its
rows resolve to, beside the row count, and names the rows that share one. A
comparison of six rows over two independent measurements says so, and the reader
who was about to treat it as six is stopped.

The comparison also counts the rows whose uncertainty rests on an interpreted
coverage factor rather than a stated one, using the interpretation rule tag from
`uncertainty.md`. Those rows carry a reading this project applied, and a
consistency statistic computed over them is partly a statement about that
reading. `error-and-failure-policy.md` already places that case in the warning
class, and the count is what the warning carries.

Rows whose uncertainty is absent cannot enter a weighted mean at all. They are
listed in the comparison, they count towards the dispersion, and they are
excluded from the consistency statistic with the exclusion stated and the rows
named. They are never given an uncertainty so that the arithmetic can proceed.

## What the report says in each case

When the values agree, meaning the Birge ratio is near one, the report gives the
rows, the dispersion with its count, the ratio, and says that the scatter is
consistent with what the sources claim. It offers no single number.

When the values differ but the difference is within what the sources claim,
meaning the intervals overlap and the ratio is at or below one, the report says
that: the values are not the same and the sources do not disagree. This is a
common and undramatic case and it reads differently from the one above, because
a reader who needs to know whether two papers conflict is asking about the
uncertainties rather than about the values.

When the values disagree by far more than their uncertainties, the report says so
in words, prints the ratio, names the rows furthest from the weighted mean, and
gives the independent source count. It does not widen any error bar, it does not
exclude an outlier, and it does not pick a winner. Removing a row from a
comparison because it disagrees is the failure this whole register was built
against, and the only thing that removes a row is a reason about the row itself,
written into the row, under the supersession rule in
`versioning-and-citation.md`.

The threshold for far more is a Birge ratio above two, and it is a reporting
convention rather than a test. The report prints the ratio itself so a reader can
apply their own, and this record does not claim the number has any other
authority.

When the comparable set is empty, or holds one row, there is nothing to compare.
That is absence rather than an error, in the class `error-and-failure-policy.md`
sets out, and the report says the register holds nothing, or holds one row, and
prints it.

## Reproducibility

A comparison that cannot be recomputed is an assertion. So a comparison records
everything needed to produce it again, and the list is not optional:

- the release identifier of the register it was computed against
- the quantity identifier
- the material identity it selected on, in full
- the condition window and every tolerance used, including the alloy tolerance
- the identifier and version of every row admitted, in the form
  `versioning-and-citation.md` fixes
- the identifier and version of every row that matched the identity but was
  excluded, with the reason for each exclusion
- the version of the tool that computed it

The excluded rows are on that list deliberately. A comparison is as much a
statement about what was left out as about what was put in, and a reader who
cannot see the exclusions cannot check the selection.

When a row in a published comparison gains a correction, nothing happens to the
published comparison. It names row versions, and those versions do not change,
so it goes on being a true statement about the versions it names. Recomputing at
the current versions produces a different comparison, and where the comparison
was written as a derived row, the staleness rule in `record-kinds.md` is what
says that the recomputation is owed and names the input that moved. Where it was
not, it was a computed answer somebody printed once, and it is reproducible
because of the list above.

## What this record does not decide

It does not decide whether this register publishes a recommended value. That is
entry 3 of the maintainer question issue, `Whether this register publishes a
recommended value`, and it is not answered anywhere in this project yet. This
record is written so that either answer fits: a comparison with no
recommendation is a complete artefact, and a recommendation, if one is ever
published, is a row under its own provenance rather than a field of a comparison.

It does not decide the output format of a comparison, nor what the command line
prints, which belong with the consumers. It does not decide the numerical
tolerances of the arithmetic, which belong with the evaluator. It does not decide
the warning vocabulary, which grows with the checks that raise the entries.
