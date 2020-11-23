use crate::{BOMError, BOM};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::{cmp, fmt, str};

const CITATION_DELIM: char = ';';
const VERSE_CHUNK_DELIM: char = ',';
const CHAPTER_VERSE_DELIM: char = ':';
const RANGE_DELIM_CANONICAL: char = '–'; // en-dash
const RANGE_DELIM_NON_CANONICAL1: char = '-'; // regular dash
const RANGE_DELIM_NON_CANONICAL2: char = '—'; // em-dash

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum Work {
    OldTestament,
    NewTestament,
    BookOfMormon,
}

impl Work {
    fn url_name(&self) -> &'static str {
        match self {
            Self::OldTestament => "ot",
            Self::NewTestament => "nt",
            Self::BookOfMormon => "bofm",
        }
    }
}

/// Everything needed to uniquely identify a single verse in a work of scripture.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerseReference {
    pub(super) work: Work,
    pub(super) book_index: usize,    // 0-based
    pub(super) chapter_index: usize, // 1-based
    pub(super) verse_index: usize,   // 1-based, None == whole chapter
}

impl VerseReference {
    /// Create a verse reference from parts.
    /// # Arguments
    /// * `work`: Work enum
    /// * `book_index`: 0 indexed, 0 = 1 Nephi, etc.
    /// * `chapter_index`: 1-indexed
    /// * `verse_index`: 1-indexed
    #[must_use]
    pub const fn new(
        work: Work,
        book_index: usize,
        chapter_index: usize,
        verse_index: usize,
    ) -> Self {
        Self {
            work,
            book_index,
            chapter_index,
            verse_index,
        }
    }

    pub fn is_valid(&self, bom: &BOM) -> bool {
        if self.chapter_index == 0 || self.verse_index == 0 {
            return false;
        }

        bom.books
            .get(self.book_index)
            .and_then(|b| b.chapters.get(self.chapter_index - 1))
            .and_then(|c| c.verses.get(self.verse_index - 1))
            .is_some()
    }

    pub fn url(&self) -> Option<String> {
        let range_collection = RangeCollection::from_verse_ref(self);
        range_collection.url()
    }
}

struct BookData {
    work: Work,
    long_name: &'static str,
    short_name: &'static str,
    url_name: &'static str,
    book_index: usize,
}

impl BookData {
    fn new(
        work: Work,
        long_name: &'static str,
        short_name: &'static str,
        url_name: &'static str,
        book_index: usize,
    ) -> BookData {
        BookData {
            work,
            long_name,
            short_name,
            url_name,
            book_index,
        }
    }
}

