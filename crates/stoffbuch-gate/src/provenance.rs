//! The locator vocabulary, and the sources nothing cites.
//!
//! A citation that names a paper leaves the reader to find the number in it,
//! and that is the part the existing collections leave out. The locator is
//! where inside the source the number is, and it is on the row rather than on
//! the source because two rows cite one paper at different tables.
//!
//! It is a vocabulary rather than prose so that a reader and a machine take the
//! same thing out of it. `docs/decisions/provenance-and-the-citation-chain.md`
//! fixes that much, fixes the four things it has to reach, and says in as many
//! words that it does not fix the syntax. The syntax is here, and the row
//! schema points at this rather than carrying a pattern of its own, because a
//! locator constrained in two places is a locator constrained differently in
//! two places the first time either moves.
//!
//! The second half of this part is a report and not a refusal. A source no row
//! cites is not wrong, it is unfinished, and the register is meant to grow a
//! source before the rows that will use it in the same week. What would be
//! wrong is nobody seeing it, so it is named in what the run prints.
//!
//! Refusal surface. This module decides whether a row is refused for the
//! locator it carries, and
//! `docs/decisions/static-analysis-and-the-refusal-surface.md` is where that
//! line is given its meaning.

use std::fmt::Write as _;
use std::path::Path;

use crate::canonical::{Node, read};
use crate::{Judged, tracked};

/// Where the rows live.
const ROWS: &str = "register/rows/";

/// Where the sources live.
const SOURCES: &str = "register/sources/";

/// One shape a locator may be written in.
///
/// The shape is the sequence of words that open its parts, and the sentence is
/// what a refusal prints, because a curator told only that their locator was
/// refused reads the vocabulary and guesses which of the shapes they were near.
struct Shape {
    /// The word opening each part, in order.
    words: &'static [&'static str],
    /// The shape written out, as a refusal prints it.
    says: &'static str,
}

/// Every shape a locator may take.
///
/// The set is closed and it is small. A vocabulary wide enough to admit
/// anything a publication numbers is prose again, and prose is what this
/// replaces; a vocabulary narrower than this refuses the four things the
/// provenance record requires a locator to be able to say.
///
/// A page and an equation together is here because the extraction vocabulary
/// asks for it by name: a transcription owes a locator resolving to a table
/// with a row or a column, or to a page and an equation.
const SHAPES: &[Shape] = &[
    Shape {
        words: &["page"],
        says: "page 1197",
    },
    Shape {
        words: &["equation"],
        says: "equation 12",
    },
    Shape {
        words: &["page", "equation"],
        says: "page 1197, equation 12",
    },
    Shape {
        words: &["table", "row"],
        says: "table IV, row 7",
    },
    Shape {
        words: &["table", "column"],
        says: "table IV, column 3",
    },
    Shape {
        words: &["figure", "curve"],
        says: "figure 2, curve as-grown",
    },
];

/// A locator split into the word opening each part and what follows it, or
/// nothing where it does not split.
///
/// The separator between parts is a comma and one space, and the separator
/// inside a part is one space. Both are exact. A locator written with two
/// spaces, or with a comma and none, is refused rather than tidied: this runs
/// over a register whose whole argument is that one claim has one spelling, and
/// a reader accepting several spellings is what makes two rows that say the
/// same thing compare unequal.
fn parts(locator: &str) -> Option<Vec<(&str, &str)>> {
    let mut out = Vec::new();
    for part in locator.split(", ") {
        let (word, rest) = part.split_once(' ')?;
        if rest.is_empty() || rest != rest.trim() {
            return None;
        }
        out.push((word, rest));
    }
    Some(out)
}

/// Whether a locator is in the vocabulary.
fn in_the_vocabulary(locator: &str) -> bool {
    let Some(parts) = parts(locator) else {
        return false;
    };
    let words: Vec<&str> = parts.iter().map(|(word, _)| *word).collect();
    SHAPES.iter().any(|shape| shape.words == words.as_slice())
}

