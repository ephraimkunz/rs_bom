#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;
use rs_bom::{VerseReference, VerseWithReference, BOM};
use serde::Serialize;

#[derive(Responder, Debug)]
enum WebBOMError {
    #[response(status = 400)]
    InvalidReference(String),
    #[response(status = 500)]
    BOMError(String),
}

#[derive(Serialize, Debug)]
struct WebVerseWithReference {
    reference: VerseReference,
    text: String,
}

impl<'a> From<VerseWithReference<'a>> for WebVerseWithReference {
    fn from(other: VerseWithReference) -> Self {
        Self {
            reference: other.reference,
            text: other.text.to_string(),
        }
    }
}

#[get("/verse/<book>/<chapter>/<verse>")]
fn single_verse(
    book: usize,
    chapter: usize,
    verse: usize,
) -> Result<Json<WebVerseWithReference>, WebBOMError> {
    let bom = BOM::from_default_parser().map_err(|e| WebBOMError::BOMError(e.to_string()))?;
    let reference = VerseReference::new(book, chapter, verse);
    bom.verse_matching(&reference)
        .map(|v| Json(v.into()))
        .ok_or_else(|| WebBOMError::InvalidReference(format!("Invalid reference: {:?}", reference)))
}

#[catch(404)]
fn not_found() -> String {
    String::from("The requested resource could not be found.")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![single_verse])
        .register(catchers![not_found])
        .launch();
}
