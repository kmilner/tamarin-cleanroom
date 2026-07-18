//! R5 — the theory-path grammar (URL <-> structured path).
//!
//! The wildcard tail after `/thy/trace/<idx>/main/` (and the same grammar the
//! `del/path/…` + `verify/…` routes take) maps to a structured [`ThyPath`].
//! Everything below is pinned by the round-2 probes (QUERIES.log [S14][L08]–
//! [L13]) — the corpus href inventory on the render side and the live
//! acceptance battery on the parse side.
//!
//! ## Segment model
//! The raw tail splits on `/` FIRST; each segment is then percent-decoded
//! independently (`me%73sage` matches `message`; an encoded `%2F` does NOT
//! split — `proof/foo%2F_` names the lemma `foo/_` [L09]). Decoding: a valid
//! `%XX` pair becomes that byte and the byte sequence is read as UTF-8 with
//! U+FFFD replacement for invalid sequences [L12]; an INVALID percent sequence
//! stays literal (`a%zzb`, trailing `a%`, `a%2` [L12]); `+` is NOT a space
//! [L12]. Head keywords match exactly and case-sensitively (`MESSAGE`,
//! `cases/RAW` rejected [L08][L09]).
//!
//! ## Acceptance shape
//! Extra segments AFTER a complete match are ignored (`help/extra`,
//! `message/`, `cases/raw/0/0/extra`, `lemma/foo/extra` all accepted
//! [L09][L11]); a missing required argument rejects (`proof`, `cases/raw/1`
//! [L08][L11]); name arguments accept ANY decoded text including the empty
//! string (`edit/`, `proof//_` accepted [L11]) — existence of the named lemma
//! is a resolution question, not a parse question [L08].
//!
//! ## The numeric-segment grammar ([L10])
//! The two `cases` indices parse like a Haskell `reads @Int` prefix: optional
//! whitespace, balanced parens, an optional `-` (whitespace allowed after it),
//! then one integer lexeme — decimal, or `0x`/`0X` hex, `0o`/`0O` octal,
//! `0b`/`0B` binary. Junk after the lexeme is IGNORED at top level (`1abc`,
//! `1_`, `(1)x` accepted) but not inside parens (`(1x)` rejected); a decimal
//! lexeme that continues as a FLOAT (`.` + digit, or `e`/`E` + optional sign +
//! digit) rejects (`1.0`, `1e2`); a leading underscore rejects (`_1` lexes as
//! an identifier, not a number — the underscore-prefix quirk) while interior/
//! trailing underscores are junk-tolerated (`1_0`, `1_`, `0x_1`); `+1` and
//! `--1` reject. The VALUE never changes the response bytes (0/0 vs 9/9 vs
//! -1/0 bodies byte-identical [L10]), so out-of-usize values are clamped
//! (negative → 0, overflow → `usize::MAX`) as a documented arbitrary choice.
//!
//! ## Render side ([S14][L13])
//! Rendered segments across the whole corpus + live pages contain only
//! `[A-Za-z0-9_.]` verbatim plus the single escape `%3C`/`%3E` in
//! `add/%3Cfirst%3E` (`<first>`). Encoding of other bytes is UNOBSERVABLE (the
//! metachar-filename channel collapsed: download/get_and_append URLs derive
//! from the theory NAME [L13]); the documented choice is: keep RFC3986
//! unreserved bytes (`A-Z a-z 0-9 - . _ ~`) raw, percent-encode every other
//! UTF-8 byte as uppercase `%XX` — which reproduces every observed href byte.
//! Numeric arguments render as plain decimal (`cases/raw/0/0`); the version
//! index in full hrefs is plain decimal too and is out of this grammar (it
//! precedes the handler).
//!
//! Out of the producer link vocabulary (no [`ThyPath`] constructor, parse →
//! `None`): `method/{lemma}/{n}[/path…]` — the server accepts it ([L11]) but
//! no fragment link the producers emit uses it, and the interface contract
//! omits it. `sources` is NOT a head (rejected live [L11]); the sources panes
//! live under `cases/{raw|refined}/{i}/{j}`.

use crate::model::ThyPath;

