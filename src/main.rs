mod lexer;
mod parser;
use lexer::lex_with_positions;
use parser::node::Node;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use serde_json::Value;

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

    let source_path = Path::new(source_folder);
    if !source_path.is_dir() {
        eprintln!("Source folder '{}' is not a valid directory.", source_folder);
        std::process::exit(1);
    }

    let mut parsed_files = Vec::new();


    fn process_folder(folder_path: &Path, parsed_files: &mut Vec<(String, Node)>) -> io::Result<()> {
        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
        
                process_folder(&path, parsed_files)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "atra") {
               
                let mut file_content = String::new();
                File::open(&path)?.read_to_string(&mut file_content)?;

       
                let mut parser = parser::parser::Parser::new(lex_with_positions(&file_content));
                match parser.parse() {
                    Ok(node) => {
                        parsed_files.push((path.display().to_string(), node));
                    }
                    Err(err) => {
                        eprintln!("Error parsing file '{}': {}", path.display(), err);
                    }
                }
            }
        }
        Ok(())
    }


    process_folder(source_path, &mut parsed_files)?;


    for (file_path, node) in &parsed_files {
        println!("File: {}\nNode: {:?}", file_path, node);
        println!("-----------------------------------");
    }

    Ok(())
}