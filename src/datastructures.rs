use std::str::FromStr;

enum BOMParseError {

}

#[derive(Debug, Default)]
struct BOM {
    title: String,
    subtitle: String,
    translator: String,
    last_updated: String,
    language: String,
    title_page_text: String,
    witness_testimonies: Vec<WitnessTestimony>, 
    books: Vec<Book>
}

impl FromStr for BOM {
    type Err = BOMParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bom = BOM::default();
        Ok(bom)
    }

}

#[derive(Debug, Default)]
struct WitnessTestimony {
    title: String,
    text: String,
    signatures: String,
}

#[derive(Debug, Default)]
struct Book {
    title: String,
    description: Option<String>,
    chapters: Vec<Chapter>
}

#[derive(Debug, Default)]
struct Chapter {
    number: u32,
    verses: Vec<Verse>
}

#[derive(Debug, Default)]
struct Verse {
    number: u32,
    text: String
}