/// Percent-decode one raw URL segment (observed rules [L12]): valid `%XX`
/// pairs become bytes, invalid percent sequences stay literal, the byte string
/// is read as UTF-8 with U+FFFD replacement, `+` is left alone.
fn decode_segment(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 3 <= bytes.len()
            && bytes[i + 1].is_ascii_hexdigit()
            && bytes[i + 2].is_ascii_hexdigit()
        {
            let hi = (bytes[i + 1] as char).to_digit(16).unwrap() as u8;
            let lo = (bytes[i + 2] as char).to_digit(16).unwrap() as u8;
            out.push(hi << 4 | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-encode one segment for an href: RFC3986 unreserved bytes raw,
/// everything else as uppercase `%XX` per UTF-8 byte. Reproduces every
/// observed rendered segment ([S14]); beyond `[A-Za-z0-9_.]` and `<`/`>` the
/// exact server-side set is UNOBSERVABLE ([L13]) — this is the documented
/// choice.
fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Parse a numeric segment the way the live server accepts one ([L10]): a
/// Haskell-`reads`-shaped integer prefix. Returns the clamped value.
fn parse_numeric(seg: &str) -> Option<usize> {
    let chars: Vec<char> = seg.chars().collect();
    // Top level: anything after the parsed item is ignored junk.
    let (value, _end) = parse_int_item(&chars, 0)?;
    Some(if value < 0 {
        0
    } else {
        usize::try_from(value).unwrap_or(usize::MAX)
    })
}

/// One parenthesized/signed integer item starting at `pos` (whitespace
/// skipped). The item's END is returned: the top-level caller ignores what
/// follows it, while a paren frame requires its `)` to follow directly
/// ([L10]: `(1)x` accepted, `(1x)` rejected).
fn parse_int_item(chars: &[char], pos: usize) -> Option<(i128, usize)> {
    let mut i = skip_ws(chars, pos);
    if i < chars.len() && chars[i] == '(' {
        let (v, after) = parse_int_item(chars, i + 1)?;
        let j = skip_ws(chars, after);
        if j < chars.len() && chars[j] == ')' {
            return Some((v, j + 1));
        }
        return None;
    }
    let negative = if i < chars.len() && chars[i] == '-' {
        i = skip_ws(chars, i + 1);
        true
    } else {
        false
    };
    let (magnitude, after) = parse_int_lexeme(chars, i)?;
    Some((if negative { -magnitude } else { magnitude }, after))
}

fn skip_ws(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// One integer lexeme: decimal, or `0x`/`0o`/`0b` (either case) with at least
/// one digit of the base. A decimal lexeme that continues as a float (`.`
/// digit / `e`/`E` [sign] digit) is NOT an integer ([L10]: `1.0`, `1e2`
/// rejected while `1abc`, `1.x`, `1ex` are junk-tolerated).
fn parse_int_lexeme(chars: &[char], i: usize) -> Option<(i128, usize)> {
    if i >= chars.len() || !chars[i].is_ascii_digit() {
        return None;
    }
    // Radix-prefixed form.
    if chars[i] == '0' && i + 1 < chars.len() {
        let radix = match chars[i + 1] {
            'x' | 'X' => Some(16),
            'o' | 'O' => Some(8),
            'b' | 'B' => Some(2),
            _ => None,
        };
        if let Some(radix) = radix {
            let mut j = i + 2;
            let mut v: i128 = 0;
            while j < chars.len() && chars[j].is_digit(radix) {
                v = v.saturating_mul(radix as i128).saturating_add(chars[j].to_digit(radix).unwrap() as i128);
                j += 1;
            }
            if j > i + 2 {
                return Some((v, j));
            }
            // No digit after the prefix: fall through — the leading `0` is the
            // lexeme and `x…` is junk ([L10]: `0x_1` accepted).
        }
    }
    let mut j = i;
    let mut v: i128 = 0;
    while j < chars.len() && chars[j].is_ascii_digit() {
        v = v.saturating_mul(10).saturating_add((chars[j] as u8 - b'0') as i128);
        j += 1;
    }
    // Float guard.
    if j < chars.len() {
        match chars[j] {
            '.' if j + 1 < chars.len() && chars[j + 1].is_ascii_digit() => return None,
            'e' | 'E' => {
                let mut k = j + 1;
                if k < chars.len() && (chars[k] == '+' || chars[k] == '-') {
                    k += 1;
                }
                if k < chars.len() && chars[k].is_ascii_digit() {
                    return None;
                }
            }
            _ => {}
        }
    }
    Some((v, j))
}

/// Parse a raw wildcard tail (still percent-encoded, `/`-separated) into a
/// structured path. `None` mirrors the live server's 404 surface for the
/// producer link vocabulary.
pub fn parse(raw: &str) -> Option<ThyPath> {
    let segs: Vec<String> = raw.split('/').map(decode_segment).collect();
    let s: Vec<&str> = segs.iter().map(String::as_str).collect();
    match s.as_slice() {
        ["help", ..] => Some(ThyPath::Help),
        ["message", ..] => Some(ThyPath::Message),
        ["rules", ..] => Some(ThyPath::Rules),
        ["tactic", ..] => Some(ThyPath::Tactic),
        ["cases", kind @ ("raw" | "refined"), i, j, ..] => Some(ThyPath::Sources {
            refined: *kind == "refined",
            source_idx: parse_numeric(i)?,
            case_idx: parse_numeric(j)?,
        }),
        ["lemma", name, ..] => Some(ThyPath::Lemma((*name).to_string())),
        ["proof", lemma, sub @ ..] => Some(ThyPath::Proof {
            lemma: (*lemma).to_string(),
            sub: sub.iter().map(|s| s.to_string()).collect(),
        }),
        ["edit", name, ..] => Some(ThyPath::Edit((*name).to_string())),
        ["add", pos, ..] => Some(ThyPath::Add((*pos).to_string())),
        ["delete", name, ..] => Some(ThyPath::Delete((*name).to_string())),
        _ => None,
    }
}

/// Render a structured path to its href segments (percent-encoded; join with
/// `/` for the wildcard tail). Byte-reproduces every harvested corpus href
/// tail in the vocabulary ([S14], tests/r5_path_grammar.rs).
pub fn render(path: &ThyPath) -> Vec<String> {
    match path {
        ThyPath::Help => vec!["help".into()],
        ThyPath::Message => vec!["message".into()],
        ThyPath::Rules => vec!["rules".into()],
        ThyPath::Tactic => vec!["tactic".into()],
        ThyPath::Sources {
            refined,
            source_idx,
            case_idx,
        } => vec![
            "cases".into(),
            if *refined { "refined" } else { "raw" }.into(),
            source_idx.to_string(),
            case_idx.to_string(),
        ],
        ThyPath::Lemma(name) => vec!["lemma".into(), encode_segment(name)],
        ThyPath::Proof { lemma, sub } => {
            let mut v = vec!["proof".into(), encode_segment(lemma)];
            v.extend(sub.iter().map(|s| encode_segment(s)));
            v
        }
        ThyPath::Edit(name) => vec!["edit".into(), encode_segment(name)],
        ThyPath::Add(pos) => vec!["add".into(), encode_segment(pos)],
        ThyPath::Delete(name) => vec!["delete".into(), encode_segment(name)],
    }
}
