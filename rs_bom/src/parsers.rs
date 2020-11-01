/// Parser for the [Gutenberg English BOM](http://www.gutenberg.org/ebooks/17) text.
pub mod gutenberg {
    use crate::{BOMParser, Book, Chapter, Verse, WitnessTestimony, BOM};
    use once_cell::sync::Lazy;
    use regex::Regex;
    use std::{borrow::Cow, fs, io, path};
    use thiserror::Error;

    /// Errors when parsing the Gutenberg text.
    #[derive(Error, Debug)]
    pub enum ParseError {
        #[error("Gutenberg corpus not found")]
        CorpusNotFound {
            #[from]
            source: io::Error,
        },

        #[error("Corpus invalid: {0}")]
        CorpusInvalid(String),
    }

    #[derive(PartialEq)]
    enum ChunkType {
        BookTitle,
        BookDescription,
        ChapterStart,
        Verse {
            short_title: String,
            verse: String,
            verse_num: usize,
        },
        Unrecognized,
    }

    impl ChunkType {
        fn new(s: &str) -> Self {
            static CHAPTER_START: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^(\d+\s+)?[A-Za-z]+\s+\d+\nChapter\s+(?P<num>\d+)$").unwrap()
            });

            // Profiling shows that extracting capture groups from this regex is the bottleneck for
            // parsing, so I've heavily optimized it here, using both information about the shortest book name
            // and the shortest verse.
            static VERSE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"(?s)^(?P<short_title>\d?[\sA-Za-z]{4,})\s+\d{1,2}:\d{1,2}\n\s+(?P<num>\d{1,2})\s+(?P<text>.{17,})$")
                .unwrap()
            });

            match s {
                _ if s.lines().count() == 1 && s.to_uppercase() == s => Self::BookTitle,
                _ if CHAPTER_START.is_match(s) => Self::ChapterStart,
                _ if VERSE.is_match(s) => {
                    let caps = VERSE.captures(s).unwrap(); // Must be valid if is_match returned true.
                    if let Ok(num) = caps["num"].parse() {
                        Self::Verse {
                            short_title: caps["short_title"].to_string(),
                            verse: caps["text"].to_string(),
                            verse_num: num,
                        }
                    } else {
                        Self::Unrecognized
                    }
                }
                _ => Self::BookDescription,
            }
        }
    }

    /// Does the work of parsing.
    pub struct Parser {
        path: Option<path::PathBuf>,
    }

    impl Parser {
        /// Path to Gutenberg corpus. Corpus must be a single file starting with
        /// 1 Nephi 1.
        #[must_use]
        pub fn new(path: &path::Path) -> Self {
            Self {
                path: Some(path.into()),
            }
        }

        /// Use the Gutenberg corpus specified at compile time and included in the binary.
        /// This makes it more convenient when using this library as part of standalone
        /// binary, since there's no additional corpus file to copy around.
        #[must_use]
        pub const fn from_default_corpus() -> Self {
            Self { path: None }
        }

        fn corpus_text(&self) -> Result<Cow<str>, ParseError> {
            match &self.path {
                None => {
                    let s = include_str!("../data/gutenberg.txt");
                    Ok(Cow::Borrowed(s))
                }
                Some(path) => {
                    let s = fs::read_to_string(path)?;
                    Ok(Cow::Owned(s))
                }
            }
        }
    }

    impl BOMParser for Parser {
        type Err = ParseError;
        fn parse(self) -> Result<BOM, Self::Err> {
            let s = self.corpus_text()?;

            let mut bom = BOM {
                title: "The Book of Mormon".to_string(),
                subtitle: "Another Testament of Jesus Christ".to_string(),
                translator: "Joseph Smith, Jr.".to_string(),
                last_updated: "February 1, 2013".to_string(),
                language: "en".to_string(),
                title_page_text: TITLE_PAGE_TEXT.to_string(),
                witness_testimonies: vec![
                    WitnessTestimony {
                        title: THREE_WITNESS_TITLE.to_string(),
                        text: THREE_WITNESS_TEXT.to_string(),
                        signatures: THREE_WITNESS_SIGNATURES.to_string(),
                    },
                    WitnessTestimony {
                        title: EIGHT_WITNESS_TITLE.to_string(),
                        text: EIGHT_WITNESS_TEXT.to_string(),
                        signatures: EIGHT_WITNESS_SIGNATURES.to_string(),
                    },
                ],
                books: vec![],
            };

            let chunks: Vec<_> = s
                .split("\n\n")
                .filter_map(|l| {
                    if l.is_empty() {
                        None
                    } else {
                        Some(l.trim_matches('\n'))
                    }
                })
                .collect();

            let mut previous_chunk = ChunkType::Verse {
                short_title: String::new(),
                verse: String::new(),
                verse_num: 0,
            }; // So we expect a title next.

            for s in chunks {
                previous_chunk = update_book_with_chunk(s, &previous_chunk, &mut bom)?;
            }

            if bom.books.is_empty() {
                return Err(ParseError::CorpusInvalid("No books found".to_string()));
            }

            Ok(bom)
        }
    }

    fn update_book_with_chunk(
        s: &str,
        previous_chunk: &ChunkType,
        bom: &mut BOM,
    ) -> Result<ChunkType, ParseError> {
        let chunk = ChunkType::new(s);
        match chunk {
            ChunkType::BookTitle => match previous_chunk {
                ChunkType::Verse { .. } => bom.books.push(Book {
                    title: s.to_string(),
                    short_title: None,
                    description: None,
                    chapters: vec![],
                }),
                _ => {
                    return Err(ParseError::CorpusInvalid(format!(
                        "Book title in incorrect location: {}",
                        s
                    )))
                }
            },
            ChunkType::BookDescription => match previous_chunk {
                ChunkType::BookTitle => {
                    if let Some(book) = bom.books.last_mut() {
                        book.description = Some(s.to_string());
                    }
                }
                _ => {
                    return Err(ParseError::CorpusInvalid(format!(
                        "Book description in incorrect location: {}",
                        s
                    )))
                }
            },
            ChunkType::ChapterStart => match previous_chunk {
                ChunkType::BookTitle | ChunkType::BookDescription | ChunkType::Verse { .. } => {
                    if let Some(book) = bom.books.last_mut() {
                        book.chapters.push(Chapter { verses: vec![] });
                    }
                }
                _ => {
                    return Err(ParseError::CorpusInvalid(format!(
                        "Chapter start in incorrect location: {}",
                        s
                    )))
                }
            },
            ChunkType::Verse {
                ref short_title,
                ref verse,
                verse_num,
            } => {
                match previous_chunk {
                    ChunkType::BookTitle
                    | ChunkType::BookDescription
                    | ChunkType::ChapterStart
                    | ChunkType::Verse { .. } => {
                        if previous_chunk == &ChunkType::BookTitle
                            || previous_chunk == &ChunkType::BookDescription
                        {
                            // Books with only 1 chapter don't have a chapter start, so insert it here.
                            if let Some(book) = bom.books.last_mut() {
                                book.chapters.push(Chapter { verses: vec![] });
                            }
                        }

                        if let Some(chapter) = bom.books.last_mut().and_then(|b| {
                            b.short_title = Some(short_title.clone());
                            b.chapters.last_mut()
                        }) {
                            let expected_verse_number = chapter.verses.len() + 1;
                            if expected_verse_number != verse_num {
                                return Err(ParseError::CorpusInvalid(format!("Parser thought this verse was {} but text says it's verse {}: {}", expected_verse_number, verse_num, s)));
                            }

                            let v = verse.replace('\n', " ");
                            chapter.verses.push(Verse { text: v })
                        }
                    }
                    _ => {
                        return Err(ParseError::CorpusInvalid(format!(
                            "Verse in incorrect location: {}",
                            s
                        )))
                    }
                }
            }
            ChunkType::Unrecognized => {
                return Err(ParseError::CorpusInvalid(format!(
                    "Unrecognized line: {}",
                    s
                )))
            }
        }

        Ok(chunk)
    }

    const TITLE_PAGE_TEXT: &str = "THE BOOK OF MORMON

