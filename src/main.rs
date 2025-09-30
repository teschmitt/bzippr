#[warn(dead_code)]
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use bwt::BwtEncoded;
use rle::RleSequence;

use bzippr::{bwt, mtf::MtfTransform, rle};

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

    let rle_enc = &RleSequence::encode(&data);
    println!("Length of RLE sequence: {}", rle_enc.len());

    let bwt_enc = BwtEncoded::encode(rle_enc);
    println!("Length of BWT transform: {}", bwt_enc.len());

    let mtf_enc: MtfTransform = MtfTransform::encode(&bwt_enc.data());
    println!("Length of MTF transform: {}", mtf_enc.len());

    println!(
        "Compression ratio: {:.2}%",
        100.0 - (100 * mtf_enc.len()) as f64 / data.len() as f64
    );

    let decompressed_data = BwtEncoded::new(mtf_enc.decode(), bwt_enc.original_index())
        .decode()
        .decode();

    assert_eq!(data, decompressed_data);

    println!("Success!");

    Ok(())
}
