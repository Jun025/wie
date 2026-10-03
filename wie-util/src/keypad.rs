//! A handset keypad text input method: multi-tap Latin and 천지인 Hangul.
//!
//! Shared by every emulated text widget that has to turn number keys into text — WIPI's
//! `InputMethodHandler` and lwc `TextComponent` (wie-wipi-java) and SKVM's `TextComponentHandler`
//! (wie-skvm). It knows nothing about the JVM: a caller keeps `tokens` (the keys typed since the
//! composition began, as characters) wherever it keeps state, and applies the returned [`Edit`] to
//! the text before its caret.
//!
//! Every key re-renders the whole composition and the edit is the difference, so a Hangul vowel
//! that moves the previous syllable's final consonant (각 + ㅏ → 가가) needs no special case.
//!
//! Layouts (docs/report/0427 §2 — none of this is documented in the repo, so read it as chosen):
//! - Latin: ITU-T E.161 letters, the digit last (`2` → A B C 2).
//! - Hangul: 천지인 — 1 ㅣ · 2 ㆍ · 3 ㅡ compose vowels by stroke; 4 ㄱㅋㄲ · 5 ㄴㄹ · 6 ㄷㅌㄸ ·
//!   7 ㅂㅍㅃ · 8 ㅅㅎㅆ · 9 ㅈㅊㅉ · 0 ㅇㅁ cycle when pressed again.

use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Hangul,
    Upper,
    Lower,
    Digit,
}

/// Remove `delete` characters before the caret, then insert `insert` there.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Edit {
    pub delete: usize,
    pub insert: Vec<char>,
}

/// One primitive on a text field: the shape both WIPI's `notifyTextChanged` (insert / replace /
/// delete) and SKVM's `TextComponent` (`insert(C)` / `replace(C)` / `delete()`) take.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    Delete,
    Replace(char),
    Insert(char),
}

impl Edit {
    pub fn ops(&self) -> Vec<Op> {
        let mut ops = Vec::new();
        let mut insert = self.insert.iter().copied();
        if self.delete > 0 {
            ops.extend(core::iter::repeat_n(Op::Delete, self.delete - 1));
            ops.push(match insert.next() {
                Some(c) => Op::Replace(c),
                None => Op::Delete,
            });
        }
        ops.extend(insert.map(Op::Insert));
        ops
    }
}

const LATIN: [&str; 10] = [" 0", ".,?!1", "ABC2", "DEF3", "GHI4", "JKL5", "MNO6", "PQRS7", "TUV8", "WXYZ9"];
const HANGUL: [&str; 10] = ["ㅇㅁ", "ㅣ", "ㆍ", "ㅡ", "ㄱㅋㄲ", "ㄴㄹ", "ㄷㅌㄸ", "ㅂㅍㅃ", "ㅅㅎㅆ", "ㅈㅊㅉ"];

/// `digit` is `0..=9`. `again`: the same key was the last one pressed, within the caller's
/// multi-tap window — it cycles the last character instead of adding one. Vowel strokes never
/// cycle; pressing ㆍ twice is ᆢ.
pub fn press(mode: Mode, tokens: &mut Vec<char>, digit: u8, again: bool) -> Edit {
    let before = render(mode, tokens);
    let group: Vec<char> = match mode {
        Mode::Hangul => HANGUL[digit as usize].chars().collect(),
        Mode::Upper => LATIN[digit as usize].chars().collect(),
        Mode::Lower => LATIN[digit as usize].chars().flat_map(char::to_lowercase).collect(),
        Mode::Digit => alloc::vec![(b'0' + digit) as char],
    };
    let cycled = match tokens.last() {
        Some(last) if again && group.len() > 1 => group.iter().position(|c| c == last),
        _ => None,
    };
    match cycled {
        Some(i) => *tokens.last_mut().unwrap() = group[(i + 1) % group.len()],
        None => tokens.push(group[0]),
    }
    diff(&before, &render(mode, tokens))
}

/// CLR inside a composition: takes back the last key (a Hangul stroke or jamo, a Latin letter).
/// `None` when nothing is being composed — the caller deletes a character itself.
pub fn back(mode: Mode, tokens: &mut Vec<char>) -> Option<Edit> {
    let before = render(mode, tokens);
    tokens.pop()?;
    Some(diff(&before, &render(mode, tokens)))
}

/// The text `tokens` stand for. A caller that can read its field checks the text still ends with
/// this before typing on: a title that rewrote the text (`setString`) ended the composition.
pub fn render(mode: Mode, tokens: &[char]) -> Vec<char> {
    match mode {
        Mode::Hangul => hangul(tokens),
        _ => tokens.to_vec(),
    }
}

fn diff(before: &[char], after: &[char]) -> Edit {
    let same = before.iter().zip(after).take_while(|(a, b)| a == b).count();
    Edit {
        delete: before.len() - same,
        insert: after[same..].to_vec(),
    }
}

