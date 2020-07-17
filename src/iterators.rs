use crate::{VerseReference, VerseWithReference, BOM};
use std::iter;

impl BOM {
    /// Iterate over all verses in the entire book.
    pub fn verses(&self) -> impl Iterator<Item = VerseWithReference> {
        VerseIter {
            bom: self,
            position: VerseReference::default(),
        }
    }
}

#[derive(Debug)]
struct VerseIter<'v> {
    bom: &'v BOM,
    position: VerseReference,
}

impl<'v> Iterator for VerseIter<'v> {
    type Item = VerseWithReference<'v>;
    fn next(&mut self) -> Option<<Self as iter::Iterator>::Item> {
        let book = self.bom.books.get(self.position.book_index)?;
        let chapter = book.chapters.get(self.position.chapter_index - 1)?;
        let verse = chapter.verses.get(self.position.verse_index - 1)?;

        let result = VerseWithReference {
            reference: self.position.clone(),
            book_title: book.short_title.as_ref().unwrap_or(&book.title).clone(),
            text: &verse.text,
        };

        self.position.verse_index += 1;
        if self.position.verse_index > chapter.verses.len() {
            self.position.verse_index = 1;
            self.position.chapter_index += 1;
            if self.position.chapter_index > book.chapters.len() {
                self.position.chapter_index = 1;
                self.position.book_index += 1; // Any overflow dealt with then they next call next().
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Book, Chapter, Verse};
    use std::cmp;

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
                short_title: None,
                description: None,
                chapters: vec![Chapter {
                    verses: vec![Verse {
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
                book_title: "Testing".to_string(),
                reference: VerseReference {
                    book_index: 0,
                    chapter_index: 1,
                    verse_index: 1,
                },
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
                    short_title: None,
                    description: None,
                    chapters: vec![
                        Chapter {
                            verses: vec![
                                Verse {
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                        Chapter {
                            verses: vec![
                                Verse {
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                    ],
                },
                Book {
                    title: "Testing2".to_string(),
                    short_title: None,
                    description: None,
                    chapters: vec![
                        Chapter {
                            verses: vec![
                                Verse {
                                    text: "hello".to_string(),
                                },
                                Verse {
                                    text: "hello".to_string(),
                                },
                            ],
                        },
                        Chapter {
                            verses: vec![
                                Verse {
                                    text: "hello".to_string(),
                                },
                                Verse {
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
        let mut prev_book = 0;
        for v in verses {
            match v.reference.book_index.cmp(&prev_book) {
                cmp::Ordering::Less => assert!(false, "Next book should be >= previous book"),
                cmp::Ordering::Equal => match v.reference.chapter_index.cmp(&prev_chap) {
                    cmp::Ordering::Less => {
                        assert!(false, "Next chapter should be >= previous chapter")
                    }
                    cmp::Ordering::Equal => match v.reference.verse_index.cmp(&prev_verse) {
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

            prev_chap = v.reference.chapter_index;
            prev_verse = v.reference.verse_index;
            prev_book = v.reference.book_index;
        }
    }
}
