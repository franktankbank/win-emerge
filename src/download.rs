use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;

use crate::error::DownloadError;

pub fn download(url: &str, output_file: &str) -> Result<String, DownloadError> {
    let client = Client::new();

    let response = client.get(url).send()?;

    let total_size = match response.content_length() {
        Some(len) => len,
        None => return Err(DownloadError::FileSize)
    };

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );

    let path = Path::new(output_file);
    let mut file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => return Err(DownloadError::FileCreation(e))
    };

    let mut source = BufReader::new(response);
    let mut buffer = [0u8; 8192];
    let mut downloaded = 0;

    loop {
        let bytes_read = match source.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => return Err(DownloadError::FileRead(e))
        };

        if let Err(e) = file.write_all(&buffer[..bytes_read]) {
            return Err(DownloadError::FileWrite(e))
        }

        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("✅ Download complete");

    println!("Saved to {}", output_file);

    Ok(output_file.to_string())
}
