use std::path;
use thiserror::Error;

mod gutenberg;
mod iterators;

pub use self::gutenberg::{GutenbergParseError, GutenbergParser};
pub use self::iterators::VerseIter;

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
    pub fn from_default_parser() -> Result<Self, BOMError>{
        let corpus_path = path::Path::new("data/gutenberg.txt");
        let parser = GutenbergParser::new(corpus_path);
        let bom = parser.parse()?;
        Ok(bom)
    }
}

#[derive(Error, Debug)]
pub enum BOMError {
    #[error("Parsing error")]
    ParsingError {
        #[from]
        source: GutenbergParseError,
    },
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
    description: Option<String>,
    chapters: Vec<Chapter>,
}

#[derive(Debug)]
struct Chapter {
    number: u32,
    verses: Vec<Verse>,
}

#[derive(Debug)]
struct Verse {
    number: u32,
    text: String,
}
