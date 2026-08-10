//! The schema check.
//!
//! Decides whether a file is a well formed row: its shape, its required
//! fields, its patterns and its canonical serialisation. It is separate from
//! the gate because it is also what a contributor runs against one file, and
//! separate from the library because a consumer reading a released register
//! should not have to carry the machinery that refuses a bad one.
//!
//! The schema exists and the check does not. What this crate holds today is
//! the fixtures under `fixtures/`, one directory per record kind, and the suite
//! below that keeps them honest. Nothing here reads a schema or decides a
//! refusal; the fixtures are run against the schema by a validator written by
//! somebody else, which is the property the format was chosen for, and the
//! command that does it is in the pull request that landed them.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// The workspace root, from this crate's own manifest directory.
    ///
    /// The tree is walked rather than asked of git, because a test that asked
    /// would fail in any copy of this tree that is not a checkout, and a
    /// mutation run works in exactly such a copy.
    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
    }

    /// One record kind's schema and the fixtures that stand for it.
    struct Kind {
        /// The schema file, under `schema/` at the workspace root.
        file: &'static str,
        /// The `$id` that file carries. Its last segment is the schema's
        /// version, and a version is a new file beside the old one rather than
        /// an edit to it, so this string moving is a change the suite sees.
        id: &'static str,
        /// The fixture directory, under `fixtures/` in this crate.
        fixtures: &'static str,
        /// The one fixture the schema accepts.
        accepted: &'static str,
        /// Every fixture the schema refuses. The file name is the reason.
        refused: &'static [&'static str],
    }

    /// Every record kind that has a schema.
    ///
    /// This is what the two set comparisons below are made against, so a schema
    /// or a fixture added without an entry here reddens the suite, and an entry
    /// naming a file that is not in the tree reddens it from the other side.
    const KINDS: &[Kind] = &[
        Kind {
            file: "measured-value.schema.json",
            id: "urn:stoffbuch:schema:measured-value:1",
            fixtures: "measured-value",
            accepted: "valid.json",
            refused: &[
                "declared-kind-is-another-kind.json",
                "missing-required-field.json",
                "unit-is-not-in-the-accepted-syntax.json",
                "unknown-field.json",
                "value-is-not-a-number.json",
            ],
        },
        Kind {
            file: "source.schema.json",
            id: "urn:stoffbuch:schema:source:1",
            fixtures: "source",
            accepted: "valid.json",
            refused: &[
                "a-standard-without-what-a-standard-needs.json",
                "arxiv-identifier-is-not-in-the-form-it-declares.json",
                "declared-kind-is-another-kind.json",
                "doi-is-not-in-the-form-it-declares.json",
                "id-is-a-row-identifier.json",
                "identifier-form-is-not-in-the-vocabulary.json",
                "isbn-is-not-in-the-form-it-declares.json",
                "missing-required-field.json",
                "report-designation-is-not-in-the-form-it-declares.json",
                "standard-designation-carries-no-year.json",
                "type-is-not-in-the-vocabulary.json",
                "unknown-field.json",
                "year-is-not-a-year.json",
            ],
        },
    ];

    /// The dialect every schema in this tree is written in.
    const DIALECT: &str = "\"$schema\": \"https://json-schema.org/draft/2020-12/schema\"";

    /// Every file directly inside a directory, sorted.
    fn listing(directory: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(directory)
            .expect("a directory that is in this tree")
            .map(|entry| {
                entry
                    .expect("a readable entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn every_schema_in_the_tree_is_one_the_suite_knows_about() {
        let mut declared: Vec<String> = KINDS.iter().map(|kind| kind.file.to_owned()).collect();
        declared.sort();
        assert_eq!(
            listing(&root().join("schema")),
            declared,
            "a schema with no entry in the suite, or an entry naming no file"
        );
    }

    #[test]
    fn every_fixture_in_the_tree_is_one_the_suite_knows_about() {
        for kind in KINDS {
            let mut declared: Vec<String> = kind
                .refused
                .iter()
                .chain(std::iter::once(&kind.accepted))
                .map(|name| (*name).to_owned())
                .collect();
            declared.sort();
            let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join(kind.fixtures);
            assert_eq!(
                listing(&directory),
                declared,
                "{}: a fixture with no entry in the suite, or an entry naming no file",
                kind.fixtures
            );
        }
    }

    #[test]
    fn every_schema_carries_the_dialect_and_the_identifier_the_suite_holds() {
        // The identifier is where the version lives, and the rule written into
        // the schema is that a new version is a new file rather than an edit to
        // this one. A version raised in place moves this string and reddens
        // here, which is the half of that rule a machine can hold. Whether the
        // constraints changed without the version moving is the review's.
        for kind in KINDS {
            let text = std::fs::read_to_string(root().join("schema").join(kind.file))
                .expect("a schema that is in this tree");
            assert!(text.contains(DIALECT), "{} names no dialect", kind.file);
            let identifier = format!("\"$id\": \"{}\"", kind.id);
            assert!(
                text.contains(&identifier),
                "{} does not carry {}",
                kind.file,
                kind.id
            );
        }
    }

    /// The lines of `later` that are not in `earlier`, counting repeats.
    fn added<'a>(earlier: &[&str], later: &[&'a str]) -> Vec<&'a str> {
        let mut left: Vec<&str> = earlier.to_vec();
        let mut extra = Vec::new();
        for line in later {
            if let Some(at) = left.iter().position(|other| other == line) {
                left.remove(at);
            } else {
                extra.push(*line);
            }
        }
        extra
    }

    #[test]
    fn every_refused_fixture_is_a_near_neighbour_of_the_accepted_one() {
        // A fixture that could not plausibly have been written by mistake
        // proves less than one that nearly is correct. Each refused fixture
        // here is the accepted one with a single line changed, added or taken
        // away, so what the schema refused is that line and not the twenty
        // other ways the file could have been wrong at the same time.
        for kind in KINDS {
            let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join(kind.fixtures);
            let accepted = std::fs::read_to_string(directory.join(kind.accepted))
                .expect("the accepted fixture is in this tree");
            let accepted: Vec<&str> = accepted.lines().collect();
            for name in kind.refused {
                let text = std::fs::read_to_string(directory.join(name))
                    .expect("a refused fixture that is in this tree");
                let refused: Vec<&str> = text.lines().collect();
                let gained = added(&accepted, &refused);
                let lost = added(&refused, &accepted);
                assert!(
                    !gained.is_empty() || !lost.is_empty(),
                    "{name} is the accepted fixture and refuses nothing"
                );
                assert!(
                    gained.len() <= 1 && lost.len() <= 1,
                    "{name} differs from {} in more than one line: gained {gained:?}, lost {lost:?}",
                    kind.accepted
                );
            }
        }
    }

    #[test]
    fn a_fixture_differing_in_two_lines_is_not_a_near_neighbour() {
        // The near miss for the comparison itself. A test that only counted
        // whether two files differ would pass a fixture rewritten end to end,
        // and such a fixture proves nothing about which line the schema
        // refused.
        let accepted = ["a", "b", "c"];
        let one_line = ["a", "x", "c"];
        let two_lines = ["a", "x", "y"];
        assert_eq!(added(&accepted, &one_line).len(), 1);
        assert_eq!(added(&one_line, &accepted).len(), 1);
        assert_eq!(added(&accepted, &two_lines).len(), 2);
    }
}
