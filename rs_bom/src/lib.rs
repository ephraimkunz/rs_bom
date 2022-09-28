use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

mod iterators;
mod parsers;
mod reference;

pub use self::parsers::gutenberg;
pub use self::reference::{RangeCollection, VerseReference, Work};

/// Plugin interface for creating a new Book of Mormon parser. Primarily designed
/// to make it easier to add new languages later.
pub trait BOMParser {
    /// Error type to be returned if there is a failure while parsing.
    type Err: std::error::Error;
    /// Parse using the parser-specific implementation.
    /// # Errors
    ///
    /// Customize type of errors returned with `Err` associated type.
    fn parse(self) -> Result<BOM, Self::Err>;
}

/// Represents a parsed copy of the Book of Mormon.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BOM {
    title: String,
    subtitle: String,
    translator: String,
    last_updated: String,
    language: String,
    title_page_text: String,
    witness_testimonies: Vec<WitnessTestimony>,
    books: Vec<Book>,
}

impl BOM {
    /// Creates a `BOM` by using the default parser.
    /// # Errors
    ///
    /// Will return `Err` if there is an error parsing the backing corpus.
    // This could happen if the corpus is corrupt, non-existant, or doesn't
    // match the expected format.
    pub fn from_default_parser() -> Result<Self, BOMError> {
        let parser = gutenberg::Parser::from_default_corpus();
        let bom = parser.parse()?;
        Ok(bom)
    }

    /// Return an iterator of verses matching the given `RangeCollection`. Any invalid
    /// verses in the `RangeCollection` are skipped.
    pub fn verses_matching(
        &self,
        range_collection: &RangeCollection,
    ) -> impl Iterator<Item = VerseWithReference> {
        range_collection
            .verse_refs(self)
            .filter_map(move |i| self.verse_matching(&i))
    }

    /// Return a single verse matching the given verse reference. If the reference is invalid,
    /// returns `None`.
    #[must_use]
    pub fn verse_matching(&self, r: &VerseReference) -> Option<VerseWithReference> {
        if r.is_valid(self) {
            let book = &self.books[r.book_index];
            let verse = &book.chapters[r.chapter_index - 1].verses[r.verse_index - 1];
            let book_title = book.short_title.as_ref().unwrap_or(&book.title).clone();

            Some(VerseWithReference {
                book_title,
                reference: r.clone(),
                text: &verse.text,
            })
        } else {
            None
        }
    }
}

/// Represents the text of a verse and it's reference.
#[derive(Debug, PartialEq, Eq)]
pub struct VerseWithReference<'v> {
    book_title: String, // Needed to display this without having to hold a reference to BOM.
    /// Reference of this verse.
    pub reference: VerseReference,
    /// Raw text of the verse.
    pub text: &'v str,
}

impl<'v> VerseWithReference<'v> {
    pub fn to_html_string(&self) -> String {
        format!(
            "<h3><a href=\"{}\">{} {}:{}</a></h3> <p>{}</p>",
            self.reference.url().unwrap_or_default(),
            self.book_title,
            self.reference.chapter_index,
            self.reference.verse_index,
            self.text
        )
    }
}

impl<'v> fmt::Display for VerseWithReference<'v> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{} {}:{}\n{}",
            self.book_title, self.reference.chapter_index, self.reference.verse_index, self.text
        )
    }
}

/// All possible errors that this library can return.
#[derive(Error, Debug)]
pub enum BOMError {
    #[error("BOM text parsing error")]
    TextParsingError {
        #[from]
        source: gutenberg::ParseError,
    },

    #[error("Reference error: {0}")]
    ReferenceError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WitnessTestimony {
    title: String,
    text: String,
    signatures: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Book {
    title: String,
    short_title: Option<String>,
    description: Option<String>,
    chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chapter {
    verses: Vec<Verse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Verse {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verse_matching_bad_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = VerseReference {
            work: Work::BookOfMormon,
            book_index: 1,
            chapter_index: 0,
            verse_index: 0,
        };

        assert_eq!(bom.verse_matching(&reference), None);
    }

    #[test]
    fn verse_matching_good_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = VerseReference {
            work: Work::BookOfMormon,
            book_index: 0,
            chapter_index: 2,
            verse_index: 15,
        };

        assert_eq!(
            bom.verse_matching(&reference),
            Some(VerseWithReference {
                book_title: "1 Nephi".to_string(),
                reference: reference.clone(),
                text: "And my father dwelt in a tent.",
            })
        );
    }

    #[test]
    fn verses_matching_bad_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference: RangeCollection = "1 Nephi 0: 1".parse().unwrap();
        assert_eq!(bom.verses_matching(&reference).count(), 0);
    }

    #[test]
    fn display_verse() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = VerseReference::new(Work::BookOfMormon, 0, 1, 1);
        let verse = bom.verse_matching(&reference).unwrap();
        assert_eq!(
            verse.to_string(),
            "1 Nephi 1:1\nI, Nephi, having been born of goodly parents, therefore I was \
        taught somewhat in all the learning of my father; and having seen \
        many afflictions in the course of my days, nevertheless, having \
        been highly favored of the Lord in all my days; yea, having had a \
        great knowledge of the goodness and the mysteries of God, \
        therefore I make a record of my proceedings in my days."
        )
    }

    #[test]
    fn verses_matching_good_reference_verse_ranges() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = "1 Nephi 3: 3-5".parse::<RangeCollection>();

        assert!(reference.is_ok());
        let reference = reference.unwrap();
        let verses: Vec<VerseWithReference> = bom.verses_matching(&reference).collect();
        assert_eq!(verses.len(), 3);
        assert_eq!(
            verses,
            vec![
                VerseWithReference {
                    book_title: "1 Nephi".to_string(),
                    reference: VerseReference {
                        work: Work::BookOfMormon,
                        book_index: 0,
                        chapter_index: 3,
                        verse_index: 3,
                    },
                    text: "For behold, Laban hath the record of the Jews and also a \
                    genealogy of my forefathers, and they are engraven upon plates of \
                    brass.",
                },
                VerseWithReference {
                    book_title: "1 Nephi".to_string(),
                    reference: VerseReference {
                        work: Work::BookOfMormon,
                        book_index: 0,
                        chapter_index: 3,
                        verse_index: 4,
                    },
                    text: "Wherefore, the Lord hath commanded me that thou and thy \
                    brothers should go unto the house of Laban, and seek the records, \
                    and bring them down hither into the wilderness.",
                },
                VerseWithReference {
                    book_title: "1 Nephi".to_string(),
                    reference: VerseReference {
                        work: Work::BookOfMormon,
                        book_index: 0,
                        chapter_index: 3,
                        verse_index: 5,
                    },
                    text: "And now, behold thy brothers murmur, saying it is a hard thing \
                    which I have required of them; but behold I have not required it \
                    of them, but it is a commandment of the Lord.",
                }
            ]
        );
    }

    #[test]
    fn verses_matching_good_reference_chapter_ranges() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = "1 Nephi 3-5".parse::<RangeCollection>();

        assert!(reference.is_ok());
        let reference = reference.unwrap();
        let verses: Vec<VerseWithReference> = bom.verses_matching(&reference).collect();
        assert_eq!(verses.len(), 91);
        assert_eq!(
            verses.first().unwrap(),
            &VerseWithReference {
                book_title: "1 Nephi".to_string(),
                reference: VerseReference {
                    work: Work::BookOfMormon,
                    book_index: 0,
                    chapter_index: 3,
                    verse_index: 1,
                },
                text: "And it came to pass that I, Nephi, returned from speaking with \
                the Lord, to the tent of my father.",
            }
        );
    }
}
