mod parser;
mod thread_pool;

use crate::error::Error;
use crate::processor::parser::{build_response_site_info, read_jsonl};
use crate::processor::thread_pool::create_pool;
use crate::structures::site_info::SitesInfo;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;

/// Extension of the files to process
const EXTENSION: &str = "jsonl";

/// Process the data from the files in the data directory
/// # Arguments
/// * `n_threads` - Number of threads to use in Rayon's thread pool.
/// If the number of threads is greater than the number of files, the number of threads will be set to the number of files
/// # Returns
/// * `Result<String, Error>` - The result returned as a string
pub fn process_data(mut n_threads: usize, data_dir: &str) -> Result<String, Error> {
    let paths = read_paths(data_dir)?;
    if paths.is_empty() {
        return Err(Error::ReadingError("No files to process".to_string()));
    }

    let n_files = paths.len();
    if n_threads > n_files {
        n_threads = n_files;
    }

    create_pool(n_threads)?.install(|| process_files(paths))
}

/// Process the files in the data directory using Rayon's thread pool, this is fork-join model
/// # Arguments
/// * `paths` - Vector of paths to the files to process
/// # Returns
/// * `Result<String, Error>` - The result returned as a string
fn process_files(paths: Vec<String>) -> Result<String, Error> {
    let result: HashMap<_, _> = paths
        .par_iter()
        .filter_map(|value| read_jsonl(value).ok())
        .flatten()
        .collect();

    build_final_response(Ok(result))
}

/// Builds the final response from the result of processing the files
/// # Arguments
/// * `result` - Result<SiteInfo, Error> of processing the files
/// # Returns
/// * The result is returned as a json string
fn build_final_response(result: Result<SitesInfo, Error>) -> Result<String, Error> {
    match result {
        Ok(sites_map) => match build_response_site_info(sites_map) {
            Ok(res) => {
                let result_string = serde_json::to_string_pretty(&res).map_err(|e| {
                    Error::ReadingError(
                        "Error converting response to json string".to_string() + &e.to_string(),
                    )
                })?;
                Ok(result_string)
            }
            Err(e) => {
                eprintln!("Error building response: {:?}", e);
                Err(Error::ReadingError("Error building response".to_string()))
            }
        },
        Err(e) => {
            eprintln!("Error processing files: {:?}", e);
            Err(Error::ReadingError("Error processing files".to_string()))
        }
    }
}

/// Reads the paths of the files in the data directory.
/// # Returns
/// * `Result<Vec<String>, Error>` - Vector of paths to the files ending with 'jsonl' in the data directory
fn read_paths(data_dir: &str) -> Result<Vec<String>, Error> {
    let mut paths = Vec::new();

    let dir_entries = fs::read_dir(data_dir)
        .map_err(|_| Error::ReadingError("Error reading data directory".to_string()))?;

    for dir_entry in dir_entries {
        let entry = dir_entry.map_err(|e| {
            Error::ReadingError("Error reading dir entry: ".to_string() + &e.to_string())
        })?;

        let path = entry.path();
        if path.extension().eq(&Some(EXTENSION.as_ref())) {
            if let Some(jsonl_path) = path.to_str() {
                paths.push(jsonl_path.to_string());
            }
        }
    }

    Ok(paths)
}
