//! The canonical form a record is stored in.
//!
//! Curation is the work here and the review of a diff is where a transposed
//! digit gets caught, so a record has one spelling and not several. Two files
//! saying the same thing in different bytes make a diff that shows a reflow
//! where a reader is looking for the field that moved.
//!
//! `docs/decisions/means-format-and-language.md` fixes the form and this
//! renders it. The rule a formatter written from habit gets wrong is the last
//! one: a number is written back exactly as its token stood in the file, never
//! parsed and printed again. `1.1200` and `1.12` are different claims about the
//! fourth figure, and a round trip through a binary floating point number
//! destroys the first of them silently and uniformly across a whole register.
//! Nothing here converts a number, so there is no step whose result could
//! depend on the machine it ran on.
//!
//! Refusal surface. This module decides whether a record is refused for the
//! form it is written in, and
//! `docs/decisions/static-analysis-and-the-refusal-surface.md` is where that
//! line is given its meaning.

use std::fmt::Write as _;
use std::path::Path;

use crate::{Judged, tracked};

/// Where the records this judges live.
const REGISTER: &str = "register/";

/// A JSON value, holding what the canonical form has to preserve.
///
/// A number is the token as it stood in the file rather than a parsed number,
/// which is the whole of the trailing zero rule. A string is decoded, because
/// the form is a statement about the escapes and two spellings of one string
/// have to render alike.
enum Node {
    Object(Vec<(String, Node)>),
    Array(Vec<Node>),
    Text(String),
    Number(String),
    Truth(bool),
    Nothing,
}

/// Why a file has no canonical form.
///
/// Each variant is a separate refusal with its own sentence, because a curator
/// told only that a file was refused reads the rule and guesses.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Fault {
    /// The bytes are not UTF-8, so there is nothing to read.
    NotText,
    /// A byte order mark, which the form excludes and which a text editor adds
    /// without being asked.
    ByteOrderMark,
    /// The bytes are not JSON. The line is where the reading stopped.
    Unreadable { line: usize, why: String },
    /// One object carries two members with one name. Refused rather than
    /// resolved: this is the case where two parsers legitimately disagree, and
    /// no ordering rule saves it.
    TwoMembersWithOneName { line: usize, name: String },
}

impl Fault {
    /// The sentence a refusal prints for this fault.
    fn says(&self) -> String {
        match self {
            Self::NotText => "the bytes are not UTF-8".to_owned(),
            Self::ByteOrderMark => {
                "the file opens with a byte order mark, which the form excludes".to_owned()
            }
            Self::Unreadable { line, why } => format!("line {line}: {why}"),
            Self::TwoMembersWithOneName { line, name } => {
                format!("line {line}: two members named {name} in one object")
            }
        }
    }
}

/// The canonical form of `bytes`, or why they have none.
///
/// This is the formatter. It is reached only from the gate, so the form is
/// something a contributor is told about by a run rather than something they
/// have to remember to invoke.
///
/// # Errors
///
/// Returns the fault where the bytes are not a JSON value this form can hold.
pub(crate) fn canonical(bytes: &[u8]) -> Result<String, Fault> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(Fault::ByteOrderMark);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Fault::NotText)?;
    let mut scan = Scan { text, at: 0 };
    let node = scan.value()?;
    scan.space();
    if scan.at < text.len() {
        return Err(scan.stop("something follows the value the file holds"));
    }
    let mut out = String::with_capacity(text.len());
    write_node(&node, 0, &mut out);
    out.push('\n');
    Ok(out)
}

/// A reader over the text of one file.
struct Scan<'a> {
    text: &'a str,
    at: usize,
}

