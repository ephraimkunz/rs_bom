use rand::Rng;
use rs_bom::{ReferenceCollection, BOM};

fn main() -> Result<(), anyhow::Error> {
    let bom = BOM::from_default_parser()?;

    let ephraim = "ephraim";
    let total = bom.verses().count();
    let num_ephraim = bom
        .verses()
        .filter(|v| v.text.to_lowercase().contains(ephraim))
        .count();

    println!(
        "{}: {} / {}\n{:.2}%\n",
        ephraim,
        num_ephraim,
        total,
        ((num_ephraim as f64) / (total as f64)) * 100f64
    );

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0, total);
    let random_verse = bom.verses().nth(r).unwrap();
    println!("{}\n", random_verse);

    let orig = "3 Ne. 5, 14 - 15, 13"; // "3 Nephi 5: 16 - 18, 9, 15 - 17, 14 - 15, 17 - 19";
    let mut complicated: ReferenceCollection = orig.parse()?;
    complicated.canonicalize();
    println!("{} canonicalized to {}\n", orig, complicated);
    for v in bom.verses_matching(complicated.verse_refs(&bom)) {
        println!("{}\n", v);
    }

    Ok(())
}
