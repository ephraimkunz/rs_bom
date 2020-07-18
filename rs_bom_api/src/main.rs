#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;
use rand::Rng;
use rocket::response::status;
use rocket_contrib::json::Json;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::{openapi, routes_with_openapi, JsonSchema};
use rs_bom::{RangeCollection, VerseReference, VerseWithReference, BOM};
use serde::Serialize;

lazy_static! {
    static ref STATIC_BOM: BOM =
        BOM::from_default_parser().expect("Failed to get BOM from defaul parser");
}

#[derive(Serialize, Debug, JsonSchema)]
struct WebVerseWithReference {
    reference: VerseReference,
    reference_string: String,
    text: String,
}

#[derive(Serialize, Debug, JsonSchema)]
struct WebParsedReference {
    original_reference: String,
    parsed_reference: String,
    is_valid: bool,
}

impl<'a> From<VerseWithReference<'a>> for WebVerseWithReference {
    fn from(other: VerseWithReference) -> Self {
        let s = other.to_string();
        let lines = s.lines().collect::<Vec<_>>();
        let reference_string = lines[0].to_string();
        let text = lines[1..].join("\n");
        Self {
            reference: other.reference,
            reference_string,
            text,
        }
    }
}

#[openapi]
#[get("/verse/<book>/<chapter>/<verse>")]
fn single_verse(
    book: usize,
    chapter: usize,
    verse: usize,
) -> Result<Json<WebVerseWithReference>, status::NotFound<String>> {
    let reference = VerseReference::new(book, chapter, verse);
    STATIC_BOM
        .verse_matching(&reference)
        .map(|v| Json(v.into()))
        .ok_or_else(|| status::NotFound(format!("Invalid reference: {:?}", reference)))
}

#[openapi]
#[get("/verses/<reference_string>")]
fn verses(
    reference_string: String,
) -> Result<Json<Vec<WebVerseWithReference>>, status::NotFound<String>> {
    let reference = RangeCollection::new(&reference_string)
        .map_err(|e| status::NotFound(format!("Error: {:}", e.to_string())))?;

    let verses: Vec<_> = STATIC_BOM
        .verses_matching(&reference)
        .map(|v| v.into())
        .collect();
    Ok(Json(verses))
}

#[openapi]
#[get("/verse/random")]
fn random_verse() -> Json<WebVerseWithReference> {
    let verses = STATIC_BOM.verses();
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0, verses.count());
    let random_verse = STATIC_BOM.verses().nth(r).unwrap();
    Json(random_verse.into())
}

#[openapi]
#[get("/canonicalize/<reference_string>")]
fn canonicalize(
    reference_string: String,
) -> Result<Json<WebParsedReference>, status::NotFound<String>> {
    let mut collection = RangeCollection::new(&reference_string)
        .map_err(|e| status::NotFound(format!("Error: {:}", e.to_string())))?;
    collection.canonicalize();

    Ok(Json(WebParsedReference {
        original_reference: reference_string,
        parsed_reference: collection.to_string(),
        is_valid: collection.is_valid(&STATIC_BOM),
    }))
}

#[catch(404)]
fn not_found() -> String {
    String::from("The requested resource could not be found.")
}

fn main() {
    rocket::ignite()
        .mount(
            "/bom_api/v1",
            routes_with_openapi![single_verse, verses, random_verse, canonicalize],
        )
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/bom_api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .register(catchers![not_found])
        .launch();
}
