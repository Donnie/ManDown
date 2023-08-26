use crate::data::Record;

// Function to update HTTP status of each website
pub fn update_http_status(records: &mut Vec<Record>) {
  for record in records {
      match get_status(&record.website) {
          Ok(status) => record.status = status,
          Err(e) => handle_error(record, &e),
      }
  }
}

fn get_status(website: &str) -> Result<usize, reqwest::Error> {
  let client = reqwest::blocking::Client::new();
  let res = client.get(website).send()?;

  Ok(res.status().as_u16() as usize)
}

// Handle failure
fn handle_error(record: &mut Record, e: &reqwest::Error) {
  println!("Error fetching {}: {:?}", record.website, e);
  record.status = 0;
}
