use csv::Error;
use std::env;
use std::path::Path;

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
struct Record {
    website: String,
    status: usize,
}

fn get_status(website: &str) -> Result<usize, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.get(website).send()?;

    Ok(res.status().as_u16() as usize)
}

// Function to read CSV
fn read_csv(filename: &str) -> Result<Vec<Record>, Error> {
    let mut reader = csv::Reader::from_path(filename)?;
    let records: Vec<Record> = reader.deserialize().filter_map(Result::ok).collect();
    Ok(records)
}

// Function to update HTTP status of each website
fn update_http_status(records: &mut Vec<Record>) {
    for record in records {
        match get_status(&record.website) {
            Ok(status) => record.status = status,
            Err(e) => println!("Error fetching {}: {:?}", record.website, e),
        }
    }
}

// Function to write updated records back to CSV
fn write_csv(filename: &str, records: Vec<Record>) -> Result<(), Error> {
    let mut writer = csv::Writer::from_path(filename)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

// Function to process CSV
fn process_csv(filename: &str) -> Result<(), Error> {
    // Read from CSV
    let mut records = read_csv(filename)?;

    println!("{:?}", records);

    // Update HTTP status of each website
    update_http_status(&mut records);

    // Write updated records back to CSV
    write_csv(filename, records)?;

    Ok(())
}

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check that a filename was provided
    if args.len() < 2 {
        println!("Please provide a filename as an argument");
        return;
    }

    let filename = &args[1];

    // Check that the file exists
    if !Path::new(filename).exists() {
        println!("The file {} does not exist", filename);
        return;
    }

    // Process the CSV file
    match process_csv(filename) {
        Ok(_) => println!("CSV file updated successfully!"),
        Err(e) => println!("Error processing CSV file: {:?}", e),
    }
}
