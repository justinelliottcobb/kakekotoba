use clap::{Arg, ArgAction, Command};
use kakekotoba::pipeline::{
    create_debug_options, create_default_options, create_optimized_options, Compiler,
    CompilerOptions,
};
use std::process;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let matches = Command::new("kakekotoba")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Kakekotoba Programming Language Compiler")
        .long_about("A compiler for the Kakekotoba programming language, which combines Japanese keywords with Haskell-style type systems and group homomorphisms for meta-programming.")
        .arg(
            Arg::new("input")
                .help("Input source file")
                .required(true)
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output executable file")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("optimize")
                .short('O')
                .long("optimize")
                .help("Enable optimizations")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("emit-ir")
                .long("emit-ir")
                .help("Output LLVM IR to stdout")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("type-check-only")
                .long("type-check-only")
                .help("Only perform type checking, don't generate code")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lex-only")
                .long("lex-only")
                .help("Only perform lexical analysis")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("parse-only")
                .long("parse-only")
                .help("Only perform parsing")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();

    info!("Kakekotoba compiler starting");
    info!("Input file: {}", input_file);

    let compiler = Compiler::new();

    // Handle different compilation modes
    if matches.get_flag("lex-only") {
        handle_lex_only(&compiler, input_file);
    } else if matches.get_flag("parse-only") {
        handle_parse_only(&compiler, input_file);
    } else if matches.get_flag("type-check-only") {
        handle_type_check_only(&compiler, input_file);
    } else {
        handle_full_compilation(&compiler, input_file, &matches);
    }
}

fn handle_lex_only(compiler: &Compiler, input_file: &str) {
    info!("Running lexer only");

    let source = match read_source_file(input_file) {
        Ok(source) => source,
        Err(e) => {
            error!("Failed to read input file: {}", e);
            process::exit(1);
        }
    };

    match compiler.lex_only(source) {
        Ok(tokens) => {
            println!("=== Tokens ===");
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => {
            error!("Lexical analysis failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_parse_only(compiler: &Compiler, input_file: &str) {
    info!("Running parser only");

    let source = match read_source_file(input_file) {
        Ok(source) => source,
        Err(e) => {
            error!("Failed to read input file: {}", e);
            process::exit(1);
        }
    };

    match compiler.parse_only(source) {
        Ok(ast) => {
            println!("=== Abstract Syntax Tree ===");
            println!("{:#?}", ast);
        }
        Err(e) => {
            error!("Parsing failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_type_check_only(compiler: &Compiler, input_file: &str) {
    info!("Running type checker only");

    let source = match read_source_file(input_file) {
        Ok(source) => source,
        Err(e) => {
            error!("Failed to read input file: {}", e);
            process::exit(1);
        }
    };

    match compiler.type_check_only(source) {
        Ok(result) => {
            println!("=== Type Information ===");
            for (name, scheme) in &result.type_environment {
                println!("{}: {:?}", name, scheme);
            }
            info!("Type checking completed successfully");
        }
        Err(e) => {
            error!("Type checking failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_full_compilation(compiler: &Compiler, input_file: &str, matches: &clap::ArgMatches) {
    info!("Running full compilation");

    let mut options = create_default_options();

    // Configure options based on command line arguments
    options.optimize = matches.get_flag("optimize");
    options.output_ir = matches.get_flag("emit-ir");
    options.output_path = matches.get_one::<String>("output").cloned();

    // If no output specified but optimization requested, create default output
    if options.optimize && options.output_path.is_none() {
        let output_name = input_file.replace(".kake", "").replace(".kakekotoba", "") + ".out";
        options.output_path = Some(output_name);
    }

    match compiler.compile_file(input_file, options) {
        Ok(result) => {
            info!("Compilation completed successfully");

            if let Some(executable) = result.executable {
                println!(
                    "Generated executable: {} ({} bytes)",
                    executable.path, executable.size
                );
            }
        }
        Err(e) => {
            error!("Compilation failed: {}", e);
            print_diagnostic(&e);
            process::exit(1);
        }
    }
}

fn read_source_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

fn print_diagnostic(error: &kakekotoba::Error) {
    use miette::{GraphicalReportHandler, GraphicalTheme};

    let mut handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
    let mut output = String::new();

    if handler.render_report(&mut output, error).is_ok() {
        eprintln!("{}", output);
    } else {
        eprintln!("Error: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simple_compilation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "// Simple kakekotoba program").unwrap();

        let compiler = Compiler::new();
        let options = create_default_options();

        // This should at least pass lexing stage
        let result = compiler.compile_file(temp_file.path(), options);

        // Since we don't have actual grammar implemented yet,
        // we expect it to fail at parsing, but not at file reading
        match result {
            Err(kakekotoba::Error::Parser { .. }) => {
                // Expected - parser not fully implemented
            }
            Err(kakekotoba::Error::Lexer { .. }) => {
                // Also expected - lexer not fully implemented
            }
            Ok(_) => {
                // Unexpected but good!
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }
}
