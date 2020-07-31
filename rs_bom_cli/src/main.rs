use anyhow::anyhow;
use rand::Rng;
use regex::RegexBuilder;
use rs_bom::{RangeCollection, VerseReference, BOM};

fn main() -> Result<(), anyhow::Error> {
    let bom = BOM::from_default_parser()?;

    let ephraim = "ephraim";
    let total = bom.verses().count();
    let num_ephraim = bom
        .verses()
        .filter(|v| v.text.to_lowercase().contains(ephraim))
        .count();

    println!("{}: {} / {}\n", ephraim, num_ephraim, total,);

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0, total);
    let random_verse = bom.verses().nth(r).unwrap();
    println!("{}\n", random_verse);

    let orig = "3 Ne. 5, 14 - 15, 13"; // "3 Nephi 5: 16 - 18, 9, 15 - 17, 14 - 15, 17 - 19";
    let mut complicated = RangeCollection::new(orig)?;
    complicated.canonicalize();
    println!("{} canonicalized to {}\n", orig, complicated);

    for v in bom.verses_matching(&complicated).take(2) {
        println!("{}\n", v);
    }

    let single = VerseReference::new(0, 1, 1);
    println!(
        "{}",
        bom.verse_matching(&single)
            .ok_or_else(|| anyhow!("Unable to validate verse reference"))?
    );

    let re = RegexBuilder::new("Gazelem")
        .case_insensitive(true)
        .build()
        .unwrap();
    let matches: Vec<_> = bom.verses().filter(|v| re.is_match(v.text)).collect();
    println!("\n\n{}", matches.len());
    for m in matches.iter().take(10) {
        println!("{}", m)
    }

    Ok(())
}