An Account Written

BY THE HAND OF MORMON

UPON PLATES

TAKEN FROM THE PLATES OF NEPHI


Wherefore, it is an abridgment of the record of the people of
Nephi, and also of the Lamanites--Written to the Lamanites, who
are a remnant of the house of Israel; and also to Jew and
Gentile--Written by way of commandment, and also by the spirit of
prophecy and of revelation--Written and sealed up, and hid up
unto the Lord, that they might not be destroyed--To come forth by
the gift and power of God unto the interpretation thereof--Sealed
by the hand of Moroni, and hid up unto the Lord, to come forth in
due time by way of the Gentile--The interpretation thereof by the
gift of God.

An abridgment taken from the Book of Ether also, which is a
record of the people of Jared, who were scattered at the time the
Lord confounded the language of the people, when they were
building a tower to get to heaven--Which is to show unto the
remnant of the House of Israel what great things the Lord hath
done for their fathers; and that they may know the covenants of
the Lord, that they are not cast off forever--And also to the
convincing of the Jew and Gentile that JESUS is the CHRIST, the
ETERNAL GOD, manifesting himself unto all nations--And now, if
there are faults they are the mistakes of men; wherefore, condemn
not the things of God, that ye may be found spotless at the
judgment-seat of Christ.

TRANSLATED BY JOSEPH SMITH, JUN.";

    const THREE_WITNESS_TITLE: &str = "THE TESTIMONY OF THREE WITNESSES";

    const THREE_WITNESS_TEXT: &str =
        "Be it known unto all nations, kindreds, tongues, and people, unto
