use crate::{BOMError, VerseReference, BOM};
use lazy_static::lazy_static;
use regex::Regex;
use std::{cmp, collections::HashMap, fmt, str};

const CITATION_DELIM: char = ';';
const VERSE_CHUNK_DELIM: char = ',';
const CHAPTER_VERSE_DELIM: char = ':';
const RANGE_DELIM_CANONICAL: char = '–'; // en-dash
const RANGE_DELIM_NON_CANONICAL1: char = '-'; // regular dash
const RANGE_DELIM_NON_CANONICAL2: char = '—'; // em-dash

lazy_static! {
    static ref BOOK_NAMES_TO_INDEX: HashMap<&'static str, usize> = vec![
        ("1 Nephi", 0),
        ("1 Ne.", 0),
        ("2 Nephi", 1),
        ("2 Ne.", 1),
        ("Jacob", 2),
        ("Enos", 3),
        ("Jarom", 4),
        ("Omni", 5),
        ("Words of Mormon", 6),
        ("W of M", 6),
        ("Mosiah", 7),
        ("Alma", 8),
        ("Helaman", 9),
        ("Hel.", 9),
        ("3 Nephi", 10),
        ("3 Ne.", 10),
        ("4 Nephi", 11),
        ("4 Ne.", 11),
        ("Mormon", 12),
        ("Morm.", 12),
        ("Ether", 13),
        ("Moroni", 14),
        ("Moro.", 14),
    ]
    .into_iter()
    .collect();

    static ref BOOK_INDEX_TO_NAMES: Vec<(&'static str, &'static str)> = vec![
        // (Long name, short name)
        ("1 Nephi", "1 Ne."),
        ("2 Nephi", "2 Ne."),
        ("Jacob", "Jacob"),
        ("Enos", "Enos"),
        ("Jarom", "Jarom"),
        ("Omni", "Omni"),
        ("Words of Mormon", "W of M"),
        ("Mosiah", "Mosiah"),
        ("Alma", "Alma"),
        ("Helaman", "Hel."),
        ("3 Nephi", "3 Ne."),
        ("4 Nephi", "4 Ne."),
        ("Mormon", "Morm."),
        ("Ether", "Ether"),
        ("Moroni", "Moro."),
    ];
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum RangeType {
    StartEndVerse {
        chapter: usize,
        start: usize,
        end: usize,
    },
    StartEndChapter {
        start: usize,
        end: usize,
    },
}

impl RangeType {
    fn chapter_range(&self) -> (usize, usize) {
        match self {
            RangeType::StartEndChapter { start, end } => (*start, *end),
            RangeType::StartEndVerse { chapter, .. } => (*chapter, *chapter),
        }
    }

    fn verse_range(&self) -> Option<(usize, usize)> {
        match self {
            RangeType::StartEndChapter { .. } => None,
            RangeType::StartEndVerse { start, end, .. } => Some((*start, *end)),
        }
    }
}

