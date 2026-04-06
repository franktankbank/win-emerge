use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DownloadResult {
    pub status: bool,
    pub message: String,
    pub path: Option<String>
}

impl DownloadResult {
    pub fn new(url: &str, output_file: &str) -> Self {
        let client = Client::new();

        let response = match client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => {
                return Self {
                    status: false,
                    message: format!("Request failed: {}", e),
                    path: None
                };
            }
        };

        let total_size = match response.content_length() {
            Some(len) => len,
            None => {
                return Self {
                    status: false,
                    message: "Couldn't determine file size.".to_string(),
                    path: None
                };
            }
        };

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        let path = Path::new(output_file);
        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                return Self {
                    status: false,
                    message: format!("Failed to create file: {}", e),
                    path: None
                };
            }
        };

        let mut source = BufReader::new(response);
        let mut buffer = [0u8; 8192];
        let mut downloaded = 0;

        loop {
            let bytes_read = match source.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return Self {
                        status: false,
                        message: format!("Read error: {}", e),
                        path: None
                    };
                }
            };

            if let Err(e) = file.write_all(&buffer[..bytes_read]) {
                return Self {
                    status: false,
                    message: format!("Write error: {}", e),
                    path: None
                };
            }

            downloaded += bytes_read as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("✅ Download complete");

        Self {
            status: true,
            message: format!("Saved to {}", output_file),
            path: Some(output_file.to_string())
        }
    }
}
