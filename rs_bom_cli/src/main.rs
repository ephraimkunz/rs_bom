use anyhow::Result;
use clap::{Arg, Command};
use rand::Rng;
use regex::Regex;
use rs_bom::{RangeCollection, BOM};
use std::{env, fs};

fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg(Arg::new("delete_cache").short('d').long("delete_cache").help("Delete any cache files before running"))
        .subcommand(
            Command::new("search")
                .about("Search by reference ('1 Nephi 5:3-6') or with a free-form string ('dwelt in a')")
                .arg(
                    Arg::new("query")
                        .help("The search query")
                        .required(true),
                )
                .arg(
                    Arg::new("num_matches")
                        .short('n')
                        .long("num_matches")
                        .help("The maximum number of search results to return")
                        .default_value("10"),
                )
                .arg(
                    Arg::new("count_matches")
                        .short('c')
                        .long("count_matches")
                        .help("First line of returned data is the total number of verses matching the query")
                ),
        )
        .subcommand(Command::new("random").about("Output a random verse"))
        .subcommand(Command::new("text").about("Output the entire Book of Mormon text"))
        .get_matches();

    let bom = get_bom(matches.is_present("delete_cache"))?;

    match matches.subcommand() {
        Some(("text", _)) => {
            let all_verses: Vec<_> = bom.verses().map(|v| v.text).collect();
            println!("{}", all_verses.join("\n"));
        }
        Some(("random", _)) => {
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0..bom.verses().count());
            let random_verse = bom.verses().nth(r).unwrap();
            println!("{}", random_verse);
        }
        Some(("search", submatches)) => {
            let search = submatches.value_of("query").unwrap();
            let matches: Vec<String>;
            let total_match_count: usize;

            // Try to parse as a reference first.
            let range = RangeCollection::new(search);
            if let Ok(range) = range {
                matches = bom.verses_matching(&range).map(|v| v.to_string()).collect();
                total_match_count = matches.len();
            } else {
                // If that failed, try to parse as free form text.
                let num_matches = submatches
                    .value_of_t("num_matches")
                    .unwrap_or_else(|e| e.exit());
                let re = Regex::new(&format!(r"(?i){}", search)).unwrap();

                total_match_count = bom.verses().filter(|v| re.is_match(v.text)).count();

                matches = bom
                    .verses()
                    .filter(|v| re.is_match(v.text))
                    .take(num_matches)
                    .map(|v| v.to_string())
                    .collect();
            }

            if submatches.is_present("count_matches") {
                println!("{}", total_match_count);
            }

            if !matches.is_empty() {
                println!("{}", matches.join("\n\n"));
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn get_bom(delete_cache: bool) -> Result<BOM> {
    // Try to de-serialize a cached bincode version (to avoid re-parsing all the source),
    // or if that fails fallback to parsing the source.
    const TEMP_FILE_NAME: &str = "rs_bom_serialized";
    let mut file_path = env::temp_dir();
    file_path.push(TEMP_FILE_NAME);

    if delete_cache {
        fs::remove_file(&file_path)?;
    }

    let mut from_cache = false;

    let bom = match fs::read(&file_path) {
        Ok(data) => match bincode::deserialize(data.as_slice()) {
            Ok(bom) => {
                from_cache = true;
                Ok(bom)
            }
            Err(_) => BOM::from_default_parser(),
        },
        Err(_) => BOM::from_default_parser(),
    }?;

    // Create a cache file if we just ended up parsing the corpus.
    // If creating the cache file or serializing the BOM struct into it fail, don't fail the
    // program. This cache is just to speed up our app and the app can continue to work without it.
    if !from_cache {
        if let Ok(out_file) = fs::File::create(&file_path) {
            let _ = bincode::serialize_into(out_file, &bom);
        }
    }

    Ok(bom)
}
