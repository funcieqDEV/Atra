mod codegen;
mod lexer;
mod parser;
use ariadne::{Color, Label, Report, ReportKind, Source};
use codegen::codegen::generate_html;
use lexer::lex_with_positions;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use parser::node::Node;
use serde_json::Value;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc::channel;

fn copy_static_files(src_dir: &Path, dest_dir: &Path) -> std::io::Result<()> {
    if !src_dir.exists() {
        return Ok(());
    }

    let static_extensions = [
        "css", "js", "javascript", "png", "jpg", "jpeg", "gif", "svg", "webp",
        "mov", "mp4", "avi", "mkv", "wav", "mp3", "m4a", "ogg", "flac",
        "pdf", "txt", "ico", "woff", "woff2", "ttf", "eot", "html", "htm",
        "json", "xml", "yaml", "yml", "md", "markdown", "csv", "tsv",
        "zip", "tar", "gz", "rar", "7z", "exe", "bat", "sh", "ps1",
        "dll", "so", "dylib", "apk", "ipa", "deb", "rpm", "msi", "pkg",
        "webmanifest", "wasm", "c", "cpp", "h", "hpp", "py", "rb", "go",
        "java", "cs", "rs", "swift", "kt", "ts", "tsx", "jsx", "html",
    ];

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        if path.is_dir() {
            let dest_subdir = dest_dir.join(&file_name);
            fs::create_dir_all(&dest_subdir)?;
            copy_static_files(&path, &dest_subdir)?;
        } else if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if static_extensions.contains(&ext_str.to_lowercase().as_str()) {
                    let dest_file = dest_dir.join(&file_name);
                    fs::copy(&path, &dest_file)?;
                    println!("Copied static file: {}", file_name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || (args[1] != "build" && args[1] != "watch") {
        eprintln!("Usage: atra <build|watch> <path/to/config.json>");
        std::process::exit(1);
    }

    let config_path = Path::new(&args[2]);
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

    let config_content = fs::read_to_string(config_path)?;
    let config: Value = serde_json::from_str(&config_content)?;

    let resolve_path = |folder: &str| {
        let path = Path::new(folder);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            config_dir.join(path)
        }
    };

    let source_path = resolve_path(
        config["source_folder"]
            .as_str()
            .expect("Expected 'source_folder' in config.json"),
    );
    let source_path = &source_path.as_path();

    let output_path = resolve_path(
        config["output_folder"]
            .as_str()
            .expect("Expected 'output_folder' in config.json"),
    );
    let output_path = &output_path.as_path();

    if args[1] == "build" {
        build_project(source_path, output_path)?;
    } else if args[1] == "watch" {
        watch_project(source_path, output_path)?;
    }

    Ok(())
}

fn build_project(source_path: &Path, output_path: &Path) -> io::Result<()> {
    if !source_path.is_dir() {
        eprintln!(
            "Source folder '{}' is not a valid directory.",
            source_path.display()
        );
        std::process::exit(1);
    }

    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
    }

    // Copy static files
    if let Err(e) = copy_static_files(source_path, output_path) {
        eprintln!("Warning: Failed to copy static files: {}", e);
    }

    let mut parsed_files = Vec::new();
    let mut global_components = std::collections::HashMap::new();

    collect_components(source_path, &mut global_components)?;
    process_folder(
        source_path,
        output_path,
        &mut parsed_files,
        &global_components,
    )?;

    Ok(())
}

fn watch_project(source_path: &Path, output_path: &Path) -> io::Result<()> {
    let (tx, rx) = channel();
    let config = notify::Config::default(); // Użycie domyślnej konfiguracji
    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Konwersja błędu notify::Error na std::io::Error
    watcher
        .watch(source_path, RecursiveMode::Recursive)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Konwersja błędu notify::Error na std::io::Error

    println!("Watching for changes in '{}'", source_path.display());

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("Change detected: {:?}", event);
                if let Err(err) = build_project(source_path, output_path) {
                    eprintln!("Error during build: {}", err);
                } else {
                    println!("Rebuild completed successfully.");
                }
            }
            Err(err) => eprintln!("Watch error: {}", err),
        }
    }
}