#[rustfmt::skip]
static BOOK_DATA: Lazy<Vec<BookData>> = Lazy::new(|| {
        vec![
        // Old Testament
        BookData::new(Work::OldTestament, "Genesis", "Gen.", "gen", 0),
        BookData::new(Work::OldTestament, "Exodus", "Ex.", "ex", 1),
        BookData::new(Work::OldTestament, "Leviticus", "Lev.", "lev", 2),
        BookData::new(Work::OldTestament, "Numbers", "Num.", "num", 3),
        BookData::new(Work::OldTestament, "Deuteronomy", "Deut.", "deut", 4),
        BookData::new(Work::OldTestament, "Joshua", "Josh.", "josh", 5),
        BookData::new(Work::OldTestament, "Judges", "Judg.", "judg", 6),
        BookData::new(Work::OldTestament, "Ruth", "Ruth", "ruth", 7),
        BookData::new(Work::OldTestament, "1 Samuel", "1 Sam.", "1-sam", 8),
        BookData::new(Work::OldTestament, "2 Samuel", "2 Sam.", "2-sam", 9),
        BookData::new(Work::OldTestament, "1 Kings", "1 Kgs.", "1-kgs", 10),
        BookData::new(Work::OldTestament, "2 Kings", "2 Kgs.", "2-kgs", 11),
        BookData::new(Work::OldTestament, "1 Chronicles", "1 Chron.", "1-chron", 12,),
        BookData::new(Work::OldTestament, "2 Chronicles", "2 Chron.", "2-chron", 13,),
        BookData::new(Work::OldTestament, "Ezra", "Ezra", "ezra", 14),
        BookData::new(Work::OldTestament, "Nehemiah", "Neh.", "neh", 15),
        BookData::new(Work::OldTestament, "Esther", "Esth.", "esth", 16),
        BookData::new(Work::OldTestament, "Job", "Job", "job", 17),
        BookData::new(Work::OldTestament, "Psalms", "Ps.", "ps", 18),
        BookData::new(Work::OldTestament, "Proverbs", "Prov.", "prov", 19),
        BookData::new(Work::OldTestament, "Ecclesiastes", "Eccl.", "eccl", 20),
        BookData::new(Work::OldTestament, "Song of Solomon", "Song.", "song", 21),
        BookData::new(Work::OldTestament, "Isaiah", "Isa.", "isa", 22),
        BookData::new(Work::OldTestament, "Jeremiah", "Jer.", "jer", 23),
        BookData::new(Work::OldTestament, "Lamentations", "Lam.", "lam", 24),
        BookData::new(Work::OldTestament, "Ezekiel", "Ezek.", "ezek", 25),
        BookData::new(Work::OldTestament, "Daniel", "Dan.", "dan", 26),
        BookData::new(Work::OldTestament, "Hosea", "Hosea", "hosea", 27),
        BookData::new(Work::OldTestament, "Joel", "Joel", "joel", 28),
        BookData::new(Work::OldTestament, "Amos", "Amos", "amos", 29),
        BookData::new(Work::OldTestament, "Obadiah", "Obad.", "obad", 30),
        BookData::new(Work::OldTestament, "Jonah", "Jonah", "jonah", 31),
        BookData::new(Work::OldTestament, "Micah", "Micah", "micah", 32),
        BookData::new(Work::OldTestament, "Nahum", "Nahum", "nahum", 33),
        BookData::new(Work::OldTestament, "Habakkuk", "Hab.", "hab", 34),
        BookData::new(Work::OldTestament, "Zephaniah", "Zeph.", "zeph", 35),
        BookData::new(Work::OldTestament, "Haggai", "Hag.", "hag", 36),
        BookData::new(Work::OldTestament, "Zechariah", "Zech.", "zech", 37),
        BookData::new(Work::OldTestament, "Malachi", "Mal.", "mal", 38),
        // New Testament
        BookData::new(Work::NewTestament, "Matthew", "Matt.", "matt", 0),
        BookData::new(Work::NewTestament, "Mark", "Mark", "mark", 1),
        BookData::new(Work::NewTestament, "Luke", "Luke", "luke", 2),
        BookData::new(Work::NewTestament, "John", "John", "john", 3),
        BookData::new(Work::NewTestament, "Acts", "Acts", "acts", 4),
        BookData::new(Work::NewTestament, "Romans", "Rom.", "rom", 5),
        BookData::new(Work::NewTestament, "1 Corinthians", "1 Cor.", "1-cor", 6),
        BookData::new(Work::NewTestament, "2 Corinthians", "2 Cor.", "2-cor", 7),
        BookData::new(Work::NewTestament, "Galatians", "Gal.", "gal", 8),
        BookData::new(Work::NewTestament, "Ephesians", "Eph.", "eph", 9),
        BookData::new(Work::NewTestament, "Philippians", "Philip.", "philip", 10),
        BookData::new(Work::NewTestament, "Colossians", "Col.", "col", 11),
        BookData::new(Work::NewTestament, "1 Thessalonians", "1 Thes.", "1-thes", 12,),
        BookData::new(Work::NewTestament, "2 Thessalonians", "2 Thes.", "2-thes", 13,),
        BookData::new(Work::NewTestament, "1 Timothy", "1 Tim.", "1-tim", 14),
        BookData::new(Work::NewTestament, "2 Timothy", "2 Tim.", "2-tim", 15),
        BookData::new(Work::NewTestament, "Titus", "Titus", "titus", 16),
        BookData::new(Work::NewTestament, "Philemon", "Philem.", "philem", 17),
        BookData::new(Work::NewTestament, "Hebrews", "Heb.", "heb", 18),
        BookData::new(Work::NewTestament, "James", "James", "james", 19),
        BookData::new(Work::NewTestament, "1 Peter", "1 Pet.", "1-pet", 20),
        BookData::new(Work::NewTestament, "2 Peter", "2 Pet.", "2-pet", 21),
        BookData::new(Work::NewTestament, "1 John", "1 Jn.", "1-jn", 22),
        BookData::new(Work::NewTestament, "2 John", "2 Jn.", "2-jn", 23),
        BookData::new(Work::NewTestament, "3 John", "3 Jn.", "3-jn", 24),
        BookData::new(Work::NewTestament, "Jude", "Jude", "jude", 25),
        BookData::new(Work::NewTestament, "Revelation", "Rev.", "rev", 26),
        // Book of Mormon
        BookData::new(Work::BookOfMormon, "1 Nephi", "1 Ne.", "1-ne", 0),
        BookData::new(Work::BookOfMormon, "2 Nephi", "2 Ne.", "2-ne", 1),
        BookData::new(Work::BookOfMormon, "Jacob", "Jacob", "jacob", 2),
        BookData::new(Work::BookOfMormon, "Enos", "Enos", "enos", 3),
        BookData::new(Work::BookOfMormon, "Jarom", "Jarom", "jarom", 4),
        BookData::new(Work::BookOfMormon, "Omni", "Omni", "omni", 5),
        BookData::new(Work::BookOfMormon, "Words of Mormon", "W of M", "w-of-m", 6),
        BookData::new(Work::BookOfMormon, "Mosiah", "Mosiah", "mosiah", 7),
        BookData::new(Work::BookOfMormon, "Alma", "Alma", "alma", 8),
        BookData::new(Work::BookOfMormon, "Helaman", "Hel.", "hel", 9),
        BookData::new(Work::BookOfMormon, "3 Nephi", "3 Ne.", "3-ne", 10),
        BookData::new(Work::BookOfMormon, "4 Nephi", "4 Ne.", "4-ne", 11),
        BookData::new(Work::BookOfMormon, "Mormon", "Morm.", "morm", 12),
        BookData::new(Work::BookOfMormon, "Ether", "Ether", "ether", 13),
        BookData::new(Work::BookOfMormon, "Moroni", "Moro.", "moro", 14),
    ]
});

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
    const fn chapter_range(&self) -> (usize, usize) {
        match self {
            Self::StartEndChapter { start, end } => (*start, *end),
            Self::StartEndVerse { chapter, .. } => (*chapter, *chapter),
        }
    }

    const fn verse_range(&self) -> Option<(usize, usize)> {
        match self {
            Self::StartEndChapter { .. } => None,
            Self::StartEndVerse { start, end, .. } => Some((*start, *end)),
        }
    }
}

