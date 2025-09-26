#[warn(dead_code)]
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use bwt::BwtEncoded;
use rle::RleSequence;

mod bwt;
mod rle;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Will bzip2 your file and shut up about it."
)]
struct Args {
    /// Path of input file to compress
    #[arg(short, long)]
    file_path: PathBuf,
    /// Path of compressed output file
    #[arg(short, long)]
    output_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let data = std::fs::read(&args.file_path)?;

    let rle_sequence = &RleSequence::encode(&data);

    println!("Length of RLE sequence: {}", rle_sequence.len());

    let bwt: BwtEncoded = rle_sequence.try_into().unwrap();

    println!("Length of BWT transform: {}", bwt.len());

    let decompressed_data: Vec<u8> = TryInto::<RleSequence>::try_into(bwt).unwrap().decode();
    assert_eq!(data, decompressed_data);

    println!(
        "Decompresssed data: {}",
        decompressed_data
            .iter()
            .map(|b| *b as char)
            .collect::<String>()
    );
    Ok(())
}
