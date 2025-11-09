# pngv

[![Crates.io](https://img.shields.io/crates/v/pngv.svg)](https://crates.io/crates/pngv)
[![Documentation](https://docs.rs/pngv/badge.svg)](https://docs.rs/pngv)
[![License](https://img.shields.io/crates/l/pngv.svg)](https://github.com/vmaspad/pngv#license)

A Rust library and CLI tool for converting PNG images to color matrices and reconstructing them as PNG or SVG files with 1x1 vector pixels.

## Features

- **Encode**: Convert PNG images to JSON color matrices (`.pngv` format) where each cell contains a hex color with alpha channel (`#RRGGBBAA`)
- **Decode**: Reconstruct PNG images from `.pngv` files with 1x1 pixels
- **SVG Export**: Generate SVG files with 1x1 vector rectangles from `.pngv` files
- Simple CLI with automatic output path generation
- Can be used as a library in your own projects

## Installation

### As a CLI tool

```bash
cargo install pngv
```

### As a library

Add this to your `Cargo.toml`:

```toml
[dependencies]
pngv = "0.1"
```

## CLI Usage

The CLI requires a command and an input file. The output path is optional and will be auto-generated if not provided.

### Basic syntax

```bash
pngv <COMMAND> <INPUT> [OUTPUT]
```

### Commands

#### Encode: PNG → .pngv

Convert a PNG image to a color matrix:

```bash
pngv encode image.png
# Creates: image.pngv

pngv encode input.png output.pngv
# Creates: output.pngv
```

#### Decode: .pngv → PNG

Reconstruct a PNG from a color matrix:

```bash
pngv decode image.pngv
# Creates: image.png

pngv decode input.pngv output.png
# Creates: output.png
```

#### SVG: .pngv → SVG

Generate an SVG with 1x1 vector pixels:

```bash
pngv svg image.pngv
# Creates: image.svg

pngv svg input.pngv output.svg
# Creates: output.svg
```

## Library Usage

```rust
use pngv::{encode_to_pngv, decode_from_pngv, decode_to_svg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Encode a PNG to .pngv format
    encode_to_pngv("image.png", "image.pngv")?;
    
    // Decode back to PNG
    decode_from_pngv("image.pngv", "output.png")?;
    
    // Export to SVG
    decode_to_svg("image.pngv", "output.svg")?;
    
    Ok(())
}
```

## .pngv Format

The `.pngv` file is a JSON array of arrays, where each element represents a pixel color:

```json
[
  ["#FF0000FF", "#00FF00FF", "#0000FFFF"],
  ["#FFFFFFFF", "#000000FF", "#808080FF"]
]
```

- Each row represents a horizontal line of pixels (top to bottom)
- Each color is in `#RRGGBBAA` format (Red, Green, Blue, Alpha in hexadecimal)
- Alpha channel is fully supported

## Use Cases

- **Pixel art editing**: Edit images as JSON matrices
- **Image analysis**: Process images as structured data
- **Vector conversion**: Convert pixel art to scalable SVG format
- **Data visualization**: Represent images as color data
- **Backup/versioning**: Store images in text-friendly format

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