impl PartialOrd for RangeType {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RangeType {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (
                Self::StartEndVerse {
                    chapter: c,
                    start: s,
                    end: e,
                },
                Self::StartEndVerse {
                    chapter: oc,
                    start: os,
                    end: oe,
                },
            ) => match c.cmp(oc) {
                cmp::Ordering::Equal => match s.cmp(os) {
                    cmp::Ordering::Equal => e.cmp(oe),
                    comp => comp,
                },
                comp => comp,
            },
            (
                Self::StartEndVerse { chapter: c, .. },
                Self::StartEndChapter { start: os, end: oe },
            ) => match c.cmp(os) {
                cmp::Ordering::Equal => c.cmp(oe),
                comp => comp,
            },
            (
                Self::StartEndChapter { start: os, end: oe },
                Self::StartEndVerse { chapter: c, .. },
            ) => match os.cmp(c) {
                cmp::Ordering::Equal => oe.cmp(c),
                comp => comp,
            },
            (
                Self::StartEndChapter { start: s, end: e },
                Self::StartEndChapter { start: os, end: oe },
            ) => match s.cmp(os) {
                cmp::Ordering::Equal => e.cmp(oe),
                comp => comp,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct VerseRangeReference {
    range_type: RangeType,
    book_index: usize,
    work: Work,
}

impl PartialOrd for VerseRangeReference {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VerseRangeReference {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.book_index.cmp(&other.book_index) {
            cmp::Ordering::Equal => self.range_type.cmp(&other.range_type),
            comp => comp,
        }
    }
}

impl VerseRangeReference {
    const fn verse_refs<'a, 'b>(&'b self, bom: &'a BOM) -> VerseRangeReferenceIter<'a, 'b> {
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

                book.and_then(|b| b.chapters.get(start - 1)).is_some()
                    && book.and_then(|b| b.chapters.get(end - 1)).is_some()
            }
            RangeType::StartEndVerse {
                chapter,
                start,
                end,
            } => {
                if chapter == 0 || start == 0 || end == 0 {
                    return false;
                }

                book.and_then(|b| b.chapters.get(chapter - 1))
                    .and_then(|c| c.verses.get(start - 1))
                    .is_some()
                    && book
                        .and_then(|b| b.chapters.get(chapter - 1))
                        .and_then(|c| c.verses.get(end - 1))
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
                        work: Work::BookOfMormon,
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
                        work: Work::BookOfMormon,
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
struct RangeCollectionIter {
    data: Vec<VerseReference>,
    index: usize,
}

impl Iterator for RangeCollectionIter {
    type Item = VerseReference;
    fn next(&mut self) -> Option<VerseReference> {
        let data = self.data.get(self.index).cloned();
        self.index += 1;
        data
    }
}

/// Represents a collection of verses that may include ranges of verses or chapters.
#[derive(Debug)]
pub struct RangeCollection {
    refs: Vec<VerseRangeReference>,
}

impl RangeCollection {
    /// Parses a given string `s` into an iterable collection.
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/Bible_citation) for some examples
    /// of reference string that can be parsed.
    /// # Errors
    ///
    /// Will return `Err` if `s` does not match a valid reference format.
    /// Note that just because a reference parses does not make it valid.
    /// Validity of a reference in a given book can be checked with `is_valid`.
    pub fn new(s: &str) -> Result<Self, BOMError> {
        s.parse()
    }

    pub fn from_verse_ref(verseref: &VerseReference) -> Self {
        Self {
            refs: vec![VerseRangeReference {
                range_type: RangeType::StartEndVerse {
                    chapter: verseref.chapter_index,
                    start: verseref.verse_index,
                    end: verseref.verse_index,
                },
                book_index: verseref.book_index,
                work: Work::BookOfMormon,
            }],
        }
    }

    pub fn url(&self) -> Option<String> {
        let r = &self.refs[0];
        let s = match r.range_type {
            RangeType::StartEndVerse {
                chapter,
                start,
                end,
            } => {
                let work = r.work.url_name();
                let book = BOOK_DATA
                    .iter()
                    .find(|d| d.work == r.work && d.book_index == r.book_index)
                    .expect("Failed to find book data for valid ref, should be impossible")
                    .url_name;
                format!(
                    "https://www.churchofjesuschrist.org/study/scriptures/{}/{}/{}?lang=eng&id=p{}-p{}#p{}",
                    work, book, chapter, start, end, start
                )
            }
            _ => return None,
        };

        Some(s)
    }

    /// Returns whether this is a valid collection. Validity means that all chapters, books,
    /// and verses specified are actually navigable references in `BOM`.
    #[must_use]
    pub fn is_valid(&self, bom: &BOM) -> bool {
        self.refs.iter().all(|r| r.is_valid(bom))
    }

    /// Iterate over the `RangeCollection`, producing `VerseReference`s.
    pub fn verse_refs(&self, bom: &BOM) -> impl Iterator<Item = VerseReference> {
        // I don't think it's very efficient to eagerly collect this iter, but I don't know how to store
        // an "in-use" iterator in struct without generators.
        let data = self.refs.iter().flat_map(|r| r.verse_refs(bom)).collect();
        RangeCollectionIter { data, index: 0 }
    }

    /// Canonicalize the `RangeCollection`. Canonicalization means sorting by the book title,
    /// using standardized book names and symbols, and collapsing ranges of chapters and verses.
    pub fn canonicalize(&mut self) {
        // Sort collection by book, chapter / chapter range, verse / verse range.
        self.refs.sort();
        let mut new_refs = vec![];

        // Collapse ranges
        let mut current_ref = self.refs[0].clone();
        let mut current_book = current_ref.book_index;
        let mut current_work = current_ref.work;
        let mut current_chap_range = current_ref.range_type.chapter_range();
        let mut current_verse_range = current_ref.range_type.verse_range();
        new_refs.push(current_ref);

        for r in self.refs.iter().skip(1) {
            let chap_range = r.range_type.chapter_range();
            let verse_range = r.range_type.verse_range();

            let in_same_work = r.work == current_work;
            let in_same_book = r.book_index == current_book;
            let overlapping_chapter_ranges =
                chap_range.0 >= current_chap_range.0 && chap_range.0 <= (current_chap_range.1 + 1);
            let is_collapsible = in_same_work && in_same_book && overlapping_chapter_ranges;
            if is_collapsible {
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
                                work: current_work,
                            };

                            current_ref = combined_ref.clone();
                            current_book = current_ref.book_index;
                            current_work = current_ref.work;
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
                                work: current_work,
                            };

                            current_ref = combined_ref.clone();
                            current_book = current_ref.book_index;
                            current_work = current_ref.work;
                            current_chap_range = current_ref.range_type.chapter_range();
                            current_verse_range = current_ref.range_type.verse_range();

                            new_refs.pop();
                            new_refs.push(combined_ref);
                            continue;
                        }
                    }
                    _ => {
                        // We know that they have overlapping chapter ranges, and that one is a full chapter (None).
                        // The right way to handle this is to keep the full chapter and eliminate single verses in it.
                        if verse_range.is_none() {
                            // Keep the new range.
                            let combined_ref = VerseRangeReference {
                                book_index: current_book,
                                range_type: RangeType::StartEndChapter {
                                    start: chap_range.0,
                                    end: chap_range.1,
                                },
                                work: current_work,
                            };

                            current_ref = combined_ref.clone();
                            current_book = current_ref.book_index;
                            current_work = current_ref.work;
                            current_chap_range = current_ref.range_type.chapter_range();
                            current_verse_range = current_ref.range_type.verse_range();

                            new_refs.pop();
                            new_refs.push(combined_ref);
                        }
                        // Since we'll just take 1 of the two references, either remove the existing one on the array
                        // and add a new one (above), or keep the one already and don't add or remove anything (here).
                        continue;
                    }
                }
            }

