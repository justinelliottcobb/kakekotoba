use clap::{Arg, ArgAction, Command};
use kakekotoba::pipeline::{create_default_options, Compiler};
use kakekotoba::repl::Repl;
use std::process;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let matches = Command::new("kakekotoba")
        .version("0.1.0")
        .about("掛詞 Programming Language")
        .subcommand_required(false)
        .subcommand(
            Command::new("run")
                .about("Run a kakekotoba source file")
                .arg(
                    Arg::new("file")
                        .help("Source file to run (.kk)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("repl").about("Start the interactive REPL"))
        .subcommand(
            Command::new("compile")
                .about("Compile a kakekotoba source file")
                .arg(
                    Arg::new("input")
                        .help("Input source file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file")
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
                    Arg::new("type-check-only")
                        .long("type-check-only")
                        .help("Only perform type checking")
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
                ),
        )
        // Legacy: bare file argument for backwards compatibility
        .arg(
            Arg::new("input")
                .help("Input source file (use 'run' or 'compile' subcommands instead)")
                .index(1),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", sub_m)) => {
            let file = sub_m.get_one::<String>("file").unwrap();
            handle_run(file);
        }
        Some(("repl", _)) => {
            let mut repl = Repl::new();
            repl.run();
        }
        Some(("compile", sub_m)) => {
            let input = sub_m.get_one::<String>("input").unwrap();
            let compiler = Compiler::new();

            if sub_m.get_flag("lex-only") {
                handle_lex_only(&compiler, input);
            } else if sub_m.get_flag("parse-only") {
                handle_parse_only(&compiler, input);
            } else if sub_m.get_flag("type-check-only") {
                handle_type_check_only(&compiler, input);
            } else {
                handle_full_compilation(&compiler, input, sub_m);
            }
        }
        _ => {
            // No subcommand — check for legacy bare file argument
            if let Some(input) = matches.get_one::<String>("input") {
                // Auto-detect: if it looks like a .kk file, run it
                handle_run(input);
            } else {
                // No arguments at all: start REPL
                let mut repl = Repl::new();
                repl.run();
            }
        }
    }
}

fn handle_run(file: &str) {
    let compiler = Compiler::new();
    match compiler.run_file(file) {
        Ok(value) => {
            // Only print if the result is non-unit
            if !matches!(value, kakekotoba::interpreter::Value::Unit) {
                println!("{}", value);
            }
        }
        Err(e) => {
            print_diagnostic(&e);
            process::exit(1);
        }
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
            print_diagnostic(&e);
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

    match compiler.parse_sexp(source) {
        Ok(ast) => {
            println!("=== Abstract Syntax Tree ===");
            println!("{:#?}", ast);
        }
        Err(e) => {
            print_diagnostic(&e);
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
            print_diagnostic(&e);
            process::exit(1);
        }
    }
}

fn handle_full_compilation(compiler: &Compiler, input_file: &str, matches: &clap::ArgMatches) {
    info!("Running full compilation");

    let mut options = create_default_options();
    options.optimize = matches.get_flag("optimize");
    options.output_path = matches.get_one::<String>("output").cloned();

    if options.optimize && options.output_path.is_none() {
        let output_name = input_file.replace(".kk", "").replace(".kakekotoba", "") + ".out";
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

    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
    let mut output = String::new();

    if handler.render_report(&mut output, error).is_ok() {
        eprintln!("{}", output);
    } else {
        eprintln!("Error: {}", error);
    }
}
