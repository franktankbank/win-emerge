use std::fs::File;
use std::io::{BufReader, BufWriter};

use zstd::stream::copy_decode;

#[derive(Debug)]
pub struct ZstdDec {
    pub status: bool,
    pub message: String
}

impl ZstdDec {
    pub fn new(input_file_str: &str, output_file_str: &str) -> Self {
        let input_file  = match File::open(input_file_str) {
            Ok(file) => file,
            Err(e) => {
                return Self {
                    status: false,
                    message: format!("Zstd decode failed: {}", e)
                }
            }
        };
        let output_file = match File::open(output_file_str) {
            Ok(file) => file,
            Err(e) => {
                return Self {
                    status: false,
                    message: format!("Zstd decode failed: {}", e)
                }
            }
        };
        let mut reader = BufReader::new(input_file);
        let mut writer = BufWriter::new(output_file);

        match copy_decode(&mut reader, &mut writer) {
            Ok(_) => {
                Self {
                    status: true,
                    message: format!("✅ Zstd decode succeeded")
                }
            },
            Err(e) => {
                Self {
                    status: false,
                    message: format!("Zstd decode failed: {}", e)
                }
            }
        }
    }
}
