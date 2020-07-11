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

    pub fn verse(&self, reference: &Reference) -> Option<VerseWithReference> {
        match reference.is_valid(self) {
            true => {
                let book = &self.books[reference.book_index];
                let verse =
                    &book.chapters[reference.chapter_index - 1].verses[reference.verse_index - 1];
                let book_title = book.short_title.as_ref().unwrap_or(&book.title).clone();
                Some(VerseWithReference {
                    book_title,
                    reference: reference.clone(),
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
    pub reference: Reference,
    pub text: &'v str,
}

impl<'v> fmt::Display for VerseWithReference<'v> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{} {}: {}\n{}",
            self.book_title, self.reference.chapter_index, self.reference.verse_index, self.text
        )
    }
}

#[derive(Error, Debug)]
pub enum BOMError {
    #[error("Parsing error")]
    ParsingError {
        #[from]
        source: GutenbergParseError,
    },

    #[error("Reference error")]
    ReferenceError {},
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    book_index: usize,    // 0-based
    chapter_index: usize, // 1-based
    verse_index: usize,   // 1-based
}

impl Reference {
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

impl Default for Reference {
    fn default() -> Self {
        Reference {
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

    #[test]
    fn verse_matching_bad_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = Reference {
            book_index: 1,
            chapter_index: 0,
            verse_index: 0,
        };

        assert_eq!(bom.verse(&reference), None);
    }

    #[test]
    fn verse_matching_good_reference() {
        let bom = BOM::from_default_parser().unwrap();
        let reference = Reference {
            book_index: 0,
            chapter_index: 2,
            verse_index: 15,
        };

        assert_eq!(
            bom.verse(&reference),
            Some(VerseWithReference {
                book_title: "1 Nephi".to_string(),
                reference: reference.clone(),
                text: "And my father dwelt in a tent.",
            })
        );
    }
}
