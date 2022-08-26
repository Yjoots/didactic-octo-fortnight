use credit::{transcode, Authority};
use csv::Writer;
use pretty_assertions::assert_eq;

#[test]
fn integration() {
    let rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(std::include_str!("./sample.csv").as_bytes());

    let mut authority = Authority::from_iter(transcode(rdr));

    let mut wtr = Writer::from_writer(vec![]);
    for client in authority.iter_clients() {
        assert_eq!(&(client.available() + client.held()), client.total());
        wtr.serialize(client).unwrap();
    }

    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    assert_eq!(
        "client,available,held,total,locked
1,-1.5000,3.0000,1.5000,false
2,-3.0000,0.0000,-3.0000,true
",
        data,
    )
}
