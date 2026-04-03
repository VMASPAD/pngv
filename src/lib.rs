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
use geo::{Coord, MultiPolygon, Polygon, LineString};
use geo::algorithm::bool_ops::unary_union;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use roxmltree::Document;
use svg::node::element::path::Data;
use svg::node::element::Path as SvgPath;
use svg::Document as SvgDocument;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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

/// Creates a rectangle `Polygon<f64>` from position and size.
fn make_rect_polygon(x: f64, y: f64, w: f64, h: f64) -> Polygon<f64> {
    Polygon::new(
        LineString::from(vec![
            Coord { x, y },
            Coord { x: x + w, y },
            Coord { x: x + w, y: y + h },
            Coord { x, y: y + h },
            Coord { x, y },
        ]),
        vec![],
    )
}

/// Perform boolean union on SVG `<rect>` elements grouped by fill color and
/// write an optimized SVG with combined paths.
///
/// # Algorithm
///
/// 1. **Parse** the input SVG and extract all `<rect>` elements
/// 2. **Group** rectangles by fill color using an `IndexMap` for stable ordering
/// 3. **Parallel color processing**: each Rayon thread handles one color group
///    entirely — one core per color for maximum throughput
/// 4. **`unary_union`** (geo 0.32 + iOverlay engine) merges all polygons of a
///    single color in one optimized sweep-line pass (O(n log n) vs O(n²))
/// 5. **Generate** a unified vectorial SVG with one `<path>` per color
pub fn union_boolean(input: &str, output: &str) -> Result<(), Box<dyn Error>> {
    // Read and parse SVG
    let input_svg = fs::read_to_string(input)?;
    let doc = Document::parse(&input_svg)?;

    // Extract viewBox / dimensions from the root <svg> element
    let svg_root = doc
        .descendants()
        .find(|n| n.has_tag_name("svg"))
        .ok_or("No <svg> root element found")?;

    let svg_width: f64 = svg_root
        .attribute("width")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let svg_height: f64 = svg_root
        .attribute("height")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let view_box = svg_root.attribute("viewBox").map(|s| s.to_string());

    // Group rects by fill attribute while preserving insertion order
    let mut color_groups: IndexMap<String, Vec<Polygon<f64>>> = IndexMap::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("rect")) {
        let x: f64 = node.attribute("x").unwrap_or("0").parse().unwrap_or(0.0);
        let y: f64 = node.attribute("y").unwrap_or("0").parse().unwrap_or(0.0);
        let w: f64 = node.attribute("width").unwrap_or("1").parse().unwrap_or(1.0);
        let h: f64 = node.attribute("height").unwrap_or("1").parse().unwrap_or(1.0);
        let fill = node.attribute("fill").unwrap_or("#000000").to_string();

        color_groups
            .entry(fill)
            .or_default()
            .push(make_rect_polygon(x, y, w, h));
    }

    let total_colors = color_groups.len();

    // Progress bar — tracks colors processed (each color is one parallel unit)
    let pb = ProgressBar::new(total_colors as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} colores (núcleos) ETA: {eta}",
        )
        .unwrap()
        .progress_chars("██░"),
    );

    // Atomic counter shared across threads for progress
    let progress_counter = Arc::new(AtomicU64::new(0));

    // ─── Parallel processing: one core per color ───────────────────────
    // Each color group is processed independently by a Rayon thread.
    // Inside each thread, `unary_union` performs the merge using the
    // iOverlay sweep-line algorithm — far faster than iterative binary union.
    let results: Vec<(String, MultiPolygon<f64>)> = color_groups
        .into_par_iter()
        .map(|(color, polygons)| {
            let unified = if polygons.len() == 1 {
                // Single polygon — no union needed
                MultiPolygon::new(polygons)
            } else {
                // unary_union: O(n log n) sweep-line merge via iOverlay
                unary_union(&polygons)
            };

            // Update progress
            let done = progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
            pb.set_position(done);

            (color, unified)
        })
        .collect();

    pb.finish_with_message("¡Procesamiento completado!");

    // ─── Build output SVG ──────────────────────────────────────────────
    let mut output_document = SvgDocument::new();

    // Set dimensions and viewBox
    if svg_width > 0.0 && svg_height > 0.0 {
        output_document = output_document
            .set("width", svg_width)
            .set("height", svg_height);
    }
    if let Some(vb) = view_box {
        output_document = output_document.set("viewBox", vb);
    } else if svg_width > 0.0 && svg_height > 0.0 {
        output_document = output_document.set(
            "viewBox",
            format!("0 0 {} {}", svg_width, svg_height),
        );
    }

    // Generate one <path> per color — each is a single unified vector shape
    for (color, multi_poly) in results {
        let mut path_data = Data::new();

        for poly in multi_poly.iter() {
            // Exterior ring
            let ext = poly.exterior();
            let mut points = ext.points();
            if let Some(first) = points.next() {
                path_data = path_data.move_to((first.x(), first.y()));
                for p in points {
                    path_data = path_data.line_to((p.x(), p.y()));
                }
                path_data = path_data.close();
            }

            // Interior rings (holes)
            for interior in poly.interiors() {
                let mut points = interior.points();
                if let Some(first) = points.next() {
                    path_data = path_data.move_to((first.x(), first.y()));
                    for p in points {
                        path_data = path_data.line_to((p.x(), p.y()));
                    }
                    path_data = path_data.close();
                }
            }
        }

        let path = SvgPath::new()
            .set("fill", color)
            .set("fill-rule", "evenodd")
            .set("d", path_data);

        output_document = output_document.add(path);
    }

    svg::save(output, &output_document)?;

    Ok(())
}