impl PartialOrd for RangeType {
    fn partial_cmp(&self, other: &RangeType) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RangeType {
    fn cmp(&self, other: &RangeType) -> cmp::Ordering {
        match (self, other) {
            (
                RangeType::StartEndVerse {
                    chapter: c,
                    start: s,
                    end: e,
                },
                RangeType::StartEndVerse {
                    chapter: oc,
                    start: os,
                    end: oe,
                },
            ) => match c.cmp(&oc) {
                cmp::Ordering::Equal => match s.cmp(&os) {
                    cmp::Ordering::Equal => e.cmp(&oe),
                    comp => comp,
                },
                comp => comp,
            },
            (
                RangeType::StartEndVerse { chapter: c, .. },
                RangeType::StartEndChapter { start: os, end: oe },
            ) => match c.cmp(&os) {
                cmp::Ordering::Equal => c.cmp(&oe),
                comp => comp,
            },
            (
                RangeType::StartEndChapter { start: os, end: oe },
                RangeType::StartEndVerse { chapter: c, .. },
            ) => match os.cmp(&c) {
                cmp::Ordering::Equal => oe.cmp(&c),
                comp => comp,
            },
            (
                RangeType::StartEndChapter { start: s, end: e },
                RangeType::StartEndChapter { start: os, end: oe },
            ) => match s.cmp(&os) {
                cmp::Ordering::Equal => e.cmp(&oe),
                comp => comp,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct VerseRangeReference {
    range_type: RangeType,
    book_index: usize,
}

impl PartialOrd for VerseRangeReference {
    fn partial_cmp(&self, other: &VerseRangeReference) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VerseRangeReference {
    fn cmp(&self, other: &VerseRangeReference) -> cmp::Ordering {
        match self.book_index.cmp(&other.book_index) {
            cmp::Ordering::Equal => self.range_type.cmp(&other.range_type),
            comp => comp,
        }
    }
}

impl VerseRangeReference {
    fn verse_refs<'a, 'b>(&'b self, bom: &'a BOM) -> VerseRangeReferenceIter<'a, 'b> {
        VerseRangeReferenceIter {
            bom,
            range_reference: self,
            current_chap_index: 0,
            current_verse_index: 0,
        }
    }

    fn is_valid(&self, bom: &BOM) -> bool {
        let book = bom.books.get(self.book_index);
        match self.range_type {
            RangeType::StartEndChapter { start, end } => {
                if start == 0 || end == 0 {
                    return false;
                }

                book.map(|b| b.chapters.get(start)).is_some()
                    && book.map(|b| b.chapters.get(end)).is_some()
            }
            RangeType::StartEndVerse {
                chapter,
                start,
                end,
            } => {
                if chapter == 0 || start == 0 || end == 0 {
                    return false;
                }

                book.and_then(|b| b.chapters.get(chapter))
                    .and_then(|c| c.verses.get(start))
                    .is_some()
                    && book
                        .and_then(|b| b.chapters.get(chapter))
                        .and_then(|c| c.verses.get(end))
                        .is_some()
            }
        }
    }
}

struct VerseRangeReferenceIter<'a, 'b> {
    bom: &'a BOM,
    range_reference: &'b VerseRangeReference,
    current_chap_index: usize,
    current_verse_index: usize,
}

impl<'a, 'b> Iterator for VerseRangeReferenceIter<'a, 'b> {
    type Item = VerseReference;
    fn next(&mut self) -> Option<VerseReference> {
        if !self.range_reference.is_valid(self.bom) {
            return None;
        }

        let book = &self.bom.books[self.range_reference.book_index];
        match self.range_reference.range_type {
            RangeType::StartEndChapter { start, end } => {
                let mut res = None;
                if self.current_chap_index + start <= end {
                    let chapter = &book.chapters[self.current_chap_index + start - 1];
                    res = Some(VerseReference {
                        book_index: self.range_reference.book_index,
                        chapter_index: self.current_chap_index + start,
                        verse_index: self.current_verse_index + 1,
                    });

                    self.current_verse_index += 1;
                    if self.current_verse_index > chapter.verses.len() {
                        self.current_verse_index = 0;
                        self.current_chap_index += 1;
                    }
                }

                res
            }
            RangeType::StartEndVerse {
                chapter,
                start,
                end,
            } => {
                let mut res = None;
                if self.current_verse_index + start <= end {
                    res = Some(VerseReference {
                        book_index: self.range_reference.book_index,
                        chapter_index: chapter,
                        verse_index: start + self.current_verse_index,
                    });
                    self.current_verse_index += 1;
                }

                res
            }
        }
    }
}

#[derive(Debug)]
struct ReferenceCollectionIter {
    data: Vec<VerseReference>,
    index: usize,
}

impl Iterator for ReferenceCollectionIter {
    type Item = VerseReference;
    fn next(&mut self) -> Option<VerseReference> {
        let data = self.data.get(self.index).cloned();
        self.index += 1;
        data
    }
}

#[derive(Debug)]
pub struct ReferenceCollection {
    refs: Vec<VerseRangeReference>,
}

impl ReferenceCollection {
    pub fn new(s: &str) -> Result<Self, BOMError> {
        s.parse()
    }

    #[allow(dead_code)] // Used by test
    fn is_valid(&self, bom: &BOM) -> bool {
        self.refs.iter().all(|r| r.is_valid(bom))
    }

    pub fn verse_refs(&self, bom: &BOM) -> impl Iterator<Item = VerseReference> {
        // I don't think it's very efficient to eagerly collect this iter, but I don't know how to store
        // an "in-use" iterator in struct without generators.
        let data = self.refs.iter().flat_map(|r| r.verse_refs(bom)).collect();
        ReferenceCollectionIter { data, index: 0 }
    }

    pub fn canonicalize(&mut self) {
        // Sort collection by book, chapter / chapter range, verse / verse range.
        self.refs.sort();
        let mut new_refs = vec![];

        // Collapse ranges
        let mut current_ref = self.refs[0].clone();
        let mut current_book = current_ref.book_index;
        let mut current_chap_range = current_ref.range_type.chapter_range();
        let mut current_verse_range = current_ref.range_type.verse_range();
        new_refs.push(current_ref);

        for r in self.refs.iter().skip(1) {
            let chap_range = r.range_type.chapter_range();
            let verse_range = r.range_type.verse_range();

            // In same book.
            if r.book_index == current_book {
                // Overlapping chapter ranges
                if chap_range.0 >= current_chap_range.0
                    && chap_range.0 <= (current_chap_range.1 + 1)
                {
                    match (verse_range, current_verse_range) {
                        (None, None) => {
                            if verse_range.is_none() && current_verse_range.is_none() {
                                // Both chapter-only ranges. Take the union of their covered area.
                                let min_chap = current_chap_range.0.min(chap_range.0);
                                let max_chap = current_chap_range.1.max(chap_range.1);
                                let combined_ref = VerseRangeReference {
                                    book_index: current_book,
                                    range_type: RangeType::StartEndChapter {
                                        start: min_chap,
                                        end: max_chap,
                                    },
                                };

                                current_ref = combined_ref.clone();
                                current_book = current_ref.book_index;
                                current_chap_range = current_ref.range_type.chapter_range();
                                current_verse_range = current_ref.range_type.verse_range();

                                new_refs.pop();
                                new_refs.push(combined_ref);
                                continue;
                            }
                        }

                        (Some(vr), Some(cvr)) => {
                            // Overlapping verse ranges
                            if vr.0 >= cvr.0 && vr.0 <= (cvr.1 + 1) {
                                let min_verse = cvr.0.min(vr.0);
                                let max_verse = cvr.1.max(vr.1);
                                let combined_ref = VerseRangeReference {
                                    book_index: current_book,
                                    range_type: RangeType::StartEndVerse {
                                        start: min_verse,
                                        end: max_verse,
                                        chapter: current_chap_range.0, // We can use any of the chapter ranges, arbitrary choice since all the same.
                                    },
                                };

                                current_ref = combined_ref.clone();
                                current_book = current_ref.book_index;
                                current_chap_range = current_ref.range_type.chapter_range();
                                current_verse_range = current_ref.range_type.verse_range();

                                new_refs.pop();
                                new_refs.push(combined_ref);
                                continue;
                            }
                        }
                        _ => {
                            // We know that they have overlapping chapter ranges, and that one is a full chapter (None).
                            // the right way to handle this is to keep the full chapter and eliminate single verses in it.
                            if verse_range.is_none() {
                                // Keep the new range.
                                let combined_ref = VerseRangeReference {
                                    book_index: current_book,
                                    range_type: RangeType::StartEndChapter {
                                        start: chap_range.0,
                                        end: chap_range.1,
                                    },
                                };

                                current_ref = combined_ref.clone();
                                current_book = current_ref.book_index;
                                current_chap_range = current_ref.range_type.chapter_range();
                                current_verse_range = current_ref.range_type.verse_range();

                                new_refs.pop();
                                new_refs.push(combined_ref);
                                continue;
                            }
                        }
                    }
                }
            }

            current_ref = r.clone();
            current_book = current_ref.book_index;
            current_chap_range = current_ref.range_type.chapter_range();
            current_verse_range = current_ref.range_type.verse_range();
            new_refs.push(r.clone()); // Nothing to combine.
        }

        self.refs = new_refs;
    }
}

/// Types of references that we'll parse:
// https://en.wikipedia.org/wiki/Bible_citation. We use the Chicago Manual of Style.

// 1. Multiple citations semi-colon delimited. Those without book names get booknames
// of the book before. Book name may be full or abbreviated.
// 2. In a single citation, the chapter is left of the semi-colon. No semi-colon means that
// everything in the citation is a chapter, not a verse.
// 3. In a single citation, chunks of verses are comma-separated on the right side of a semicolon.
// A en-dash is used to mark ranges.
impl str::FromStr for ReferenceCollection {
    type Err = BOMError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let citations = s.split(CITATION_DELIM);
        let mut references = vec![];
        for citation in citations {
            let chapter_verse_split: Vec<_> = citation.split(CHAPTER_VERSE_DELIM).collect();
            match chapter_verse_split.len() {
                1 => {
                    // Everything should be treated as a chapter.
                    let chapter_chunk_split: Vec<_> = citation.split(VERSE_CHUNK_DELIM).collect();
                    let first_chunk = chapter_chunk_split[0];
                    let (end_of_name_index, book_index) = extract_book_name(first_chunk)?;
                    let mut chapter_chunks = vec![&first_chunk[end_of_name_index..]];
                    chapter_chunks.extend(&chapter_chunk_split[1..]);

                    for chapter_chunk in chapter_chunks {
                        let (start, end) = extract_range(chapter_chunk)?;
                        let reference = VerseRangeReference {
                            book_index,
                            range_type: RangeType::StartEndChapter { start, end },
                        };
                        references.push(reference);
                    }
                }
                2 => {
                    // First is the chapter, everything else is the verse.
                    let book_chapter_chunk = chapter_verse_split[0];
                    let (end_of_name_index, book_index) = extract_book_name(book_chapter_chunk)
                        .or_else(|e| {
                            // Use the previous book if it exists.
                            if let Some(prev) = references.last() {
                                Ok((0, prev.book_index))
                            } else {
                                Err(e)
                            }
                        })?;

                    let chapter = extract_number(&book_chapter_chunk[end_of_name_index..])?;

                    let verse_chunk_split: Vec<_> =
                        chapter_verse_split[1].split(VERSE_CHUNK_DELIM).collect();
                    for verse_chunk in verse_chunk_split {
                        let (start, end) = extract_range(verse_chunk)?;
                        let reference = VerseRangeReference {
                            book_index,
                            range_type: RangeType::StartEndVerse {
                                chapter,
                                start,
                                end,
                            },
                        };
                        references.push(reference);
                    }
                }
                _ => {
                    return Err(BOMError::ReferenceError(format!(
                        "More than 1 '{}' in a single citation",
                        CHAPTER_VERSE_DELIM
                    )))
                }
            };
        }

        if references.is_empty() {
            return Err(BOMError::ReferenceError(format!(
                "Unable to parse any references from string: {}",
                s
            )));
        }

        Ok(ReferenceCollection { refs: references })
    }
}

fn extract_range(s: &str) -> Result<(usize, usize), BOMError> {
    let split = s
        .split(|s| {
            s == RANGE_DELIM_CANONICAL
                || s == RANGE_DELIM_NON_CANONICAL1
                || s == RANGE_DELIM_NON_CANONICAL2
        })
        .collect::<Vec<_>>();
    match split.len() {
        1 => {
            let num = extract_number(split[0])?;
            Ok((num, num))
        }
        2 => {
            let lower = extract_number(split[0])?;
            let upper = extract_number(split[1])?;
            if lower >= upper {
                return Err(BOMError::ReferenceError(format!("Range is invalid: {}", s)));
            }

            Ok((lower, upper))
        }
        _ => Err(BOMError::ReferenceError(format!(
            "Too many dashes (-) found in {}",
            s
        ))),
    }
}

fn extract_book_name(s: &str) -> Result<(usize, usize), BOMError> {
    lazy_static! {
        static ref POSSIBLE_BOOK_NAME: Regex =
            Regex::new(r"^(?P<name>(\d\s)?[A-Za-z ]+\.?)\s+").unwrap();
    }

    let s_trimmed = s.trim();
    if POSSIBLE_BOOK_NAME.is_match(s_trimmed) {
        let caps = POSSIBLE_BOOK_NAME.captures(s_trimmed).ok_or_else(|| {
            BOMError::ReferenceError(format!("Book name not found as expected in {}", s_trimmed))
        })?;
        let cap = caps["name"].trim();
        let trimmed = cap.trim();
        if BOOK_NAMES_TO_INDEX.contains_key(trimmed) {
            let index = s.find(trimmed).unwrap(); // We just found it via regex.
            return Ok((
                index + trimmed.len(),
                *BOOK_NAMES_TO_INDEX.get(trimmed).unwrap(),
            ));
        }
    }

    Err(BOMError::ReferenceError(format!(
        "Book name not found as expected in {}",
        s
    )))
}

fn extract_number(s: &str) -> Result<usize, BOMError> {
    let s = s.trim();
    s.parse::<usize>()
        .map_err(|_| BOMError::ReferenceError(format!("Unable to parse number from {}", s)))
}

impl fmt::Display for ReferenceCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.refs.is_empty() {
            return Ok(());
        }

        // Use values guaranteed to not be the first.
        let mut previous_book = 1000;
        let mut previous_chapter = 1000;

        for (i, reference) in self.refs.iter().enumerate() {
            let new_book = previous_book != reference.book_index;
            if new_book {
                if i != 0 {
                    write!(f, "{} ", CITATION_DELIM)?;
                }

                write!(f, "{} ", BOOK_INDEX_TO_NAMES[reference.book_index].1)?;
                previous_book = reference.book_index;
            }

            match reference.range_type {
                RangeType::StartEndChapter { start, end } => {
                    if !new_book {
                        write!(f, "{} ", VERSE_CHUNK_DELIM)?
                    }

                    if start == end {
                        write!(f, "{}", start)?
                    } else {
                        write!(f, "{}{}{}", start, RANGE_DELIM_CANONICAL, end)?
                    }
                }
                RangeType::StartEndVerse {
                    chapter,
                    start,
                    end,
                } => {
                    if !new_book && chapter == previous_chapter {
                        write!(f, "{} ", VERSE_CHUNK_DELIM)?
                    } else {
                        if !new_book && i != 0 {
                            write!(f, "{} ", CITATION_DELIM)?;
                        }

                        write!(f, "{}{}", chapter, CHAPTER_VERSE_DELIM)?;
                        previous_chapter = chapter;
                    }

                    if start == end {
                        write!(f, "{}", start)?
                    } else {
                        write!(f, "{}{}{}", start, RANGE_DELIM_CANONICAL, end)?
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! roundtrip_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $value;
                let parsed = input.parse::<ReferenceCollection>();
                if let Ok(parsed) = parsed {
                    let formatted = parsed.to_string();
                    assert_eq!(
                        formatted, input,
                        "Roundtrip from string -> parsed -> string failed"
                    );
                } else {
                    assert!(
                        false,
                        format!("Input '{}' should have parsed without error", input)
                    );
                }
            }
        )*
        }
    }

    roundtrip_tests! {
        roundtrip_0: "Alma 3:16",
        roundtrip_1: "Alma 3:16–17",
        roundtrip_2: "Alma 3:16, 18",
        roundtrip_3: "Alma 3:16, 18–20; 13:2–4, 7–8",
        roundtrip_4: "Alma 5–8",
        roundtrip_5: "Alma 8",
        roundtrip_6: "Alma 8, 10",
        roundtrip_7: "Alma 32:31; Mosiah 1:1; 3:2",
        roundtrip_9: "1 Ne. 1:1",
        roundtrip_11: "2 Ne. 1:1",
        roundtrip_13: "W of M 1:1",
        roundtrip_15: "Hel. 1:1",
        roundtrip_17: "3 Ne. 1:1",
        roundtrip_19: "4 Ne. 1:1",
        roundtrip_20: "Morm. 1:1",
        roundtrip_22: "Moro. 1:1",
    }

    #[test]
    fn reference_collection_canonicalization() {
        let cases = vec![
            // Spacing
            ("  Alma  3   :  16 ", "Alma 3:16"),
            // Joining ranges, ordering of books and chapters
            (
                "Alma 3:18–19, 16–17; Mosiah 3:18",
                "Mosiah 3:18; Alma 3:16–19",
            ),
            ("Alma 3:18–19, 16–17; Alma 3; Alma 4", "Alma 3–4"),
            ("Alma 3:16, 17, 18–19", "Alma 3:16–19"),
            ("Alma 3:16, 18, 19", "Alma 3:16, 18–19"),
            ("Alma 16, 18, 19", "Alma 16, 18–19"),
            // Convert to en-dashes
            ("Alma 3:16-17", "Alma 3:16–17"),
            ("Alma 3:16—17", "Alma 3:16–17"),
            // Move to abbreviations
            ("Moroni 1:1", "Moro. 1:1"),
            ("Moroni 1:1", "Moro. 1:1"),
            ("Mormon 1:1", "Morm. 1:1"),
            ("4 Nephi 1:1", "4 Ne. 1:1"),
            ("3 Nephi 1:1", "3 Ne. 1:1"),
            ("Helaman 1:1", "Hel. 1:1"),
            ("Words of Mormon 1:1", "W of M 1:1"),
            ("2 Nephi 1:1", "2 Ne. 1:1"),
            ("1 Nephi 1:1", "1 Ne. 1:1"),
        ];

        for (input, expected) in cases {
            let parsed = input.parse::<ReferenceCollection>();
            if let Ok(mut parsed) = parsed {
                parsed.canonicalize();
                let formatted = parsed.to_string();
                assert_eq!(formatted, expected, "Canonicalization failed");
            } else {
                assert!(
                    false,
                    format!("Input {} should have parsed without error", input)
                );
            }
        }
    }

    macro_rules! illegal_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let case = $value;
                let bom = BOM::from_default_parser().unwrap();
                let result = case.parse::<ReferenceCollection>();
                match result {
                    Ok(parsed) => assert!(
                        !parsed.is_valid(&bom),
                        format!("Should have failed to validate reference {}", case)
                    ),
                    _ => assert!(
                        result.is_err(),
                        format!("Should have failed to parse reference {}", case)
                    )
                };
            }
        )*
        }
    }

    illegal_tests! {
        illegal_0: "Alma 100:5",
        illegal_1: "",
        illegal_2: "100:5",
        illegal_3: "23 Nephi: 11, 5",
        illegal_4: "Ephraim 1:1",
        illegal_5: "MeNephi 1:1",
        illegal_6: "1 Nephi 5:100",
        illegal_7: "1 Nephi 1: 5-1",
        illegal_8: "1 Nephi 1: 5-5", // Should this be illegal? Or should be just treat as a non-range?
        illegal_9: "1 Nephi 0: 1",
        illegal_10: "1 Nephi 5: 0",
        illegal_11: "1 Nephi 1:1, 1:2",
        illegal_12: "1 Nephi 1:1; 1 Nephi 5: 0", // Should this be illegal? Should any incorrect citations in a list fail the whole list?
        illegal_13: "Ephraim 5",
    }
}
