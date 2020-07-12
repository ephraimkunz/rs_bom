use crate::{BOMError, Reference, BOM};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt, iter, str,
};

#[derive(Debug)]
pub struct ReferenceCollection {
    refs: Vec<Reference>,
}

impl ReferenceCollection {
    fn is_valid(&self, bom: &BOM) -> bool {
        self.refs.iter().all(|r| r.is_valid(bom))
    }
}

impl iter::FromIterator<Reference> for ReferenceCollection {
    fn from_iter<I: IntoIterator<Item = Reference>>(iter: I) -> Self {
        ReferenceCollection {
            refs: iter.into_iter().collect(),
        }
    }
}

const CITATION_DELIM: char = ';';
const VERSE_CHUNK_DELIM: char = ',';
const CHAPTER_VERSE_DELIM: char = ':';
const RANGE_DELIM_CANONICAL: char = '–'; // en-dash
const RANGE_DELIM_NON_CANONICAL1: char = '-'; // regular dash
const RANGE_DELIM_NON_CANONICAL2: char = '—'; // em-dash

lazy_static! {
    // Mapping from valid names to book index.
    static ref BOOK_NAMES: HashMap<&'static str, usize> = vec![
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

    // Mapping from valid names to book index.
    static ref CANONICAL_NAMES: Vec<(&'static str, &'static str)> = vec![
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
        ("1 Nephi", "1 Ne."),
        ("3 Nephi", "3 Ne."),
        ("4 Nephi", "4 Ne."),
        ("Mormon", "Morm."),
        ("Ether", "Ether"),
        ("Moroni", "Moro."),
    ];
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
            let chapter_verse_split = citation.split(CHAPTER_VERSE_DELIM);
            match chapter_verse_split.count() {
                1 => {
                    // Everything should be treated as a chapter.
                    let chapter_chunk_split: Vec<_> = citation.split(VERSE_CHUNK_DELIM).collect();
                    let first_chunk = chapter_chunk_split[0];
                    let (name, book_index) = extract_book_name(first_chunk)?;
                    let index = first_chunk.find(&name).unwrap(); // We just found it via regex.
                    let mut chapter_chunks = vec![&first_chunk[index + name.len()..]];
                    chapter_chunks.extend(&chapter_chunk_split[1..]);

                    let mut chapter_set = HashSet::new();
                    for chapter_chunk in chapter_chunks {
                        let split = chapter_chunk
                            .split(|s| {
                                return s == RANGE_DELIM_CANONICAL
                                    || s == RANGE_DELIM_NON_CANONICAL1
                                    || s == RANGE_DELIM_NON_CANONICAL2;
                            })
                            .collect::<Vec<_>>();
                        if split.len() == 1 {
                            let num = extract_number(split[0])?;
                            chapter_set.insert(num);
                        } else if split.len() != 2 {
                            return Err(BOMError::ReferenceError(format!(
                                "Too many dashes (-) found in {}",
                                chapter_chunk
                            )));
                        } else {
                            let lower = extract_number(split[0])?;
                            let upper = extract_number(split[1])?;
                            if lower >= upper {
                                return Err(BOMError::ReferenceError(format!(
                                    "Range is invalid: {}",
                                    chapter_chunk
                                )));
                            }

                            for chap in lower..=upper {
                                chapter_set.insert(chap);
                            }
                        }
                    }

                    for chapter in chapter_set {
                        references.push(Reference {
                            book_index,
                            chapter_index: chapter,
                            verse_index: None,
                        });
                    }
                }
                2 => {
                    // First is the chapter, everything else is the verse.
                }
                _ => {
                    return Err(BOMError::ReferenceError(format!(
                        "More than 1 '{}' in a single citation",
                        CHAPTER_VERSE_DELIM
                    )))
                }
            };
        }

        if references.len() == 0 {
            return Err(BOMError::ReferenceError(format!(
                "Unable to parse any references from string: {}",
                s
            )));
        }

        Ok(ReferenceCollection { refs: references })
    }
}

fn extract_book_name(s: &str) -> Result<(String, usize), BOMError> {
    lazy_static! {
        static ref POSSIBLE_BOOK_NAME: Regex =
            Regex::new(r"^(?P<name>(\d\s)?[A-Za-z ]+\.?)\s+").unwrap();
    }

    let s = s.trim();
    if POSSIBLE_BOOK_NAME.is_match(s) {
        let caps = POSSIBLE_BOOK_NAME
            .captures(s)
            .ok_or(BOMError::ReferenceError(format!(
                "Book name not found as expected in {}",
                s
            )))?;
        let cap = &caps["name"];
        if BOOK_NAMES.contains_key(cap) {
            return Ok((cap.to_string(), *BOOK_NAMES.get(cap).unwrap()));
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
        for reference in &self.refs {
            match reference.verse_index {
                Some(v) => {}
                None => write!(
                    f,
                    "{} {}",
                    CANONICAL_NAMES[reference.book_index].1, reference.chapter_index
                )?,
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
        roundtrip_3: "Alma 3:16,18–20; 13:2–4, 7–8",
        roundtrip_4: "Alma 5–8",
        roundtrip_5: "Alma 8",
        roundtrip_6: "Alma 8, 10",
        roundtrip_7: "Alma 32:31; Mosiah 1:1; 3:2",
        roundtrip_8: "1 Nephi 1:1",
        roundtrip_9: "1 Ne. 1:1",
        roundtrip_10: "2 Nephi 1:1",
        roundtrip_11: "2 Ne. 1:1",
        roundtrip_12: "Words of Mormon 1:1",
        roundtrip_13: "W of M 1:1",
        roundtrip_14: "Helaman 1:1",
        roundtrip_15: "Hel. 1:1",
        roundtrip_16: "3 Nephi 1:1",
        roundtrip_17: "3 Ne. 1:1",
        roundtrip_18: "4 Nephi 1:1",
        roundtrip_19: "4 Ne. 1:1",
        roundtrip_20: "Mormon 1:1",
        roundtrip_21: "Morm. 1:1",
        roundtrip_22: "Moroni 1:1",
        roundtrip_23: "Moro. 1:1",
    }

    #[test]
    fn reference_collection_canonicalization() {
        // Spacing, move to abbreviations, joining ranges, ordering of books/citations?, to en-dashes
        let cases = vec![
            ("Alma 3:16", "Alma 3:16"),
            ("Alma 3:16;;", "Alma 3:16"), // I guess we can allow empty citations?
        ];

        for (input, expected) in cases {
            let parsed = input.parse::<ReferenceCollection>();
            if let Ok(parsed) = parsed {
                let formatted = parsed.to_string();
                assert_eq!(
                    formatted, expected,
                    "Roundtrip from string -> parsed -> string failed"
                );
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
                        parsed.is_valid(&bom),
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
