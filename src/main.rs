use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Debug)]
struct RleRun {
    value: u8,
    count: u64,
}

#[derive(Debug)]
struct RleSequence(Vec<RleRun>);

impl From<&[u8]> for RleSequence {
    fn from(data: &[u8]) -> Self {
        let mut sequence = Vec::new();
        if !data.is_empty() {
            let mut current_value = data[0];
            let mut current_count = 1;

            for &byte in &data[1..] {
                if byte == current_value {
                    current_count += 1;
                } else {
                    sequence.push(RleRun {
                        value: current_value,
                        count: current_count,
                    });
                    current_value = byte;
                    current_count = 1;
                }
            }

            sequence.push(RleRun {
                value: current_value,
                count: current_count,
            });
        }

        RleSequence(sequence)
    }
}

impl Into<Vec<u8>> for RleSequence {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();
        for run in self.0 {
            data.extend(std::iter::repeat(run.value).take(run.count as usize));
        }
        data
    }
}

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

    let rle_sequence = RleSequence::from(&data[..]);
    let decompressed_data: Vec<u8> = rle_sequence.into();
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