/// What a run says when a locator is not in the vocabulary.
fn refusal(wrong: &[String]) -> String {
    let mut why = format!(
        "{} row(s) carry a locator that is not in the vocabulary:\n\n",
        wrong.len()
    );
    for one in wrong {
        let _ = writeln!(why, "{one}");
    }
    let _ = writeln!(
        why,
        "\nA locator says where inside the source the number is, in one of these \
         shapes:\n"
    );
    for shape in SHAPES {
        let _ = writeln!(why, "  {}", shape.says);
    }
    let _ = writeln!(
        why,
        "\nThe words are the ones above, in lower case, and what follows each one is \
         what\nthe publication printed. Parts are separated by a comma and one space. \
         The\nabbreviations a citation is usually written with, p. and Fig. and Tab., \
         are not\nin it: a locator is followed rather than read, and the reader \
         following it is\nas often a program as a person."
    );
    why
}

/// Whether a tracked path is a record in one of the register's trees.
fn is_a_record(name: &str, under: &str) -> bool {
    name.starts_with(under)
        && Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

/// Refuses a row whose locator is not in the vocabulary, and names every source
/// no row cites.
///
/// What it does not reach is in the note the run prints beside it. A record
/// this cannot read, and a row carrying no locator where this can find one, are
/// both counted there rather than refused: which fields a record owes is the
/// schema's question, and answering it here would be a second answer that can
/// disagree with the first.
pub(crate) fn every_locator_is_in_the_vocabulary(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let mut wrong = Vec::new();
    let mut rows = 0_usize;
    let mut without = 0_usize;
    let mut unread = 0_usize;
    let mut cited: Vec<String> = Vec::new();

    for name in tracked.iter().filter(|name| is_a_record(name, ROWS)) {
        let Ok(bytes) = std::fs::read(root.join(name)) else {
            unread += 1;
            continue;
        };
        let Ok(record) = read(&bytes) else {
            unread += 1;
            continue;
        };
        rows += 1;
        let provenance = record.member("provenance");
        for link in provenance
            .and_then(|provenance| provenance.member("chain"))
            .and_then(Node::elements)
            .unwrap_or_default()
        {
            if let Some(source) = link.member("source").and_then(Node::text) {
                cited.push(source.to_owned());
            }
        }
        match provenance
            .and_then(|provenance| provenance.member("locator"))
            .and_then(Node::text)
        {
            None => without += 1,
            Some(locator) if in_the_vocabulary(locator) => {}
            Some(locator) => wrong.push(format!("  {name}\n      {locator}")),
        }
    }

    if !wrong.is_empty() {
        return Judged::Refused(refusal(&wrong));
    }

    let mut sources = 0_usize;
    let mut uncited = Vec::new();
    for name in tracked.iter().filter(|name| is_a_record(name, SOURCES)) {
        let Ok(bytes) = std::fs::read(root.join(name)) else {
            unread += 1;
            continue;
        };
        let Ok(record) = read(&bytes) else {
            unread += 1;
            continue;
        };
        sources += 1;
        match record.member("id").and_then(Node::text) {
            Some(id) if cited.iter().any(|source| source == id) => {}
            _ => uncited.push(name.clone()),
        }
    }

    let mut note = format!("{rows} row(s) read under {ROWS}, {sources} source(s) under {SOURCES}");
    if !uncited.is_empty() {
        let _ = write!(note, ", {} that no row cites:", uncited.len());
        for name in &uncited {
            let _ = write!(note, "\n  {name}");
        }
    }
    if without > 0 {
        let _ = write!(note, "\n{without} row(s) carry no locator this could read");
    }
    if unread > 0 {
        let _ = write!(note, "\n{unread} record(s) could not be read");
    }
    Judged::Nothing(Some(note))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{a_tree, note, refusal};

    /// A row carrying one locator, in the canonical form and nothing more than
    /// this part reads.
    fn a_row(locator: &str, source: &str) -> Vec<u8> {
        format!(
            "{{\n  \"provenance\": {{\n    \"chain\": [\n      {{\n        \"source\": \
             \"{source}\"\n      }}\n    ],\n    \"locator\": \"{locator}\"\n  }}\n}}\n"
        )
        .into_bytes()
    }

    /// A source carrying one identifier.
    fn a_source(id: &str) -> Vec<u8> {
        format!("{{\n  \"id\": \"{id}\"\n}}\n").into_bytes()
    }

    #[test]
    fn every_shape_in_the_vocabulary_is_one_the_reader_takes() {
        // The set and the reader are two statements of one rule, and this is
        // what stops a shape being written into the list that nothing accepts.
        for shape in SHAPES {
            assert!(in_the_vocabulary(shape.says), "{}", shape.says);
        }
    }

    #[test]
    fn a_locator_in_the_vocabulary_is_not_refused() {
        let tree = a_tree(
            "locator-in-the-vocabulary",
            &[(
                "register/rows/sb-A/sb-A@1.json",
                &a_row("page 1197", "sb-src-A"),
            )],
        );
        assert_eq!(
            note(every_locator_is_in_the_vocabulary(tree.at())),
            "1 row(s) read under register/rows/, 0 source(s) under register/sources/"
        );
    }

    #[test]
    fn the_abbreviation_a_citation_is_written_with_is_refused() {
        // The near neighbour of the row above, differing in the two characters
        // a curator writing from habit actually types. Both name page 1197 and
        // a reader would follow either, which is why the pair is the one worth
        // holding: a fixture naming no page at all would prove only that the
        // reader refuses nonsense.
        let tree = a_tree(
            "locator-abbreviated",
            &[(
                "register/rows/sb-A/sb-A@1.json",
                &a_row("p. 1197", "sb-src-A"),
            )],
        );
        let why = refusal(every_locator_is_in_the_vocabulary(tree.at()));
        assert!(
            why.contains("1 row(s) carry a locator that is not in the vocabulary"),
            "{why}"
        );
        assert!(why.contains("  register/rows/sb-A/sb-A@1.json"), "{why}");
        assert!(why.contains("      p. 1197"), "{why}");
        assert!(why.contains("page 1197, equation 12"), "{why}");
    }

    #[test]
    fn a_part_separated_by_a_comma_and_no_space_is_refused() {
        // The whole vocabulary rests on the separator being exact, so the
        // one-character miss is the fixture, not a locator with no comma in it.
        assert!(in_the_vocabulary("table IV, row 7"));
        assert!(!in_the_vocabulary("table IV,row 7"));
    }

    #[test]
    fn a_word_the_vocabulary_does_not_carry_is_refused_however_well_it_reads() {
        assert!(!in_the_vocabulary("section 3"));
        assert!(!in_the_vocabulary("Page 1197"));
        assert!(!in_the_vocabulary("figure 2, panel b"));
    }

    #[test]
    fn a_part_with_nothing_after_its_word_is_refused() {
        assert!(!in_the_vocabulary("page"));
        assert!(!in_the_vocabulary("page "));
        assert!(!in_the_vocabulary("table IV, row"));
    }

    #[test]
    fn a_second_space_inside_a_part_is_refused_and_one_is_not() {
        assert!(in_the_vocabulary("figure 2, curve as grown"));
        assert!(!in_the_vocabulary("page  1197"));
    }

    #[test]
    fn a_source_a_row_cites_is_not_named_and_one_nothing_cites_is() {
        let tree = a_tree(
            "locator-uncited-source",
            &[
                (
                    "register/rows/sb-A/sb-A@1.json",
                    &a_row("page 1197", "sb-src-CITED"),
                ),
                (
                    "register/sources/sb-src-CITED/sb-src-CITED@1.json",
                    &a_source("sb-src-CITED"),
                ),
                (
                    "register/sources/sb-src-ALONE/sb-src-ALONE@1.json",
                    &a_source("sb-src-ALONE"),
                ),
            ],
        );
        let note = note(every_locator_is_in_the_vocabulary(tree.at()));
        assert!(
            note.contains("2 source(s) under register/sources/"),
            "{note}"
        );
        assert!(note.contains("1 that no row cites:"), "{note}");
        assert!(
            note.contains("  register/sources/sb-src-ALONE/sb-src-ALONE@1.json"),
            "{note}"
        );
        assert!(!note.contains("sb-src-CITED/"), "{note}");
    }

    #[test]
    fn a_row_this_cannot_read_a_locator_out_of_is_counted_rather_than_refused() {
        let tree = a_tree(
            "locator-absent",
            &[
                (
                    "register/rows/sb-A/sb-A@1.json",
                    b"{\n  \"kind\": \"x\"\n}\n",
                ),
                ("register/rows/sb-B/sb-B@1.json", b"{\n"),
            ],
        );
        let note = note(every_locator_is_in_the_vocabulary(tree.at()));
        assert!(
            note.contains("1 row(s) read under register/rows/"),
            "{note}"
        );
        assert!(
            note.contains("1 row(s) carry no locator this could read"),
            "{note}"
        );
        assert!(note.contains("1 record(s) could not be read"), "{note}");
    }

    #[test]
    fn a_tracked_row_absent_from_the_working_copy_is_counted_rather_than_passed_over() {
        // The three ways a record goes unread are three places in this check,
        // and a run that passed one of them over silently would report fewer
        // records than it was given while saying nothing about the difference.
        // Each has its own fixture below for that reason.
        let tree = a_tree(
            "locator-row-gone",
            &[
                (
                    "register/rows/sb-A/sb-A@1.json",
                    &a_row("page 1197", "sb-src-A"),
                ),
                (
                    "register/rows/sb-B/sb-B@1.json",
                    &a_row("page 1198", "sb-src-A"),
                ),
            ],
        );
        std::fs::remove_file(tree.at().join("register/rows/sb-B/sb-B@1.json"))
            .expect("a file this test just wrote");
        assert_eq!(
            note(every_locator_is_in_the_vocabulary(tree.at())),
            "1 row(s) read under register/rows/, 0 source(s) under register/sources/\n\
             1 record(s) could not be read"
        );
    }

    #[test]
    fn a_tracked_source_absent_from_the_working_copy_is_counted_rather_than_passed_over() {
        let tree = a_tree(
            "locator-source-gone",
            &[
                (
                    "register/rows/sb-A/sb-A@1.json",
                    &a_row("page 1197", "sb-src-CITED"),
                ),
                (
                    "register/sources/sb-src-CITED/sb-src-CITED@1.json",
                    &a_source("sb-src-CITED"),
                ),
                (
                    "register/sources/sb-src-GONE/sb-src-GONE@1.json",
                    &a_source("sb-src-GONE"),
                ),
            ],
        );
        std::fs::remove_file(
            tree.at()
                .join("register/sources/sb-src-GONE/sb-src-GONE@1.json"),
        )
        .expect("a file this test just wrote");
        assert_eq!(
            note(every_locator_is_in_the_vocabulary(tree.at())),
            "1 row(s) read under register/rows/, 1 source(s) under register/sources/\n\
             1 record(s) could not be read"
        );
    }

    #[test]
    fn a_source_this_cannot_read_is_counted_rather_than_named_as_uncited() {
        // A source whose bytes are not a record has no identifier to compare,
        // and reporting it as one no row cites would be a second refusal
        // wearing the report's clothes.
        let tree = a_tree(
            "locator-source-unreadable",
            &[
                (
                    "register/rows/sb-A/sb-A@1.json",
                    &a_row("page 1197", "sb-src-A"),
                ),
                ("register/sources/sb-src-A/sb-src-A@1.json", b"{\n"),
            ],
        );
        assert_eq!(
            note(every_locator_is_in_the_vocabulary(tree.at())),
            "1 row(s) read under register/rows/, 0 source(s) under register/sources/\n\
             1 record(s) could not be read"
        );
    }
}