impl Scan<'_> {
    /// The line the reading has reached, counting from one.
    fn line(&self) -> usize {
        self.text[..self.at].matches('\n').count() + 1
    }

    /// A fault at the position the reading stopped.
    fn stop(&self, why: &str) -> Fault {
        Fault::Unreadable {
            line: self.line(),
            why: why.to_owned(),
        }
    }

    /// The byte at the reading position, if the text has not run out.
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    /// Steps over the whitespace JSON allows between tokens.
    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    /// Steps over `want`, or says what was there instead.
    fn take(&mut self, want: u8) -> Result<(), Fault> {
        if self.peek() == Some(want) {
            self.at += 1;
            return Ok(());
        }
        Err(self.stop(&format!("expected {}", char::from(want))))
    }

    /// Reads one value.
    fn value(&mut self) -> Result<Node, Fault> {
        self.space();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Node::Text(self.string()?)),
            Some(b't') => self.word("true").map(|()| Node::Truth(true)),
            Some(b'f') => self.word("false").map(|()| Node::Truth(false)),
            Some(b'n') => self.word("null").map(|()| Node::Nothing),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.stop("expected a value")),
            None => Err(self.stop("the file ends before its value does")),
        }
    }

    /// Steps over one of the three bare words.
    fn word(&mut self, want: &str) -> Result<(), Fault> {
        if self.text[self.at..].starts_with(want) {
            self.at += want.len();
            return Ok(());
        }
        Err(self.stop(&format!("expected {want}")))
    }

    /// Reads an object, keeping the line each member was written on.
    fn object(&mut self) -> Result<Node, Fault> {
        self.take(b'{')?;
        let mut members: Vec<(String, Node)> = Vec::new();
        let mut lines: Vec<usize> = Vec::new();
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Node::Object(members));
        }
        loop {
            self.space();
            let line = self.line();
            let name = self.string()?;
            self.space();
            self.take(b':')?;
            let value = self.value()?;
            if let Some(at) = members.iter().position(|(seen, _)| *seen == name) {
                return Err(Fault::TwoMembersWithOneName {
                    line: lines[at].max(line),
                    name,
                });
            }
            members.push((name, value));
            lines.push(line);
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Node::Object(members));
                }
                _ => return Err(self.stop("expected , or } after a member")),
            }
        }
    }

    /// Reads an array.
    fn array(&mut self) -> Result<Node, Fault> {
        self.take(b'[')?;
        let mut elements = Vec::new();
        self.space();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Node::Array(elements));
        }
        loop {
            elements.push(self.value()?);
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Node::Array(elements));
                }
                _ => return Err(self.stop("expected , or ] after an element")),
            }
        }
    }

    /// Reads a string, decoding its escapes.
    fn string(&mut self) -> Result<String, Fault> {
        self.take(b'"')?;
        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.stop("the file ends inside a string"));
            };
            match byte {
                b'"' => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.at += 1;
                    out.push(self.escape()?);
                }
                0x00..=0x1F => {
                    return Err(self.stop("a control character inside a string is not escaped"));
                }
                _ => {
                    let rest = &self.text[self.at..];
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.stop("the file ends inside a string"))?;
                    self.at += character.len_utf8();
                    out.push(character);
                }
            }
        }
    }

    /// Reads one escape, after its backslash.
    fn escape(&mut self) -> Result<char, Fault> {
        let Some(byte) = self.peek() else {
            return Err(self.stop("the file ends inside an escape"));
        };
        self.at += 1;
        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{8}'),
            b'f' => Ok('\u{c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.escaped_code_point(),
            _ => Err(self.stop("an escape this grammar does not have")),
        }
    }

    /// Reads a `\u` escape and the low half of a surrogate pair where one
    /// follows, which is the only way a JSON file can carry a character above
    /// the basic plane.
    fn escaped_code_point(&mut self) -> Result<char, Fault> {
        let first = self.four_hex_digits()?;
        if !(0xD800..0xDC00).contains(&first) {
            return char::from_u32(first).ok_or_else(|| self.stop("not a character"));
        }
        if !self.text[self.at..].starts_with("\\u") {
            return Err(self.stop("half of a surrogate pair with no other half"));
        }
        self.at += 2;
        let second = self.four_hex_digits()?;
        if !(0xDC00..0xE000).contains(&second) {
            return Err(self.stop("half of a surrogate pair with no other half"));
        }
        let joined = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        char::from_u32(joined).ok_or_else(|| self.stop("not a character"))
    }

    /// Reads the four hexadecimal digits of a `\u` escape.
    ///
    /// Every one of the four is checked to be a hexadecimal digit before the
    /// number is read, because the ordinary way of reading one accepts a
    /// leading sign and would take a spelling this grammar does not have.
    fn four_hex_digits(&mut self) -> Result<u32, Fault> {
        let rest = self.text.get(self.at..self.at + 4);
        let digits = rest.ok_or_else(|| self.stop("an escape with fewer than four digits"))?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(self.stop("an escape whose digits are not hexadecimal"));
        }
        let value = u32::from_str_radix(digits, 16)
            .map_err(|_| self.stop("an escape whose digits are not hexadecimal"))?;
        self.at += 4;
        Ok(value)
    }

    /// Reads a number and keeps its token.
    ///
    /// The token is taken as it stands. Nothing here turns it into a number, so
    /// there is no representation for it to lose figures to and no platform
    /// difference for it to pick up.
    fn number(&mut self) -> Result<Node, Fault> {
        let from = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => self.digits(),
            _ => return Err(self.stop("a number with no digits")),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.stop("a number with no digits after its point"));
            }
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.stop("a number with no digits in its exponent"));
            }
            self.digits();
        }
        Ok(Node::Number(self.text[from..self.at].to_owned()))
    }

    /// Steps over a run of digits.
    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
    }
}

