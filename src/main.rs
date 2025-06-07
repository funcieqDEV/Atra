mod codegen;
mod lexer;
mod parser;
use codegen::codegen::generate_html;
use lexer::lex_with_positions;
use parser::node::Node;
use serde_json::Value;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "build" {
        eprintln!("Usage: atra build <path/to/config.json>");
        std::process::exit(1);
    }

    let config_path = &args[2];

    let config_content = fs::read_to_string(config_path)?;
    let config: Value = serde_json::from_str(&config_content)?;
    let source_folder = config["source_folder"]
        .as_str()
        .expect("Expected 'source_folder' in config.json");
    let output_folder = config["output_folder"]
        .as_str()
        .expect("Expected 'output_folder' in config.json");

    let source_path = Path::new(source_folder);
    if !source_path.is_dir() {
        eprintln!(
            "Source folder '{}' is not a valid directory.",
            source_folder
        );
        std::process::exit(1);
    }

    let output_path = Path::new(output_folder);
    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
    }

    let mut parsed_files = Vec::new();

    fn process_folder(
        folder_path: &Path,
        output_base: &Path,
        parsed_files: &mut Vec<(String, Node)>,
    ) -> io::Result<()> {
        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let relative_path = path.strip_prefix(folder_path).unwrap();
                let output_subfolder = output_base.join(relative_path);
                fs::create_dir_all(&output_subfolder)?;

                process_folder(&path, &output_subfolder, parsed_files)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "atra") {
                let mut file_content = String::new();
                File::open(&path)?.read_to_string(&mut file_content)?;

                let mut parser = parser::parser::Parser::new(lex_with_positions(&file_content));
                match parser.parse() {
                    Ok(node) => {
                        parsed_files.push((path.display().to_string(), node.clone()));

                        let relative_path = path.strip_prefix(folder_path).unwrap();
                        let output_file_path =
                            output_base.join(relative_path).with_extension("html");
                        let mut output_file = File::create(output_file_path)?;
                        let html_content = generate_html(&node, 0, true);
                        output_file.write_all(html_content.as_bytes())?;
                    }
                    Err(err) => {
                        eprintln!("Error parsing file '{}': {}", path.display(), err);
                    }
                }
            }
        }
        Ok(())
    }

    process_folder(source_path, output_path, &mut parsed_files)?;

    Ok(())
}