fn collect_components(
    folder_path: &Path,
    components: &mut std::collections::HashMap<String, codegen::codegen::ComponentDefinition>,
) -> io::Result<()> {
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_components(&path, components)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "atrac") {
            let mut file_content = String::new();
            File::open(&path)?.read_to_string(&mut file_content)?;

            match lex_with_positions(&file_content) {
                Ok(tokens) => {
                    let mut parser = parser::parser::Parser::new(tokens);
                    match parser.parse() {
                        Ok(node) => {
                            let mut temp_components = std::collections::HashMap::new();
                            generate_html(&node, 0, true, &mut temp_components);
                            components.extend(temp_components);
                        }
                        Err(err) => {
                            eprintln!(
                                "❌ Error parsing component file '{}':\n   {}",
                                path.display(),
                                err
                            );
                        }
                    }
                }
                Err(lex_err) => {
                    print_lex_error(&lex_err, &file_content, path.display().to_string().as_str());
                }
            }
        }
    }
    Ok(())
}

fn process_folder(
    folder_path: &Path,
    output_base: &Path,
    parsed_files: &mut Vec<(String, Node)>,
    global_components: &std::collections::HashMap<String, codegen::codegen::ComponentDefinition>,
) -> io::Result<()> {
    let mut components = global_components.clone(); // Skopiuj globalne komponenty

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let relative_path = path.strip_prefix(folder_path).unwrap();
            let output_subfolder = output_base.join(relative_path);
            fs::create_dir_all(&output_subfolder)?;

            process_folder(&path, &output_subfolder, parsed_files, global_components)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "atra") {
            let mut file_content = String::new();
            File::open(&path)?.read_to_string(&mut file_content)?;

            match lex_with_positions(&file_content) {
                Ok(tokens) => {
                    let mut parser = parser::parser::Parser::new(tokens);
                    match parser.parse() {
                        Ok(node) => {
                            parsed_files.push((path.display().to_string(), node.clone()));

                            let relative_path = path.strip_prefix(folder_path).unwrap();
                            let output_file_path = output_base.join(relative_path).with_extension("html");
                            let mut output_file = File::create(output_file_path)?;
                            let html_content = generate_html(&node, 0, true, &mut components);
                            output_file.write_all(html_content.as_bytes())?;
                        }
                        Err(err) => {
                            print_parse_error(&err, &file_content, path.display().to_string().as_str());
                        }
                    }
                }
                Err(lex_err) => {
                    print_lex_error(&lex_err, &file_content, path.display().to_string().as_str());
                }
            }
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "atrac") {
            println!("Loaded component file: {}", path.display());
        }
    }
    Ok(())
}

fn print_parse_error(error: &parser::error::ParseError, source: &str, filename: &str) {
    let span = error.span();
    let source_len = source.len();


    let valid_span = if span.0 > span.1 || span.1 > source_len {
        (0, source_len.min(1)) // Use safe default span
    } else {
        span
    };

    let report = Report::build(ReportKind::Error, filename, valid_span.0)
        .with_code(error.error_code())
        .with_message(format!("Parse Error: {}", error));

    let report = if valid_span.0 < valid_span.1 && valid_span.1 <= source_len {
        report.with_label(
            Label::new((filename, valid_span.0..valid_span.1))
                .with_message(get_error_message(error))
                .with_color(Color::Red),
        )
    } else {
        report
    };

    let report = match error {
        parser::error::ParseError::UnclosedBlock { start_span, .. } => {
            if start_span.0 < start_span.1 && start_span.1 <= source_len {
                report.with_label(
                    Label::new((filename, start_span.0..start_span.1))
                        .with_message("Block started here")
                        .with_color(Color::Blue),
                )
            } else {
                report
            }
        },
        _ => report,
    };

    report
        .finish()
        .print((filename, Source::from(source)))
        .unwrap();
}

