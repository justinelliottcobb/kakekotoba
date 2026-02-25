use crate::ast::Program;
use crate::codegen::CodeGenerator;
use crate::error::{Error, Result};
use crate::inference::TypeInference;
use crate::lexer::Lexer;
use crate::parser::Parser;
use inkwell::context::Context;
use std::path::Path;
use tracing::{info, instrument, warn};

pub struct Compiler {
    context: Context,
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub optimize: bool,
    pub output_ir: bool,
    pub type_check_only: bool,
    pub output_path: Option<String>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            optimize: false,
            output_ir: false,
            type_check_only: false,
            output_path: None,
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            context: Context::create(),
        }
    }

    #[instrument(skip(self, source))]
    pub fn compile_source(
        &self,
        source: String,
        options: CompilerOptions,
    ) -> Result<CompilationResult> {
        info!("Starting compilation pipeline");

        // Stage 1: Lexical Analysis
        info!("Running lexer");
        let tokens = self.lex(source)?;

        // Stage 2: Parsing
        info!("Running parser");
        let ast = self.parse(tokens)?;

        // Stage 3: Type Inference and Checking
        info!("Running type checker");
        let type_info = self.type_check(&ast)?;

        if options.type_check_only {
            return Ok(CompilationResult {
                ast: Some(ast),
                type_info: Some(type_info),
                ir: None,
                executable: None,
            });
        }

        // Stage 4: Code Generation
        info!("Running code generator");
        let ir = self.generate_code(&ast)?;

        if options.output_ir {
            println!("=== Generated LLVM IR ===");
            ir.print_to_stderr();
        }

        // Stage 5: Optimization (if requested)
        if options.optimize {
            info!("Running optimizations");
            self.optimize_ir(&ir)?;
        }

        // Stage 6: Executable Generation (if output path specified)
        let executable = if let Some(output_path) = options.output_path {
            info!("Generating executable: {}", output_path);
            Some(self.generate_executable(&ir, &output_path)?)
        } else {
            None
        };

        info!("Compilation completed successfully");

        Ok(CompilationResult {
            ast: Some(ast),
            type_info: Some(type_info),
            ir: Some(ir),
            executable,
        })
    }

    #[instrument(skip(self, path))]
    pub fn compile_file<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        path: P,
        options: CompilerOptions,
    ) -> Result<CompilationResult> {
        let source = std::fs::read_to_string(&path).map_err(|e| Error::Io(e))?;

        info!("Compiling file: {:?}", path);
        self.compile_source(source, options)
    }

    fn lex(&self, source: String) -> Result<Vec<crate::lexer::Token>> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize()
    }

    fn parse(&self, tokens: Vec<crate::lexer::Token>) -> Result<Program> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn type_check(&self, ast: &Program) -> Result<TypeInferenceResult> {
        let mut inference = TypeInference::new();
        let type_env = inference.infer_program(ast)?;

        Ok(TypeInferenceResult {
            type_environment: type_env,
        })
    }

    fn generate_code(&self, ast: &Program) -> Result<IRModule> {
        let mut codegen = CodeGenerator::new(&self.context, "kakekotoba_module")?;
        codegen.compile_program(ast)?;

        Ok(IRModule {
            _context: &self.context,
            module: codegen.get_module(),
        })
    }

    fn optimize_ir(&self, _ir: &IRModule) -> Result<()> {
        // TODO: Implement LLVM optimization passes
        warn!("IR optimization not yet implemented");
        Ok(())
    }

    fn generate_executable(&self, _ir: &IRModule, output_path: &str) -> Result<ExecutableInfo> {
        // TODO: Implement executable generation
        warn!("Executable generation not yet implemented");

        Ok(ExecutableInfo {
            path: output_path.to_string(),
            size: 0,
        })
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CompilationResult {
    pub ast: Option<Program>,
    pub type_info: Option<TypeInferenceResult>,
    pub ir: Option<IRModule>,
    pub executable: Option<ExecutableInfo>,
}

#[derive(Debug)]
pub struct TypeInferenceResult {
    pub type_environment: std::collections::HashMap<String, crate::types::TypeScheme>,
}

pub struct IRModule<'ctx> {
    _context: &'ctx Context,
    module: &'ctx inkwell::module::Module<'ctx>,
}

impl<'ctx> IRModule<'ctx> {
    pub fn print_to_stderr(&self) {
        self.module.print_to_stderr();
    }
}

#[derive(Debug)]
pub struct ExecutableInfo {
    pub path: String,
    pub size: u64,
}

// Pipeline stages for testing and debugging
impl Compiler {
    pub fn lex_only(&self, source: String) -> Result<Vec<crate::lexer::Token>> {
        self.lex(source)
    }

    pub fn parse_only(&self, source: String) -> Result<Program> {
        let tokens = self.lex(source)?;
        self.parse(tokens)
    }

    pub fn type_check_only(&self, source: String) -> Result<TypeInferenceResult> {
        let tokens = self.lex(source)?;
        let ast = self.parse(tokens)?;
        self.type_check(&ast)
    }
}

// Utility functions for the CLI
pub fn create_default_options() -> CompilerOptions {
    CompilerOptions::default()
}

pub fn create_optimized_options(output_path: String) -> CompilerOptions {
    CompilerOptions {
        optimize: true,
        output_ir: false,
        type_check_only: false,
        output_path: Some(output_path),
    }
}

pub fn create_debug_options() -> CompilerOptions {
    CompilerOptions {
        optimize: false,
        output_ir: true,
        type_check_only: false,
        output_path: None,
    }
}