whom this work shall come: That we, through the grace of God the
Father, and our Lord Jesus Christ, have seen the plates which
contain this record, which is a record of the people of Nephi,
and also of the Lamanites, their brethren, and also of the people
of Jared, who came from the tower of which hath been spoken. And
we also know that they have been translated by the gift and power
of God, for his voice hath declared it unto us; wherefore we know
of a surety that the work is true. And we also testify that we
have seen the engravings which are upon the plates; and they have
been shown unto us by the power of God, and not of man. And we
declare with words of soberness, that an angel of God came down
from heaven, and he brought and laid before our eyes, that we
beheld and saw the plates, and the engravings thereon; and we
know that it is by the grace of God the Father, and our Lord
Jesus Christ, that we beheld and bear record that these things
are true. And it is marvelous in our eyes. Nevertheless, the
voice of the Lord commanded us that we should bear record of it;
wherefore, to be obedient unto the commandments of God, we bear
testimony of these things. And we know that if we are faithful
in Christ, we shall rid our garments of the blood of all men, and
be found spotless before the judgment-seat of Christ, and shall
dwell with him eternally in the heavens. And the honor be to the
Father, and to the Son, and to the Holy Ghost, which is one God.
Amen.";

    const THREE_WITNESS_SIGNATURES: &str = "OLIVER COWDERY
DAVID WHITMER
MARTIN HARRIS";

    const EIGHT_WITNESS_TITLE: &str = "THE TESTIMONY OF EIGHT WITNESSES";

    const EIGHT_WITNESS_TEXT: &str =
        "Be it known unto all nations, kindreds, tongues, and people, unto
whom this work shall come: That Joseph Smith, Jun., the
translator of this work, has shown unto us the plates of which
hath been spoken, which have the appearance of gold; and as many
of the leaves as the said Smith has translated we did handle with
our hands; and we also saw the engravings thereon, all of which
has the appearance of ancient work, and of curious workmanship.
And this we bear record with words of soberness, that the said
Smith has shown unto us, for we have seen and hefted, and know of
a surety that the said Smith has got the plates of which we have
spoken. And we give our names unto the world, to witness unto
the world that which we have seen. And we lie not, God bearing
witness of it.";

    const EIGHT_WITNESS_SIGNATURES: &str = "CHRISTIAN WHITMER
JACOB WHITMER
PETER WHITMER, JUN.
JOHN WHITMER
HIRAM PAGE
JOSEPH SMITH, SEN.
HYRUM SMITH
SAMUEL H. SMITH";

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn error_for_empty_document() {
            let parser = Parser::new(path::Path::new("testdata/empty_file.txt"));
            assert!(parser.parse().is_err())
        }

        #[test]
        fn error_for_invalid_path() {
            let parser = Parser::new(path::Path::new("testing123"));
            assert!(parser.parse().is_err())
        }

        #[test]
        fn error_for_invalid_document() {
            let parser = Parser::new(path::Path::new("testdata/bad_data_file.txt"));
            assert!(parser.parse().is_err())
        }
    }
}
