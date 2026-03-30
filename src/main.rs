use std::env;
use std::path::PathBuf;
use pngv::{encode_to_pngv, decode_from_pngv, decode_to_svg, union_boolean};

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