// 천지인 vowels by stroke (ㅣ ㆍ ㅡ). Every prefix of an entry is an entry, so strokes group greedily.
// ㆍ and ᆢ alone are the two half-typed vowels a handset shows while waiting for the next stroke.
const VOWELS: [(&str, char); 23] = [
    ("ㅣ", 'ㅣ'),
    ("ㅡ", 'ㅡ'),
    ("ㆍ", 'ㆍ'),
    ("ㆍㆍ", 'ᆢ'),
    ("ㅣㆍ", 'ㅏ'),
    ("ㅣㆍㆍ", 'ㅑ'),
    ("ㆍㅣ", 'ㅓ'),
    ("ㆍㆍㅣ", 'ㅕ'),
    ("ㆍㅡ", 'ㅗ'),
    ("ㆍㆍㅡ", 'ㅛ'),
    ("ㅡㆍ", 'ㅜ'),
    ("ㅡㆍㆍ", 'ㅠ'),
    ("ㅣㆍㅣ", 'ㅐ'),
    ("ㅣㆍㆍㅣ", 'ㅒ'),
    ("ㆍㅣㅣ", 'ㅔ'),
    ("ㆍㆍㅣㅣ", 'ㅖ'),
    ("ㆍㅡㅣ", 'ㅚ'),
    ("ㆍㅡㅣㆍ", 'ㅘ'),
    ("ㆍㅡㅣㆍㅣ", 'ㅙ'),
    ("ㅡㆍㅣ", 'ㅟ'),
    ("ㅡㆍㆍㅣ", 'ㅝ'),
    ("ㅡㆍㆍㅣㅣ", 'ㅞ'),
    ("ㅡㅣ", 'ㅢ'),
];
const STROKES: [char; 3] = ['ㅣ', 'ㆍ', 'ㅡ'];
const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const JUNG: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const JONG: &str = "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";
const DOUBLE_JONG: [(char, char, char); 11] = [
    ('ㄱ', 'ㅅ', 'ㄳ'),
    ('ㄴ', 'ㅈ', 'ㄵ'),
    ('ㄴ', 'ㅎ', 'ㄶ'),
    ('ㄹ', 'ㄱ', 'ㄺ'),
    ('ㄹ', 'ㅁ', 'ㄻ'),
    ('ㄹ', 'ㅂ', 'ㄼ'),
    ('ㄹ', 'ㅅ', 'ㄽ'),
    ('ㄹ', 'ㅌ', 'ㄾ'),
    ('ㄹ', 'ㅍ', 'ㄿ'),
    ('ㄹ', 'ㅎ', 'ㅀ'),
    ('ㅂ', 'ㅅ', 'ㅄ'),
];

fn double_jong(first: char, second: char) -> Option<char> {
    DOUBLE_JONG.iter().find(|&&(a, b, _)| a == first && b == second).map(|&(_, _, d)| d)
}

fn vowel(strokes: &[char]) -> Option<char> {
    VOWELS.iter().find(|(s, _)| s.chars().eq(strokes.iter().copied())).map(|&(_, v)| v)
}

fn index(table: &str, c: char) -> Option<u32> {
    table.chars().position(|t| t == c).map(|i| i as u32)
}

#[derive(Default)]
struct Syllable {
    cho: Option<char>,
    jung: Option<char>,
    jong: Option<char>,
}

impl Syllable {
    fn flush(&mut self, out: &mut Vec<char>) {
        match (self.cho, self.jung.and_then(|v| index(JUNG, v))) {
            (Some(c), Some(v)) => {
                let jong = self.jong.and_then(|j| index(JONG, j)).map_or(0, |j| j + 1);
                let code = 0xAC00 + (index(CHO, c).unwrap() * 21 + v) * 28 + jong;
                out.push(char::from_u32(code).unwrap());
            }
            _ => out.extend([self.cho, self.jung].into_iter().flatten()),
        }
        *self = Syllable::default();
    }
}

