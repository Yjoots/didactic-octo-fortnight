use credit::{transcode, Authority};
use csv::Writer;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("Expected path to input file as argument")?;

    let rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)?;

    let mut authority = Authority::from_iter(transcode(rdr));

    let mut wtr = Writer::from_writer(std::io::stdout());
    for client in authority.iter_clients() {
        wtr.serialize(client)?;
    }

    Ok(())
}
