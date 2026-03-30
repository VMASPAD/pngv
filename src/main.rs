use std::env;
use std::path::PathBuf;
use pngv::{encode_to_pngv, decode_from_pngv, decode_to_svg};
use geo::{coord, MultiPolygon, Polygon, Rect, BooleanOps};
use geo_booleanop::boolean::BooleanOp;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use roxmltree::Document;
use std::fs;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document as SvgDocument;

fn union_boolean(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
   // 1. Configurar Rayon para usar 6 núcleos
    rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();

    // 2. Leer el archivo SVG original
    let input_svg = fs::read_to_string(input).expect("Error leyendo input.svg. Verifica que el archivo exista.");
    let doc = Document::parse(&input_svg).expect("Error parseando la estructura del SVG.");

    // 3. Agrupar rectángulos por color manteniendo el orden original (Z-Index)
    let mut color_groups: IndexMap<String, Vec<Polygon<f64>>> = IndexMap::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("rect")) {
        let x: f64 = node.attribute("x").unwrap_or("0").parse().unwrap_or(0.0);
        let y: f64 = node.attribute("y").unwrap_or("0").parse().unwrap_or(0.0);
        let w: f64 = node.attribute("width").unwrap_or("1").parse().unwrap_or(1.0);
        let h: f64 = node.attribute("height").unwrap_or("1").parse().unwrap_or(1.0);
        let fill = node.attribute("fill").unwrap_or("#000000").to_string();

        let rect = Rect::new(coord! { x: x, y: y }, coord! { x: x + w, y: y + h });
        color_groups.entry(fill).or_default().push(rect.into());
    }

    // 4. Configurar la barra de progreso
    let total_polygons: usize = color_groups.values().map(|v| v.len()).sum();
    let pb = ProgressBar::new(total_polygons as u64);
    pb.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} rectángulos ETA: {eta}"
    ).unwrap().progress_chars("##-"));

    let mut output_document = SvgDocument::new();

    // 5. Procesamiento paralelo de las uniones booleanas
    for (color, polygons) in color_groups {
        if polygons.is_empty() { continue; }

        let pb_clone = pb.clone();
let count = polygons.len() as u64;
        // Unión en cascada dividida en múltiples hilos
let unified_shape = polygons
            .into_par_iter()
            .map(|poly| MultiPolygon(vec![poly]))
            .reduce(
                || MultiPolygon(vec![]),
                |a: MultiPolygon<f64>, b: MultiPolygon<f64>| a.union(&b),
            );
pb.inc(count);
        // 6. Generar comandos SVG manejando contornos y agujeros (interiores)
        let mut path_data = Data::new();
        
        for poly in unified_shape.into_iter() {
            // Contorno exterior
            let ext = poly.exterior();
            let mut points = ext.points();
            if let Some(first) = points.next() {
                path_data = path_data.move_to((first.x(), first.y()));
                for p in points { path_data = path_data.line_to((p.x(), p.y())); }
                path_data = path_data.close();
            }

            // Agujeros interiores
            for interior in poly.interiors() {
                let mut points = interior.points();
                if let Some(first) = points.next() {
                    path_data = path_data.move_to((first.x(), first.y()));
                    for p in points { path_data = path_data.line_to((p.x(), p.y())); }
                    path_data = path_data.close();
                }
            }
        }

        // 7. Añadir el path al documento usando fill-rule="evenodd"
        let path = Path::new()
            .set("fill", color)
            .set("fill-rule", "evenodd")
            .set("d", path_data);

        output_document = output_document.add(path);
    }

    pb.finish_with_message("¡Procesamiento completado!");

    // 8. Guardar el nuevo SVG optimizado
    svg::save(&output, &output_document).expect("Error al guardar output.svg");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let input = &args[2];
    let output = if args.len() > 3 {
        args[3].clone()
    } else {
        generate_output_path(input, command)
    };

    let result = match command.as_str() {
        "encode" => {
            encode_to_pngv(input, &output)
                .map(|_| format!("✓ Matrix saved to {}", output))
        }
        "decode" => {
            decode_from_pngv(input, &output)
                .map(|_| format!("✓ PNG image generated at {}", output))
        }
        "svg" => {
            decode_to_svg(input, &output)
                .map(|_| format!("✓ SVG image generated at {}", output))
        }
        "compress" => {
            union_boolean(input, &output)
                .map(|_| format!("✓ SVG image compressed at {}", output))
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    };

    match result {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("pngv - PNG to color matrix converter");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    {} <COMMAND> <INPUT> [OUTPUT]", program);
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("    encode    Convert PNG to .pngv color matrix");
    eprintln!("    decode    Convert .pngv to PNG image");
    eprintln!("    svg       Convert .pngv to SVG with 1x1 vector pixels");
    eprintln!();
    eprintln!("ARGUMENTS:");
    eprintln!("    <INPUT>     Input file path");
    eprintln!("    [OUTPUT]    Output file path (optional, auto-generated if not provided)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    {} encode image.png", program);
    eprintln!("    {} decode image.pngv output.png", program);
    eprintln!("    {} svg image.pngv", program);
}

fn generate_output_path(input: &str, command: &str) -> String {
    let path = PathBuf::from(input);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    
    match command {
        "encode" => format!("{}.pngv", stem),
        "decode" => format!("{}.png", stem),
        "svg" => format!("{}.svg", stem),
        _ => "output".to_string(),
    }
}