fn hangul(tokens: &[char]) -> Vec<char> {
    // Strokes → vowels first (greedy), consonants as they are.
    let mut units: Vec<char> = Vec::new();
    let mut strokes: Vec<char> = Vec::new();
    for &t in tokens {
        if STROKES.contains(&t) {
            strokes.push(t);
            if vowel(&strokes).is_none() {
                strokes.pop();
                units.push(vowel(&strokes).unwrap());
                strokes = alloc::vec![t];
            }
            continue;
        }
        if !strokes.is_empty() {
            units.push(vowel(&strokes).unwrap());
            strokes.clear();
        }
        units.push(t);
    }
    if !strokes.is_empty() {
        units.push(vowel(&strokes).unwrap());
    }

    let mut out = Vec::new();
    let mut s = Syllable::default();
    for u in units {
        if index(CHO, u).is_some() {
            match (s.cho, s.jung, s.jong) {
                (Some(_), Some(_), None) if index(JONG, u).is_some() => s.jong = Some(u),
                (Some(_), Some(_), Some(j)) if double_jong(j, u).is_some() => s.jong = double_jong(j, u),
                (None, None, None) => s.cho = Some(u),
                _ => {
                    s.flush(&mut out);
                    s.cho = Some(u);
                }
            }
        } else if index(JUNG, u).is_some() {
            match (s.cho, s.jung, s.jong) {
                (Some(_), None, _) => s.jung = Some(u),
                // The final consonant (or a double final's second half) starts the next syllable.
                (Some(_), Some(_), Some(j)) => {
                    let (keep, moved) = DOUBLE_JONG.iter().find(|&&(_, _, d)| d == j).map_or((None, j), |&(a, b, _)| (Some(a), b));
                    s.jong = keep;
                    s.flush(&mut out);
                    s.cho = Some(moved);
                    s.jung = Some(u);
                }
                _ => {
                    s.flush(&mut out);
                    s.jung = Some(u);
                }
            }
        } else {
            // ㆍ or ᆢ: shown as typed until the next stroke makes it a vowel.
            s.flush(&mut out);
            out.push(u);
        }
    }
    s.flush(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};

    use super::{Edit, Mode, Op, back, press, render};

    /// Types `keys` (`'0'..='9'`; `','` = wait out the multi-tap window; `'<'` = CLR) into a text.
    fn typed(mode: Mode, keys: &str) -> String {
        let (mut text, mut tokens, mut last) = (Vec::new(), Vec::new(), None);
        for k in keys.chars() {
            let edit = match k {
                ',' => {
                    last = None;
                    continue;
                }
                '<' => {
                    last = None;
                    back(mode, &mut tokens).unwrap_or(Edit {
                        delete: 1,
                        insert: Vec::new(),
                    })
                }
                _ => {
                    let again = last == Some(k);
                    last = Some(k);
                    press(mode, &mut tokens, k as u8 - b'0', again)
                }
            };
            text.truncate(text.len().saturating_sub(edit.delete));
            text.extend(edit.insert);
        }
        text.into_iter().collect()
    }

    #[test]
    fn latin_cycles_and_wraps_to_the_digit() {
        assert_eq!(typed(Mode::Upper, "22,2"), "BA");
        assert_eq!(typed(Mode::Upper, "22222"), "A"); // A B C 2 A
        assert_eq!(typed(Mode::Lower, "4433"), "he");
        assert_eq!(typed(Mode::Digit, "223"), "223");
        assert_eq!(typed(Mode::Upper, "55<2"), "A");
    }

    #[test]
    fn cheonjiin_composes_syllables() {
        assert_eq!(typed(Mode::Hangul, "412"), "가");
        assert_eq!(typed(Mode::Hangul, "4124"), "각");
        assert_eq!(typed(Mode::Hangul, "4124,412"), "각가");
        assert_eq!(typed(Mode::Hangul, "41241"), "가기"); // the final moves on to the vowel after it
        assert_eq!(typed(Mode::Hangul, "883"), "흐");
        assert_eq!(typed(Mode::Hangul, "88123"), "하ㅡ"); // ㅣㆍㅡ is no vowel: ㅡ starts a new one
        assert_eq!(typed(Mode::Hangul, "0221551"), "여리"); // ㅇ ㆍㆍㅣ ㄴ→ㄹ ㅣ
        assert_eq!(typed(Mode::Hangul, "8121,881"), "새히");
        assert_eq!(typed(Mode::Hangul, "512558"), "낤"); // ㄴ→ㄹ final, then ㅅ makes it ㄽ
        assert_eq!(typed(Mode::Hangul, "02312"), "와"); // ㅇ ㆍㅡ ㅣ ㆍ
        assert_eq!(typed(Mode::Hangul, "032"), "우");
        assert_eq!(typed(Mode::Hangul, "42"), "ㄱㆍ");
        assert_eq!(typed(Mode::Hangul, "41242"), "각ㆍ");
        assert_eq!(typed(Mode::Hangul, "412421"), "가거");
        assert_eq!(typed(Mode::Hangul, "666"), "ㄸ");
        assert_eq!(typed(Mode::Hangul, "4126661"), "가띠"); // ㄸ is never a final
    }

    #[test]
    fn clr_takes_back_one_key() {
        assert_eq!(typed(Mode::Hangul, "4124<"), "가");
        assert_eq!(typed(Mode::Hangul, "412<"), "기");
        assert_eq!(typed(Mode::Hangul, "412<<<"), "");
        assert_eq!(typed(Mode::Hangul, "<"), "");
        assert_eq!(render(Mode::Hangul, &['ㄱ', 'ㅣ', 'ㆍ']), ['가']);
    }

    #[test]
    fn edits_become_field_operations() {
        let ops = |delete, insert: &str| {
            Edit {
                delete,
                insert: insert.chars().collect(),
            }
            .ops()
        };
        assert_eq!(ops(0, "가"), [Op::Insert('가')]);
        assert_eq!(ops(1, "가거"), [Op::Replace('가'), Op::Insert('거')]);
        assert_eq!(ops(2, "고"), [Op::Delete, Op::Replace('고')]);
        assert_eq!(ops(1, ""), [Op::Delete]);
    }
}