/// Writes one value in the canonical form.
///
/// An object or an array with nothing in it is written on one line, because the
/// rule puts one member or element on a line and such a value has none. The
/// alternative is a line holding only indentation, which the same rule forbids.
fn write_node(node: &Node, depth: usize, out: &mut String) {
    match node {
        Node::Object(members) => write_object(members, depth, out),
        Node::Array(elements) => write_array(elements, depth, out),
        Node::Text(text) => write_text(text, out),
        Node::Number(token) => out.push_str(token),
        Node::Truth(true) => out.push_str("true"),
        Node::Truth(false) => out.push_str("false"),
        Node::Nothing => out.push_str("null"),
    }
}

/// Writes an object with its members in the order their names sort in.
fn write_object(members: &[(String, Node)], depth: usize, out: &mut String) {
    if members.is_empty() {
        out.push_str("{}");
        return;
    }
    let mut sorted: Vec<&(String, Node)> = members.iter().collect();
    sorted.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    out.push_str("{\n");
    for (at, (name, value)) in sorted.iter().enumerate() {
        indent(depth + 1, out);
        write_text(name, out);
        out.push_str(": ");
        write_node(value, depth + 1, out);
        if at + 1 < sorted.len() {
            out.push(',');
        }
        out.push('\n');
    }
    indent(depth, out);
    out.push('}');
}

/// Writes an array, keeping the order it was written in.
fn write_array(elements: &[Node], depth: usize, out: &mut String) {
    if elements.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    for (at, element) in elements.iter().enumerate() {
        indent(depth + 1, out);
        write_node(element, depth + 1, out);
        if at + 1 < elements.len() {
            out.push(',');
        }
        out.push('\n');
    }
    indent(depth, out);
    out.push(']');
}

