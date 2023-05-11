use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use csv::{ReaderBuilder, WriterBuilder};

// A function to read the CSV file and return the client secret with the highest quota
pub fn get_client_secret_with_highest_quota(csv: &Path) -> Result<(String, u32), Box<dyn Error>> {
    // Open the CSV file
    let mut path = PathBuf::new();
    path.push(csv.parent().unwrap());
    let file = File::open(csv)?;
    let reader = BufReader::new(file);

    // Build the CSV reader and parse the records
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);
    let mut client_secret_map: HashMap<String, u32> = HashMap::new();
    for result in csv_reader.records() {
        let record = result?;
        let client_secret = record.get(0).unwrap_or("").to_string();
        let quota = record.get(1).unwrap_or("10000").parse::<u32>().unwrap_or(10000);
        client_secret_map.insert(client_secret, quota);
    }

    // Find the client secret with the highest quota
    let (client_secret, quota) = client_secret_map.iter().max_by_key(|&(_, quota)| quota).ok_or_else(|| {
        Box::new(std::io::Error::new(ErrorKind::Other, "No client secret found"))
    })?;
    path.push(client_secret);
    Ok((path.to_str().unwrap().to_string(), *quota))
}

pub fn update_quota_in_csv(file_path: PathBuf, client_secret_name: &str, new_quota: u32) -> Result<(), Box<dyn Error>> {
    // Open the CSV file for reading and writing
    let file = File::open(&file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut wtr = csv::Writer::from_writer(File::create(file_path)?);

    // Loop over each record in the CSV file
    for result in rdr.records() {
        let record = result?;
        let secret_name = &record[0];
        let quota: u32 = record[1].parse()?;

        // Update the quota for the specified client_secret
        if secret_name == client_secret_name {
            wtr.write_record(&[secret_name, new_quota.to_string().as_str()])?;
        } else {
            wtr.write_record(&[secret_name, quota.to_string().as_str()])?;
        }
    }

    // Flush the CSV writer to write changes to the file
    wtr.flush()?;
    Ok(())
}

// A function that reads through a folder with client_secret.json files and creates a CSV with the file names
// in one column and a quota of 10000 in another. The header is set to the date the CSV file was created.
pub fn create_client_secret_csv(folder_path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let folder = Path::new(folder_path);

    // Check if a CSV file already exists
    let mut csv_file_path = folder.to_owned();
    let mut csv_file_exists = false;
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "csv" {
            csv_file_path = path.clone();
            csv_file_exists = true;
            break;
        }
    }
    if csv_file_exists {
        let metadata = fs::metadata(&csv_file_path)?;
        let created_time = metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if current_time - created_time <= 86400 {
            return Ok(csv_file_path);
        } else { fs::remove_file(&csv_file_path)?; }
    }

    // Read the client secret files and build a map with the file names as keys and a quota of 10000 as values
    let mut client_secret_map: HashMap<String, u32> = HashMap::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "json" && path.file_name().unwrap_or_default().to_str().unwrap_or("").contains("client_secret") {
                    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                    client_secret_map.insert(file_name, 10000);
                }
            }
        }
    }

   // Get the current timestamp
   let now = SystemTime::now();
   let timestamp = now.duration_since(UNIX_EPOCH)?.as_secs();

    // Create the CSV file and write the data to it
    csv_file_path.push(format!("client_secret_{}.csv", timestamp));
    let file = File::create(&csv_file_path)?;
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(file);
    for (client_secret, quota) in client_secret_map.iter() {
        writer.write_record(&[client_secret.to_string(), quota.to_string()]).expect("writer error!");
    }
    writer.flush()?;

    Ok(csv_file_path)
}

fn get_csv_file_path(folder: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(folder).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "csv" {
            return Some(path);
        }
    }
    None
}