            // Nothing to collapse, just add the reference.
            current_ref = r.clone();
            current_book = current_ref.book_index;
            current_work = current_ref.work;
            current_chap_range = current_ref.range_type.chapter_range();
            current_verse_range = current_ref.range_type.verse_range();
            new_refs.push(r.clone());
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
impl str::FromStr for RangeCollection {
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
                    let (end_of_name_index, book_index, work) = extract_book_name(first_chunk)?;
                    let mut chapter_chunks = vec![&first_chunk[end_of_name_index..]];
                    chapter_chunks.extend(&chapter_chunk_split[1..]);

                    for chapter_chunk in chapter_chunks {
                        let (start, end) = extract_range(chapter_chunk)?;
                        let reference = VerseRangeReference {
                            book_index,
                            range_type: RangeType::StartEndChapter { start, end },
                            work,
                        };
                        references.push(reference);
                    }
                }
                2 => {
                    // First is the chapter, everything else is the verse.
                    let book_chapter_chunk = chapter_verse_split[0];
                    let (end_of_name_index, book_index, work) =
                        extract_book_name(book_chapter_chunk).or_else(|e| {
                            // Use the previous book if it exists.
                            references
                                .last()
                                .map_or(Err(e), |prev| Ok((0, prev.book_index, prev.work)))
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
                            work,
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

        Ok(Self { refs: references })
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

fn book_data_from_candidate_title(candidate: &str) -> Option<&BookData> {
    BOOK_DATA
        .iter()
        .find(|d| d.long_name == candidate || d.short_name == candidate)
}

fn extract_book_name(s: &str) -> Result<(usize, usize, Work), BOMError> {
    static POSSIBLE_BOOK_NAME: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(?P<name>(\d\s)?[A-Za-z ]+\.?)\s+").unwrap());

    let s_trimmed = s.trim();
    if POSSIBLE_BOOK_NAME.is_match(s_trimmed) {
        let caps = POSSIBLE_BOOK_NAME.captures(s_trimmed).ok_or_else(|| {
            BOMError::ReferenceError(format!("Book name not found as expected in {}", s_trimmed))
        })?;
        let cap = caps["name"].trim();
        let trimmed = cap.trim();
        if let Some(book_data) = book_data_from_candidate_title(trimmed) {
            let index = s.find(trimmed).unwrap(); // We just found it via regex.
            return Ok((index + trimmed.len(), book_data.book_index, book_data.work));
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

impl fmt::Display for RangeCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.refs.is_empty() {
            return Ok(());
        }

        // Use values guaranteed to not be the first.
        let mut previous_book = 1000;
        let mut previous_chapter = 1000;
        let mut previous_work: Option<Work> = None;

        for (i, reference) in self.refs.iter().enumerate() {
            let new_book = previous_book != reference.book_index;
            let new_work = previous_work.is_none() || previous_work.unwrap() != reference.work;
            let new_book_title = new_book || new_work;
            if new_book_title {
                if i != 0 {
                    write!(f, "{} ", CITATION_DELIM)?;
                }

                // It should be impossible to create a RangeCollection with an invalid book index (since it would
                // have failed to parse the string), so we can be sure it's legitimate at this point.
                let book_data = BOOK_DATA
                    .iter()
                    .find(|d| d.work == reference.work && d.book_index == reference.book_index)
                    .unwrap();
                write!(f, "{} ", book_data.short_name)?;
                previous_book = reference.book_index;
                previous_work = Some(reference.work);
            }

            match reference.range_type {
                RangeType::StartEndChapter { start, end } => {
                    if !new_book_title {
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
                    if !new_book_title && chapter == previous_chapter {
                        write!(f, "{} ", VERSE_CHUNK_DELIM)?
                    } else {
                        if !new_book_title && i != 0 {
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
    use concat_idents::concat_idents;

    macro_rules! roundtrip_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $value;
                let parsed = input.parse::<RangeCollection>();
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

    roundtrip_tests! {
        // From https://en.wikipedia.org/wiki/Bible_citation wikipedia page
        roundtrip_bible_0: "John 3",
        roundtrip_bible_1: "John 1–3",
        roundtrip_bible_2: "John 3:16",
        roundtrip_bible_3: "John 3:16–17",
        roundtrip_bible_4: "John 6:14, 44",

        // Others
        roundtrip_bible_5: "Gen. 6:14",
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
            (
                "1 Nephi 1; 1 Nephi 2; 1 Nephi 1:1-3; 1 Nephi 5:6",
                "1 Ne. 1–2; 5:6",
            ),
            ("Alma 3:18–19, 16–17; Alma 3; Alma 4", "Alma 3–4"),
            ("Alma 3:16, 17, 18–19", "Alma 3:16–19"),
            ("Alma 3:16, 18, 19", "Alma 3:16, 18–19"),
            ("Alma 16, 18, 19", "Alma 16, 18–19"),
            ("1 Nephi 1; 2 Nephi 1", "1 Ne. 1; 2 Ne. 1"),
            ("Genesis 1; 1 Nephi 1", "Gen. 1; 1 Ne. 1"), // Make sure that same chapter index across different works is not joined.
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
            let parsed = input.parse::<RangeCollection>();
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

    #[test]
    fn is_valid_huge_chapter() {
        let bom = BOM::from_default_parser().unwrap();
        let parsed = "Alma 1000".parse::<RangeCollection>().unwrap();
        assert!(!parsed.is_valid(&bom));
    }

    #[test]
    fn is_valid_last_verse_in_chapter() {
        let bom = BOM::from_default_parser().unwrap();
        let parsed = "Alma 63:17".parse::<RangeCollection>().unwrap();
        assert!(parsed.is_valid(&bom));
    }

    macro_rules! illegal_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let case = $value;
                let bom = BOM::from_default_parser().unwrap();
                let result = case.parse::<RangeCollection>();
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
        illegal_14: "Alma 5:5-6-",
    }

    macro_rules! bom_urls_reachable {
        ($($test_name_postfix:ident:$book_index:expr,)*) => {
        $(
            concat_idents!(fn_name = test_urls_reachable, _, $test_name_postfix {
                #[test]
                #[ignore] // These tests take a long time to run.
                fn fn_name() {
                    let bom = BOM::from_default_parser().unwrap();
                    let work = Work::BookOfMormon;
                    let book_index = $book_index;
                    let mut chapter_index = 1;
                    let mut verse_index = 1;
                    let mut verse_ref = VerseReference::new(work, book_index, chapter_index, verse_index);
                    let mut is_valid = verse_ref.is_valid(&bom);

                    while is_valid {
                        let url = verse_ref.url().unwrap();
                        let resp = ureq::get(&url.to_string()).redirects(0).call();
                        assert!(resp.ok(), "url failed: {}", url);

                        verse_index += 15; // Speed up.
                        verse_ref = VerseReference::new(work, book_index, chapter_index, verse_index);
                        is_valid = verse_ref.is_valid(&bom);
                        if !is_valid && chapter_index < 40 { // For speed
                            verse_index = 1;
                            chapter_index += 1;
                            verse_ref = VerseReference::new(work, book_index, chapter_index, verse_index);
                            is_valid = verse_ref.is_valid(&bom);
                        }
                    }
                }
            });
        )*
        }
    }

    bom_urls_reachable! {nephi1:0, nephi2:1, jacob:2, enos:3, jarom:4, omni:5, wofm:6, mosiah:7, alma:8, helaman:9, nephi3:10, nephi4:11, mormon:12, ether:13, moroni:14,}
}
