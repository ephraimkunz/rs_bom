use anyhow::Result;
use clap::{Parser, Subcommand};
use rand::Rng;
use regex::Regex;
use rs_bom::{RangeCollection, BOM};
use std::{env, fs};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Delete any cache files before running
    #[arg(short, long)]
    delete_cache: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search by reference ('1 Nephi 5:3-6') or with a free-form string ('dwelt in a')
    Search {
        /// The search query
        query: String,

        /// The maximum number of search results to return
        #[arg(short, long, default_value_t = 10)]
        num_matches: usize,

        /// First line of the returned data is the total number of verses matching the query
        #[arg(short, long)]
        count_matches: bool,
    },
    /// Output a random verse
    Random,
    /// Output the entire Book of Mormon text
    Text,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let bom = get_bom(cli.delete_cache)?;

    match cli.command {
        Commands::Text => {
            let all_verses: Vec<_> = bom.verses().map(|v| v.text).collect();
            println!("{}", all_verses.join("\n"));
        }
        Commands::Random => {
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0..bom.verses().count());
            let random_verse = bom.verses().nth(r).unwrap();
            println!("{}", random_verse);
        }
        Commands::Search {
            query,
            num_matches,
            count_matches,
        } => {
            let matches: Vec<String>;
            let total_match_count: usize;

            // Try to parse as a reference first.
            let range = RangeCollection::new(&query);
            if let Ok(range) = range {
                matches = bom.verses_matching(&range).map(|v| v.to_string()).collect();
                total_match_count = matches.len();
            } else {
                // If that failed, try to parse as free form text.
                let re = Regex::new(&format!(r"(?i){}", query)).unwrap();

                total_match_count = bom.verses().filter(|v| re.is_match(v.text)).count();

                matches = bom
                    .verses()
                    .filter(|v| re.is_match(v.text))
                    .take(num_matches)
                    .map(|v| v.to_string())
                    .collect();
            }

            if count_matches {
                println!("{}", total_match_count);
            }

            if !matches.is_empty() {
                println!("{}", matches.join("\n\n"));
            }
        }
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
