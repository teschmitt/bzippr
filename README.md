# bzippr

A Rust implementation of the `bzip2` compression algorithm.

## Overview

bzippr is a command-line tool that implements the `bzip2` compression algorithm from scratch. The project aims to provide a comprehensive Rust implementation of all stages in the bzip2 compression pipeline.

## Features

- ✅ Run-Length Encoding (RLE) compression and decompression
- ✅ Move-to-Front (MTF) transformation
- ✅ Burrows-Wheeler Transform (BWT) implementation
- ⏳ Huffman Coding
- ⏳ Full bzip2 format support
- ⏳ Command-line interface for file compression

## Implementation Status

The project has successfully implemented the following compression stages:

1. **Run-Length Encoding (RLE)**: A lossless data compression technique that encodes consecutive repeated data elements.
2. **Move-to-Front Transform (MTF)**: An algorithm that reorders data based on recency of occurrence, improving compression for certain types of data.
3. **Burrows-Wheeler Transform (BWT)**: A reversible transformation that rearranges input data to improve compressibility.

These components are fully implemented with comprehensive test coverage, including edge cases and larger datasets.

## Installation

```bash
cargo build --release
```

## Usage

The command-line interface is still in development. The core compression algorithms can be used as a library.

```rust
use bzippr::{bwt, mtf, rle};

// Example usage will be provided once CLI is complete
```

## Future Work

- Implement Huffman Coding stage
- Integrate all compression stages into the full bzip2 pipeline
- Develop a user-friendly command-line interface
- Add benchmarking against the original bzip2 implementation
- Support for additional bzip2 features (block sizes, compression levels)

## License

Gnu AGPLv3 (see LICENSE file for full terms)

## Contributing

This project is in active development. Feel free to contribute by opening issues or submitting pull requests.
