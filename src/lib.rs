//! # pngv
//!
//! A library for converting PNG images to color matrices and reconstructing them
//! as PNG or SVG files with 1x1 pixel vectors.
//!
//! ## Features
//!
//! - **Encode**: Convert PNG images to JSON color matrices (`.pngv` format)
//! - **Decode**: Reconstruct PNG images from `.pngv` files
//! - **SVG Export**: Generate SVG files with 1x1 vector pixels from `.pngv` files
//!
//! ## Example
//!
//! ```no_run
//! use pngv::{encode_to_pngv, decode_from_pngv, decode_to_svg};
//!
//! // Encode a PNG to .pngv format
//! encode_to_pngv("image.png", "image.pngv").unwrap();
//!
//! // Decode back to PNG
//! decode_from_pngv("image.pngv", "output.png").unwrap();
//!
//! // Export to SVG
//! decode_to_svg("image.pngv", "output.svg").unwrap();
//! ```

use std::error::Error;
use std::fs;
use image::io::Reader as ImageReader;
use image::{RgbaImage, Rgba};
use std::path::Path;

/// Encodes a PNG image to a `.pngv` color matrix file
///
/// Reads a PNG image and saves a JSON matrix where each cell contains
/// a hex color with alpha channel in the format `#RRGGBBAA`.
///
/// # Arguments
///
/// * `input_path` - Path to the input PNG file
/// * `output_path` - Path for the output `.pngv` file
///
/// # Example
///
/// ```no_run
/// use pngv::encode_to_pngv;
/// encode_to_pngv("image.png", "image.pngv").unwrap();
/// ```
pub fn encode_to_pngv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    let img = ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    let mut matrix: Vec<Vec<String>> = Vec::with_capacity(height as usize);
    for y in 0..height {
        let mut row: Vec<String> = Vec::with_capacity(width as usize);
        for x in 0..width {
            let p = rgba.get_pixel(x, y);
            let [r, g, b, a] = p.0;
            row.push(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a));
        }
        matrix.push(row);
    }

    let json = serde_json::to_string_pretty(&matrix)?;
    fs::write(output_path, json)?;
    
    Ok(())
}

/// Decodes a `.pngv` file and reconstructs it as a PNG image
///
/// Reads a `.pngv` JSON matrix and generates a PNG image with 1x1 pixels,
/// drawing from top to bottom following the matrix structure.
///
/// # Arguments
///
/// * `input_path` - Path to the input `.pngv` file
/// * `output_path` - Path for the output PNG file
///
/// # Example
///
/// ```no_run
/// use pngv::decode_from_pngv;
/// decode_from_pngv("image.pngv", "output.png").unwrap();
/// ```
pub fn decode_from_pngv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let json = fs::read_to_string(input_path)?;
    let matrix: Vec<Vec<String>> = serde_json::from_str(&json)?;
    
    if matrix.is_empty() {
        return Err("Matrix is empty".into());
    }
    
    let height = matrix.len() as u32;
    let width = matrix[0].len() as u32;
    
    let mut img = RgbaImage::new(width, height);
    
    // Iterate matrix row by row (top to bottom)
    for (y, row) in matrix.iter().enumerate() {
        for (x, hex_color) in row.iter().enumerate() {
            let rgba = parse_hex_color(hex_color)?;
            img.put_pixel(x as u32, y as u32, rgba);
        }
    }
    
    img.save(output_path)?;
    
    Ok(())
}

/// Decodes a `.pngv` file and generates an SVG with 1x1 vector pixels
///
/// Reads a `.pngv` JSON matrix and creates an SVG file where each pixel
/// is represented as a 1x1 vector rectangle.
///
/// # Arguments
///
/// * `input_path` - Path to the input `.pngv` file
/// * `output_path` - Path for the output SVG file
///
/// # Example
///
/// ```no_run
/// use pngv::decode_to_svg;
/// decode_to_svg("image.pngv", "output.svg").unwrap();
/// ```
pub fn decode_to_svg(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let json = fs::read_to_string(input_path)?;
    let matrix: Vec<Vec<String>> = serde_json::from_str(&json)?;
    
    if matrix.is_empty() {
        return Err("Matrix is empty".into());
    }
    
    let height = matrix.len();
    let width = matrix[0].len();
    
    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, height, width, height
    ));
    svg.push('\n');
    
    // Iterate matrix row by row (top to bottom)
    for (y, row) in matrix.iter().enumerate() {
        for (x, hex_color) in row.iter().enumerate() {
            // Convert #RRGGBBAA to SVG rgba format
            let rgba = hex_to_svg_color(hex_color)?;
            svg.push_str(&format!(
                r#"  <rect x="{}" y="{}" width="1" height="1" fill="{}" />"#,
                x, y, rgba
            ));
            svg.push('\n');
        }
    }
    
    svg.push_str("</svg>\n");
    
    fs::write(output_path, svg)?;
    
    Ok(())
}

/// Converts a hex color `#RRGGBBAA` to SVG `rgba(r,g,b,a)` format
fn hex_to_svg_color(hex: &str) -> Result<String, Box<dyn Error>> {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() != 8 {
        return Err(format!("Invalid hex color: {}", hex).into());
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    let a = u8::from_str_radix(&hex[6..8], 16)?;
    
    // Convert alpha from 0-255 to 0.0-1.0
    let alpha = a as f32 / 255.0;
    
    Ok(format!("rgba({},{},{},{:.3})", r, g, b, alpha))
}

/// Parses a hex color `#RRGGBBAA` to `Rgba<u8>`
fn parse_hex_color(hex: &str) -> Result<Rgba<u8>, Box<dyn Error>> {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() != 8 {
        return Err(format!("Invalid hex color: {}", hex).into());
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    let a = u8::from_str_radix(&hex[6..8], 16)?;
    
    Ok(Rgba([r, g, b, a]))
}
