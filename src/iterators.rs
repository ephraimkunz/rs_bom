use crate::BOM;
use std::{cmp, fmt, iter};

impl BOM {
    pub fn verses(&self) -> VerseIter {
        VerseIter {
            bom: self,
            book_index: 0,
            chapter_index: 0,
            verse_index: 0,
        }
    }
}

#[derive(Debug)]
pub struct VerseWithReference<'v> {
    pub book: String, // Currently not a reference, because it shouldn't be too space consuming.
    pub chapter: u32,
    pub verse: u32,
    pub text: &'v str,
}

impl<'v> cmp::PartialEq for VerseWithReference<'v> {
    fn eq(&self, other: &Self) -> bool {
        self.book == other.book && self.chapter == other.chapter && self.verse == other.verse
    }
}

impl<'v> fmt::Display for VerseWithReference<'v> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{} {}: {}\n{}",
            self.book, self.chapter, self.verse, self.text
        )
    }
}

#[derive(Debug)]
pub struct VerseIter<'v> {
    bom: &'v BOM,
    book_index: usize,
    chapter_index: usize,
    verse_index: usize,
}

impl<'v> Iterator for VerseIter<'v> {
    type Item = VerseWithReference<'v>;
    fn next(&mut self) -> Option<<Self as iter::Iterator>::Item> {
        let book = self.bom.books.iter().nth(self.book_index)?;
        let chapter = book.chapters.iter().nth(self.chapter_index)?;
        let verse = chapter.verses.iter().nth(self.verse_index)?;

        let result = VerseWithReference {
            book: book.title.to_string(),
            chapter: chapter.number,
            verse: verse.number,
            text: &verse.text,
        };

        self.verse_index += 1;
        if self.verse_index >= chapter.verses.len() {
            self.verse_index = 0;
            self.chapter_index += 1;
            if self.chapter_index >= book.chapters.len() {
                self.chapter_index = 0;
                self.book_index += 1; // Any overflow dealt with then they next call next().
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Book, Chapter, Verse};

    #[test]
    fn empty_verse_iter() {
        let bom = BOM {
            title: "test title".to_string(),
            subtitle: "test subtitle".to_string(),
            translator: "test translator".to_string(),
            last_updated: "test updated".to_string(),
            language: "test language".to_string(),
            title_page_text: "test title page".to_string(),
            witness_testimonies: vec![],
            books: vec![],
        };

        let num_iterations = bom.verses().count();
        assert_eq!(num_iterations, 0);
    }

    #[test]
    fn single_book_chapter_verse_iter() {
        let bom = BOM {
            title: "test title".to_string(),
            subtitle: "test subtitle".to_string(),
            translator: "test translator".to_string(),
            last_updated: "test updated".to_string(),
            language: "test language".to_string(),
            title_page_text: "test title page".to_string(),
            witness_testimonies: vec![],
            books: vec![Book {
                title: "Testing".to_string(),
                description: None,
                chapters: vec![Chapter {
                    number: 1,
                    verses: vec![Verse {
                        number: 1,
                        text: "hello".to_string(),
                    }],
                }],
            }],
        };

        let verses: Vec<_> = bom.verses().collect();
        let num_iterations = verses.len();
        assert_eq!(num_iterations, 1);
        assert_eq!(
            verses[0],
            VerseWithReference {
                book: "Testing".to_string(),
                chapter: 1,
                verse: 1,
                text: "hello"
            }
        )
    }

    #[test]
    fn multiple_book_chapter_verse_iter() {
        let bom = BOM {
            title: "test title".to_string(),
            subtitle: "test subtitle".to_string(),
            translator: "test translator".to_string(),
            last_updated: "test updated".to_string(),
            language: "test language".to_string(),
            title_page_text: "test title page".to_string(),
            witness_testimonies: vec![],
            books: vec![
                Book {
                    title: "Testing".to_string(),
                    description: None,
                    chapters: vec![
                        Chapter {
                            number: 1,
                            verses: vec![
                                Verse {
                                    number: 1,
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    number: 2,
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                        Chapter {
                            number: 2,
                            verses: vec![
                                Verse {
                                    number: 1,
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    number: 2,
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                    ],
                },
                Book {
                    title: "Testing2".to_string(),
                    description: None,
                    chapters: vec![
                        Chapter {
                            number: 1,
                            verses: vec![
                                Verse {
                                    number: 1,
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    number: 2,
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                        Chapter {
                            number: 2,
                            verses: vec![
                                Verse {
                                    number: 1,
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    number: 2,
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                    ],
                },
            ],
        };

        let verses: Vec<_> = bom.verses().collect();
        let num_iterations = verses.len();
        assert_eq!(num_iterations, 8);

        let mut prev_chap = 0;
        let mut prev_verse = 0;
        let mut prev_book = String::new();
        for v in verses {
            match v.book.cmp(&prev_book) {
                cmp::Ordering::Less => assert!(false, "Next book should be >= previous book"),
                cmp::Ordering::Equal => match v.chapter.cmp(&prev_chap) {
                    cmp::Ordering::Less => {
                        assert!(false, "Next chapter should be >= previous chapter")
                    }
                    cmp::Ordering::Equal => match v.verse.cmp(&prev_verse) {
                        cmp::Ordering::Less | cmp::Ordering::Equal => assert!(
                            false,
                            "In the same chapter, next verse should be >= previous verse"
                        ),
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }

            prev_chap = v.chapter;
            prev_verse = v.verse;
            prev_book = v.book;
        }
    }
}
