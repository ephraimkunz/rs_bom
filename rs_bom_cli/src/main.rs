use anyhow::Result;
use clap::{value_t, App, AppSettings, Arg, SubCommand};
use rand::Rng;
use regex::Regex;
use rs_bom::{RangeCollection, BOM};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("search")
                .about("Search by reference ('1 Nephi 5:3-6') or with a free-form string ('dwelt in a')")
                .arg(
                    Arg::with_name("query")
                        .help("The search query")
                        .required(true),
                )
                .arg(
                    Arg::with_name("num_matches")
                        .short("n")
                        .long("num_matches")
                        .help("The maximum number of search results to return")
                        .default_value("10"),
                ),
        )
        .subcommand(SubCommand::with_name("random").about("Output a random verse"))
        .subcommand(SubCommand::with_name("text").about("Output the entire Book of Mormon text"))
        .get_matches();

    let bom = get_bom()?;

    match matches.subcommand() {
        ("text", _) => {
            let all_verses: Vec<_> = bom.verses().map(|v| v.text).collect();
            println!("{}", all_verses.join("\n"));
        }
        ("random", _) => {
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0, bom.verses().count());
            let random_verse = bom.verses().nth(r).unwrap();
            println!("{}", random_verse);
        }
        ("search", Some(submatches)) => {
            let search = submatches.value_of("query").unwrap();

            // Try to parse as a reference first.
            let range = RangeCollection::new(search);
            if let Ok(range) = range {
                let verses: Vec<_> = bom.verses_matching(&range).map(|v| v.to_string()).collect();
                println!("{}", verses.join("\n\n"));
            } else {
                // If that failed, try to parse as free form text.
                let num_matches = value_t!(submatches.value_of("num_matches"), usize)
                    .unwrap_or_else(|e| e.exit());
                let re = Regex::new(&format!(r"(?i){}", search)).unwrap();
                let verses: Vec<_> = bom
                    .verses()
                    .filter(|v| re.is_match(v.text))
                    .take(num_matches)
                    .map(|v| v.to_string())
                    .collect();
                println!("{}", verses.join("\n\n"))
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn get_bom() -> Result<BOM> {
    // Try to de-serialize a cached bincode versions (to avoid re-parsing all the source),
    // or if that fails fallback to parsing the source.
    const TEMP_FILE_NAME: &str = "rs_bom_serialized";
    let mut file_path = env::temp_dir();
    file_path.push(TEMP_FILE_NAME);
    println!("{:?}", file_path);

    let bom = match fs::read(&file_path) {
        Ok(data) => match bincode::deserialize(data.as_slice()) {
            Err(_) => BOM::from_default_parser(),
            Ok(bom) => Ok(bom),
        },
        _ => BOM::from_default_parser(),
    }?;

    let out_file = fs::File::create(file_path)?;
    bincode::serialize_into(out_file, &bom)?;
    Ok(bom)
}
