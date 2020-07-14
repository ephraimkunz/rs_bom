use std::{fmt, path};
use thiserror::Error;

mod gutenberg;
mod iterators;
mod reference;

pub use self::gutenberg::{GutenbergParseError, GutenbergParser};
pub use self::iterators::VerseIter;
pub use self::reference::ReferenceCollection;

pub trait BOMParser {
    type Err: std::error::Error;
    fn parse(self) -> Result<BOM, Self::Err>;
}

#[derive(Debug)]
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
    pub fn from_default_parser() -> Result<Self, BOMError> {
        let corpus_path = path::Path::new("data/gutenberg.txt");
        let parser = GutenbergParser::new(corpus_path);
        let bom = parser.parse()?;
        Ok(bom)
    }

    pub fn verses_matching<I: IntoIterator<Item = VerseReference>>(
        &self,
        verse_references: I,
    ) -> impl Iterator<Item = VerseWithReference> {
        verse_references
            .into_iter()
            .filter_map(move |i| self.verse_matching(&i))
    }

    pub fn verse_matching(&self, r: &VerseReference) -> Option<VerseWithReference> {
        match r.is_valid(self) {
            true => {
                let book = &self.books[r.book_index];
                let verse = &book.chapters[r.chapter_index - 1].verses[r.verse_index - 1];
                let book_title = book.short_title.as_ref().unwrap_or(&book.title).clone();

                Some(VerseWithReference {
                    book_title,
                    reference: r.clone(),
                    text: &verse.text,
                })
            }
            false => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VerseWithReference<'v> {
    book_title: String, // Needed to display this without having to hold a reference to BOM.
    pub reference: VerseReference,
    pub text: &'v str,
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

#[derive(Error, Debug)]
pub enum BOMError {
    #[error("BOM text parsing error")]
    TextParsingError {
        #[from]
        source: GutenbergParseError,
    },

    #[error("Reference error: {0}")]
    ReferenceError(String),
}

// Everything needed to uniquely identify a single verse.
#[derive(Debug, Clone, PartialEq)]
pub struct VerseReference {
    book_index: usize,    // 0-based
    chapter_index: usize, // 1-based
    verse_index: usize,   // 1-based, None == whole chapter
}

impl VerseReference {
    fn is_valid(&self, bom: &BOM) -> bool {
        if self.chapter_index == 0 || self.verse_index == 0 {
            return false;
        }

        bom.books
            .get(self.book_index)
            .and_then(|b| b.chapters.get(self.chapter_index - 1))
            .and_then(|c| c.verses.get(self.verse_index - 1))
            .is_some()
    }
}

impl Default for VerseReference {
    fn default() -> Self {
        VerseReference {
            book_index: 0,
            chapter_index: 1,
            verse_index: 1,
        }
    }
}

#[derive(Debug)]
struct WitnessTestimony {
    title: String,
    text: String,
    signatures: String,
}

#[derive(Debug)]
struct Book {
    title: String,
    short_title: Option<String>,
    description: Option<String>,
    chapters: Vec<Chapter>,
}

#[derive(Debug)]
struct Chapter {
    verses: Vec<Verse>,
}

#[derive(Debug)]
struct Verse {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn verse_matching_bad_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = VerseReference {
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
        let reference = VerseReference {
            book_index: 0,
            chapter_index: 0,
            verse_index: 15,
        };

        assert_eq!(bom.verses_matching(iter::once(reference)).count(), 0);
    }

    // #[test]
    // fn verses_matching_good_reference() {
    //     let bom = BOM::from_default_parser().unwrap();
    //     let reference = "1 Nephi 3: 3-5".parse::<ReferenceCollection>();

    //     assert!(reference.is_ok());
    //     let reference = reference.unwrap();
    //     let verses: Vec<VerseWithReference> =
    //         bom.verses_matching(reference.verse_refs(&bom)).collect();
    //     assert_eq!(verses.len(), 3);
    //     assert_eq!(
    //         verses,
    //         vec![
    //             VerseWithReference {
    //                 book_title: "1 Nephi".to_string(),
    //                 reference: VerseReference {
    //                     book_index: 0,
    //                     chapter_index: 3,
    //                     verse_index: 3,
    //                 },
    //                 text: "For behold, Laban hath the record of the Jews and also a a genealogy of my forefathers, \
    //                 and they are engraven upon plates of brass.",
    //             },
    //             VerseWithReference {
    //                 book_title: "1 Nephi".to_string(),
    //                 reference: VerseReference {
    //                     book_index: 0,
    //                     chapter_index: 3,
    //                     verse_index: 4,
    //                 },
    //                 text: "Wherefore, the Lord hath commanded me that thou and thy brothers should go unto the house \
    //                 of Laban, and seek the records, and bring them down hither into the wilderness.",
    //             },
    //             VerseWithReference {
    //                 book_title: "1 Nephi".to_string(),
    //                 reference: VerseReference {
    //                     book_index: 0,
    //                     chapter_index: 3,
    //                     verse_index: 5,
    //                 },
    //                 text: "And now, behold thy brothers murmur, saying it is a hard thing which I have required of \
    //                 them; but behold I have not required it of them, but it is a commandment of the Lord.",
    //             }
    //         ]
    //     );
    // }
}
