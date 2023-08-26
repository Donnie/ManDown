use csv::Error;

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Record {
    pub website: String,
    pub status: usize,
    pub user: usize,
}

// Function to read CSV
pub fn read_csv(filename: &str) -> Result<Vec<Record>, Error> {
    let mut reader = csv::Reader::from_path(filename)?;
    let records: Vec<Record> = reader.deserialize().filter_map(Result::ok).collect();
    Ok(records)
}

// Function to write updated records back to CSV
pub fn write_csv(filename: &str, records: Vec<Record>) -> Result<(), Error> {
    let mut writer = csv::Writer::from_path(filename)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}
