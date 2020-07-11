use rs_bom::BOM;
use rand::Rng;

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
    println!("{}", random_verse);

    Ok(())
}