/// Writes two spaces per level of nesting.
fn indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// Writes a string with the escapes the grammar requires and no others.
///
/// The hexadecimal digits of a `\u` escape are lower case, so that one string
/// has one spelling. Everything a JSON string may carry unescaped is written
/// unescaped, which keeps a curator's own words readable in the file.
fn write_text(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// What a run says about the first place a file and its form part company.
///
/// The comparison is over bytes rather than over lines, because the two
/// differences a line comparison cannot see are exactly the two this form is
/// most often broken by: a line ending and a final line feed. Where the byte
/// that differs does not print, the sentence says so rather than showing a
/// reader two lines that look identical.
fn first_difference(found: &str, wanted: &str) -> Option<String> {
    if found == wanted {
        return None;
    }
    let at = found
        .as_bytes()
        .iter()
        .zip(wanted.as_bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| found.len().min(wanted.len()));
    let line = found[..at.min(found.len())].matches('\n').count() + 1;
    let has = nth_line(found, line);
    let wants = nth_line(wanted, line);
    if has == wants {
        return Some(format!(
            "line {line} differs from the form in a byte that does not print, \
             such as a line ending or the line feed the file ends with"
        ));
    }
    Some(format!(
        "line {line} reads   {has}\n      the form is   {wants}"
    ))
}

/// One line of a rendering, counting from one, or a sentence where the
/// rendering has no such line.
fn nth_line(text: &str, line: usize) -> String {
    text.lines().nth(line - 1).map_or_else(
        || "nothing, the text ends above here".to_owned(),
        str::to_owned,
    )
}

/// Refuses a record that is not stored in the canonical form.
///
/// The subject is every tracked file under `register/` with the format's
/// extension. What is under `register/` is the published product, and the form
/// is a promise made about that product rather than about this project's own
/// files, so nothing else is read and the count says how many were.
pub(crate) fn every_record_is_in_the_canonical_form(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let mut wrong = Vec::new();
    let mut read = 0_usize;
    let mut unread = 0_usize;

    for name in tracked.iter().filter(|name| is_a_record(name)) {
        let Ok(bytes) = std::fs::read(root.join(name)) else {
            unread += 1;
            continue;
        };
        read += 1;
        match canonical(&bytes) {
            Err(fault) => wrong.push(format!("  {name}\n      {}", fault.says())),
            Ok(form) => {
                let found = String::from_utf8_lossy(&bytes);
                if let Some(said) = first_difference(&found, &form) {
                    wrong.push(format!("  {name}\n      {said}"));
                }
            }
        }
    }

    if !wrong.is_empty() {
        return Judged::Refused(refusal(&wrong));
    }

    let mut note = format!("{read} record(s) read under {REGISTER}");
    if unread > 0 {
        let _ = write!(note, ", {unread} tracked but not in the working copy");
    }
    Judged::Nothing(Some(note))
}

/// What a run says when a record is not in the form.
fn refusal(wrong: &[String]) -> String {
    let mut why = format!(
        "{} record(s) are not in the canonical form:\n\n",
        wrong.len()
    );
    for one in wrong {
        let _ = writeln!(why, "{one}");
    }
    let _ = writeln!(
        why,
        "\nThe form is fixed in docs/decisions/means-format-and-language.md and a \
         record\nstored in it makes a diff that shows the field that changed rather \
         than a\nreflow. Two spaces per level of nesting, one member or element to a \
         line,\nobject keys in the order their names sort in, and a number written back \
         exactly\nas its digits stood."
    );
    why
}

/// Whether a tracked path is a record this form is a promise about.
fn is_a_record(name: &str) -> bool {
    name.starts_with(REGISTER)
        && Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fixture below is a byte string in this file rather than a file in
    /// the tree. The form is a statement about line endings and trailing bytes,
    /// and a fixture carrying those as a tracked file would have them
    /// normalised on the way in and would prove nothing.
    /// `docs/decisions/fixtures-and-what-a-test-may-be-about.md` is where that
    /// convention is written down.
    fn canonical_row() -> &'static [u8] {
        b"{\n  \"digits\": \"1.1200\",\n  \"unit\": \"eV\"\n}\n"
    }

    #[test]
    fn a_record_in_the_form_is_left_exactly_as_it_stands() {
        let form = canonical(canonical_row()).expect("a row this form can hold");
        assert_eq!(form.as_bytes(), canonical_row());
    }

    #[test]
    fn keys_out_of_order_are_not_the_form_and_the_form_puts_them_back() {
        // The near neighbour of the row above, differing in the order of two
        // members and in nothing else.
        let swapped = b"{\n  \"unit\": \"eV\",\n  \"digits\": \"1.1200\"\n}\n";
        let form = canonical(swapped).expect("a row this form can hold");
        assert_ne!(form.as_bytes(), swapped.as_slice());
        assert_eq!(form.as_bytes(), canonical_row());
    }

    #[test]
    fn the_indentation_is_two_spaces_a_level_and_four_is_refused() {
        let four = b"{\n    \"digits\": \"1.1200\",\n    \"unit\": \"eV\"\n}\n";
        let form = canonical(four).expect("a row this form can hold");
        assert_eq!(form.as_bytes(), canonical_row());
    }

    #[test]
    fn carriage_returns_are_not_the_form() {
        // Byte for byte the row above as a converting checkout writes it.
        let written = b"{\r\n  \"digits\": \"1.1200\",\r\n  \"unit\": \"eV\"\r\n}\r\n";
        let form = canonical(written).expect("a row this form can hold");
        assert_eq!(form.as_bytes(), canonical_row());
    }

    #[test]
    fn a_missing_trailing_line_feed_is_not_the_form() {
        let without = b"{\n  \"digits\": \"1.1200\",\n  \"unit\": \"eV\"\n}";
        let form = canonical(without).expect("a row this form can hold");
        assert_eq!(form.as_bytes(), canonical_row());
    }

    #[test]
    fn a_number_with_more_figures_than_a_double_carries_survives() {
        // The case the form exists for. Under the representation a formatter
        // written from habit uses, this token comes back as 1 and the claim
        // about the twenty-ninth figure is gone, silently.
        let digits = "1.00000000000000000000000000001";
        let row = format!("{{\n  \"value\": {digits}\n}}\n");
        let form = canonical(row.as_bytes()).expect("a row this form can hold");
        assert_eq!(form, row);
        assert!(form.contains(digits), "{form}");

        let naive = digits.parse::<f64>().expect("a number a double will take");
        assert_ne!(
            naive.to_string(),
            digits,
            "the naive representation would have to lose this for the test to mean anything"
        );
    }

    #[test]
    fn a_trailing_zero_survives_and_a_double_would_not_have_kept_it() {
        let row = "{\n  \"digits\": 0.020\n}\n";
        assert_eq!(canonical(row.as_bytes()).expect("a row"), row);
        let naive = "0.020".parse::<f64>().expect("a number a double will take");
        assert_eq!(naive.to_string(), "0.02");
    }

    #[test]
    fn an_exponent_keeps_the_spelling_it_was_written_with() {
        let row = "{\n  \"digits\": 1.5E+03\n}\n";
        assert_eq!(canonical(row.as_bytes()).expect("a row"), row);
    }

    #[test]
    fn two_members_with_one_name_are_refused_rather_than_resolved() {
        let twice = b"{\n  \"unit\": \"eV\",\n  \"unit\": \"meV\"\n}\n";
        assert_eq!(
            canonical(twice),
            Err(Fault::TwoMembersWithOneName {
                line: 3,
                name: "unit".to_owned(),
            })
        );
    }

    #[test]
    fn two_members_with_names_that_merely_look_alike_are_not_refused() {
        // The near neighbour. A check that compared names loosely would refuse
        // an object carrying a value and its unit, which is most of a row.
        let neighbour = b"{\n  \"unit\": \"eV\",\n  \"units\": \"meV\"\n}\n";
        assert!(canonical(neighbour).is_ok());
    }

    #[test]
    fn a_byte_order_mark_is_refused_by_name() {
        let mut with = vec![0xEF, 0xBB, 0xBF];
        with.extend_from_slice(canonical_row());
        assert_eq!(canonical(&with), Err(Fault::ByteOrderMark));
        assert!(canonical(canonical_row()).is_ok(), "the neighbour");
    }

    #[test]
    fn bytes_that_are_not_text_are_refused_by_name() {
        assert_eq!(canonical(&[0x7B, 0xFF, 0x7D]), Err(Fault::NotText));
    }

    #[test]
    fn a_file_that_is_not_json_is_refused_at_the_line_the_reading_stopped_on() {
        let broken = b"{\n  \"digits\": \"1.1200\",\n  \"unit\": eV\n}\n";
        let Err(Fault::Unreadable { line, why }) = canonical(broken) else {
            panic!("a file that is not json");
        };
        assert_eq!(line, 3);
        assert!(why.contains("expected a value"), "{why}");
    }

    #[test]
    fn something_after_the_value_the_file_holds_is_refused() {
        // One record per file is the whole of the layout, so a second value
        // after the first is a file two readers disagree about. The form
        // written back would be the first value alone, with everything after
        // it gone and nothing in the diff to say so.
        let two = b"{\n  \"digits\": \"1.1200\"\n}\n{\n  \"digits\": \"1.13\"\n}\n";
        let Err(Fault::Unreadable { line, why }) = canonical(two) else {
            panic!("a file holding two values");
        };
        assert_eq!(line, 4);
        assert!(why.contains("something follows the value"), "{why}");
        assert!(
            canonical(b"{\n  \"digits\": \"1.1200\"\n}\n").is_ok(),
            "the neighbour, which is the same first value with nothing after it"
        );
    }

    #[test]
    fn an_escape_whose_digits_are_not_hexadecimal_is_refused() {
        // The ordinary way of reading a number in a base accepts a leading
        // sign, so this spelling is read as the character A by a reader that
        // hands its four bytes straight over, and the file passes carrying an
        // escape the grammar does not have.
        let signed = b"{\n  \"note\": \"\\u+041\"\n}\n";
        let Err(Fault::Unreadable { why, .. }) = canonical(signed) else {
            panic!("an escape whose digits are not hexadecimal");
        };
        assert!(why.contains("not hexadecimal"), "{why}");
        let neighbour = b"{\n  \"note\": \"\\u0041\"\n}\n";
        assert_eq!(
            canonical(neighbour).expect("a row"),
            "{\n  \"note\": \"A\"\n}\n"
        );
    }

    #[test]
    fn a_refusal_names_which_of_the_faults_it_is() {
        // A check that started refusing everything passes a test that only
        // asked whether something was refused, so every fixture above asserts
        // the fault and this asserts that the four say different things.
        let said = [
            Fault::NotText.says(),
            Fault::ByteOrderMark.says(),
            Fault::Unreadable {
                line: 3,
                why: "expected a value".to_owned(),
            }
            .says(),
            Fault::TwoMembersWithOneName {
                line: 3,
                name: "unit".to_owned(),
            }
            .says(),
        ];
        for (at, one) in said.iter().enumerate() {
            assert!(!one.is_empty());
            assert!(
                !said[at + 1..].contains(one),
                "two faults say the same thing: {one}"
            );
        }
    }

    #[test]
    fn a_negative_number_keeps_its_sign() {
        // A coefficient in a published fit is as often negative as not, and
        // the sign is a byte the reader steps over rather than one it writes
        // down. A reader that steps over it wrongly either refuses the row or
        // reads a different number out of it, and both are silent.
        let row = "{\n  \"digits\": -1.1200\n}\n";
        assert_eq!(canonical(row.as_bytes()).expect("a row"), row);
        let exponent = "{\n  \"digits\": 1.5e-03\n}\n";
        assert_eq!(canonical(exponent.as_bytes()).expect("a row"), exponent);
    }

    #[test]
    fn every_escape_the_grammar_has_is_read_and_written_back() {
        // One row per escape rather than one row carrying all of them, so a
        // failure names the escape the reader lost. Each of these is already
        // the form's own spelling, so the row it is in comes back byte for
        // byte; a reader that has forgotten one refuses a record for being
        // correct, which is the refusal a curator has no way to act on.
        for escape in ["\\\"", "\\\\", "\\b", "\\f", "\\n", "\\r", "\\t"] {
            let row = format!("{{\n  \"note\": \"a{escape}b\"\n}}\n");
            let form = canonical(row.as_bytes())
                .unwrap_or_else(|fault| panic!("{escape}: {}", fault.says()));
            assert_eq!(form, row, "{escape}");
        }
    }

    #[test]
    fn a_control_character_inside_a_string_is_refused_and_its_escape_is_not() {
        // A tab pasted into a note out of a table is how this arrives. Taken
        // rather than refused, the byte reaches the register unescaped, where
        // the grammar does not allow it and the next reader will not take the
        // file at all.
        let raw = b"{\n  \"note\": \"two\tcolumns\"\n}\n";
        let Err(Fault::Unreadable { line, why }) = canonical(raw) else {
            panic!("a control character inside a string");
        };
        assert_eq!(line, 2);
        assert!(why.contains("control character"), "{why}");
        let escaped = "{\n  \"note\": \"two\\tcolumns\"\n}\n";
        assert_eq!(canonical(escaped.as_bytes()).expect("a row"), escaped);
    }

    #[test]
    fn an_escape_the_grammar_does_not_require_is_written_out() {
        // A solidus may be escaped and need not be, so two spellings of one
        // string reach the tree unless the form picks one.
        let escaped = b"{\n  \"locator\": \"table 2\\/3\"\n}\n";
        let form = canonical(escaped).expect("a row");
        assert_eq!(form, "{\n  \"locator\": \"table 2/3\"\n}\n");
    }

    #[test]
    fn an_escape_the_grammar_does_require_is_kept_and_its_digits_are_lower_case() {
        let upper = b"{\n  \"note\": \"a\\u001Fb\"\n}\n";
        let form = canonical(upper).expect("a row");
        assert_eq!(form, "{\n  \"note\": \"a\\u001fb\"\n}\n");
    }

    #[test]
    fn a_character_above_the_basic_plane_survives_its_surrogate_pair() {
        // Written as a pair because that is the only way a JSON escape reaches
        // above the basic plane, and read back as the one character it is.
        let pair = b"{\n  \"note\": \"\\ud83d\\udd2c\"\n}\n";
        let one = char::from_u32(0x0001_F52C).expect("a character");
        let form = canonical(pair).expect("a row");
        assert_eq!(form, format!("{{\n  \"note\": \"{one}\"\n}}\n"));
    }

    #[test]
    fn half_a_surrogate_pair_with_no_other_half_is_refused() {
        let lonely = b"{\n  \"note\": \"\\ud83d\"\n}\n";
        let Err(Fault::Unreadable { why, .. }) = canonical(lonely) else {
            panic!("half a pair");
        };
        assert!(why.contains("surrogate pair"), "{why}");
    }

    #[test]
    fn an_empty_object_and_an_empty_array_are_written_on_one_line() {
        let empties = "{\n  \"chain\": [],\n  \"conditions\": {}\n}\n";
        assert_eq!(canonical(empties.as_bytes()).expect("a row"), empties);
    }

    #[test]
    fn an_array_keeps_the_order_it_was_written_in() {
        // A chain is ordered from where the curator got the value back to the
        // earliest source, so sorting one would change what the row says.
        let chain = "{\n  \"chain\": [\n    \"second\",\n    \"first\"\n  ]\n}\n";
        assert_eq!(canonical(chain.as_bytes()).expect("a row"), chain);
    }

    #[test]
    fn only_a_file_under_the_register_with_the_extension_is_a_record() {
        assert!(is_a_record(
            "register/rows/sb-4KQ7N2XJ0P/sb-4KQ7N2XJ0P@1.json"
        ));
        assert!(!is_a_record(
            "register/rows/sb-4KQ7N2XJ0P/sb-4KQ7N2XJ0P@1.tsv"
        ));
        assert!(!is_a_record("schema/measured-value.schema.json"));
        assert!(!is_a_record(
            "crates/stoffbuch-schema/fixtures/x/valid.json"
        ));
    }

    #[test]
    fn the_difference_reported_is_the_first_one() {
        let found = "a\nb\nc\n";
        let said = first_difference(found, "a\nB\nC\n").expect("two texts that differ");
        assert!(said.starts_with("line 2 reads   b"), "{said}");
        assert!(said.contains("the form is   B"), "{said}");
        assert_eq!(first_difference(found, found), None);
    }

    #[test]
    fn a_file_carrying_a_line_the_form_does_not_have_is_reported_at_that_line() {
        // Where one rendering runs on past the end of the other there is no
        // differing byte to point at, so the position taken is the end of the
        // shorter one. Taken at the end of the longer one instead, the line
        // reported is past the end of both, and both sides read as nothing,
        // which prints the sentence for a byte that does not show and hides
        // the line a curator has to go and look at.
        let found = "a\nb\nc\n";
        let said = first_difference(found, "a\nb\n").expect("two texts that differ");
        assert!(said.starts_with("line 3 reads   c"), "{said}");
        assert!(
            said.contains("the form is   nothing, the text ends above here"),
            "{said}"
        );
    }

    #[test]
    fn a_difference_no_reader_could_see_is_said_to_be_one() {
        // The near miss for the report itself. A line comparison would return
        // nothing here, and a run that printed nothing while refusing a file
        // is one nobody can act on.
        let said = first_difference("a\r\nb\n", "a\nb\n").expect("two texts that differ");
        assert!(said.contains("does not print"), "{said}");
        let ending = first_difference("a\nb", "a\nb\n").expect("two texts that differ");
        assert!(ending.contains("does not print"), "{ending}");
    }

    #[test]
    fn every_record_in_this_tree_reads_and_writes_back_to_the_same_bytes() {
        // Over every record rather than over an example: the set is walked out
        // of the tree, so a record added to it is covered without anybody
        // remembering to add it here. The tree is walked rather than asked of
        // git, because a test that asked would fail in any copy of this tree
        // that is not a checkout.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let mut seen = 0_usize;
        for file in records_under(&root) {
            let bytes = std::fs::read(&file).expect("a record that is in this tree");
            let form = canonical(&bytes)
                .unwrap_or_else(|fault| panic!("{}: {}", file.display(), fault.says()));
            assert_eq!(
                form.as_bytes(),
                bytes.as_slice(),
                "{} is not in the canonical form",
                file.display()
            );
            seen += 1;
        }
        assert!(seen > 0, "no record was read, so this proved nothing");
    }

    /// Every file in this tree the canonical form is a promise about: what is
    /// under `register/`, and the row fixtures, which are rows in the stored
    /// format and are the only ones the tree holds today.
    fn records_under(root: &Path) -> Vec<std::path::PathBuf> {
        let register = root.join("register");
        let mut found = Vec::new();
        walk(&register, &mut found);
        walk(&root.join("crates"), &mut found);
        found.retain(|path| {
            let is_json = path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
            let is_a_fixture = path.components().any(|part| part.as_os_str() == "fixtures");
            is_json && (is_a_fixture || path.starts_with(&register))
        });
        found.sort();
        found
    }

    /// Every file under a directory, however deep, and nothing if it is absent.
    fn walk(at: &Path, into: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(at) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, into);
            } else {
                into.push(path);
            }
        }
    }

    // The check, given a tree.
    //
    // Everything above this line hands the formatter its bytes directly. The
    // check itself asks git what is tracked, so the only way to give it a
    // subject is to build a tree and index one, and until it was given one the
    // counters in its report, the branch that chooses between refusing and
    // passing, and the filter deciding which files it reads were all reachable
    // by nothing. The builder is the gate's own rather than a second one
    // written here, so there is one answer to what a tree is.

    use crate::tests::{a_tree, note, somewhere};

    /// One record, in the form.
    const ROW: &[u8] = b"{\n  \"digits\": \"1.1200\",\n  \"unit\": \"eV\"\n}\n";

    /// What the check wrote when it refused.
    ///
    /// Named apart from this module's own `refusal`, which is the text rather
    /// than the reading of a verdict.
    fn what_it_wrote(judged: Judged) -> String {
        crate::tests::refusal(judged)
    }

    #[test]
    fn a_tree_of_records_in_the_form_is_reported_with_the_count_of_what_was_read() {
        // The two files beside the record are the ones that decide what the
        // subject is. A block is under the register and is not this format, and
        // a schema is this format and is not under the register, so a check
        // reading either of them is reading something the form is not a promise
        // about and its count says it examined more than it did.
        let tree = a_tree(
            "canonical-clean",
            &[
                ("register/rows/sb-A/sb-A@1.json", ROW),
                (
                    "register/rows/sb-A/sb-A@1.n-and-k.tsv",
                    b"T\tn\n300\t3.42\n",
                ),
                ("schema/measured-value.schema.json", b"{\n  \"a\": 1\n}\n"),
            ],
        );
        assert_eq!(
            note(every_record_is_in_the_canonical_form(tree.at())),
            "1 record(s) read under register/"
        );
    }

    #[test]
    fn a_record_out_of_the_form_is_refused_with_the_file_and_the_line() {
        let swapped = b"{\n  \"unit\": \"eV\",\n  \"digits\": \"1.1200\"\n}\n";
        let tree = a_tree(
            "canonical-out-of-order",
            &[("register/rows/sb-A/sb-A@1.json", swapped)],
        );
        let why = what_it_wrote(every_record_is_in_the_canonical_form(tree.at()));
        assert!(
            why.contains("1 record(s) are not in the canonical form"),
            "{why}"
        );
        assert!(why.contains("  register/rows/sb-A/sb-A@1.json"), "{why}");
        assert!(why.contains("line 2 reads"), "{why}");
    }

    #[test]
    fn a_record_that_is_not_this_format_at_all_is_refused_by_the_fault_it_has() {
        // A row written in the format the format decision rejected, which is
        // the mistake a curator coming from the neighbouring collection makes.
        let tree = a_tree(
            "canonical-not-json",
            &[("register/rows/sb-A/sb-A@1.json", b"digits: 1.1200\n")],
        );
        let why = what_it_wrote(every_record_is_in_the_canonical_form(tree.at()));
        assert!(why.contains("line 1: expected a value"), "{why}");
    }

    #[test]
    fn a_tracked_record_absent_from_the_working_copy_is_counted_rather_than_passed_over() {
        let tree = a_tree(
            "canonical-absent",
            &[
                ("register/rows/sb-A/sb-A@1.json", ROW),
                ("register/rows/sb-B/sb-B@1.json", ROW),
            ],
        );
        std::fs::remove_file(tree.at().join("register/rows/sb-B/sb-B@1.json"))
            .expect("a file this test just wrote");
        assert_eq!(
            note(every_record_is_in_the_canonical_form(tree.at())),
            "1 record(s) read under register/, 1 tracked but not in the working copy"
        );
    }

    #[test]
    fn a_directory_git_knows_nothing_about_is_not_reported_as_a_register_with_no_records() {
        // The near miss for the whole apparatus. `git ls-files` fails outside a
        // checkout, and a check reading that failure as an empty list would
        // report every record clean without having read one.
        let at = somewhere("canonical-not-a-repository");
        let judged = every_record_is_in_the_canonical_form(&at);
        let _ = std::fs::remove_dir_all(&at);
        assert!(
            matches!(judged, Judged::CouldNotJudge(_)),
            "expected a run that could not judge, and got {judged:?}"
        );
    }
}
