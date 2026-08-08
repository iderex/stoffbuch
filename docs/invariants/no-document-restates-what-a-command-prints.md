Id: no-document-restates-what-a-command-prints
Held: no
Retired-by: a check that reads the structure of a document rather than its text, or a generator that writes the lists into documents so that a stale one is a diff rather than a drift.
Rule: no document lists the parts of the gate, the workflow guards, or the fields of a schema. A document that needs to talk about a set points at the command that prints it.

Three issues each require this of every other document, and a search of the tree
for text cannot hold it. It is recorded here as unheld rather than approximated,
because a pattern that half works is worse than no pattern: it makes a green run
mean something it does not.

A list has no spelling. What the rule forbids is a shape, several names appearing
together in a document where one pointer belongs, and the names it forbids are
the same names that appear legitimately elsewhere. `docs/quality-parity-walk.md`
restates every name from the target board on purpose, and says so, because a
rename has to show as a change to that file. A search that refused a name in a
document would refuse that file first, and the repair would be an exception list
that grows until the invariant means nothing.

Nor can the search be inverted. Refusing a document that does not point at the
command would refuse every document that has no business mentioning the set at
all, which is most of them.

What holds it today is the review, and the fact that the sets live in one place
each so that a document restating one has to have copied it. That is weaker than
a check and it is what there is.

Whoever retires this should know which direction is cheaper. Reading a document's
structure means parsing prose, which is the expensive half. Writing the lists
into the documents from the thing that owns them is the other half, and it turns
a drift into a diff, which is the shape this project prefers everywhere else. It
costs a generator and it costs the rule that no document lists the set, replacing
it with a rule that no person writes the list by hand.
