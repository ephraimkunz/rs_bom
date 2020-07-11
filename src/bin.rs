use rs_bom::{BOM};

fn main() -> Result<(), anyhow::Error> {
    let bom = BOM::from_default_parser()?;

    let ephraim = "ephraim";
    let total = bom.verses().count();
    let num_ephraim = bom
        .verses()
        .filter(|v| v.text.to_lowercase().contains(ephraim))
        .count();

    println!(
        "{}: {} / {}\n{:.2}%",
        ephraim,
        num_ephraim,
        total,
        ((num_ephraim as f64) / (total as f64)) * 100f64
    );

    Ok(())
}
