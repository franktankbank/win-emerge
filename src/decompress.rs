use std::fs::File;
use std::io::{BufReader, BufWriter};

use zstd::stream::copy_decode;

use crate::error::ZstdError;

pub fn decode_zstd(input_file_str: &str, output_file_str: &str) -> Result<(), ZstdError> {
    let input_file  = File::open(input_file_str)?;
    let output_file = File::create(output_file_str)?;
    let mut reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    copy_decode(&mut reader, &mut writer)?;

    println!("✅ Zstd decode succeeded");

    Ok(())
}
