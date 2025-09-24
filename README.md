# bzippr

A Rust implementation of `bzip2` compression algorithm.

## Overview

bzippr is a command-line tool that implements the `bzip2` compression algorithm from scratch. Currently supports run-length encoding (RLE) with Burrows-Wheeler Transform (BWT) implementation in progress.

## Features

- Run-Length Encoding (RLE) compression and decompression
- Burrows-Wheeler Transform implementation (in development)
- Command-line interface for file compression
- Built with Rust for memory safety and performance

## Installation

```bash
cargo build --release
```

## Usage

```bash
bzippr --file-path <input_file> [--output-path <output_file>]
```

### Options

- `-f, --file-path <FILE>`: Path of input file to compress (required)
- `-o, --output-path <FILE>`: Path of compressed output file (optional)

### Example

```bash
# Compress a file
bzippr -f example.txt -o example.bz2
```

## Development Status

This is a work-in-progress implementation of the `bzip2` algorithm:

- ✅ Run-Length Encoding
- 🚧 Burrows-Wheeler Transform
- ⏳ Huffman Coding
- ⏳ Full bzip2 format support

## License

Gnu AGPLv3 (see LICENSE file for full terms)

## Contributing

This project is in active development. Feel free to contribute by opening issues or submitting pull requests.