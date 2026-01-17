use std::process;

use clap::Parser;
use mproto_codegen::{Database, Module};

#[derive(Parser, Debug)]
#[command(
    version = "0.0",
    about = "Generates [de]serialization code for a mproto schema in a target language.",
    long_about = None,
)]
struct Args {
    /// Path to the schema file
    #[arg(index = 1)]
    schema_file: String,

    /// Path to generate package in
    #[arg(short, long, default_value_t = String::from("./"))]
    output_dir: String,

    /// Name of the package or module to generate
    #[arg(short, long)]
    name: String,

    /// Language to generate package for.
    ///
    /// Supported values are: "rust", "typescript"
    #[arg(short, long)]
    language: String,

    /// Generate a directory containing an importable package instead of a single source file.
    #[arg(short, long)]
    package: bool,
}

fn main() {
    let args = Args::parse();

    // Parse input file
    let type_defs = match mproto_codegen::parse::parse_file(&args.schema_file) {
        Ok(type_defs) => type_defs,
        Err(e) => {
            println!("ERROR: Failed to parse {}: {:?}", args.schema_file, e);
            process::exit(1);
        }
    };

    // Generate package
    match args.language.as_ref() {
        "typescript" => {
            if args.package {
                mproto_codegen::codegen::js::js_package_gen(
                    &args.output_dir,
                    &args.name,
                    &type_defs,
                )
                .expect("gen typescript package");
            } else {
                mproto_codegen::codegen::js::js_module_gen(
                    format!("{}/{}.ts", args.output_dir, args.name),
                    &type_defs,
                )
                .expect("gen typescript module");
            }
        }
        "rust" => {
            if args.package {
                mproto_codegen::codegen::rust::rust_package_gen(
                    &args.output_dir,
                    &args.name,
                    &type_defs,
                )
                .expect("gen rust package");
            } else {
                let local_module = Module::from_type_defs(type_defs.clone());
                let db = Database::new(local_module);
                mproto_codegen::codegen::rust::rust_module_gen(
                    &db,
                    format!("{}/{}.rs", args.output_dir, args.name),
                    &type_defs,
                    false,
                )
                .expect("gen rust module");
            }
        }
        _ => {
            println!("ERROR: Unsupported language '{}'", args.language);
        }
    }
}