fn get_error_message(error: &parser::error::ParseError) -> String {
    match error {
        parser::error::ParseError::UnexpectedToken { found, expected, .. } => {
            if let Some(exp) = expected {
                format!("Expected {}, found {:?}", exp, found)
            } else {
                format!("Unexpected token {:?}", found)
            }
        },
        parser::error::ParseError::ExpectedToken { expected, found, .. } => {
            format!("Expected {}, found {:?}", expected, found)
        },
        parser::error::ParseError::UnexpectedEof { expected, .. } => {
            format!("Unexpected end of file, expected {}", expected)
        },
        parser::error::ParseError::UnclosedBlock { symbol, .. } => {
            format!("Missing closing '{}'", symbol)
        },
        parser::error::ParseError::UnmatchedClosingBracket { bracket_type, .. } => {
            format!("Unmatched closing bracket '{}'", bracket_type)
        },
        parser::error::ParseError::TooManyClosingBrackets { bracket_type, .. } => {
            format!("Too many closing '{}' brackets", bracket_type)
        },
        parser::error::ParseError::MismatchedBrackets { expected, found, .. } => {
            format!("Expected closing '{}', but found '{}'", expected, found)
        },
        parser::error::ParseError::EmptyBlock { block_type, .. } => {
            format!("Empty {} block is not allowed", block_type)
        },
        parser::error::ParseError::InvalidTokenInContext { token, context, .. } => {
            format!("Token {:?} is not allowed in {} context", token, context)
        },
        parser::error::ParseError::MissingSemicolon { .. } => {
            "Missing semicolon".to_string()
        },
        parser::error::ParseError::InvalidEscapeSequence { sequence, .. } => {
            format!("Invalid escape sequence: {}", sequence)
        },
        parser::error::ParseError::NestedBlocksNotAllowed { block_type, .. } => {
            format!("Nested {} blocks are not allowed", block_type)
        },
        parser::error::ParseError::InvalidIdentifier { name, .. } => {
            format!("Invalid identifier: '{}'", name)
        },
        parser::error::ParseError::InvalidCssProperty { .. } => {
            "Expected 'property: value' format".to_string()
        },
        parser::error::ParseError::InvalidAttribute { .. } => {
            "Expected 'name=\"value\"' format".to_string()
        },
        parser::error::ParseError::AmbiguousComponent { name, .. } => {
            format!("Component '{}' has ambiguous syntax", name)
        },
        parser::error::ParseError::MissingSpecialFunctionArgs { name, .. } => {
            format!("Function '{}' requires arguments", name)
        },
        parser::error::ParseError::InvalidTextElement { .. } => {
            "Text element requires string literal".to_string()
        },
        parser::error::ParseError::DuplicateElement { element_type, .. } => {
            format!("Duplicate {} found", element_type)
        },
        parser::error::ParseError::UnclosedString { .. } => {
            "String literal not properly closed with quote".to_string()
        },
    }
}

fn print_lex_error(error: &lexer::LexError, source: &str, filename: &str) {
    let span = error.span();
    let source_len = source.len();


    let valid_span = if span.0 > span.1 || span.1 > source_len {
        (0, source_len.min(1)) // Use safe default span
    } else {
        span
    };

    let report = Report::build(ReportKind::Error, filename, valid_span.0)
        .with_code(error.error_code())
        .with_message(format!("Lex Error: {}", error));

    let report = if valid_span.0 < valid_span.1 && valid_span.1 <= source_len {
        report.with_label(
            Label::new((filename, valid_span.0..valid_span.1))
                .with_message(get_lex_error_message(error))
                .with_color(Color::Red),
        )
    } else {
        report
    };

    report
        .finish()
        .print((filename, Source::from(source)))
        .unwrap();
}

fn get_lex_error_message(error: &lexer::LexError) -> String {
    match error {
        lexer::LexError::UnterminatedString { .. } => {
            "Unterminated string literal".to_string()
        }
        lexer::LexError::InvalidCharacter { char, .. } => {
            format!("Unexpected character: '{}'", char)
        }
        lexer::LexError::InvalidEscapeSequence { sequence, .. } => {
            format!("Invalid escape sequence: {}", sequence)
        }
        lexer::LexError::EmptyStringLiteral { .. } => {
            "Empty string literal".to_string()
        }
    }
}